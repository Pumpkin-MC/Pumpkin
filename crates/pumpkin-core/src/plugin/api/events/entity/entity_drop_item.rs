use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity drops an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDropItemEvent {
    /// The ID of the entity.
    pub entity_id: i32,
    /// Registry name of the item.
    pub item_name: String,
    /// Amount dropped.
    pub count: u8,
}

impl EntityDropItemEvent {
    #[must_use]
    pub const fn new(entity_id: i32, item_name: String, count: u8) -> Self {
        Self {
            entity_id,
            item_name,
            count,
            cancelled: false,
        }
    }
}
