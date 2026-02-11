use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

use crate::broadcaster::{Broadcaster, ServerEvent};
use crate::cache::AppCache;

pub type DbPool = SqlitePool;

/// Create a SQLite database connection pool
pub async fn create_pool(db_path: &str) -> Result<DbPool, sqlx::Error> {
    let database_url = format!("sqlite:{}?mode=rwc", db_path);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Enable WAL mode for concurrent reads across instances
    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys=ON")
        .execute(&pool)
        .await?;

    info!("SQLite database pool created at {}", db_path);
    Ok(pool)
}

/// Initialize database tables
pub async fn initialize_tables(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS events (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            description TEXT,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            location TEXT,
            max_participants INTEGER,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            CHECK (end_time > start_time),
            CHECK (max_participants IS NULL OR max_participants > 0)
        )"
    )
    .execute(pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS participants (
            id TEXT PRIMARY KEY NOT NULL,
            event_id TEXT NOT NULL REFERENCES events(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'registered',
            registered_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            UNIQUE (event_id, email)
        )"
    )
    .execute(pool)
    .await?;

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_events_start_time ON events(start_time)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_participants_event_id ON participants(event_id)")
        .execute(pool)
        .await?;
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_participants_email ON participants(email)")
        .execute(pool)
        .await?;

    // Notification table for cross-instance sync
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS change_notifications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            channel TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL
        )"
    )
    .execute(pool)
    .await?;

    info!("Database tables initialized");
    Ok(())
}

/// Insert a change notification for cross-instance sync
pub async fn insert_notification(pool: &DbPool, channel: &str, payload: &str) {
    let now = chrono::Utc::now().to_rfc3339();
    if let Err(e) = sqlx::query("INSERT INTO change_notifications (channel, payload, created_at) VALUES (?, ?, ?)")
        .bind(channel)
        .bind(payload)
        .bind(&now)
        .execute(pool)
        .await
    {
        error!("Failed to insert notification: {}", e);
    }
}

/// Get the current maximum notification ID
pub async fn get_max_notification_id(pool: &DbPool) -> i64 {
    sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(id) FROM change_notifications")
        .fetch_one(pool)
        .await
        .unwrap_or(Some(0))
        .unwrap_or(0)
}

/// Poll for new notifications and broadcast them (cross-instance sync)
pub async fn start_notification_poller(
    pool: DbPool,
    broadcaster: Broadcaster,
    cache: AppCache,
    last_id: Arc<Mutex<i64>>,
) {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let current_last_id = {
            let guard = last_id.lock().await;
            *guard
        };

        match sqlx::query_as::<_, (i64, String, String)>(
            "SELECT id, channel, payload FROM change_notifications WHERE id > ? ORDER BY id ASC",
        )
        .bind(current_last_id)
        .fetch_all(&pool)
        .await
        {
            Ok(notifications) => {
                if !notifications.is_empty() {
                    let mut guard = last_id.lock().await;
                    for (id, channel, payload) in &notifications {
                        // Invalidate cache based on notification channel
                        cache.invalidate_for_channel(channel).await;

                        let event = ServerEvent {
                            channel: channel.clone(),
                            payload: payload.clone(),
                        };
                        broadcaster.broadcast(event);
                        *guard = *id;
                    }
                }
            }
            Err(e) => {
                error!("Failed to poll notifications: {}", e);
            }
        }

        // Clean up old notifications (keep last hour)
        let _ = sqlx::query(
            "DELETE FROM change_notifications WHERE created_at < datetime('now', '-1 hour')",
        )
        .execute(&pool)
        .await;
    }
}
