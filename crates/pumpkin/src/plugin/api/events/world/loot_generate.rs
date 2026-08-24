use pumpkin_macros::{Event, cancellable};

/// An event that occurs when loot is generated.
#[cancellable]
#[derive(Event, Clone)]
pub struct LootGenerateEvent {
    /// Loot table identifier.
    pub loot_table: String,
}

impl LootGenerateEvent {
    #[must_use]
    pub const fn new(loot_table: String) -> Self {
        Self {
            loot_table,
            cancelled: false,
        }
    }
}
