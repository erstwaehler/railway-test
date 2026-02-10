use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerEvent {
    pub channel: String,
    pub payload: String,
}

/// Broadcaster for Server-Sent Events
#[derive(Clone)]
pub struct Broadcaster {
    sender: Arc<broadcast::Sender<ServerEvent>>,
}

impl Broadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self {
            sender: Arc::new(sender),
        }
    }

    /// Broadcast an event to all connected SSE clients
    pub fn broadcast(&self, event: ServerEvent) {
        let receiver_count = self.sender.receiver_count();
        debug!(
            "Broadcasting event on channel '{}' to {} receivers",
            event.channel, receiver_count
        );

        // Send to all subscribers, ignore if no receivers
        let _ = self.sender.send(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.sender.subscribe()
    }
}

impl Default for Broadcaster {
    fn default() -> Self {
        Self::new()
    }
}
