#![allow(clippy::doc_markdown, clippy::missing_const_for_fn)]

use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;
use uuid::Uuid;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player uses a bucket on an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerBucketEntityEvent {
    /// The player who used the bucket.
    pub player: Arc<Player>,

    /// The entity UUID.
    pub entity_uuid: Uuid,

    /// The entity type key (e.g., "minecraft:axolotl").
    pub entity_type: String,

    /// The bucket item in hand.
    pub original_bucket_key: String,

    /// The resulting bucket item (if any).
    pub entity_bucket_key: String,

    /// The hand used ("HAND" or "OFF_HAND").
    pub hand: String,
}

impl PlayerBucketEntityEvent {
    #[must_use]
    pub fn new(
        player: Arc<Player>,
        entity_uuid: Uuid,
        entity_type: String,
        original_bucket_key: String,
        entity_bucket_key: String,
        hand: String,
    ) -> Self {
        Self {
            player,
            entity_uuid,
            entity_type,
            original_bucket_key,
            entity_bucket_key,
            hand,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerBucketEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
