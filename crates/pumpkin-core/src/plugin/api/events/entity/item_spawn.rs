use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

/// An event that occurs when an item entity spawns in the world.
#[cancellable]
#[derive(Event, Clone)]
pub struct ItemSpawnEvent {
    /// The ID of the item entity.
    pub entity_id: i32,
    /// Position of the item entity.
    pub position: Vector3<f64>,
    /// Item registry name.
    pub item_name: String,
}

impl ItemSpawnEvent {
    #[must_use]
    pub const fn new(entity_id: i32, position: Vector3<f64>, item_name: String) -> Self {
        Self {
            entity_id,
            position,
            item_name,
            cancelled: false,
        }
    }
}
