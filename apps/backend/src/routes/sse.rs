use axum::{
    extract::State,
    response::{sse::Event, Sse},
};
use futures::stream::Stream;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt as _;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, error};

// Type alias for our app state
type AppState = crate::AppState;

/// SSE endpoint that streams events to clients
pub async fn event_stream(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    debug!("New SSE client connected");

    let receiver = state.broadcaster.subscribe();
    let stream = BroadcastStream::new(receiver);

    let event_stream = stream
        .filter_map(|result| match result {
            Ok(event) => {
                debug!("Sending event to SSE client: {:?}", event);
                Some(Ok(Event::default()
                    .event(&event.channel)
                    .data(event.payload)))
            }
            Err(e) => {
                error!("Broadcast stream error: {}", e);
                None
            }
        });

    Sse::new(event_stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
