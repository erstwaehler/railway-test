pub mod broadcaster;
pub mod cache;
pub mod db;
pub mod models;
pub mod routes;

use axum::Json;
use serde::Serialize;

use cache::AppCache;
use db::DbPool;
use broadcaster::Broadcaster;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub broadcaster: Broadcaster,
    pub cache: AppCache,
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
    })
}
