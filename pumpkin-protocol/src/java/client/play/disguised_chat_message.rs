use pumpkin_data::packet::clientbound::PLAY_DISGUISED_CHAT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

/// Sends a chat message that is not cryptographically signed by a player.
///
/// Introduced to support server-side "disguised" identities (like /say or NPC chat),
/// this packet bypasses the player-to-player chat signing requirements while
/// still allowing the client to format the message using the standard chat registry.
#[derive(Serialize)]
#[java_packet(PLAY_DISGUISED_CHAT)]
pub struct CDisguisedChatMessage<'a> {
    /// The raw content of the message.
    pub message: &'a TextComponent,
    /// An index into the `minecraft:chat_type` registry.
    /// This determines the decoration (e.g., "<%s> %s" or "[%s -> %s] %s").
    pub chat_type: VarInt,
    /// The name shown as the "sender" of the message.
    pub sender_name: &'a TextComponent,
    /// The optional name shown as the "target" (used for private messages/whispers).
    pub target_name: Option<&'a TextComponent>,
}

impl<'a> CDisguisedChatMessage<'a> {
    pub fn new(
        message: &'a TextComponent,
        chat_type: VarInt,
        sender_name: &'a TextComponent,
        target_name: Option<&'a TextComponent>,
    ) -> Self {
        Self {
            message,
            chat_type,
            sender_name,
            target_name,
        }
    }
}
