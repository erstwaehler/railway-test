use moka::future::Cache;
use std::time::Duration;

use crate::models::{Event, Participant};

/// In-memory cache with TTL for events and participants
#[derive(Clone)]
pub struct AppCache {
    pub events_list: Cache<String, Vec<Event>>,
    pub event: Cache<String, Event>,
    pub participants: Cache<String, Vec<Participant>>,
    pub participant: Cache<String, Participant>,
}

impl AppCache {
    pub fn new(ttl_secs: u64) -> Self {
        let ttl = Duration::from_secs(ttl_secs);
        Self {
            events_list: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(10)
                .build(),
            event: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(1000)
                .build(),
            participants: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(1000)
                .build(),
            participant: Cache::builder()
                .time_to_live(ttl)
                .max_capacity(5000)
                .build(),
        }
    }

    /// Invalidate all event-related caches
    pub async fn invalidate_events(&self) {
        self.events_list.invalidate_all();
        self.event.invalidate_all();
    }

    /// Invalidate caches for a specific event
    pub async fn invalidate_event(&self, event_id: &str) {
        self.events_list.invalidate_all();
        self.event.remove(event_id).await;
    }

    /// Invalidate all participant-related caches
    pub async fn invalidate_participants(&self) {
        self.participants.invalidate_all();
        self.participant.invalidate_all();
    }

    /// Invalidate caches based on notification channel
    pub async fn invalidate_for_channel(&self, channel: &str) {
        match channel {
            "event_changes" => self.invalidate_events().await,
            "participant_changes" => self.invalidate_participants().await,
            _ => {}
        }
    }
}
