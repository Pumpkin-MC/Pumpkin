use pumpkin_data::packet::clientbound::PLAY_DAMAGE_EVENT;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_DAMAGE_EVENT)]
pub struct CDamageEvent {
    entity_id: VarInt,
    source_type_id: VarInt,
    source_cause_id: VarInt,
    source_direct_id: VarInt,
    source_position: Option<Vector3<f64>>,
}

impl CDamageEvent {
    pub fn new(
        entity_id: VarInt,
        source_type_id: VarInt,
        source_cause_id: Option<VarInt>,
        source_direct_id: Option<VarInt>,
        source_position: Option<Vector3<f64>>,
    ) -> Self {
        Self {
            entity_id,
            source_type_id,
            source_cause_id: source_cause_id.map_or(VarInt(0), |id| VarInt(id.0 + 1)),
            source_direct_id: source_direct_id.map_or(VarInt(0), |id| VarInt(id.0 + 1)),
            source_position,
        }
    }
}
