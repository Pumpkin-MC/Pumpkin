use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_PLAYER_CHAT;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, version::MinecraftVersion};

use crate::{
    ClientPacket, WritingError,
    codec::{bit_set::BitSet, var_int::VarInt},
    ser::NetworkWriteExt,
};

/// Sends a cryptographically signed player chat message to the client.
///
/// This packet is the backbone of the modern secure chat system. It includes
/// tracking indices, digital signatures, and context for reporting.
#[java_packet(PLAY_PLAYER_CHAT)]
pub struct CPlayerChatMessage {
    /// Incremental index for messages sent TO this specific client.
    /// Starts at 0 on login; client disconnects if the sequence is broken.
    pub global_index: VarInt,
    /// The UUID of the player who sent the message.
    pub sender: uuid::Uuid,
    /// Incremental index for messages sent BY the sender player.
    /// Used by the client to verify the order of the sender's history.
    pub index: VarInt,
    /// The RSA signature (256 bytes) verifying the message's authenticity.
    pub message_signature: Option<Box<[u8]>>,
    /// The raw plain-text content of the message.
    pub message: String,
    /// Epoch timestamp (milliseconds) when the message was sent.
    pub timestamp: i64,
    /// A random 64-bit value used to ensure signature uniqueness.
    pub salt: i64,
    /// Last 20 message signatures seen by the sender, providing context
    /// for chat reporting and ensuring no messages were omitted.
    pub previous_messages: Box<[PreviousMessage]>,
    /// Optional formatted version of the message (e.g., if the server
    /// added colors or links that aren't in the signed raw text).
    pub unsigned_content: Option<TextComponent>,
    /// Indicates if the message should be hidden or partially masked
    /// by the client's profanity filter.
    pub filter_type: FilterType,
    /// ID of the chat type registry entry (e.g., "chat", "`say_command`").
    /// Usually `(index + 1)`.
    pub chat_type: VarInt,
    /// The display name of the sender.
    pub sender_name: TextComponent,
    /// The display name of the target (used in private messages).
    pub target_name: Option<TextComponent>,
}

impl CPlayerChatMessage {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
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

//TODO: Check if we need this custom impl
impl ClientPacket for CPlayerChatMessage {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
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
