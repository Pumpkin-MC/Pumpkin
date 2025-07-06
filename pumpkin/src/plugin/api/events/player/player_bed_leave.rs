use crate::entity::player::Player;
use crate::plugin::player::PlayerEvent;
use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// Called when a player attempts to leave a bed.
///
/// This event can be cancelled to prevent the player from leaving the bed.
///
/// If not cancelled and `set_bed_spawn` is `true`, the player's respawn location
/// will be updated to the bed's position.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBedLeaveEvent {
    /// The player who is attempting to leave the bed.
    pub player: Arc<Player>,

    /// The bed block being left.
    pub bed: Block,

    /// Whether the player's spawn point should be updated to this bed.
    ///
    /// This is typically `true` in survival mode, unless explicitly disabled
    /// (e.g. by a plugin or in adventure mode).
    pub set_bed_spawn: bool,
}

impl PlayerBedLeaveEvent {
    /// Creates a new `PlayerBedLeaveEvent`.
    ///
    /// # Arguments
    ///
    /// * `player` - The player entering the bed.
    /// * `bed` - The block representing the bed.
    /// * `set_bed_spawn` - Whether the player's respawn location should be updated.
    pub fn new(player: Arc<Player>, bed: Block, set_bed_spawn: bool) -> Self {
        Self {
            player,
            bed,
            set_bed_spawn,
            cancelled: false,
        }
    }

    /// Returns the bed block the player is leaving.
    #[must_use]
    pub fn get_bed(&self) -> Block {
        self.bed.clone()
    }

    /// Returns `true` if the player's spawn location should be set to the bed's position.
    #[must_use]
    pub fn should_set_spawn_location(&self) -> bool {
        self.set_bed_spawn
    }
}

impl PlayerEvent for PlayerBedLeaveEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
