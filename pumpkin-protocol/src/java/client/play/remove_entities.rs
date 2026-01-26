use pumpkin_data::packet::clientbound::PLAY_REMOVE_ENTITIES;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Sent by the server to instruct the client to remove (despawn) one or more entities.
///
/// This is typically sent when an entity leaves the player's tracking range,
/// is killed, or is otherwise removed from the world.
#[derive(Serialize)]
#[java_packet(PLAY_REMOVE_ENTITIES)]
pub struct CRemoveEntities<'a> {
    /// A list of entity IDs to be removed.
    pub entity_ids: &'a [VarInt],
}

impl<'a> CRemoveEntities<'a> {
    pub fn new(entity_ids: &'a [VarInt]) -> Self {
        Self { entity_ids }
    }
}
