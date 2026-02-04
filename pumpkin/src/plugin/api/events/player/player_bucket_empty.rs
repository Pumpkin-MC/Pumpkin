use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::vector3::Vector3;

use super::PlayerEvent;

/// An event that occurs when a player empties a bucket.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBucketEmptyEvent {
    /// The player who emptied the bucket.
    pub player: Arc<Player>,

    /// The target block position.
    pub position: Vector3<f64>,

    /// The key of the block being clicked (e.g., "minecraft:stone").
    pub block_key: String,

    /// The face that was interacted with.
    pub face: Option<BlockDirection>,

    /// The bucket item key (e.g., "minecraft:water_bucket").
    pub bucket_item_key: String,

    /// The hand used ("HAND" or "OFF_HAND").
    pub hand: String,
}

impl PlayerBucketEmptyEvent {
    #[must_use]
    pub fn new(
        player: Arc<Player>,
        position: Vector3<f64>,
        block_key: String,
        face: Option<BlockDirection>,
        bucket_item_key: String,
        hand: String,
    ) -> Self {
        Self {
            player,
            position,
            block_key,
            face,
            bucket_item_key,
            hand,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerBucketEmptyEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
