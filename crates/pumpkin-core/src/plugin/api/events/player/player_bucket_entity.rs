use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player captures an entity with a bucket.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBucketEntityEvent {
    /// The player using the bucket.
    pub player: Arc<Player>,

    /// The ID of the captured entity.
    pub entity_id: i32,

    /// The resulting bucket item name.
    pub bucket_item: String,
}

impl PlayerEvent for PlayerBucketEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
