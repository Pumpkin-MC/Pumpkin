use std::sync::atomic::{AtomicU64, Ordering};

use tracing_serde_structured::AsSerde;

use crate::wit;

pub(crate) struct WitSubscriber {
    next_id: AtomicU64,
}

impl WitSubscriber {
    pub fn new() -> Self {
        Self {
            next_id: AtomicU64::new(1),
        }
    }
}

impl tracing::Subscriber for WitSubscriber {
    fn enabled(&self, _metadata: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _attrs: &tracing::span::Attributes<'_>) -> tracing::span::Id {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        tracing::span::Id::from_u64(id)
    }

    fn record(&self, _span: &tracing::span::Id, _values: &tracing::span::Record<'_>) {}

    fn record_follows_from(&self, _span: &tracing::span::Id, _follows: &tracing::span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let serialized =
            postcard::to_allocvec(&event.as_serde()).expect("failed to serialize tracing event");
        wit::pumpkin::plugin::logging::log_tracing(&serialized);
    }

    fn enter(&self, _span: &tracing::span::Id) {}

    fn exit(&self, _span: &tracing::span::Id) {}
}

/// The log severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
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

pub fn log(level: LogLevel, message: &str) {
    wit::pumpkin::plugin::logging::log(level.to_wit(), message);
}
