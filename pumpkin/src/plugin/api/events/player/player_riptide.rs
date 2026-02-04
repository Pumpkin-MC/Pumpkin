use pumpkin_macros::Event;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player uses riptide.
#[derive(Event, Clone)]
pub struct PlayerRiptideEvent {
    /// The player using riptide.
    pub player: Arc<Player>,

    /// The item used.
    pub item_stack: ItemStack,

    /// The riptide velocity.
    pub velocity: Vector3<f64>,
}

impl PlayerRiptideEvent {
    /// Creates a new instance of `PlayerRiptideEvent`.
    pub fn new(player: Arc<Player>, item_stack: ItemStack, velocity: Vector3<f64>) -> Self {
        Self {
            player,
            item_stack,
            velocity,
        }
    }
}

impl PlayerEvent for PlayerRiptideEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
