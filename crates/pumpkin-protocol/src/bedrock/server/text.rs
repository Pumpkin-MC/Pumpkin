use crate::{
    codec::var_uint::VarUInt,
    serial::{PacketRead, PacketReadSlice, PacketWrite, read_str_slice},
};
use pumpkin_macros::packet;
use std::borrow::Cow;
use std::io::{Error, Read, Write};

/// Transmits in-game chat messages, system alerts, popups, and raw JSON text between client and server.
#[derive(Debug)]
#[packet(9)]
pub struct SText<'a> {
    /// Whether translation is requested for localization strings in the message.
    pub needs_translation: bool,
    /// Formatting category and presentation target of the text.
    pub r#type: TextPacketType,
    /// Sender username (included in chat, whisper, and announcement packets).
    pub source_name: Cow<'a, str>,
    /// Primary text content or translation key.
    pub message: Cow<'a, str>,
    /// Localization substitution arguments for translation or popup texts.
    pub parameters: Vec<Cow<'a, str>>,
    /// Sender Xbox Live User ID (XUID).
    pub xuid: Cow<'a, str>,
    /// Platform-specific chat identifier (e.g. Nintendo Switch chat isolation).
    pub platform_chat_id: Cow<'a, str>,
    /// Optional profanity-filtered message for clients with filtering enabled.
    pub filtered_message: Option<Cow<'a, str>>,
}

impl SText<'_> {
    const fn get_category(&self) -> u8 {
        match self.r#type {
            TextPacketType::Raw
            | TextPacketType::Tip
            | TextPacketType::System
            | TextPacketType::JsonWhisper
            | TextPacketType::JsonAnnouncement
            | TextPacketType::Json => 0,

            TextPacketType::Chat | TextPacketType::Whisper | TextPacketType::Announcement => 1,

            TextPacketType::Translation | TextPacketType::Popup | TextPacketType::JukeboxPopup => 2,
        }
    }
}

impl SText<'static> {
    #[must_use]
    pub fn new(message: String, source_name: String) -> Self {
        Self::chat_with_xuid(message, source_name, String::new(), None)
    }

    #[must_use]
    pub fn chat_with_xuid(
        message: String,
        source_name: String,
        xuid: String,
        filtered_message: Option<String>,
    ) -> Self {
        Self {
            needs_translation: false,
            r#type: TextPacketType::Chat,
            source_name: Cow::Owned(source_name),
            message: Cow::Owned(if message.is_empty() {
                " ".to_string()
            } else {
                message
            }),
            parameters: Vec::new(),
            xuid: Cow::Owned(xuid),
            platform_chat_id: Cow::Borrowed(""),
            filtered_message: filtered_message.map(Cow::Owned),
        }
    }

    #[must_use]
    pub fn translation(message: String, parameters: Vec<String>) -> Self {
        Self {
            needs_translation: true,
            r#type: TextPacketType::Translation,
            source_name: Cow::Borrowed(""),
            message: Cow::Owned(message),
            parameters: parameters.into_iter().map(Cow::Owned).collect(),
            xuid: Cow::Borrowed(""),
            platform_chat_id: Cow::Borrowed(""),
            filtered_message: None,
        }
    }

    #[must_use]
    pub fn system_message(message: String) -> Self {
        Self {
            needs_translation: false,
            r#type: TextPacketType::System,
            source_name: Cow::Borrowed(""),
            message: Cow::Owned(if message.is_empty() {
                " ".to_string()
            } else {
                message
            }),
            parameters: Vec::new(),
            xuid: Cow::Borrowed(""),
            platform_chat_id: Cow::Borrowed(""),
            filtered_message: None,
        }
    }

    #[must_use]
    pub const fn json(message: String) -> Self {
        Self {
            needs_translation: false,
            r#type: TextPacketType::Json,
            source_name: Cow::Borrowed(""),
            message: Cow::Owned(message),
            parameters: Vec::new(),
            xuid: Cow::Borrowed(""),
            platform_chat_id: Cow::Borrowed(""),
            filtered_message: None,
        }
    }
}

impl<'a> PacketReadSlice<'a> for SText<'a> {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error> {
        let needs_translation = bool::read_slice(buf)?;
        let _category = u8::read_slice(buf)?;
        let r#type = TextPacketType::read_slice(buf)?;

        let mut source_name = "";
        let message;
        let mut parameters: Vec<Cow<'a, str>> = Vec::new();

        match r#type {
            TextPacketType::Chat | TextPacketType::Whisper | TextPacketType::Announcement => {
                source_name = read_str_slice(buf)?;
                message = read_str_slice(buf)?;
            }
            TextPacketType::Raw
            | TextPacketType::Tip
            | TextPacketType::System
            | TextPacketType::JsonWhisper
            | TextPacketType::Json
            | TextPacketType::JsonAnnouncement => {
                message = read_str_slice(buf)?;
            }
            TextPacketType::Translation | TextPacketType::Popup | TextPacketType::JukeboxPopup => {
                message = read_str_slice(buf)?;
                let count = VarUInt::read_slice(buf)?.0 as usize;
                for _ in 0..count {
                    parameters.push(Cow::Borrowed(read_str_slice(buf)?));
                }
            }
        }

        let xuid = read_str_slice(buf)?;
        let platform_chat_id = read_str_slice(buf)?;

        let filtered_message = bool::read_slice(buf)?
            .then(|| read_str_slice(buf).map(Cow::Borrowed))
            .transpose()?;

        Ok(Self {
            needs_translation,
            r#type,
            source_name: Cow::Borrowed(source_name),
            message: Cow::Borrowed(message),
            parameters,
            xuid: Cow::Borrowed(xuid),
            platform_chat_id: Cow::Borrowed(platform_chat_id),
            filtered_message,
        })
    }
}

impl PacketRead for SText<'static> {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let needs_translation = bool::read(reader)?;
        let _category = u8::read(reader)?;
        let r#type = TextPacketType::read(reader)?;

        let mut source_name = String::new();
        #[expect(unused)]
        let mut message = String::new();
        let mut parameters = Vec::new();

        match r#type {
            TextPacketType::Chat | TextPacketType::Whisper | TextPacketType::Announcement => {
                source_name = String::read(reader)?;
                message = String::read(reader)?;
            }
            TextPacketType::Raw
            | TextPacketType::Tip
            | TextPacketType::System
            | TextPacketType::JsonWhisper
            | TextPacketType::Json
            | TextPacketType::JsonAnnouncement => {
                message = String::read(reader)?;
            }
            TextPacketType::Translation | TextPacketType::Popup | TextPacketType::JukeboxPopup => {
                message = String::read(reader)?;
                let count = VarUInt::read(reader)?.0 as usize;
                for _ in 0..count {
                    parameters.push(Cow::Owned(String::read(reader)?));
                }
            }
        }

        let xuid = String::read(reader)?;
        let platform_chat_id = String::read(reader)?;

        let filtered_message = bool::read(reader)?
            .then(|| String::read(reader).map(Cow::Owned))
            .transpose()?;

        Ok(Self {
            needs_translation,
            r#type,
            source_name: Cow::Owned(source_name),
            message: Cow::Owned(message),
            parameters,
            xuid: Cow::Owned(xuid),
            platform_chat_id: Cow::Owned(platform_chat_id),
            filtered_message,
        })
    }
}

impl PacketWrite for SText<'_> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.needs_translation.write(writer)?;

        let category = self.get_category();
        category.write(writer)?;

        self.r#type.write(writer)?;

        match self.r#type {
            TextPacketType::Chat | TextPacketType::Whisper | TextPacketType::Announcement => {
                self.source_name.as_ref().write(writer)?;
                self.message.as_ref().write(writer)?;
            }
            TextPacketType::Raw
            | TextPacketType::Tip
            | TextPacketType::System
            | TextPacketType::JsonWhisper
            | TextPacketType::Json
            | TextPacketType::JsonAnnouncement => {
                self.message.as_ref().write(writer)?;
            }
            TextPacketType::Translation | TextPacketType::Popup | TextPacketType::JukeboxPopup => {
                self.message.as_ref().write(writer)?;
                VarUInt(self.parameters.len() as u32).write(writer)?;
                for param in &self.parameters {
                    param.as_ref().write(writer)?;
                }
            }
        }

        self.xuid.as_ref().write(writer)?;
        self.platform_chat_id.as_ref().write(writer)?;

        if let Some(msg) = &self.filtered_message {
            true.write(writer)?;
            msg.as_ref().write(writer)?;
        } else {
            false.write(writer)?;
        }

        Ok(())
    }
}

/// Formatting type and destination UI layer for Bedrock text packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PacketRead, PacketReadSlice, PacketWrite)]
#[repr(u8)]
pub enum TextPacketType {
    /// Unformatted raw text message.
    Raw = 0,
    /// Standard player chat message.
    Chat = 1,
    /// Translatable localization key with arguments.
    Translation = 2,
    /// Brief popup message displayed above the action bar.
    Popup = 3,
    /// Jukebox disc track notification popup.
    JukeboxPopup = 4,
    /// Small tip text displayed above the hotbar.
    Tip = 5,
    /// Server system notification message.
    System = 6,
    /// Private whisper message directed to a specific player.
    Whisper = 7,
    /// Global server announcement message.
    Announcement = 8,
    /// Formatted JSON whisper message.
    JsonWhisper = 9,
    /// Formatted raw JSON message.
    Json = 10,
    /// Formatted JSON announcement.
    JsonAnnouncement = 11,
}
