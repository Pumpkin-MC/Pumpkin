use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

/// An event that occurs when an entity teleports.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityTeleportEvent {
    /// The entity ID.
    pub entity_id: i32,

    /// The origin position.
    pub from_position: Vector3<f64>,

    /// The destination position.
    pub to_position: Vector3<f64>,
}

impl EntityTeleportEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        from_position: Vector3<f64>,
        to_position: Vector3<f64>,
    ) -> Self {
        Self {
            entity_id,
            from_position,
            to_position,
            cancelled: false,
        }
    }
}
