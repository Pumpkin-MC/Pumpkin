use std::borrow::Cow;

use pumpkin_data::packet::clientbound::PLAY_PLAYER_CHAT;
use pumpkin_util::text::TextComponent;

use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::{VarInt, codec::bit_set::BitSet};

#[derive(Serialize, Deserialize)]
#[packet(PLAY_PLAYER_CHAT)]
pub struct CPlayerChatMessage<'a> {
    global_index: VarInt,
    #[serde(with = "uuid::serde::compact")]
    sender: uuid::Uuid,
    index: VarInt,
    message_signature: Option<Box<[u8]>>, // always 256
    message: Cow<'a, str>,
    timestamp: i64,
    salt: i64,
    previous_messages_count: VarInt,
    previous_messages: Box<[PreviousMessage]>, // max 20
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
        global_index: VarInt,
        sender: uuid::Uuid,
        index: VarInt,
        message_signature: Option<Box<[u8]>>,
        message: Cow<'a, str>,
        timestamp: i64,
        salt: i64,
        previous_messages: Box<[PreviousMessage]>,
        unsigned_content: Option<TextComponent>,
        filter_type: FilterType,
        chat_type: VarInt,
        sender_name: TextComponent,
        target_name: Option<TextComponent>,
    ) -> Self {
        Self {
            global_index,
            sender,
            index,
            message_signature,
            message,
            timestamp,
            salt,
            previous_messages_count: VarInt(previous_messages.len() as i32),
            previous_messages,
            unsigned_content,
            filter_type,
            chat_type,
            sender_name,
            target_name,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PreviousMessage {
    message_id: VarInt,
    signature: Option<Box<[u8]>>, // Always 256
}

#[derive(Serialize, Deserialize)]
pub enum FilterType {
    /// Message is not filtered at all
    PassThrough,
    /// Message is fully filtered
    FullyFiltered,
    /// Only some characters in the message are filtered
    PartiallyFiltered(BitSet),
}
