use axum::{body::Body, http::{Request, StatusCode}, Router};
use serde_json::json;
use std::sync::Arc;
use medusa_rust::{api::auth::AuthService, state::AppState};
use tower::util::ServiceExt;

// dummy service always returns success with fixed id/email
struct DummyService;

#[async_trait::async_trait]
impl AuthService for DummyService {
    async fn authenticate(&self, _actor: medusa_rust::api::auth::ActorType, _provider: &str, _data: medusa_rust::api::auth::AuthData) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        Ok(medusa_rust::api::auth::AuthResult { success: true, error: None, auth_identity: Some(medusa_rust::api::auth::AuthIdentity { id: uuid::Uuid::new_v4(), email: "foo@bar".into() }), location: None })
    }
    async fn validate_callback(&self, _actor: medusa_rust::api::auth::ActorType, _provider: &str, _data: medusa_rust::api::auth::AuthData) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        self.authenticate(_actor, _provider, _data).await
    }
    async fn register(&self, actor: medusa_rust::api::auth::ActorType, provider: &str, data: medusa_rust::api::auth::AuthData) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        self.authenticate(actor, provider, data).await
    }
    async fn reset_password(&self, _actor: medusa_rust::api::auth::ActorType, _provider: &str, _data: medusa_rust::api::auth::AuthData) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        Ok(medusa_rust::api::auth::AuthResult { success: true, error: None, auth_identity: None, location: None })
    }
    async fn update(&self, _actor: medusa_rust::api::auth::ActorType, _provider: &str, _data: medusa_rust::api::auth::AuthData) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        Ok(medusa_rust::api::auth::AuthResult { success: true, error: None, auth_identity: None, location: None })
    }
}

fn make_state() -> AppState {
    let pool = sqlx::PgPool::connect_lazy("postgres://127.0.0.1/nope").unwrap();
    AppState {
        db: Arc::new(pool),
        cache: Arc::new(moka::future::Cache::new(100)),
        storage: Arc::new(medusa_rust::storage::s3::S3Storage::new(&medusa_rust::state::StorageConfig::default())),
        storage_config: Arc::new(medusa_rust::state::StorageConfig::default()),
        jwt_secret: "secret".into(),
        auth_service: Arc::new(DummyService),
        payment_methods: Arc::new(tokio::sync::Mutex::new(Vec::new())),
    }
}

fn app() -> Router {
    medusa_rust::api::build_router(make_state())
}

// integration tests run in a separate crate – tokio macro doesn't register
// with the harness, so drive an explicit runtime instead.
#[test]
fn test_provider_returns_token() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("POST")
            .uri("/auth/customer/email")
            .header("content-type", "application/json")
            .body(Body::from(json!({"email":"a","password":"b"}).to_string()))
            .unwrap();
        let resp: axum::http::Response<axum::body::Body> = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let b = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&b).unwrap();
        assert!(v.get("token").is_some());
    });
}

#[test]
fn test_session_post() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = medusa_rust::auth::jwt::encode_store_token(&uuid::Uuid::new_v4(), "e", "secret", 24).unwrap();
        let req = Request::builder()
            .method("POST")
            .uri("/auth/session")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp: axum::http::Response<axum::body::Body> = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let b = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&b).unwrap();
        assert!(v.get("user").is_some());
    });
}

#[test]
fn test_token_refresh() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = medusa_rust::auth::jwt::encode_store_token(&uuid::Uuid::new_v4(), "e", "secret", 24).unwrap();
        let req = Request::builder()
            .method("POST")
            .uri("/auth/token/refresh")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp: axum::http::Response<axum::body::Body> = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let b = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&b).unwrap();
        assert!(v.get("token").is_some());
    });
}

// additional endpoints
#[test]
fn test_register_returns_token() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("POST")
            .uri("/auth/customer/email/register")
            .header("content-type", "application/json")
            .body(Body::from(json!({"email":"a","password":"b"}).to_string()))
            .unwrap();
        let resp: axum::http::Response<axum::body::Body> = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let b = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&b).unwrap();
        assert!(v.get("token").is_some());
    });
}

#[test]
fn test_callback_returns_token() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("POST")
            .uri("/auth/customer/email/callback")
            .header("content-type", "application/json")
            .body(Body::from(json!({"code":"x"}).to_string()))
            .unwrap();
        let resp: axum::http::Response<axum::body::Body> = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let b = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&b).unwrap();
        assert!(v.get("token").is_some());
    });
}

#[test]
fn test_reset_password_status() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("POST")
            .uri("/auth/customer/email/reset-password")
            .header("content-type", "application/json")
            .body(Body::from(json!({"email":"a"}).to_string()))
            .unwrap();
        let resp: axum::http::Response<axum::body::Body> = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    });
}

#[test]
fn test_update_success() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("POST")
            .uri("/auth/customer/email/update")
            .header("content-type", "application/json")
            .body(Body::from(json!({"foo":"bar"}).to_string()))
            .unwrap();
        let resp: axum::http::Response<axum::body::Body> = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let b = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&b).unwrap();
        assert_eq!(v.get("success").and_then(|s| s.as_bool()), Some(true));
    });
}
