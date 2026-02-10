mod broadcaster;
mod db;
mod models;
mod routes;

use axum::{
    routing::{delete, get, post, put},
    Router,
    Json,
};
use serde::Serialize;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use broadcaster::Broadcaster;
use db::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub broadcaster: Broadcaster,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
    })
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database connection string from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Initialize database pool
    let db_pool = db::create_pool(&database_url)
        .await
        .expect("Failed to create database pool");

    // Create broadcaster for SSE
    let broadcaster = Broadcaster::new();

    // Start database listener in background
    tokio::spawn(db::start_listener(database_url.clone(), broadcaster.clone()));

    // Create shared application state
    let app_state = AppState {
        db_pool,
        broadcaster,
    };

    // Build application router
    let cors_origin = std::env::var("CORS_ORIGIN").unwrap_or_else(|_| "".to_string());
    
    // Check CORS_ORIGIN requirement
    if cors_origin.is_empty() {
        let rust_log = std::env::var("RUST_LOG").unwrap_or_default();

        if rust_log != "debug" {
            panic!("CORS_ORIGIN environment variable must be set in production mode");
        } else {
            tracing::warn!("CORS_ORIGIN is not set - allowing all origins in debug mode");
        }
    }
    
    let cors_layer = if cors_origin.is_empty() {
        CorsLayer::new()
            .allow_origin(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
            .allow_headers(tower_http::cors::Any)
    } else {
        CorsLayer::new()
            .allow_origin(cors_origin.parse::<axum::http::HeaderValue>().expect("Invalid CORS_ORIGIN"))
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST, axum::http::Method::PUT, axum::http::Method::DELETE])
            .allow_headers([axum::http::header::CONTENT_TYPE])
    };

    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Event routes
        .route("/api/events", get(routes::events::list_events))
        .route("/api/events", post(routes::events::create_event))
        .route("/api/events/{id}", get(routes::events::get_event))
        .route("/api/events/{id}", put(routes::events::update_event))
        .route("/api/events/{id}", delete(routes::events::delete_event))
        
        // Participant routes
        .route("/api/events/{event_id}/participants", get(routes::participants::list_participants))
        .route("/api/participants", post(routes::participants::create_participant))
        .route("/api/participants/{id}", get(routes::participants::get_participant))
        .route("/api/participants/{id}", put(routes::participants::update_participant_status))
        .route("/api/participants/{id}", delete(routes::participants::delete_participant))
        
        // SSE stream endpoint
        .route("/api/events/stream", get(routes::sse::event_stream))
        
        // Add CORS middleware
        .layer(cors_layer)
        
        // Add state
        .with_state(app_state);

    // Start server
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
