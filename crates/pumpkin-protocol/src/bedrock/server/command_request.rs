// Last verified for v2169

use pumpkin_macros::packet;
use std::borrow::Cow;
use uuid::Uuid;

use crate::serial::{PacketRead, PacketReadSlice};

/// Sent by the client to request server-side execution of a slash command.
#[derive(Debug, PacketRead, PacketReadSlice)]
#[packet(77)]
pub struct SCommandRequest<'a> {
    /// Raw command line string (with leading slash).
    pub command: Cow<'a, str>,
    /// Origin context information identifying the command sender.
    pub origin: CommandOriginData<'a>,
    /// Whether the command originates from an internal engine subsystem.
    pub is_internal: bool,

    // TODO: enum CurrentCmdVersion
    /// Protocol command syntax version string.
    pub version: Cow<'a, str>,
}

/// Metadata describing the origin and sender of an executed command.
#[derive(Debug, PacketRead, PacketReadSlice)]
pub struct CommandOriginData<'a> {
    /// Origin type string (e.g. `player`, `command_block`, `server`).
    pub r#type: Cow<'a, str>,
    /// UUID uniquely identifying the sender instance.
    pub uuid: Uuid,
    /// Request identifier string for correlating command outputs.
    pub request_id: Cow<'a, str>,
    /// Entity unique ID of the player executing the command, if applicable.
    pub player_id: i64,
}
