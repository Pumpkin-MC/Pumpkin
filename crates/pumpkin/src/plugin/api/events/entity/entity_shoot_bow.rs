use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity shoots a bow or crossbow.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityShootBowEvent {
    /// The ID of the shooting entity.
    pub entity_id: i32,

    /// The registry name of the weapon item.
    pub weapon_name: String,

    /// The shot force/velocity factor.
    pub force: f32,
}

impl EntityShootBowEvent {
    #[must_use]
    pub const fn new(entity_id: i32, weapon_name: String, force: f32) -> Self {
        Self {
            entity_id,
            weapon_name,
            force,
            cancelled: false,
        }
    }
}
