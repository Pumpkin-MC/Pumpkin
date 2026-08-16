use pumpkin_macros::packet;
use std::borrow::Cow;
use uuid::Uuid;

use crate::serial::{PacketRead, PacketReadSlice};

#[derive(Debug, PacketRead, PacketReadSlice)]
#[packet(77)]
pub struct SCommandRequest<'a> {
    pub command: Cow<'a, str>,
    pub command_type: Cow<'a, str>,
    pub command_uuid: Uuid,
    pub request_id: Cow<'a, str>,
    pub player_actor_unique_id: i64,
    pub is_internal_source: bool,
    pub version: Cow<'a, str>,
}
