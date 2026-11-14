use std::sync::atomic::{AtomicU64, Ordering};

use tracing_serde_structured::AsSerde;

use crate::wit;

/// A [`tracing::Subscriber`] that forwards events to the host server over WIT.
///
/// Installed automatically as the global subscriber when the plugin is loaded.
pub(crate) struct WitSubscriber {
    next_id: AtomicU64,
}

impl WitSubscriber {
    /// Creates a new `WitSubscriber` with the span ID counter starting at `1`.
    pub const fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
        }
    }
}

impl tracing::Subscriber for WitSubscriber {
    /// Always returns `true` - all log levels are forwarded to the host.
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    /// Allocates a new monotonically increasing span ID.
    fn new_span(&self, _attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        tracing::span::Id::from_u64(id)
    }

    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    /// Serialises the tracing event with `postcard` and sends it to the host via WIT.
    fn event(&self, event: &tracing::Event<'_>) {
        if let Ok(serialized) = postcard::to_allocvec(&event.as_serde()) {
            wit::pumpkin::plugin::logging::log_tracing(&serialized);
        }
    }

    fn enter(&self, _span: &tracing::span::Id) {}

    fn exit(&self, _span: &tracing::span::Id) {}
}

pub use crate::wit::pumpkin::plugin::logging::{Level as LogLevel, log};
