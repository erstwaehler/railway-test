use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::db;
use crate::models::{Event, CreateEvent};

// Type alias for our app state
type AppState = crate::AppState;

/// List all events
pub async fn list_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<serde_json::Value>)> {
    // Check cache first
    if let Some(events) = state.cache.events_list.get("all").await {
        return Ok(Json(events));
    }

    let events = sqlx::query_as::<_, Event>(
        "SELECT id, title, description, start_time, end_time, location, max_participants, created_at, updated_at 
         FROM events 
         ORDER BY start_time DESC"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch events: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?;

    // Populate cache
    state.cache.events_list.insert("all".to_string(), events.clone()).await;

    Ok(Json(events))
}

/// Get a single event by ID
pub async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Event>, (StatusCode, Json<serde_json::Value>)> {
    let id_str = id.to_string();

    // Check cache first
    if let Some(event) = state.cache.event.get(&id_str).await {
        return Ok(Json(event));
    }

    let event = sqlx::query_as::<_, Event>(
        "SELECT id, title, description, start_time, end_time, location, max_participants, created_at, updated_at 
         FROM events 
         WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching event: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Event not found" })),
        )
    })?;

    // Populate cache
    state.cache.event.insert(id_str, event.clone()).await;

    Ok(Json(event))
}

/// Create a new event
pub async fn create_event(
    State(state): State<AppState>,
    Json(payload): Json<CreateEvent>,
) -> Result<(StatusCode, Json<Event>), (StatusCode, Json<serde_json::Value>)> {
    if payload.end_time <= payload.start_time {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "end_time must be after start_time" })),
        ));
    }

    if let Some(max) = payload.max_participants {
        if max <= 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "max_participants must be greater than 0" })),
            ));
        }
    }

    if payload.title.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "title is required" })),
        ));
    }

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let event = sqlx::query_as::<_, Event>(
        "INSERT INTO events (id, title, description, start_time, end_time, location, max_participants, created_at, updated_at) 
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) 
         RETURNING id, title, description, start_time, end_time, location, max_participants, created_at, updated_at"
    )
    .bind(id)
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.start_time)
    .bind(&payload.end_time)
    .bind(&payload.location)
    .bind(&payload.max_participants)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        if let Some(db_error) = e.as_database_error() {
            if db_error.message().contains("CHECK constraint failed") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Invalid event values" })),
                );
            }
        }
        tracing::error!("Failed to create event: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?;

    // Invalidate cache and notify other instances
    state.cache.invalidate_events().await;
    let notification_payload = json!({
        "operation": "INSERT",
        "table": "events",
        "id": event.id,
        "timestamp": chrono::Utc::now()
    }).to_string();
    db::insert_notification(&state.db_pool, "event_changes", &notification_payload).await;

    Ok((StatusCode::CREATED, Json(event)))
}

/// Update an event
pub async fn update_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateEvent>,
) -> Result<Json<Event>, (StatusCode, Json<serde_json::Value>)> {
    if payload.end_time <= payload.start_time {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "end_time must be after start_time" })),
        ));
    }

    if let Some(max) = payload.max_participants {
        if max <= 0 {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "max_participants must be greater than 0" })),
            ));
        }
    }

    if payload.title.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "title is required" })),
        ));
    }

    let now = chrono::Utc::now();

    let event = sqlx::query_as::<_, Event>(
        "UPDATE events 
         SET title = ?, description = ?, start_time = ?, end_time = ?, location = ?, max_participants = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, title, description, start_time, end_time, location, max_participants, created_at, updated_at"
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.start_time)
    .bind(&payload.end_time)
    .bind(&payload.location)
    .bind(&payload.max_participants)
    .bind(now)
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        if let Some(db_error) = e.as_database_error() {
            if db_error.message().contains("CHECK constraint failed") {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Invalid event values" })),
                );
            }
        }
        tracing::error!("Failed to update event: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Event not found" })),
        )
    })?;

    // Invalidate cache and notify other instances
    state.cache.invalidate_event(&id.to_string()).await;
    let notification_payload = json!({
        "operation": "UPDATE",
        "table": "events",
        "id": event.id,
        "timestamp": chrono::Utc::now()
    }).to_string();
    db::insert_notification(&state.db_pool, "event_changes", &notification_payload).await;

    Ok(Json(event))
}

/// Delete an event
pub async fn delete_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query("DELETE FROM events WHERE id = ?")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete event: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Event not found" })),
        ));
    }

    // Invalidate cache and notify other instances
    state.cache.invalidate_event(&id.to_string()).await;
    state.cache.invalidate_participants().await;
    let notification_payload = json!({
        "operation": "DELETE",
        "table": "events",
        "id": id,
        "timestamp": chrono::Utc::now()
    }).to_string();
    db::insert_notification(&state.db_pool, "event_changes", &notification_payload).await;

    Ok(StatusCode::NO_CONTENT)
}
