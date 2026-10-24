use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

/// An event that occurs when a lightning bolt strikes in a world.
#[cancellable]
#[derive(Event, Clone)]
pub struct LightningStrikeEvent {
    /// The location of the strike.
    pub position: Vector3<f64>,

    /// Whether this strike was spawned by an effect.
    pub is_effect: bool,
}

impl LightningStrikeEvent {
    #[must_use]
    pub const fn new(position: Vector3<f64>, is_effect: bool) -> Self {
        Self {
            position,
            is_effect,
            cancelled: false,
        }
    }
}
