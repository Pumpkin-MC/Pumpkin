// Last verified for v2169

use pumpkin_macros::packet;
use std::borrow::Cow;
use uuid::Uuid;

use crate::serial::{PacketRead, PacketReadSlice};

#[derive(Debug, PacketRead, PacketReadSlice)]
#[packet(77)]
pub struct SCommandRequest<'a> {
    pub command: Cow<'a, str>,
    pub origin: CommandOriginData<'a>,
    pub is_internal: bool,

    // TODO: enum CurrentCmdVersion
    pub version: Cow<'a, str>,
}

#[derive(Debug, PacketRead, PacketReadSlice)]
pub struct CommandOriginData<'a> {
    pub r#type: Cow<'a, str>,
    pub uuid: Uuid,
    pub request_id: Cow<'a, str>,
    pub player_id: i64,
}
