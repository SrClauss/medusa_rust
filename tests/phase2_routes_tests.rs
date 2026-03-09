// Tests for Phase 1-4 new admin routes (price_preferences, campaigns,
// fulfillment_sets, claims, exchanges, returns extended, workflow_executions,
// notifications resend, fulfillment_providers, payment_collections,
// refund_reasons, reservations, product_tags, product_types, payments,
// plugins, shipping_option_types, feature_flags).
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
    }
}

fn app() -> Router {
    medusa_rust::api::build_router(make_state())
}

fn admin_token() -> String {
    medusa_rust::auth::jwt::encode_admin_token(&uuid::Uuid::new_v4(), "admin@test.com", "secret", 24).unwrap()
}

// ─── Price Preferences ────────────────────────────────────────────────────────

#[test]
fn test_price_preferences_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/price-preferences").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_price_preferences_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/price-preferences").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"attribute":"currency_code","value":"usd","is_tax_inclusive":false}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_price_preferences_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/price-preferences/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_price_preferences_delete() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/price-preferences/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Campaigns ────────────────────────────────────────────────────────────────

#[test]
fn test_campaigns_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/campaigns").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_campaigns_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/campaigns").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"Summer Sale","campaign_identifier":"SUMMER"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_campaigns_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/campaigns/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_campaigns_delete() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/campaigns/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Fulfillment Sets ─────────────────────────────────────────────────────────

#[test]
fn test_fulfillment_sets_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/fulfillment-sets/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_fulfillment_sets_delete() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/fulfillment-sets/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_fulfillment_sets_create_service_zone() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/fulfillment-sets/{}/service-zones", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"name":"Zone A"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Claims ───────────────────────────────────────────────────────────────────

#[test]
fn test_claims_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/claims").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_claims_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/claims/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_claims_cancel() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/claims/{}/cancel", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_claims_confirm() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/claims/{}/confirm", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Exchanges ────────────────────────────────────────────────────────────────

#[test]
fn test_exchanges_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/exchanges").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_exchanges_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/exchanges").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"order_id": uuid::Uuid::new_v4()}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_exchanges_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/exchanges/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_exchanges_cancel() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/exchanges/{}/cancel", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Returns (extended) ───────────────────────────────────────────────────────

#[test]
fn test_returns_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/returns/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_returns_cancel() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/returns/{}/cancel", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_returns_receive_items() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/returns/{}/receive-items", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_returns_receive_confirm() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/returns/{}/receive/confirm", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_returns_request() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/returns/{}/request", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Workflow Executions ──────────────────────────────────────────────────────

#[test]
fn test_workflow_executions_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/workflows-executions").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_workflow_executions_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/workflows-executions/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_workflow_executions_run() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/workflows-executions/my-workflow/run").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"input":{}}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Notifications (resend) ───────────────────────────────────────────────────

#[test]
fn test_notifications_resend() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/notifications/{}/resend", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Fulfillment Providers ────────────────────────────────────────────────────

#[test]
fn test_fulfillment_providers_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/fulfillment-providers").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_fulfillment_providers_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/fulfillment-providers/manual").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Payment Collections (admin) ──────────────────────────────────────────────

#[test]
fn test_payment_collections_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/payment-collections").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_payment_collections_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/payment-collections").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"type":"order_edit","amount":1000,"currency_code":"usd"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_payment_collections_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/payment-collections/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_payment_collections_delete() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/payment-collections/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Refund Reasons ───────────────────────────────────────────────────────────

#[test]
fn test_refund_reasons_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/refund-reasons").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_refund_reasons_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/refund-reasons").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"label":"Wrong item"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_refund_reasons_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/refund-reasons/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_refund_reasons_delete() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/refund-reasons/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Reservations ─────────────────────────────────────────────────────────────

#[test]
fn test_reservations_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/reservations").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_reservations_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/reservations").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"inventory_item_id": uuid::Uuid::new_v4(), "location_id": uuid::Uuid::new_v4(), "quantity": 5}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_reservations_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/reservations/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_reservations_delete() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/reservations/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Product Tags (admin) ─────────────────────────────────────────────────────

#[test]
fn test_admin_product_tags_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/product-tags").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_admin_product_tags_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/product-tags").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"value":"Summer"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_admin_product_tags_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/product-tags/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_admin_product_tags_delete() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/product-tags/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Product Types (admin) ────────────────────────────────────────────────────

#[test]
fn test_admin_product_types_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/product-types").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_admin_product_types_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/product-types").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"value":"Physical"}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_admin_product_types_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/product-types/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_admin_product_types_delete() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("DELETE").uri(format!("/admin/product-types/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Payments (admin) ─────────────────────────────────────────────────────────

#[test]
fn test_payments_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/payments").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_payments_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/payments/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_payments_capture() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/payments/{}/capture", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from("{}")).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// ─── Plugins ──────────────────────────────────────────────────────────────────

#[test]
fn test_plugins_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/plugins").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Shipping Option Types ────────────────────────────────────────────────────

#[test]
fn test_shipping_option_types_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/shipping-option-types").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

#[test]
fn test_shipping_option_types_create() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("POST").uri("/admin/shipping-option-types").header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"label":"Flat Rate","code":"FLAT","description":""}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_shipping_option_types_get() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("GET").uri(format!("/admin/shipping-option-types/{}", id)).header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Feature Flags ────────────────────────────────────────────────────────────

#[test]
fn test_feature_flags_list() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder().method("GET").uri("/admin/feature-flags").header("authorization", format!("Bearer {}", token)).body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// ─── Promotions (buy-rules / target-rules batch) ──────────────────────────────

#[test]
fn test_promotions_buy_rules_batch() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/promotions/{}/buy-rules/batch", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"rules":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

#[test]
fn test_promotions_target_rules_batch() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let id = uuid::Uuid::new_v4();
        let req = Request::builder().method("POST").uri(format!("/admin/promotions/{}/target-rules/batch", id)).header("authorization", format!("Bearer {}", token)).header("content-type", "application/json").body(Body::from(json!({"rules":[]}).to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}
