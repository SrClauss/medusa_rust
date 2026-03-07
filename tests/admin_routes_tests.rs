use axum::{body::Body, http::{Request, StatusCode}, Router};
use serde_json::json;
use std::sync::Arc;
use medusa_rust::{api::auth::AuthService, state::AppState};
use tower::util::ServiceExt;

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

fn admin_token() -> String {
    medusa_rust::auth::jwt::encode_admin_token(&uuid::Uuid::new_v4(), "admin@test.com", "secret", 24).unwrap()
}

// ─── Products ─────────────────────────────────────────────────────────────────

#[test]
fn test_products_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/products").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_products_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/products").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_products_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/products").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"title":"Test Product"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_products_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/products/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_products_delete_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/products/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_products_variants_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/products/{}/variants", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_products_create_variant_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/products/{}/variants", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"title":"Variant 1"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_products_options_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/products/{}/options", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Collections ──────────────────────────────────────────────────────────────

#[test]
fn test_collections_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/collections").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_collections_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/collections").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_collections_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/collections").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"title":"Summer Collection"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_collections_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/collections/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_collections_products_batch_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/collections/{}/products/batch", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"product_ids":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Orders ───────────────────────────────────────────────────────────────────

#[test]
fn test_orders_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/orders").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_orders_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/orders").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_orders_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/orders/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_orders_cancel_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/orders/{}/cancel", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_orders_complete_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/orders/{}/complete", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_orders_archive_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/orders/{}/archive", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_orders_fulfillment_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/orders/{}/fulfillments", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"items":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_orders_refund_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/orders/{}/refunds", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"amount":0}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_orders_swap_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/orders/{}/swaps", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"return_items":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_orders_claim_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/orders/{}/claims", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"type":"replace"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Customers ────────────────────────────────────────────────────────────────

#[test]
fn test_customers_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/customers").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_customers_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/customers").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_customers_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/customers").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"email":"c@test.com","first_name":"A","last_name":"B"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_customers_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/customers/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Product Categories ───────────────────────────────────────────────────────

#[test]
fn test_categories_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/product-categories").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_categories_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/product-categories").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_categories_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/product-categories").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"Electronics"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_categories_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/product-categories/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Regions ──────────────────────────────────────────────────────────────────

#[test]
fn test_regions_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/regions").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_regions_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/regions").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_regions_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/regions").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"North America","currency_code":"usd"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_regions_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/regions/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_regions_add_country_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/regions/{}/countries", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"country_code":"US"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_regions_fulfillment_providers_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/regions/{}/fulfillment-providers", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"provider_id":"manual"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Discounts ────────────────────────────────────────────────────────────────

#[test]
fn test_discounts_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/discounts").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_discounts_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/discounts").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_discounts_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/discounts").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"code":"SAVE10","rule":{"type":"percentage","value":10}}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_discounts_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/discounts/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_discounts_get_by_code_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/discounts/code/SAVE10").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_discounts_conditions_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/discounts/{}/conditions", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Shipping Options ─────────────────────────────────────────────────────────

#[test]
fn test_shipping_options_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/shipping-options").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_shipping_options_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/shipping-options").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_shipping_options_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/shipping-options").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"Standard Shipping"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_shipping_options_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/shipping-options/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_shipping_options_delete_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/shipping-options/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Users ────────────────────────────────────────────────────────────────────

#[test]
fn test_users_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/users").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_users_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/users").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_users_me_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/users/me").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_users_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/users/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_users_password_token_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/users/password-token").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"email":"admin@test.com"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_users_reset_password_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/users/reset-password").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"token":"tok","password":"new"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Price Lists ──────────────────────────────────────────────────────────────

#[test]
fn test_price_lists_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/price-lists").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_price_lists_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/price-lists").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_price_lists_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/price-lists").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"Sale","type":"sale","prices":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_price_lists_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/price-lists/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_price_lists_prices_batch_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/price-lists/{}/prices/batch", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"prices":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Inventory ────────────────────────────────────────────────────────────────

#[test]
fn test_inventory_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/inventory-items").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_inventory_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/inventory-items").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_inventory_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/inventory-items").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"sku":"SKU-001"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_inventory_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/inventory-items/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_inventory_location_levels_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/inventory-items/{}/location-levels", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Tax Rates ────────────────────────────────────────────────────────────────

#[test]
fn test_tax_rates_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/tax-rates").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_tax_rates_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/tax-rates").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_tax_rates_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/tax-rates").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"VAT","rate":20.0}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_tax_rates_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/tax-rates/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_tax_rates_delete_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/tax-rates/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Uploads ──────────────────────────────────────────────────────────────────

#[test]
fn test_uploads_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("POST").uri("/admin/uploads").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_uploads_delete_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("DELETE").uri("/admin/uploads").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"file_keys":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Return Reasons ───────────────────────────────────────────────────────────

#[test]
fn test_return_reasons_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/return-reasons").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_return_reasons_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/return-reasons").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_return_reasons_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/return-reasons").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"value":"wrong_item","label":"Wrong Item"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_return_reasons_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/return-reasons/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_return_reasons_delete_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/return-reasons/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Swaps ────────────────────────────────────────────────────────────────────

#[test]
fn test_swaps_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/swaps").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_swaps_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/swaps").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_swaps_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/swaps/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Draft Orders sub-routes ──────────────────────────────────────────────────

#[test]
fn test_draft_orders_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/draft-orders").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"email":"d@test.com"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_draft_orders_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/draft-orders/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_draft_orders_add_line_item_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/draft-orders/{}/line-items", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"quantity":1}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_draft_orders_pay_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/draft-orders/{}/pay", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Batch Jobs sub-routes ────────────────────────────────────────────────────

#[test]
fn test_batch_jobs_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/batch-jobs").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"type":"product-export","context":{}}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_batch_jobs_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/batch-jobs/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_batch_jobs_confirm_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/batch-jobs/{}/confirm", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_batch_jobs_cancel_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/batch-jobs/{}/cancel", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Order Edits sub-routes ───────────────────────────────────────────────────

#[test]
fn test_order_edits_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/order-edits").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"order_id": uuid::Uuid::new_v4()}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_order_edits_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/order-edits/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_order_edits_request_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/order-edits/{}/request", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_order_edits_confirm_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/order-edits/{}/confirm", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_order_edits_decline_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/order-edits/{}/decline", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_order_edits_cancel_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/order-edits/{}/cancel", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Promotions sub-routes ────────────────────────────────────────────────────

#[test]
fn test_promotions_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/promotions/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_promotions_rules_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/promotions/{}/rules", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"rules":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Notifications sub-routes ─────────────────────────────────────────────────

#[test]
fn test_notifications_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/notifications/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Sales Channels sub-routes ────────────────────────────────────────────────

#[test]
fn test_sales_channels_products_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/sales-channels/{}/products", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"product_ids":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Stock Locations sub-routes ───────────────────────────────────────────────

#[test]
fn test_stock_locations_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/stock-locations/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_stock_locations_fulfillment_sets_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/stock-locations/{}/fulfillment-sets", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"FS-1"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Stores sub-routes ────────────────────────────────────────────────────────

#[test]
fn test_stores_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/stores/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Customer Groups sub-routes ───────────────────────────────────────────────

#[test]
fn test_customer_groups_customers_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/customer-groups/{}/customers", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Tax Regions sub-routes ───────────────────────────────────────────────────

#[test]
fn test_tax_regions_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/tax-regions/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Shipping Profiles sub-routes ─────────────────────────────────────────────

#[test]
fn test_shipping_profiles_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/shipping-profiles/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Invites sub-routes ───────────────────────────────────────────────────────

#[test]
fn test_invites_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/invites").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"user_email":"new@test.com"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_invites_accept_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/invites/{}/accept", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"token":"tok"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── API Keys sub-routes ──────────────────────────────────────────────────────

#[test]
fn test_api_keys_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/api-keys").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"title":"My Key","type":"publishable"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}
