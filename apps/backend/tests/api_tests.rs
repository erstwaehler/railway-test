use axum::{
    body::Body,
    http::{Request, StatusCode, Method},
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};
use tower::ServiceExt;

// Import from the backend crate
use backend::{AppState, cache::AppCache, db, broadcaster::Broadcaster};

/// Helper to create a test app state with a temporary SQLite database
async fn create_test_state() -> (AppState, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let db_pool = db::create_pool(db_path_str).await.unwrap();
    db::initialize_tables(&db_pool).await.unwrap();

    let cache = AppCache::new(60);
    let broadcaster = Broadcaster::new();

    let state = AppState {
        db_pool,
        broadcaster,
        cache,
    };

    (state, dir)
}

/// Helper to build the test router (same as main.rs)
fn build_app(state: AppState) -> Router {
    use backend::routes;

    Router::new()
        .route("/health", get(backend::health_check))
        .route("/api/events/stream", get(routes::sse::event_stream))
        .route("/api/events", get(routes::events::list_events).post(routes::events::create_event))
        .route("/api/events/:id", get(routes::events::get_event).put(routes::events::update_event).delete(routes::events::delete_event))
        .route("/api/events/:id/participants", get(routes::participants::list_participants))
        .route("/api/participants", post(routes::participants::create_participant))
        .route("/api/participants/:id", get(routes::participants::get_participant).put(routes::participants::update_participant_status).delete(routes::participants::delete_participant))
        .with_state(state)
}

/// Helper to extract JSON body from response
async fn body_json(response: axum::http::Response<Body>) -> Value {
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}

// =====================
// Health Check Tests
// =====================

#[tokio::test]
async fn test_health_check() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["status"], "healthy");
    assert!(body["timestamp"].is_string());
}

// =====================
// Event CRUD Tests
// =====================

#[tokio::test]
async fn test_create_and_get_event() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    // Create event
    let create_body = json!({
        "title": "Test Event",
        "description": "A test event",
        "start_time": "2026-03-01T10:00:00Z",
        "end_time": "2026-03-01T12:00:00Z",
        "location": "Room 1",
        "max_participants": 10
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(create_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let event = body_json(response).await;
    assert_eq!(event["title"], "Test Event");
    assert_eq!(event["description"], "A test event");
    assert_eq!(event["max_participants"], 10);

    let event_id = event["id"].as_str().unwrap();

    // Get event
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/events/{}", event_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let fetched = body_json(response).await;
    assert_eq!(fetched["id"], event_id);
    assert_eq!(fetched["title"], "Test Event");
}

#[tokio::test]
async fn test_list_events() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    // Create two events
    for title in &["Event A", "Event B"] {
        let body = json!({
            "title": title,
            "start_time": "2026-03-01T10:00:00Z",
            "end_time": "2026-03-01T12:00:00Z"
        });

        app.clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/events")
                    .header("Content-Type", "application/json")
                    .body(Body::from(body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
    }

    // List events
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let events = body_json(response).await;
    assert_eq!(events.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_update_event() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    // Create event
    let create_body = json!({
        "title": "Original",
        "start_time": "2026-03-01T10:00:00Z",
        "end_time": "2026-03-01T12:00:00Z"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(create_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let event = body_json(response).await;
    let event_id = event["id"].as_str().unwrap();

    // Update event
    let update_body = json!({
        "title": "Updated Title",
        "start_time": "2026-03-01T10:00:00Z",
        "end_time": "2026-03-01T14:00:00Z",
        "location": "New Room"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/api/events/{}", event_id))
                .header("Content-Type", "application/json")
                .body(Body::from(update_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated = body_json(response).await;
    assert_eq!(updated["title"], "Updated Title");
    assert_eq!(updated["location"], "New Room");
}

#[tokio::test]
async fn test_delete_event() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    // Create event
    let create_body = json!({
        "title": "To Delete",
        "start_time": "2026-03-01T10:00:00Z",
        "end_time": "2026-03-01T12:00:00Z"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(create_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let event = body_json(response).await;
    let event_id = event["id"].as_str().unwrap();

    // Delete event
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/events/{}", event_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/events/{}", event_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// =====================
// Event Validation Tests
// =====================

#[tokio::test]
async fn test_create_event_invalid_time_range() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    let body = json!({
        "title": "Bad Event",
        "start_time": "2026-03-01T14:00:00Z",
        "end_time": "2026-03-01T10:00:00Z"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_event_empty_title() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    let body = json!({
        "title": "  ",
        "start_time": "2026-03-01T10:00:00Z",
        "end_time": "2026-03-01T12:00:00Z"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_nonexistent_event() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/events/00000000-0000-0000-0000-000000000000")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

// =====================
// Participant Tests
// =====================

#[tokio::test]
async fn test_create_and_list_participants() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    // Create event
    let event_body = json!({
        "title": "Event",
        "start_time": "2026-03-01T10:00:00Z",
        "end_time": "2026-03-01T12:00:00Z"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(event_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let event = body_json(response).await;
    let event_id = event["id"].as_str().unwrap();

    // Create participant
    let part_body = json!({
        "event_id": event_id,
        "name": "John Doe",
        "email": "john@example.com"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/participants")
                .header("Content-Type", "application/json")
                .body(Body::from(part_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let participant = body_json(response).await;
    assert_eq!(participant["name"], "John Doe");
    assert_eq!(participant["status"], "registered");

    // List participants
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/events/{}/participants", event_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let participants = body_json(response).await;
    assert_eq!(participants.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_update_participant_status() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    // Create event
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": "Event",
                    "start_time": "2026-03-01T10:00:00Z",
                    "end_time": "2026-03-01T12:00:00Z"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let event = body_json(response).await;
    let event_id = event["id"].as_str().unwrap();

    // Create participant
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/participants")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "event_id": event_id,
                    "name": "Jane",
                    "email": "jane@test.com"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let participant = body_json(response).await;
    let part_id = participant["id"].as_str().unwrap();

    // Update status
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::PUT)
                .uri(format!("/api/participants/{}", part_id))
                .header("Content-Type", "application/json")
                .body(Body::from(json!({"status": "confirmed"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let updated = body_json(response).await;
    assert_eq!(updated["status"], "confirmed");
}

#[tokio::test]
async fn test_duplicate_participant_rejected() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    // Create event
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": "Event",
                    "start_time": "2026-03-01T10:00:00Z",
                    "end_time": "2026-03-01T12:00:00Z"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let event = body_json(response).await;
    let event_id = event["id"].as_str().unwrap();

    let part_body = json!({
        "event_id": event_id,
        "name": "John",
        "email": "john@test.com"
    });

    // First registration
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/participants")
                .header("Content-Type", "application/json")
                .body(Body::from(part_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Duplicate registration
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/participants")
                .header("Content-Type", "application/json")
                .body(Body::from(part_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_event_full_rejected() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    // Create event with max_participants = 1
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": "Small Event",
                    "start_time": "2026-03-01T10:00:00Z",
                    "end_time": "2026-03-01T12:00:00Z",
                    "max_participants": 1
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let event = body_json(response).await;
    let event_id = event["id"].as_str().unwrap();

    // First participant
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/participants")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "event_id": event_id,
                    "name": "Alice",
                    "email": "alice@test.com"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Second participant should fail (event full)
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/participants")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "event_id": event_id,
                    "name": "Bob",
                    "email": "bob@test.com"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

// =====================
// Cache Tests
// =====================

#[tokio::test]
async fn test_cache_is_populated_on_read() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state.clone());

    // Create event
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": "Cached Event",
                    "start_time": "2026-03-01T10:00:00Z",
                    "end_time": "2026-03-01T12:00:00Z"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let event = body_json(response).await;
    let event_id = event["id"].as_str().unwrap();

    // First GET (populates cache)
    app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/events/{}", event_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Verify cache is populated
    assert!(state.cache.event.get(&event_id.to_string()).await.is_some());
}

#[tokio::test]
async fn test_cache_invalidated_on_write() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state.clone());

    // Create and list events (populates list cache)
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": "Event 1",
                    "start_time": "2026-03-01T10:00:00Z",
                    "end_time": "2026-03-01T12:00:00Z"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // List to populate cache
    app.clone()
        .oneshot(
            Request::builder()
                .uri("/api/events")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(state.cache.events_list.get("all").await.is_some());

    // Create another event (should invalidate list cache)
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": "Event 2",
                    "start_time": "2026-03-01T10:00:00Z",
                    "end_time": "2026-03-01T12:00:00Z"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Cache should be invalidated
    assert!(state.cache.events_list.get("all").await.is_none());
}

// =====================
// Database Tests
// =====================

#[tokio::test]
async fn test_db_initialization() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("init_test.db");
    let db_path_str = db_path.to_str().unwrap();

    let pool = db::create_pool(db_path_str).await.unwrap();
    db::initialize_tables(&pool).await.unwrap();

    // Tables should exist
    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('events', 'participants', 'change_notifications')")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 3);
}

#[tokio::test]
async fn test_notification_insert_and_poll() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("notify_test.db");
    let db_path_str = db_path.to_str().unwrap();

    let pool = db::create_pool(db_path_str).await.unwrap();
    db::initialize_tables(&pool).await.unwrap();

    // Insert notification
    db::insert_notification(&pool, "event_changes", "{\"test\": true}")
        .await
        .unwrap();

    // Check it was stored
    let max_id = db::get_max_notification_id(&pool).await;
    assert!(max_id > 0);

    // Poll and verify
    let notifications: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT id, channel, payload FROM change_notifications WHERE id > 0"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].1, "event_changes");
}

// =====================
// Delete Participant Tests
// =====================

#[tokio::test]
async fn test_delete_participant() {
    let (state, _temp_dir) = create_test_state().await;
    let app = build_app(state);

    // Create event
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/events")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "title": "Event",
                    "start_time": "2026-03-01T10:00:00Z",
                    "end_time": "2026-03-01T12:00:00Z"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let event = body_json(response).await;
    let event_id = event["id"].as_str().unwrap();

    // Create participant
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/api/participants")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({
                    "event_id": event_id,
                    "name": "Jane",
                    "email": "jane@test.com"
                }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let participant = body_json(response).await;
    let part_id = participant["id"].as_str().unwrap();

    // Delete participant
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::DELETE)
                .uri(format!("/api/participants/{}", part_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify deleted
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/participants/{}", part_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
