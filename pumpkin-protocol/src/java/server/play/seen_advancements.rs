use std::io::Read;

use pumpkin_data::packet::serverbound::PLAY_SEEN_ADVANCEMENTS;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::{ReadingError, ServerPacket, VarInt, ser::NetworkReadExt};

/// Sent when the player opens/closes the advancement screen or switches tabs.
#[java_packet(PLAY_SEEN_ADVANCEMENTS)]
pub struct SSeenAdvancements {
    /// 0 = opened tab, 1 = closed screen.
    pub action: VarInt,
    /// Present only when action is 0 (opened tab).
    pub tab_id: Option<ResourceLocation>,
}

impl ServerPacket for SSeenAdvancements {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;
        let action = read.get_var_int()?;
        let tab_id = if action.0 == 0 {
            Some(read.get_string()?)
        } else {
            None
        };
        Ok(Self { action, tab_id })
    }
}
