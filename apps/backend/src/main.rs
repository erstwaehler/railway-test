use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use backend::{AppState, health_check};
use backend::{broadcaster::Broadcaster, cache::AppCache, db, routes};

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

    // Data directory from environment (writable filesystem path)
    let data_dir = std::env::var("DATA_DIR")
        .unwrap_or_else(|_| "/run/media".to_string());

    // Ensure data directory exists
    std::fs::create_dir_all(&data_dir)
        .expect("Failed to create data directory");

    let db_path = format!("{}/data.db", data_dir);

    // Initialize SQLite database pool
    let db_pool = db::create_pool(&db_path)
        .await
        .expect("Failed to create database pool");

    // Initialize database tables
    db::initialize_tables(&db_pool)
        .await
        .expect("Failed to initialize database tables");

    // Create in-memory cache with TTL
    let cache_ttl: u64 = std::env::var("CACHE_TTL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);

    let cache = AppCache::new(cache_ttl);

    // Create broadcaster for SSE
    let broadcaster = Broadcaster::new();

    // Start notification poller for cross-instance sync
    let last_id = Arc::new(Mutex::new(
        db::get_max_notification_id(&db_pool).await,
    ));
    tokio::spawn(db::start_notification_poller(
        db_pool.clone(),
        broadcaster.clone(),
        cache.clone(),
        last_id,
    ));

    // Create shared application state
    let app_state = AppState {
        db_pool,
        broadcaster,
        cache,
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
        
        // SSE stream endpoint (static route must be before :id param to avoid matchit capture)
        .route("/api/events/stream", get(routes::sse::event_stream))
        
        // Event routes
        .route("/api/events", get(routes::events::list_events).post(routes::events::create_event))
        .route("/api/events/:id", get(routes::events::get_event).put(routes::events::update_event).delete(routes::events::delete_event))
        
        // Participant routes
        .route("/api/events/:id/participants", get(routes::participants::list_participants))
        .route("/api/participants", post(routes::participants::create_participant))
        .route("/api/participants/:id", get(routes::participants::get_participant).put(routes::participants::update_participant_status).delete(routes::participants::delete_participant))
        
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
