use pumpkin_data::packet::clientbound::PLAY_ENTITY_EVENT;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

/// Sends a status update for a specific entity.
///
/// This packet is a "catch-all" for various entity triggers that don't
/// warrant a complex packet of their own. It primarily handles visual
/// and logical state triggers, such as tool breaking, totem usage,
/// or sheep shearing.
#[derive(Serialize, Deserialize)]
#[java_packet(PLAY_ENTITY_EVENT)]
pub struct CEntityStatus {
    /// The Entity ID of the entity affected by the status change.
    pub entity_id: i32,
    /// The ID of the status/event to trigger.
    /// See the table below for common entity statuses.
    pub entity_status: i8,
}

impl CEntityStatus {
    #[must_use]
    pub const fn new(entity_id: i32, entity_status: i8) -> Self {
        Self {
            entity_id,
            entity_status,
        }
    }
}
