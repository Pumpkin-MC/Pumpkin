use pumpkin_data::packet::clientbound::PLAY_PLAYER_INFO_REMOVE;
use pumpkin_macros::java_packet;
use serde::{Serialize, ser::SerializeSeq};

/// Sent by the server to remove one or more players from the client's player list (tab list).
///
/// This packet is typically used when a player leaves the server or becomes invisible
/// to the recipient (e.g., moving out of tracking range).
#[derive(Serialize)]
#[java_packet(PLAY_PLAYER_INFO_REMOVE)]
pub struct CRemovePlayerInfo<'a> {
    /// A list of UUIDs corresponding to the players that should be removed.
    ///
    /// The field uses a custom serializer to format the UUIDs according to the
    /// Minecraft protocol's VarInt-prefixed array format.
    #[serde(serialize_with = "serialize_slice_uuids")]
    pub players: &'a [uuid::Uuid],
}

impl<'a> CRemovePlayerInfo<'a> {
    #[must_use]
    pub const fn new(players: &'a [uuid::Uuid]) -> Self {
        Self { players }
    }
}

fn serialize_slice_uuids<S: serde::Serializer>(
    uuids: &[uuid::Uuid],
    serializer: S,
) -> Result<S::Ok, S::Error> {
    let mut seq = serializer.serialize_seq(Some(uuids.len()))?;
    for uuid in uuids {
        seq.serialize_element(uuid.as_bytes())?;
    }
    seq.end()
}
