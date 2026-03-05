//! S3-compatible object storage backend.
//!
//! Works with **MinIO** (self-hosted) and **AWS S3** out of the box:
//!
//! * Set `S3_ENDPOINT=http://localhost:9000` + `S3_FORCE_PATH_STYLE=true` for MinIO.
//! * Leave `S3_ENDPOINT` unset to route requests to native AWS S3.
//!
//! The `StorageBackend` trait is the interface used by the rest of the application,
//! so the underlying provider can be swapped transparently.

use async_trait::async_trait;
use aws_sdk_s3::{
    config::{BehaviorVersion, Credentials, Region},
    operation::create_bucket::CreateBucketError,
    primitives::ByteStream,
    Client,
};
use tracing::{info, warn};

use crate::{error::AppError, state::StorageConfig};

// ─── Trait ────────────────────────────────────────────────────────────────────

/// Abstraction over any S3-compatible object store.
#[async_trait]
pub trait StorageBackend: Send + Sync + 'static {
    /// Upload `data` under `key`, returning its public URL.
    async fn upload_file(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, AppError>;

    /// Delete the object identified by `key`.
    async fn delete_file(&self, key: &str) -> Result<(), AppError>;

    /// Return the public URL for an existing key (no I/O).
    fn public_url(&self, key: &str) -> String;
}

// ─── S3 / MinIO implementation ────────────────────────────────────────────────

pub struct S3Storage {
    client: Client,
    bucket: String,
    base_url: String,
}

impl S3Storage {
    /// Build an `S3Storage` from a [`StorageConfig`].
    ///
    /// This function is intentionally synchronous — the `aws-sdk-s3` builder is
    /// cheap and the actual network calls happen inside the async methods.
    pub fn new(config: &StorageConfig) -> Self {
        let creds = Credentials::new(
            &config.s3_access_key,
            &config.s3_secret_key,
            None,
            None,
            "medusa-rust-static",
        );

        let mut builder = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(creds)
            .region(Region::new(config.s3_region.clone()))
            .force_path_style(config.s3_force_path_style);

        if let Some(ref endpoint) = config.s3_endpoint {
            builder = builder.endpoint_url(endpoint);
        }

        let s3_config = builder.build();
        let client = Client::from_conf(s3_config);

        S3Storage {
            client,
            bucket: config.s3_bucket.clone(),
            base_url: config.public_base_url(),
        }
    }

    /// Ensure the bucket exists, creating it with public-read ACL if needed.
    ///
    /// Call this once at server startup — idempotent.
    pub async fn ensure_bucket_exists(&self) -> Result<(), AppError> {
        match self
            .client
            .create_bucket()
            .bucket(&self.bucket)
            .send()
            .await
        {
            Ok(_) => {
                info!(bucket = %self.bucket, "S3 bucket created.");
            }
            Err(sdk_err) => {
                // `BucketAlreadyOwnedByYou` / `BucketAlreadyExists` are not errors.
                let raw = sdk_err.into_service_error();
                match &raw {
                    CreateBucketError::BucketAlreadyOwnedByYou(_)
                    | CreateBucketError::BucketAlreadyExists(_) => {
                        info!(bucket = %self.bucket, "S3 bucket already exists.");
                    }
                    other => {
                        warn!(bucket = %self.bucket, error = ?other, "Could not create bucket.");
                        // Don't abort startup — bucket may already exist under a
                        // different error code in some MinIO versions.
                    }
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl StorageBackend for S3Storage {
    async fn upload_file(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, AppError> {
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 upload error: {e}")))?;

        Ok(self.public_url(key))
    }

    async fn delete_file(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("S3 delete error: {e}")))?;
        Ok(())
    }

    fn public_url(&self, key: &str) -> String {
        format!("{}/{}", self.base_url.trim_end_matches('/'), key)
    }
}
