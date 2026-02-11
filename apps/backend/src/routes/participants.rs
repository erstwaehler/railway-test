use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::db;
use crate::models::{Participant, CreateParticipant, UpdateParticipantStatus};

// Type alias for our app state
type AppState = crate::AppState;

/// List all participants for an event
pub async fn list_participants(
    State(state): State<AppState>,
    Path(event_id): Path<Uuid>,
) -> Result<Json<Vec<Participant>>, (StatusCode, Json<serde_json::Value>)> {
    let key = event_id.to_string();

    // Check cache first
    if let Some(participants) = state.cache.participants.get(&key).await {
        return Ok(Json(participants));
    }

    let participants = sqlx::query_as::<_, Participant>(
        "SELECT id, event_id, name, email, status, registered_at, updated_at 
         FROM participants 
         WHERE event_id = ? 
         ORDER BY registered_at ASC"
    )
    .bind(event_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch participants: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?;

    // Populate cache
    state.cache.participants.insert(key, participants.clone()).await;

    Ok(Json(participants))
}

/// Get a single participant by ID
pub async fn get_participant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Participant>, (StatusCode, Json<serde_json::Value>)> {
    let id_str = id.to_string();

    // Check cache first
    if let Some(participant) = state.cache.participant.get(&id_str).await {
        return Ok(Json(participant));
    }

    let participant = sqlx::query_as::<_, Participant>(
        "SELECT id, event_id, name, email, status, registered_at, updated_at 
         FROM participants 
         WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error fetching participant: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Participant not found" })),
        )
    })?;

    // Populate cache
    state.cache.participant.insert(id_str, participant.clone()).await;

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
        tracing::error!("Failed to start transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?;

    let max_participants = sqlx::query_scalar::<_, Option<i32>>(
        "SELECT max_participants FROM events WHERE id = ?"
    )
    .bind(&payload.event_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Database error checking event: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
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
            "SELECT count(*) FROM participants WHERE event_id = ?"
        )
        .bind(&payload.event_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Database error counting participants: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        })?;

        if current_count >= max as i64 {
            return Err((
                StatusCode::CONFLICT,
                Json(json!({ "error": "Event is full" })),
            ));
        }
    }

    let id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let participant = sqlx::query_as::<_, Participant>(
        "INSERT INTO participants (id, event_id, name, email, status, registered_at, updated_at)
         VALUES (?, ?, ?, ?, 'registered', ?, ?)
         RETURNING id, event_id, name, email, status, registered_at, updated_at"
    )
    .bind(id)
    .bind(&payload.event_id)
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(now)
    .bind(now)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if let Some(db_error) = e.as_database_error() {
            if db_error.message().contains("UNIQUE constraint failed") {
                return (
                    StatusCode::CONFLICT,
                    Json(json!({ "error": "Participant already registered" })),
                );
            }
        }
        tracing::error!("Failed to create participant: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?;

    // Invalidate cache and notify other instances
    state.cache.invalidate_participants().await;
    let notification_payload = json!({
        "operation": "INSERT",
        "table": "participants",
        "id": participant.id,
        "event_id": participant.event_id,
        "timestamp": chrono::Utc::now()
    }).to_string();
    if let Err(e) = db::insert_notification(
        &state.db_pool,
        "participant_changes",
        &notification_payload,
    )
    .await
    {
        tracing::error!("Failed to insert participant notification: {}", e);
    }

    Ok((StatusCode::CREATED, Json(participant)))
}

/// Update participant status
pub async fn update_participant_status(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateParticipantStatus>,
) -> Result<Json<Participant>, (StatusCode, Json<serde_json::Value>)> {
    let now = chrono::Utc::now();

    let participant = sqlx::query_as::<_, Participant>(
        "UPDATE participants 
         SET status = ?, updated_at = ?
         WHERE id = ?
         RETURNING id, event_id, name, email, status, registered_at, updated_at"
    )
    .bind(&payload.status)
    .bind(now)
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update participant: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Participant not found" })),
        )
    })?;

    // Invalidate cache and notify other instances
    state.cache.invalidate_participants().await;
    let notification_payload = json!({
        "operation": "UPDATE",
        "table": "participants",
        "id": participant.id,
        "event_id": participant.event_id,
        "timestamp": chrono::Utc::now()
    }).to_string();
    if let Err(e) = db::insert_notification(
        &state.db_pool,
        "participant_changes",
        &notification_payload,
    )
    .await
    {
        tracing::error!("Failed to insert participant notification: {}", e);
    }

    Ok(Json(participant))
}

/// Delete a participant
pub async fn delete_participant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Get event_id before deletion for notification
    let event_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT event_id FROM participants WHERE id = ?"
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch participant: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "Internal server error" })),
        )
    })?;

    let result = sqlx::query("DELETE FROM participants WHERE id = ?")
        .bind(id)
        .execute(&state.db_pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete participant: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Internal server error" })),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Participant not found" })),
        ));
    }

    // Invalidate cache and notify other instances
    state.cache.invalidate_participants().await;
    let notification_payload = json!({
        "operation": "DELETE",
        "table": "participants",
        "id": id,
        "event_id": event_id,
        "timestamp": chrono::Utc::now()
    }).to_string();
    if let Err(e) = db::insert_notification(
        &state.db_pool,
        "participant_changes",
        &notification_payload,
    )
    .await
    {
        tracing::error!("Failed to insert participant notification: {}", e);
    }

    Ok(StatusCode::NO_CONTENT)
}
