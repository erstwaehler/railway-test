use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio_postgres::{AsyncMessage, NoTls};
use tracing::{info, error, warn};

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

/// Start PostgreSQL LISTEN/NOTIFY listener
pub async fn start_listener(database_url: String) {
    loop {
        match listen_to_notifications(&database_url).await {
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

async fn listen_to_notifications(database_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse connection string
    let config = database_url.parse::<tokio_postgres::Config>()?;
    
    // Connect to database
    let (client, mut connection) = config.connect(NoTls).await?;
    
    info!("Listener connected to database");

    // Listen to channels
    client.execute("LISTEN event_changes", &[]).await?;
    client.execute("LISTEN participant_changes", &[]).await?;
    
    info!("Listening to event_changes and participant_changes channels");

    // Process notifications
    loop {
        tokio::select! {
            message = connection.recv() => {
                match message {
                    Some(Ok(AsyncMessage::Notification(notification))) => {
                        handle_notification(notification);
                    }
                    Some(Err(e)) => {
                        error!("Connection error: {}", e);
                        return Err(e.into());
                    }
                    None => {
                        warn!("Connection closed");
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}

fn handle_notification(notification: tokio_postgres::Notification) {
    info!(
        "Received notification on channel '{}': {}",
        notification.channel(),
        notification.payload()
    );
    
    // TODO: Parse payload and broadcast to WebSocket clients
    match notification.channel() {
        "event_changes" => {
            info!("Event change detected: {}", notification.payload());
        }
        "participant_changes" => {
            info!("Participant change detected: {}", notification.payload());
        }
        _ => {
            warn!("Unknown channel: {}", notification.channel());
        }
    }
}
