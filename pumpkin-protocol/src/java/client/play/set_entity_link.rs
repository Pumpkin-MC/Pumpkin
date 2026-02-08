use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_LINK;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// Sent when an entity is attached to another (e.g. leash, riding).
///
/// Set `holding_entity_id` to -1 to detach.
#[derive(Serialize)]
#[java_packet(PLAY_SET_ENTITY_LINK)]
pub struct CSetEntityLink {
    pub attached_entity_id: i32,
    pub holding_entity_id: i32,
}

impl CSetEntityLink {
    #[must_use]
    pub const fn new(attached_entity_id: i32, holding_entity_id: i32) -> Self {
        Self {
            attached_entity_id,
            holding_entity_id,
        }
    }
}
