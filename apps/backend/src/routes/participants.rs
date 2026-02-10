use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use sqlx::postgres::PgDatabaseError;
use uuid::Uuid;

use crate::models::{Participant, CreateParticipant, UpdateParticipantStatus};

// Type alias for our app state
type AppState = crate::AppState;

/// List all participants for an event
pub async fn list_participants(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<Vec<Participant>>, (StatusCode, Json<serde_json::Value>)> {
    let participants = sqlx::query_as::<_, Participant>(
        "SELECT id, event_id, name, email, status, registered_at, updated_at 
         FROM participants 
         WHERE event_id = $1 
         ORDER BY registered_at ASC"
    )
    .bind(event_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to fetch participants: {}", e) })),
        )
    })?;

    Ok(Json(participants))
}

/// Get a single participant by ID
pub async fn get_participant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Participant>, (StatusCode, Json<serde_json::Value>)> {
    let participant = sqlx::query_as::<_, Participant>(
        "SELECT id, event_id, name, email, status, registered_at, updated_at 
         FROM participants 
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
            Json(json!({ "error": "Participant not found" })),
        )
    })?;

    Ok(Json(participant))
}

/// Create a new participant
pub async fn create_participant(
    State(state): State<AppState>,
    Json(payload): Json<CreateParticipant>,
) -> Result<(StatusCode, Json<Participant>), (StatusCode, Json<serde_json::Value>)> {
    if payload.name.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "name is required" })),
        ));
    }

    if payload.email.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "email is required" })),
        ));
    }

    let mut tx = state.db_pool.begin().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to start transaction: {}", e) })),
        )
    })?;

    let max_participants = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT max_participants FROM events WHERE id = $1 FOR UPDATE"
    )
    .bind(&payload.event_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Database error: {}", e) })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Event not found" })),
        )
    })?;

    if let Some(max) = max_participants {
        let current_count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM participants WHERE event_id = $1"
        )
        .bind(&payload.event_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Database error: {}", e) })),
            )
        })?;

        if current_count >= max as i64 {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({ "error": "Event is full" })),
            ));
        }
    }

    let participant = sqlx::query_as::<_, Participant>(
        "INSERT INTO participants (event_id, name, email, status)
         VALUES ($1, $2, $3, 'registered')
         RETURNING id, event_id, name, email, status, registered_at, updated_at"
    )
    .bind(&payload.event_id)
    .bind(&payload.name)
    .bind(&payload.email)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if let Some(db_error) = e.as_database_error() {
            if let Some(pg_error) = db_error.downcast_ref::<PgDatabaseError>() {
                if pg_error.code() == "23505" {
                    return (
                        StatusCode::CONFLICT,
                        Json(json!({ "error": "Participant already registered" })),
                    );
                }
            }
        }
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to create participant: {}", e) })),
        )
    })?;

    tx.commit().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to commit transaction: {}", e) })),
        )
    })?;

    Ok((StatusCode::CREATED, Json(participant)))
}

/// Update participant status
pub async fn update_participant_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateParticipantStatus>,
) -> Result<Json<Participant>, (StatusCode, Json<serde_json::Value>)> {
    let participant = sqlx::query_as::<_, Participant>(
        "UPDATE participants 
         SET status = $1, updated_at = NOW()
         WHERE id = $2
         RETURNING id, event_id, name, email, status, registered_at, updated_at"
    )
    .bind(&payload.status)
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to update participant: {}", e) })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Participant not found" })),
        )
    })?;

    Ok(Json(participant))
}

/// Delete a participant
pub async fn delete_participant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let result = sqlx::query("DELETE FROM participants WHERE id = $1")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to delete participant: {}", e) })),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Participant not found" })),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}
