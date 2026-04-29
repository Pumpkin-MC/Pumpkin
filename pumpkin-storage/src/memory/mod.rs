/// Format-agnostic, in-memory storage.
///
/// Unlike [`VanillaStorage`](crate::VanillaStorage), this backend holds domain
/// values directly (no serialization, no on-disk layout). Intended for tests,
/// ephemeral servers, and embedded contexts where persistence is not needed.
///
/// Domain fields are added alongside their corresponding trait implementations.
#[derive(Debug, Default)]
pub struct MemoryStorage;

impl MemoryStorage {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
