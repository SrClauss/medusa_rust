//! Admin plugins handlers — stub
use axum::{extract::{Query, State}, Json};
use serde::Deserialize;
use crate::{error::AppError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct ListParams {
    #[serde(default = "d20")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}
fn d20() -> i64 { 20 }

pub async fn list(
    State(state): State<AppState>,
    Query(p): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pm = state.plugin_mgr.lock().await;
    let list: Vec<&dyn crate::plugins::Plugin> = pm.list();
    let items: Vec<serde_json::Value> = list
        .iter()
        .map(|p| serde_json::json!({"id": p.id(), "kind": format!("{}", p.kind())}))
        .collect();

    Ok(Json(serde_json::json!({
        "plugins": items,
        "count": items.len(),
        "offset": p.offset,
        "limit": p.limit,
    })))
}
