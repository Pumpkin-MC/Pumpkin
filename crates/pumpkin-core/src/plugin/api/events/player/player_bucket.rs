use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

/// An event that occurs when a player empties a bucket.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBucketEmptyEvent {
    /// The player emptying the bucket.
    pub player: Arc<Player>,

    /// The target block position where the bucket is emptied.
    pub block_pos: BlockPos,

    /// The bucket item used.
    pub bucket: String,
}

impl PlayerBucketEmptyEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, block_pos: BlockPos, bucket: String) -> Self {
        Self {
            player,
            block_pos,
            bucket,
            cancelled: false,
        }
    }
}

/// An event that occurs when a player fills a bucket.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBucketFillEvent {
    /// The player filling the bucket.
    pub player: Arc<Player>,

    /// The target block position being targeted for filling.
    pub block_pos: BlockPos,

    /// The bucket item used.
    pub bucket: String,
}

impl PlayerBucketFillEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, block_pos: BlockPos, bucket: String) -> Self {
        Self {
            player,
            block_pos,
            bucket,
            cancelled: false,
        }
    }
}
