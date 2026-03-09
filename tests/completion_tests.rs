// Completion tests — covers every route that was previously untested:
//   - GET /admin/products/{id}/variants/{variant_id}
//   - GET /admin/product-variants  (standalone list)
//   - GET /admin/promotions/{id}/buy-rules
//   - GET /admin/promotions/{id}/target-rules
//   - GET /admin/promotions/rule-attribute-options/{rule_type}
//   - GET /admin/promotions/rule-value-options/{rule_type}/{rule_attribute_id}
//   - POST /wizard/import
//   - GET  /wizard/import/{job_id}/status
//   - GET  /health
//   - Moka cache namespace helpers
//   - StorageConfig public_base_url variants
use axum::{body::Body, http::{Request, StatusCode}, Router};
use std::sync::Arc;
use medusa_rust::{api::auth::AuthService, state::AppState};
use tower::util::ServiceExt;

// ─── Test harness ─────────────────────────────────────────────────────────────

struct DummyAuth;

#[async_trait::async_trait]
impl AuthService for DummyAuth {
    async fn authenticate(
        &self,
        _actor: medusa_rust::api::auth::ActorType,
        _provider: &str,
        _data: medusa_rust::api::auth::AuthData,
    ) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        Ok(medusa_rust::api::auth::AuthResult {
            success: true,
            error: None,
            auth_identity: Some(medusa_rust::api::auth::AuthIdentity {
                id: uuid::Uuid::new_v4(),
                email: "test@example.com".into(),
            }),
            location: None,
        })
    }
    async fn validate_callback(
        &self,
        actor: medusa_rust::api::auth::ActorType,
        provider: &str,
        data: medusa_rust::api::auth::AuthData,
    ) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        self.authenticate(actor, provider, data).await
    }
    async fn register(
        &self,
        actor: medusa_rust::api::auth::ActorType,
        provider: &str,
        data: medusa_rust::api::auth::AuthData,
    ) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        self.authenticate(actor, provider, data).await
    }
    async fn reset_password(
        &self,
        _actor: medusa_rust::api::auth::ActorType,
        _provider: &str,
        _data: medusa_rust::api::auth::AuthData,
    ) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        Ok(medusa_rust::api::auth::AuthResult {
            success: true,
            error: None,
            auth_identity: None,
            location: None,
        })
    }
    async fn update(
        &self,
        _actor: medusa_rust::api::auth::ActorType,
        _provider: &str,
        _data: medusa_rust::api::auth::AuthData,
    ) -> Result<medusa_rust::api::auth::AuthResult, medusa_rust::error::AppError> {
        Ok(medusa_rust::api::auth::AuthResult {
            success: true,
            error: None,
            auth_identity: None,
            location: None,
        })
    }
}

fn make_state() -> AppState {
    let pool = sqlx::PgPool::connect_lazy("postgres://127.0.0.1/nope").unwrap();
    AppState {
        db: Arc::new(pool),
        cache: Arc::new(moka::future::Cache::new(100)),
        storage: Arc::new(medusa_rust::storage::s3::S3Storage::new(
            &medusa_rust::state::StorageConfig::default(),
        )),
        storage_config: Arc::new(medusa_rust::state::StorageConfig::default()),
        jwt_secret: "secret".into(),
        auth_service: Arc::new(DummyAuth),
        payment_methods: Arc::new(tokio::sync::Mutex::new(Vec::new())),
        plugin_mgr: Arc::new(tokio::sync::Mutex::new(medusa_rust::plugins::PluginManager::new())),
        event_bus: Arc::new(medusa_rust::EventBus::new()),
    }
}

fn app() -> Router {
    medusa_rust::api::build_router(make_state())
}

fn admin_token() -> String {
    medusa_rust::auth::jwt::encode_admin_token(
        &uuid::Uuid::new_v4(),
        "admin@test.com",
        "secret",
        24,
    )
    .unwrap()
}

// ─── Health check ─────────────────────────────────────────────────────────────

#[test]
fn test_health_check() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    });
}

// ─── Product Variants — GET /admin/products/{id}/variants/{variant_id} ────────

#[test]
fn test_product_variant_get_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let pid = uuid::Uuid::new_v4();
        let vid = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(format!("/admin/products/{}/variants/{}", pid, vid))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_product_variant_get_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let pid = uuid::Uuid::new_v4();
        let vid = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(format!("/admin/products/{}/variants/{}", pid, vid))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Standalone Product Variants — GET /admin/product-variants ────────────────

#[test]
fn test_product_variants_standalone_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/product-variants")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_product_variants_standalone_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/product-variants")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Promotions Buy Rules — GET /admin/promotions/{id}/buy-rules ──────────────

#[test]
fn test_promotions_buy_rules_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(format!("/admin/promotions/{}/buy-rules", id))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_promotions_buy_rules_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(format!("/admin/promotions/{}/buy-rules", id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Promotions Target Rules — GET /admin/promotions/{id}/target-rules ────────

#[test]
fn test_promotions_target_rules_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(format!("/admin/promotions/{}/target-rules", id))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_promotions_target_rules_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(format!("/admin/promotions/{}/target-rules", id))
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Promotions Rule Attribute Options ────────────────────────────────────────

#[test]
fn test_promotions_rule_attribute_options_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/promotions/rule-attribute-options/rules")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_promotions_rule_attribute_options_rules_type() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/promotions/rule-attribute-options/rules")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(resp.status(), StatusCode::OK);
    });
}

#[test]
fn test_promotions_rule_attribute_options_buy_rules_type() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/promotions/rule-attribute-options/buy-rules")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    });
}

#[test]
fn test_promotions_rule_attribute_options_target_rules_type() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/promotions/rule-attribute-options/target-rules")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    });
}

// ─── Promotions Rule Value Options ────────────────────────────────────────────

#[test]
fn test_promotions_rule_value_options_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/promotions/rule-value-options/rules/cart.subtotal")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_promotions_rule_value_options_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/promotions/rule-value-options/rules/cart.subtotal")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(resp.status(), StatusCode::OK);
    });
}

// ─── Wizard Import ────────────────────────────────────────────────────────────

#[test]
fn test_wizard_import_no_file_returns_bad_request() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        // Sending a multipart request without a "file" field should return 400.
        let boundary = "boundary_abc123";
        let body = format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"other\"\r\n\r\nhello\r\n--{boundary}--\r\n",
            boundary = boundary
        );
        let req = Request::builder()
            .method("POST")
            .uri("/wizard/import")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .body(Body::from(body))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    });
}

#[test]
fn test_wizard_import_status() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let job_id = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method("GET")
            .uri(format!("/wizard/import/{}/status", job_id))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(resp.status(), StatusCode::OK);
    });
}

#[test]
fn test_wizard_import_status_method_not_allowed_on_post() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let job_id = uuid::Uuid::new_v4();
        let req = Request::builder()
            .method("DELETE")
            .uri(format!("/wizard/import/{}/status", job_id))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Moka cache namespace helpers ─────────────────────────────────────────────

#[test]
fn test_cache_ns_key_format() {
    use medusa_rust::storage::cache::ns;
    assert_eq!(ns::key(ns::PRODUCTS, "abc"), "products:abc");
    assert_eq!(ns::key(ns::CATEGORIES, "xyz"), "categories:xyz");
    assert_eq!(ns::key(ns::REGIONS, "r1"), "regions:r1");
    assert_eq!(ns::key(ns::PRICE_LISTS, "p1"), "price_lists:p1");
    assert_eq!(ns::key(ns::SESSION, "s1"), "session:s1");
    assert_eq!(ns::key(ns::SHIPPING, "sh1"), "shipping_options:sh1");
}

#[test]
fn test_cache_insert_and_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        use medusa_rust::storage::cache::{build_cache, ns};
        let cache = build_cache(100, 300);
        let key = ns::key(ns::PRODUCTS, "test-product-123");
        let val = serde_json::json!({"id": "test-product-123", "title": "Test Product"});
        cache.insert(key.clone(), val.clone()).await;
        let fetched = cache.get(&key).await;
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap(), val);
    });
}

#[test]
fn test_cache_miss_returns_none() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        use medusa_rust::storage::cache::build_cache;
        let cache = build_cache(100, 300);
        let fetched = cache.get("nonexistent-key").await;
        assert!(fetched.is_none());
    });
}

// ─── StorageConfig public URL variants ────────────────────────────────────────

#[test]
fn test_storage_config_default_public_url_aws() {
    let cfg = medusa_rust::state::StorageConfig {
        s3_endpoint: None,
        s3_bucket: "my-bucket".into(),
        s3_region: "us-east-1".into(),
        s3_access_key: "key".into(),
        s3_secret_key: "secret".into(),
        s3_force_path_style: false,
        s3_public_url: None,
        upload_dir: "uploads".into(),
    };
    let url = cfg.public_base_url();
    assert_eq!(url, "https://my-bucket.s3.us-east-1.amazonaws.com");
}

#[test]
fn test_storage_config_minio_public_url() {
    let cfg = medusa_rust::state::StorageConfig {
        s3_endpoint: Some("http://localhost:9000".into()),
        s3_bucket: "medusa-uploads".into(),
        s3_region: "us-east-1".into(),
        s3_access_key: "minioadmin".into(),
        s3_secret_key: "minioadmin".into(),
        s3_force_path_style: true,
        s3_public_url: None,
        upload_dir: "uploads".into(),
    };
    let url = cfg.public_base_url();
    assert_eq!(url, "http://localhost:9000/medusa-uploads");
}

#[test]
fn test_storage_config_custom_cdn_url() {
    let cfg = medusa_rust::state::StorageConfig {
        s3_endpoint: Some("http://localhost:9000".into()),
        s3_bucket: "medusa-uploads".into(),
        s3_region: "us-east-1".into(),
        s3_access_key: "minioadmin".into(),
        s3_secret_key: "minioadmin".into(),
        s3_force_path_style: true,
        s3_public_url: Some("https://cdn.example.com".into()),
        upload_dir: "uploads".into(),
    };
    let url = cfg.public_base_url();
    assert_eq!(url, "https://cdn.example.com");
}

// ─── S3Storage public_url helper (no network I/O) ─────────────────────────────

#[test]
fn test_s3storage_public_url_minio() {
    use medusa_rust::storage::s3::StorageBackend;
    let cfg = medusa_rust::state::StorageConfig {
        s3_endpoint: Some("http://localhost:9000".into()),
        s3_bucket: "medusa-uploads".into(),
        s3_region: "us-east-1".into(),
        s3_access_key: "minioadmin".into(),
        s3_secret_key: "minioadmin".into(),
        s3_force_path_style: true,
        s3_public_url: None,
        upload_dir: "uploads".into(),
    };
    let storage = medusa_rust::storage::s3::S3Storage::new(&cfg);
    let url = storage.public_url("products/image.jpg");
    assert_eq!(url, "http://localhost:9000/medusa-uploads/products/image.jpg");
}

#[test]
fn test_s3storage_public_url_aws() {
    use medusa_rust::storage::s3::StorageBackend;
    let cfg = medusa_rust::state::StorageConfig {
        s3_endpoint: None,
        s3_bucket: "my-bucket".into(),
        s3_region: "eu-west-1".into(),
        s3_access_key: "key".into(),
        s3_secret_key: "secret".into(),
        s3_force_path_style: false,
        s3_public_url: None,
        upload_dir: "uploads".into(),
    };
    let storage = medusa_rust::storage::s3::S3Storage::new(&cfg);
    let url = storage.public_url("assets/photo.png");
    assert_eq!(url, "https://my-bucket.s3.eu-west-1.amazonaws.com/assets/photo.png");
}

// ─── Base64 encode/decode round-trip (wizard internals) ───────────────────────

#[test]
fn test_wizard_base64_round_trip() {
    use medusa_rust::wizard::zip_import::{base64_encode, base64_decode};
    let original = b"Hello, MedusaRust! \x00\xFF\xAB";
    let encoded = base64_encode(original);
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn test_wizard_base64_empty() {
    use medusa_rust::wizard::zip_import::{base64_encode, base64_decode};
    let encoded = base64_encode(b"");
    assert_eq!(encoded, "");
    let decoded = base64_decode("").unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn test_wizard_base64_invalid_input() {
    use medusa_rust::wizard::zip_import::base64_decode;
    // Non-base64 character should fail
    let result = base64_decode("!!!!");
    assert!(result.is_err());
}

// ─── Wizard slugify helper ─────────────────────────────────────────────────────

#[test]
fn test_slugify_basic() {
    assert_eq!(medusa_rust::wizard::slugify("Hello World"), "hello-world");
}

#[test]
fn test_slugify_accents() {
    // accented characters should be normalized / stripped
    let slug = medusa_rust::wizard::slugify("Café Açaí");
    assert!(!slug.contains(' '), "slug should not contain spaces");
    assert_eq!(slug, slug.to_lowercase(), "slug should be lowercase");
}

#[test]
fn test_slugify_special_chars() {
    let slug = medusa_rust::wizard::slugify("Product #1 (Special)");
    assert!(!slug.contains('#'));
    assert!(!slug.contains('('));
    assert!(!slug.contains(')'));
}

// ─── Promotions rule attribute options response shape ─────────────────────────

#[test]
fn test_promotions_rule_attribute_options_response_shape() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/promotions/rule-attribute-options/rules")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // Medusa JS v2 compatible shape: { "attributes": [...] }
        assert!(body.get("attributes").is_some(), "response must have 'attributes' key");
        let attrs = body["attributes"].as_array().unwrap();
        assert!(!attrs.is_empty(), "rules type should have at least one attribute");
        // Each attribute should have id, label, field_type
        for attr in attrs {
            assert!(attr.get("id").is_some());
            assert!(attr.get("label").is_some());
            assert!(attr.get("field_type").is_some());
        }
    });
}

#[test]
fn test_promotions_rule_value_options_response_shape() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/promotions/rule-value-options/rules/cart.subtotal")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        // Medusa JS v2 compatible shape: { "values": [...], "count": N }
        assert!(body.get("values").is_some(), "response must have 'values' key");
        assert!(body.get("count").is_some(), "response must have 'count' key");
        assert!(body.get("rule_type").is_some());
        assert!(body.get("rule_attribute_id").is_some());
    });
}
