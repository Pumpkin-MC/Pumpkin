use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player interacts at a specific location on an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInteractAtEntityEvent {
    /// The player.
    pub player: Arc<Player>,
    /// Target entity ID.
    pub entity_id: i32,
    /// Clicked X.
    pub clicked_x: f64,
    /// Clicked Y.
    pub clicked_y: f64,
    /// Clicked Z.
    pub clicked_z: f64,
    /// Hand used (0 = main hand, 1 = off hand).
    pub hand: u8,
}

impl PlayerInteractAtEntityEvent {
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        entity_id: i32,
        clicked_x: f64,
        clicked_y: f64,
        clicked_z: f64,
        hand: u8,
    ) -> Self {
        Self {
            player,
            entity_id,
            clicked_x,
            clicked_y,
            clicked_z,
            hand,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerInteractAtEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
