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
        plugin_mgr: Arc::new(tokio::sync::Mutex::new(medusa_rust::plugins::PluginManager::new())),
        event_bus: Arc::new(medusa_rust::EventBus::new()),
    }
}

fn app() -> Router {
    medusa_rust::api::build_router(make_state())
}

fn customer_token() -> String {
    medusa_rust::auth::jwt::encode_store_token(&uuid::Uuid::new_v4(), "customer@test.com", "secret", 24).unwrap()
}

// ─── Store Products ────────────────────────────────────────────────────────────

#[test]
fn test_store_products_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/products").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_products_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/products/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Collections ────────────────────────────────────────────────────────

#[test]
fn test_store_collections_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/collections").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_collections_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/collections/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Product Categories ─────────────────────────────────────────────────

#[test]
fn test_store_categories_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/product-categories").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_categories_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/product-categories/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Product Tags ───────────────────────────────────────────────────────

#[test]
fn test_store_product_tags_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/product-tags").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_product_tags_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/product-tags/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Product Types ──────────────────────────────────────────────────────

#[test]
fn test_store_product_types_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/product-types").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_product_types_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/product-types/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Regions ────────────────────────────────────────────────────────────

#[test]
fn test_store_regions_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/regions").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_regions_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/regions/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Return Reasons ─────────────────────────────────────────────────────

#[test]
fn test_store_return_reasons_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/return-reasons").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_return_reasons_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/return-reasons/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Payment Providers ──────────────────────────────────────────────────

#[test]
fn test_store_payment_providers_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/payment-providers").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store Store Credit Accounts ──────────────────────────────────────────────

#[test]
fn test_store_credits_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/store-credit-accounts/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Carts ──────────────────────────────────────────────────────────────

#[test]
fn test_store_carts_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("POST").uri("/store/carts").header("content-type", "application/json").body(Body::from(json!({"region_id": uuid::Uuid::new_v4()}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/carts/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_store_carts_update() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}", id)).header("content-type", "application/json").body(Body::from(json!({"region_id": uuid::Uuid::new_v4()}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_add_line_item() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/line-items", id)).header("content-type", "application/json").body(Body::from(json!({"variant_id": uuid::Uuid::new_v4(), "quantity": 1}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_update_line_item() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let line_id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/line-items/{}", id, line_id)).header("content-type", "application/json").body(Body::from(json!({"quantity": 2}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_delete_line_item() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let line_id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/store/carts/{}/line-items/{}", id, line_id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_create_payment_sessions() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/payment-sessions", id)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_select_payment_session() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/payment-session", id)).header("content-type", "application/json").body(Body::from(json!({"provider_id":"stripe"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_delete_payment_session() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/store/carts/{}/payment-sessions/stripe", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_refresh_payment_session() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/payment-sessions/stripe/refresh", id)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_add_shipping_method() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/shipping-methods", id)).header("content-type", "application/json").body(Body::from(json!({"option_id": uuid::Uuid::new_v4()}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_complete() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/complete", id)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_calculate_taxes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/taxes", id)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_apply_discount() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/discounts/SAVE10", id)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_carts_add_store_credit() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/store-credits", id)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store Customers ──────────────────────────────────────────────────────────

#[test]
fn test_store_customers_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("POST").uri("/store/customers").header("content-type", "application/json").body(Body::from(json!({"email":"c@test.com","password":"pass123"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_customers_password_token() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("POST").uri("/store/customers/password-token").header("content-type", "application/json").body(Body::from(json!({"email":"c@test.com"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_customers_password_reset() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("POST").uri("/store/customers/password-reset").header("content-type", "application/json").body(Body::from(json!({"token":"tok","email":"c@test.com","password":"new"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_customers_me_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/customers/me").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_store_customers_me_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = customer_token();
        let req = Request::builder().method("GET").uri("/store/customers/me").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_store_customers_orders_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = customer_token();
        let req = Request::builder().method("GET").uri("/store/customers/me/orders").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_store_customers_addresses_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = customer_token();
        let req = Request::builder().method("GET").uri("/store/customers/me/addresses").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_store_customers_add_address_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = customer_token();
        let req = Request::builder().method("POST").uri("/store/customers/me/addresses").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"address":{"address_1":"123 Main St","country_code":"US"}}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_customers_payment_methods_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = customer_token();
        let req = Request::builder().method("GET").uri("/store/customers/me/payment-methods").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Orders ─────────────────────────────────────────────────────────────

#[test]
fn test_store_orders_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/orders").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_orders_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/orders/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_store_orders_transfer_cancel() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/orders/{}/transfer/cancel", id)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_orders_transfer_decline() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/orders/{}/transfer/decline", id)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store Swaps ──────────────────────────────────────────────────────────────

#[test]
fn test_store_swaps_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("POST").uri("/store/swaps").header("content-type", "application/json").body(Body::from(json!({"order_id": uuid::Uuid::new_v4(), "return_items": []}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_swaps_get_by_cart() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let cart_id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/swaps/{}", cart_id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Returns ────────────────────────────────────────────────────────────

#[test]
fn test_store_returns_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("POST").uri("/store/returns").header("content-type", "application/json").body(Body::from(json!({"order_id": uuid::Uuid::new_v4(), "items": []}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store Shipping Options ───────────────────────────────────────────────────

#[test]
fn test_store_shipping_options_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/shipping-options").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_shipping_options_get_for_cart() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let cart_id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/shipping-options/{}", cart_id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store Payment Collections ────────────────────────────────────────────────

#[test]
fn test_store_payment_collections_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("POST").uri("/store/payment-collections").header("content-type", "application/json").body(Body::from(json!({"cart_id": uuid::Uuid::new_v4()}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_payment_collections_add_session() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/payment-collections/{}/payment-sessions", id)).header("content-type", "application/json").body(Body::from(json!({"provider_id":"stripe"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store Auth ───────────────────────────────────────────────────────────────

#[test]
fn test_store_auth_login() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("POST").uri("/store/auth").header("content-type", "application/json").body(Body::from(json!({"email":"c@test.com","password":"pass"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_auth_get_session() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/auth").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_auth_logout() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("DELETE").uri("/store/auth").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store Gift Cards ─────────────────────────────────────────────────────────

#[test]
fn test_store_gift_cards_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/store/gift-cards/{}", id)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}
