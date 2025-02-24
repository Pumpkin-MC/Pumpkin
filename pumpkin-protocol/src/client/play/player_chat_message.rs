use pumpkin_data::packet::clientbound::PLAY_PLAYER_CHAT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::Serialize;

use crate::{VarInt, codec::bit_set::BitSet};

#[derive(Serialize)]
#[packet(PLAY_PLAYER_CHAT)]
pub struct CPlayerChatMessage<'a> {
    #[serde(with = "uuid::serde::compact")]
    sender: uuid::Uuid,
    index: VarInt,
    message_signature: Option<&'a [u8]>,
    message: &'a str,
    timestamp: i64,
    salt: i64,
    previous_messages_count: VarInt,
    previous_messages: &'a [PreviousMessage<'a>], // max 20
    unsigned_content: Option<TextComponent>,
    filter_type: FilterType,
    /// This should not be zero, (index + 1)
    chat_type: VarInt,
    sender_name: TextComponent,
    target_name: Option<TextComponent>,
}

impl<'a> CPlayerChatMessage<'a> {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        sender: uuid::Uuid,
        index: VarInt,
        message_signature: Option<&'a [u8]>,
        message: &'a str,
        timestamp: i64,
        salt: i64,
        previous_messages: &'a [PreviousMessage<'a>],
        unsigned_content: Option<TextComponent>,
        filter_type: FilterType,
        chat_type: VarInt,
        sender_name: TextComponent,
        target_name: Option<TextComponent>,
    ) -> Self {
        Self {
            sender,
            index,
            message_signature,
            message,
            timestamp,
            salt,
            previous_messages_count: previous_messages.len().into(),
            previous_messages,
            unsigned_content,
            filter_type,
            chat_type,
            sender_name,
            target_name,
        }
    }
}

#[derive(Serialize)]
pub struct PreviousMessage<'a> {
    message_id: VarInt,
    signature: Option<&'a [u8]>,
}

#[derive(Serialize)]
pub enum FilterType {
    /// Message is not filtered at all
    PassThrough,
    /// Message is fully filtered
    FullyFiltered,
    /// Only some characters in the message are filtered
    PartiallyFiltered(BitSet),
}
