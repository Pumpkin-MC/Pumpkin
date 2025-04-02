use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_PLAYER_CHAT;
use pumpkin_macros::packet;
use pumpkin_util::text::TextComponent;

use crate::{
    ClientPacket,
    codec::{bit_set::BitSet, var_int::VarInt},
    ser::{NetworkWriteExt, WritingError},
};

#[packet(PLAY_PLAYER_CHAT)]
pub struct CPlayerChatMessage {
    /// An index that increases for every message sent TO the client
    global_index: VarInt,
    sender: uuid::Uuid,
    /// An index that increases for every message sent BY the client
    index: VarInt,
    message_signature: Option<Box<[u8]>>, // always 256
    message: String,
    timestamp: i64,
    salt: i64,
    previous_messages: Box<[PreviousMessage]>, // max 20
    unsigned_content: Option<TextComponent>,
    filter_type: FilterType,
    /// This should not be zero, (index + 1)
    chat_type: VarInt,
    sender_name: TextComponent,
    target_name: Option<TextComponent>,
}

impl CPlayerChatMessage {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        global_index: VarInt,
        sender: uuid::Uuid,
        index: VarInt,
        message_signature: Option<Box<[u8]>>,
        message: String,
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
            previous_messages,
            unsigned_content,
            filter_type,
            chat_type,
            sender_name,
            target_name,
        }
    }
}

impl ClientPacket for CPlayerChatMessage {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        write.write_var_int(&self.global_index)?;
        write.write_uuid(&self.sender)?;
        write.write_var_int(&self.index)?;
        write.write_option(&self.message_signature, |p, v| p.write_slice(v))?;
        write.write_string(&self.message)?;
        write.write_i64_be(self.timestamp)?;
        write.write_i64_be(self.salt)?;
        write.write_list(&self.previous_messages, |p, v| {
            p.write_var_int(&v.id)?;
            if let Some(signature) = &v.signature {
                p.write_slice(signature)?;
            }
            Ok(())
        })?;
        write.write_option(&self.unsigned_content, |p, v| p.write_slice(&v.encode()))?;
        write.write_var_int(&VarInt(match self.filter_type {
            FilterType::PassThrough => 0,
            FilterType::FullyFiltered => 1,
            FilterType::PartiallyFiltered(_) => 2,
        }))?;
        write.write_var_int(&self.chat_type)?;
        write.write_slice(&self.sender_name.encode())?;
        write.write_option(&self.target_name, |p, v| p.write_slice(&v.encode()))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct PreviousMessage {
    pub id: VarInt,
    pub signature: Option<Box<[u8]>>, // Always 256
}

pub enum FilterType {
    /// Message is not filtered at all
    PassThrough,
    /// Message is fully filtered
    FullyFiltered,
    /// Only some characters in the message are filtered
    PartiallyFiltered(BitSet),
}
