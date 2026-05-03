use pumpkin_macros::packet;
use uuid::Uuid;

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

#[derive(Debug, PacketRead)]
#[packet(77)]
pub struct SCommandRequest {
    pub command: String,

    // Command Origin
    pub command_type: String,
    pub command_uuid: Uuid,
    pub request_id: String,
    pub player_actor_unique_id: i64,

    pub is_internal_source: bool,
    pub version: String,
}
