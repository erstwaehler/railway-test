use sqlx::{PgPool, postgres::{PgPoolOptions, PgListener}};
use tracing::{info, error, warn};

use crate::broadcaster::{Broadcaster, ServerEvent};

pub type DbPool = PgPool;

/// Create a database connection pool
pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;
    
    info!("Database pool created successfully");
    Ok(pool)
}

/// Start PostgreSQL LISTEN/NOTIFY listener with broadcaster
pub async fn start_listener(database_url: String, broadcaster: Broadcaster) {
    loop {
        match listen_to_notifications(&database_url, broadcaster.clone()).await {
            Ok(_) => {
                warn!("Listener connection closed, reconnecting...");
            }
            Err(e) => {
                error!("Listener error: {}, reconnecting in 5 seconds...", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

async fn listen_to_notifications(database_url: &str, broadcaster: Broadcaster) -> Result<(), sqlx::Error> {
    // Create a listener
    let mut listener = PgListener::connect(database_url).await?;
    
    info!("Listener connected to database");

    // Listen to channels
    listener.listen("event_changes").await?;
    listener.listen("participant_changes").await?;
    
    info!("Listening to event_changes and participant_changes channels");

    // Process notifications
    loop {
        let notification = listener.recv().await?;
        handle_notification(notification.channel(), notification.payload(), &broadcaster);
    }
}

fn handle_notification(channel: &str, payload: &str, broadcaster: &Broadcaster) {
    info!("Received notification on channel '{}'", channel);
    tracing::debug!("Notification payload: {}", payload);
    
    // Broadcast to all SSE clients
    let event = ServerEvent {
        channel: channel.to_string(),
        payload: payload.to_string(),
    };
    
    broadcaster.broadcast(event);
}
