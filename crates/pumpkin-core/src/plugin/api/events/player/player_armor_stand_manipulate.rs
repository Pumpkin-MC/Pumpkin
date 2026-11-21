use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player manipulates an armor stand.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerArmorStandManipulateEvent {
    /// The player manipulating the armor stand.
    pub player: Arc<Player>,

    /// The ID of the armor stand entity.
    pub armor_stand_id: i32,

    /// The armor stand slot.
    pub slot: u8,
}

impl PlayerEvent for PlayerArmorStandManipulateEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
