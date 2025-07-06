use crate::entity::player::Player;
use crate::plugin::player::PlayerEvent;
use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// Called when a player attempts to enter a bed.
///
/// This event can be cancelled to prevent the player from entering the bed.
///
/// As there are no event `Result`s implemented yet, I won't implement the methods to manipulate them for this event.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBedEnterEvent {
    /// The player who is attempting to enter the bed.
    pub player: Arc<Player>,

    /// The bed block being entered.
    pub bed: Block,

    /// Represents the outcome of this event.
    pub bed_enter_result: BedEnterResult,
}

impl PlayerBedEnterEvent {
    /// Creates a new `PlayerBedEnterEvent`.
    ///
    /// # Arguments
    ///
    /// * `player` - The player entering the bed.
    /// * `bed` - The block representing the bed.
    /// * `bed_enter_result` - Describes whether the attempt was successful or not and why.
    ///
    /// # Returns
    /// A new instance of `PlayerBedEnterEvent`.
    pub fn new(player: Arc<Player>, bed: Block, bed_enter_result: BedEnterResult) -> Self {
        Self {
            player,
            bed,
            bed_enter_result,
            cancelled: false,
        }
    }

    /// Returns the bed block the player is entering.
    #[must_use]
    pub fn get_bed(&self) -> Block {
        self.bed.clone()
    }

    /// Returns the `BedEnterResult` of the attempt of entering the bed.
    #[must_use]
    pub fn get_bed_enter_result(&self) -> BedEnterResult {
        self.bed_enter_result.clone()
    }

    pub fn use_bed(&self) {
        // todo()!
    }

    pub fn set_use_bed(&self) {
        // todo()!
    }
}

impl PlayerEvent for PlayerBedEnterEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}

/// The possible results of a player trying to enter a bed.
#[derive(Clone)]
pub enum BedEnterResult {
    Ok,
    NotPossibleHere,
    NotPossibleNow,
    TooFarAway,
    Obstructed,
    NotSafe,
    OtherProblem,
}
