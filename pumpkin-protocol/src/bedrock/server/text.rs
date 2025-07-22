use std::io::{Error, Read, Write};

use pumpkin_macros::packet;

use crate::{
    codec::var_uint::VarUInt,
    serial::{PacketRead, PacketWrite},
};

#[derive(Debug)]
#[packet(9)]
pub struct SText {
    // https://mojang.github.io/bedrock-protocol-docs/html/TextPacket.html
    pub r#type: TextPacketType,
    pub localize: bool,
    pub player_name: String,
    pub message: String,
    pub parameters: Vec<String>,
    pub sender_xuid: String,
    pub platform_id: String,
    pub filtered_message: String,
}

impl SText {
    pub fn new(message: String, player_name: String) -> Self {
        Self {
            r#type: TextPacketType::Chat,
            localize: false,
            player_name,
            message,
            parameters: Vec::with_capacity(0),
            sender_xuid: String::with_capacity(0),
            platform_id: String::with_capacity(0),
            filtered_message: String::with_capacity(0),
        }
    }
}

impl PacketRead for SText {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let typ = TextPacketType::read(reader)?;
        let localize = bool::read(reader)?;

        let mut player_name = String::with_capacity(0);
        let message;
        let mut parameters = Vec::with_capacity(0);

        match typ {
            TextPacketType::Raw
            | TextPacketType::Tip
            | TextPacketType::SystemMessage
            | TextPacketType::TextObjectWhisper
            | TextPacketType::TextObject
            | TextPacketType::TextObjectAnnouncement => {
                message = String::read(reader)?;
            }
            TextPacketType::Chat | TextPacketType::Whisper | TextPacketType::Announcement => {
                player_name = String::read(reader)?;
                message = String::read(reader)?;
            }
            TextPacketType::Translate | TextPacketType::Popup | TextPacketType::JukeboxPopup => {
                message = String::read(reader)?;
                let count = VarUInt::read(reader)?.0 as usize;
                parameters = Vec::with_capacity(count);
                for _ in 0..count {
                    parameters.push(String::read(reader)?);
                }
            }
        }
        let sender_xuid = String::read(reader)?;
        let platform_id = String::read(reader)?;
        let filtered_message = String::read(reader)?;

        Ok(Self {
            r#type: typ,
            localize,
            player_name,
            message,
            parameters,
            sender_xuid,
            platform_id,
            filtered_message,
        })
    }
}

impl PacketWrite for SText {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.r#type.write(writer)?;
        self.localize.write(writer)?;
        match self.r#type {
            TextPacketType::Raw
            | TextPacketType::Tip
            | TextPacketType::SystemMessage
            | TextPacketType::TextObjectWhisper
            | TextPacketType::TextObject
            | TextPacketType::TextObjectAnnouncement => {
                self.message.write(writer)?;
            }
            TextPacketType::Chat | TextPacketType::Whisper | TextPacketType::Announcement => {
                self.player_name.write(writer)?;
                self.message.write(writer)?;
            }
            TextPacketType::Translate | TextPacketType::Popup | TextPacketType::JukeboxPopup => {
                self.message.write(writer)?;
                VarUInt(self.parameters.len() as u32).write(writer)?;
                for param in &self.parameters {
                    param.write(writer)?;
                }
            }
        }
        self.sender_xuid.write(writer)?;
        self.platform_id.write(writer)?;
        self.filtered_message.write(writer)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum TextPacketType {
    Raw = 0,
    Chat = 1,
    Translate = 2,
    Popup = 3,
    JukeboxPopup = 4,
    Tip = 5,
    SystemMessage = 6,
    Whisper = 7,
    Announcement = 8,
    TextObjectWhisper = 9,
    TextObject = 10,
    TextObjectAnnouncement = 11,
}

impl PacketRead for TextPacketType {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        Ok(match u8::read(reader)? {
            0 => Self::Raw,
            1 => Self::Chat,
            2 => Self::Translate,
            3 => Self::Popup,
            4 => Self::JukeboxPopup,
            5 => Self::Tip,
            6 => Self::SystemMessage,
            7 => Self::Whisper,
            8 => Self::Announcement,
            9 => Self::TextObjectWhisper,
            10 => Self::TextObject,
            11 => Self::TextObjectAnnouncement,
            _ => return Err(Error::new(std::io::ErrorKind::InvalidData, "")),
        })
    }
}

impl PacketWrite for TextPacketType {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)
    }
}
