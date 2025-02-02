use pumpkin_data::packet::clientbound::PLAY_ENTITY_EVENT;
use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet(PLAY_ENTITY_EVENT)]
pub struct CEntityStatus {
    entity_id: i32,
    entity_status: i8,
}

impl CEntityStatus {
    pub fn new(entity_id: i32, entity_status: i8) -> Self {
        Self {
            entity_id,
            entity_status,
        }
    }
}
