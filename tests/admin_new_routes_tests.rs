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

// ─── Sales Channels ───────────────────────────────────────────────────────────

#[test]
fn test_sales_channels_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/sales-channels").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_sales_channels_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/sales-channels").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_sales_channels_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/sales-channels").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"Test"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_sales_channels_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/sales-channels/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Stock Locations ──────────────────────────────────────────────────────────

#[test]
fn test_stock_locations_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/stock-locations").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_stock_locations_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/stock-locations").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_stock_locations_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/stock-locations").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"WH-1"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Stores ───────────────────────────────────────────────────────────────────

#[test]
fn test_stores_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/stores").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_stores_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/stores").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Customer Groups ──────────────────────────────────────────────────────────

#[test]
fn test_customer_groups_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/customer-groups").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_customer_groups_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/customer-groups").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_customer_groups_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/customer-groups").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"VIP"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── API Keys ─────────────────────────────────────────────────────────────────

#[test]
fn test_api_keys_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/api-keys").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_api_keys_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/api-keys").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_api_keys_revoke_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/api-keys/{}/revoke", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Invites ──────────────────────────────────────────────────────────────────

#[test]
fn test_invites_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/invites").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_invites_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/invites").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Tax Providers ────────────────────────────────────────────────────────────

#[test]
fn test_tax_providers_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/tax-providers").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Tax Regions ──────────────────────────────────────────────────────────────

#[test]
fn test_tax_regions_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/tax-regions").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_tax_regions_create_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/tax-regions").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"country_code":"US"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Shipping Profiles ────────────────────────────────────────────────────────

#[test]
fn test_shipping_profiles_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/admin/shipping-profiles").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

#[test]
fn test_shipping_profiles_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/shipping-profiles").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Payment Providers (admin) ────────────────────────────────────────────────

#[test]
fn test_admin_payment_providers_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/payment-providers").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Store: Locales ───────────────────────────────────────────────────────────

#[test]
fn test_store_locales_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/locales").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store: Payment Collections ───────────────────────────────────────────────

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

// ─── Store: Store Credit Accounts ─────────────────────────────────────────────

#[test]
fn test_store_credits_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder().method("GET").uri("/store/store-credit-accounts").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store: Cart Gift Cards, Promotions, Store Credits ────────────────────────

#[test]
fn test_store_cart_add_gift_card_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let cart_id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/gift-cards", cart_id)).header("content-type", "application/json").body(Body::from(json!({"code":"GC-1234"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_cart_add_promotion_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let cart_id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/carts/{}/promotions", cart_id)).header("content-type", "application/json").body(Body::from(json!({"code":"PROMO10"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store: Order Transfers ───────────────────────────────────────────────────

#[test]
fn test_store_order_transfer_request_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let order_id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/orders/{}/transfer/request", order_id)).header("content-type", "application/json").body(Body::from(json!({"email":"test@test.com"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_store_order_transfer_accept_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let order_id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/orders/{}/transfer/accept", order_id)).header("content-type", "application/json").body(Body::from(json!({"token":"abc"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Store: Shipping Option Calculate ─────────────────────────────────────────

#[test]
fn test_store_shipping_option_calculate_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/store/shipping-options/{}/calculate", id)).header("content-type", "application/json").body(Body::from(json!({"cart_id": uuid::Uuid::new_v4()}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}
