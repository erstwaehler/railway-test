use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::models::{Event, CreateEvent};

// Type alias for our app state
type AppState = crate::AppState;

/// List all events
pub async fn list_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<Event>>, (StatusCode, Json<serde_json::Value>)> {
    let events = sqlx::query_as::<_, Event>(
        "SELECT id, title, description, start_time, end_time, location, max_participants, created_at, updated_at 
         FROM events 
         ORDER BY start_time DESC"
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to fetch events: {}", e) })),
        )
    })?;

    Ok(Json(events))
}

/// Get a single event by ID
pub async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Event>, (StatusCode, Json<serde_json::Value>)> {
    let event = sqlx::query_as::<_, Event>(
        "SELECT id, title, description, start_time, end_time, location, max_participants, created_at, updated_at 
         FROM events 
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Event not found" })),
        )
    })?;

    Ok(Json(event))
}

/// Create a new event
pub async fn create_event(
    State(state): State<AppState>,
    Json(payload): Json<CreateEvent>,
) -> Result<(StatusCode, Json<Event>), (StatusCode, Json<serde_json::Value>)> {
    let event = sqlx::query_as::<_, Event>(
        "INSERT INTO events (title, description, start_time, end_time, location, max_participants) 
         VALUES ($1, $2, $3, $4, $5, $6) 
         RETURNING id, title, description, start_time, end_time, location, max_participants, created_at, updated_at"
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.start_time)
    .bind(&payload.end_time)
    .bind(&payload.location)
    .bind(&payload.max_participants)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to create event: {}", e) })),
        )
    })?;

    Ok((StatusCode::CREATED, Json(event)))
}

/// Update an event
pub async fn update_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<CreateEvent>,
) -> Result<Json<Event>, (StatusCode, Json<serde_json::Value>)> {
    let event = sqlx::query_as::<_, Event>(
        "UPDATE events 
         SET title = $1, description = $2, start_time = $3, end_time = $4, location = $5, max_participants = $6, updated_at = NOW()
         WHERE id = $7
         RETURNING id, title, description, start_time, end_time, location, max_participants, created_at, updated_at"
    )
    .bind(&payload.title)
    .bind(&payload.description)
    .bind(&payload.start_time)
    .bind(&payload.end_time)
    .bind(&payload.location)
    .bind(&payload.max_participants)
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to update event: {}", e) })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Event not found" })),
        )
    })?;

    Ok(Json(event))
}

/// Delete an event
pub async fn delete_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query("DELETE FROM events WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to delete event: {}", e) })),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Event not found" })),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
