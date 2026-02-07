use crate::{
    codec::var_uint::VarUInt,
    serial::{PacketRead, PacketWrite},
};
use pumpkin_macros::packet;
use std::io::{Error, ErrorKind, Read, Write};

#[derive(Debug)]
#[packet(9)]
pub struct SText {
    pub needs_translation: bool,
    pub r#type: TextPacketType,
    pub source_name: String,
    pub message: String,
    pub parameters: Vec<String>,
    pub xuid: String,
    pub platform_chat_id: String,
    pub filtered_message: Option<String>,
}

impl SText {
    #[must_use]
    pub fn new(message: String, source_name: String) -> Self {
        Self {
            needs_translation: false,
            r#type: TextPacketType::Chat,
            source_name,
            message: if message.is_empty() {
                " ".to_string()
            } else {
                message
            },
            parameters: Vec::new(),
            xuid: String::new(),
            platform_chat_id: String::new(),
            filtered_message: None,
        }
    }

    #[must_use]
    pub fn system_message(message: String) -> Self {
        Self {
            needs_translation: false,
            r#type: TextPacketType::System,
            source_name: String::new(),
            message: if message.is_empty() {
                " ".to_string()
            } else {
                message
            },
            parameters: Vec::new(),
            xuid: String::new(),
            platform_chat_id: String::new(),
            filtered_message: None,
        }
    }

    const fn get_dummy_strings(category: u8) -> &'static [&'static str] {
        match category {
            0 => &[
                "raw",
                "tip",
                "systemMessage",
                "textObjectWhisper",
                "textObjectAnnouncement",
                "textObject",
            ],
            1 => &["chat", "whisper", "announcement"],
            2 => &["translate", "popup", "jukeboxPopup"],
            _ => &[],
        }
    }

    const fn get_category(&self) -> u8 {
        match self.r#type {
            TextPacketType::Raw
            | TextPacketType::Tip
            | TextPacketType::System
            | TextPacketType::JsonWhisper
            | TextPacketType::JsonAnnouncement
            | TextPacketType::Json => 0, // CATEGORY_MESSAGE_ONLY

            TextPacketType::Chat | TextPacketType::Whisper | TextPacketType::Announcement => 1, // CATEGORY_AUTHORED_MESSAGE

            TextPacketType::Translation | TextPacketType::Popup | TextPacketType::JukeboxPopup => 2, // CATEGORY_MESSAGE_WITH_PARAMETERS
        }
    }
}

impl PacketRead for SText {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let needs_translation = bool::read(reader)?;

        let category = u8::read(reader)?;

        let expected_dummies = Self::get_dummy_strings(category);
        for &expected in expected_dummies {
            let actual = String::read(reader)?;
            if actual != expected {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Dummy string mismatch: expected {expected}, got {actual}"),
                ));
            }
        }

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
                    parameters.push(String::read(reader)?);
                }
            }
        }

        let xuid = String::read(reader)?;
        let platform_chat_id = String::read(reader)?;

        let has_filtered = bool::read(reader)?;
        let filtered_message = if has_filtered {
            Some(String::read(reader)?)
        } else {
            None
        };

        Ok(Self {
            needs_translation,
            r#type,
            source_name,
            message,
            parameters,
            xuid,
            platform_chat_id,
            filtered_message,
        })
    }
}

impl PacketWrite for SText {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.needs_translation.write(writer)?;

        let category = self.get_category();
        category.write(writer)?;

        for &dummy in Self::get_dummy_strings(category) {
            dummy.to_string().write(writer)?;
        }

        self.r#type.write(writer)?;

        match self.r#type {
            TextPacketType::Chat | TextPacketType::Whisper | TextPacketType::Announcement => {
                self.source_name.write(writer)?;
                self.message.write(writer)?;
            }
            TextPacketType::Raw
            | TextPacketType::Tip
            | TextPacketType::System
            | TextPacketType::JsonWhisper
            | TextPacketType::Json
            | TextPacketType::JsonAnnouncement => {
                self.message.write(writer)?;
            }
            TextPacketType::Translation | TextPacketType::Popup | TextPacketType::JukeboxPopup => {
                self.message.write(writer)?;
                VarUInt(self.parameters.len() as u32).write(writer)?;
                for param in &self.parameters {
                    param.write(writer)?;
                }
            }
        }

        self.xuid.write(writer)?;
        self.platform_chat_id.write(writer)?;

        if let Some(msg) = &self.filtered_message {
            true.write(writer)?;
            msg.write(writer)?;
        } else {
            false.write(writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TextPacketType {
    Raw = 0,
    Chat = 1,
    Translation = 2,
    Popup = 3,
    JukeboxPopup = 4,
    Tip = 5,
    System = 6,
    Whisper = 7,
    Announcement = 8,
    JsonWhisper = 9,
    Json = 10,
    JsonAnnouncement = 11,
}

impl PacketWrite for TextPacketType {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)
    }
}

impl PacketRead for TextPacketType {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        match u8::read(reader)? {
            0 => Ok(Self::Raw),
            1 => Ok(Self::Chat),
            2 => Ok(Self::Translation),
            3 => Ok(Self::Popup),
            4 => Ok(Self::JukeboxPopup),
            5 => Ok(Self::Tip),
            6 => Ok(Self::System),
            7 => Ok(Self::Whisper),
            8 => Ok(Self::Announcement),
            9 => Ok(Self::JsonWhisper),
            10 => Ok(Self::Json),
            11 => Ok(Self::JsonAnnouncement),
            _ => Err(Error::new(ErrorKind::InvalidData, "Unknown Text Type")),
        }
    }
}
