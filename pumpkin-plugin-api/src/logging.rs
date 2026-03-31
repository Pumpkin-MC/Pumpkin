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
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
        }
    }
}

impl tracing::Subscriber for WitSubscriber {
    /// Always returns `true` — all log levels are forwarded to the host.
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    /// Allocates a new monotonically increasing span ID.
    fn new_span(&self, _attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        tracing::span::Id::from_u64(id)
    }

    /// No-op — span field recording is not forwarded to the host.
    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    /// No-op — causality links are not forwarded to the host.
    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    /// Serialises the tracing event with `postcard` and sends it to the host via WIT.
    fn event(&self, event: &tracing::Event<'_>) {
        let serialized =
            postcard::to_allocvec(&event.as_serde()).expect("failed to serialize tracing event");
        wit::pumpkin::plugin::logging::log_tracing(&serialized);
    }

    /// No-op — span entry is not tracked.
    fn enter(&self, _span: &tracing::span::Id) {}

    /// No-op — span exit is not tracked.
    fn exit(&self, _span: &tracing::span::Id) {}
}

/// The log severity level used with [`log`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Very fine-grained diagnostic information.
    Trace,
    /// Diagnostic information useful during development.
    Debug,
    /// General informational messages.
    Info,
    /// Potentially harmful situations that do not stop execution.
    Warn,
    /// Errors that may require attention.
    Error,
}

impl LogLevel {
    /// Converts this level to the WIT-generated `Level` type expected by the host.
    fn to_wit(self) -> wit::pumpkin::plugin::logging::Level {
        match self {
            LogLevel::Trace => wit::pumpkin::plugin::logging::Level::Trace,
            LogLevel::Debug => wit::pumpkin::plugin::logging::Level::Debug,
            LogLevel::Info => wit::pumpkin::plugin::logging::Level::Info,
            LogLevel::Warn => wit::pumpkin::plugin::logging::Level::Warn,
            LogLevel::Error => wit::pumpkin::plugin::logging::Level::Error,
        }
    }
}

/// Sends a log message to the server at the given severity level.
///
/// Prefer the standard `tracing` macros (`tracing::info!`, `tracing::warn!`, etc.)
/// for structured logging; use this function when you need a direct, low-level log call.
pub fn log(level: LogLevel, message: &str) {
    wit::pumpkin::plugin::logging::log(level.to_wit(), message);
}
