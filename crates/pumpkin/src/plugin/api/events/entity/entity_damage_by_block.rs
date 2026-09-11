use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when an entity takes damage from a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDamageByBlockEvent {
    /// The ID of the entity taking damage.
    pub entity_id: i32,

    /// The position of the damaging block, if known.
    pub damager_pos: Option<BlockPos>,

    /// The amount of damage dealt.
    pub damage: f32,

    /// The damage cause.
    pub cause: String,
}
