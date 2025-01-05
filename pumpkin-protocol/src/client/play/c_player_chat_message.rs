use pumpkin_core::text::Text;

use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::{codec::bit_set::BitSet, VarInt};

#[derive(Serialize)]
#[client_packet("play:player_chat")]
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
    unsigned_content: Option<Text>,
    filter_type: FilterType,
    chat_type: VarInt,
    sender_name: Text,
    target_name: Option<Text>,
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
        unsigned_content: Option<impl Into<Text>>,
        filter_type: FilterType,
        chat_type: VarInt,
        sender_name: impl Into<Text>,
        target_name: Option<impl Into<Text>>,
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
            unsigned_content: unsigned_content.map(|x| x.into()),
            filter_type,
            chat_type,
            sender_name: sender_name.into(),
            target_name: target_name.map(|x| x.into()),
        }
    }
}

#[derive(Serialize)]
pub struct PreviousMessage<'a> {
    message_id: VarInt,
    signature: Option<&'a [u8]>,
}

#[derive(Serialize)]
#[repr(i32)]
pub enum FilterType {
    /// Message is not filtered at all
    PassThrough = 0,
    /// Message is fully filtered
    FullyFiltered = 1,
    /// Only some characters in the message are filtered
    PartiallyFiltered(BitSet) = 2,
}
