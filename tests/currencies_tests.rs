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

fn admin_token() -> String {
    medusa_rust::auth::jwt::encode_admin_token(&uuid::Uuid::new_v4(), "admin@test.com", "secret", 24).unwrap()
}

// GET /admin/currencies — requires admin auth
#[test]
fn test_currencies_list_requires_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/currencies")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    });
}

// GET /admin/currencies — with valid admin token, routes to handler
#[test]
fn test_currencies_list_with_auth() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/currencies")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
    });
}

// GET /admin/currencies/{code} — routes to get handler
#[test]
fn test_currencies_get_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("GET")
            .uri("/admin/currencies/usd")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}

// PUT /admin/currencies/{code} — routes to update handler
#[test]
fn test_currencies_update_routes() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let app = app();
        let token = admin_token();
        let req = Request::builder()
            .method("PUT")
            .uri("/admin/currencies/usd")
            .header("authorization", format!("Bearer {}", token))
            .header("content-type", "application/json")
            .body(Body::from(json!({"includes_tax": true}).to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_ne!(resp.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(resp.status(), StatusCode::NOT_FOUND);
        assert_ne!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    });
}
