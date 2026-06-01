use crate::{
    codec::{var_long::VarLong, var_uint::VarUInt},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use std::io::{Error, Write};
use uuid::Uuid;

#[packet(63)]
pub struct CPlayerList {
    pub action: u8,
    pub entries: Vec<PlayerListEntry>,
}

impl PacketWrite for CPlayerList {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.action.write(writer)?;
        VarUInt(self.entries.len() as u32).write(writer)?;
        match self.action {
            Self::ACTION_ADD => {
                for entry in &self.entries {
                    entry.write(writer)?;
                }
                for entry in &self.entries {
                    entry.skin.is_trusted.write(writer)?;
                }
            }
            Self::ACTION_REMOVE => {
                for entry in &self.entries {
                    entry.uuid.write(writer)?;
                }
            }
            _ => return Err(Error::other("Invalid PlayerList action")),
        }
        Ok(())
    }
}

impl CPlayerList {
    pub const ACTION_ADD: u8 = 0;
    pub const ACTION_REMOVE: u8 = 1;
}

pub struct PlayerListEntry {
    pub uuid: Uuid,
    pub entity_unique_id: VarLong,
    pub username: String,
    pub xuid: String,
    pub platform_chat_id: String,
    pub build_platform: i32,
    pub skin: Skin,
    pub is_teacher: bool,
    pub is_host: bool,
    pub is_sub_client: bool,
    pub player_color: [u8; 4], // ARGB
}

impl PacketWrite for PlayerListEntry {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.uuid.write(writer)?;
        self.entity_unique_id.write(writer)?;
        self.username.write(writer)?;
        self.xuid.write(writer)?;
        self.platform_chat_id.write(writer)?;
        self.build_platform.write(writer)?;
        self.skin.write(writer)?;
        self.is_teacher.write(writer)?;
        self.is_host.write(writer)?;
        self.is_sub_client.write(writer)?;
        self.player_color.write(writer)
    }
}

#[derive(Clone)]
pub struct Skin {
    pub skin_id: String,
    pub play_fab_id: String,
    pub resource_patch: Vec<u8>,
    pub image_width: u32,
    pub image_height: u32,
    pub skin_data: Vec<u8>,
    pub animations: Vec<SkinAnimation>,
    pub cape_width: u32,
    pub cape_height: u32,
    pub cape_data: Vec<u8>,
    pub geometry_data: Vec<u8>,
    pub animation_data: Vec<u8>,
    pub geometry_data_engine_version: Vec<u8>,
    pub cape_id: String,
    pub full_id: String,
    pub arm_size: String,
    pub skin_color: String,
    pub persona_pieces: Vec<PersonaPiece>,
    pub piece_tint_colors: Vec<PieceTintColor>,
    pub is_premium: bool,
    pub is_persona: bool,
    pub persona_cape_on_classic: bool,
    pub is_primary_user: bool,
    pub override_appearance: bool,
    pub is_trusted: bool,
}

impl Skin {
    #[must_use]
    pub fn steve() -> Self {
        Self {
            skin_id: "Standard_Custom".to_string(),
            play_fab_id: String::new(),
            resource_patch: r#"{"geometry":{"default":"geometry.humanoid.custom"}}"#.into(),
            image_width: 64,
            image_height: 64,
            // 64 * 64 * 4 = 16384 bytes of raw RGBA data
            skin_data: vec![0; 16384],
            animations: Vec::new(),
            cape_width: 0,
            cape_height: 0,
            cape_data: Vec::new(),
            geometry_data: Vec::new(),
            animation_data: Vec::new(),
            geometry_data_engine_version: Vec::new(),
            cape_id: String::new(),
            full_id: "Standard_Custom".to_string(),
            arm_size: "wide".to_string(),
            skin_color: "#0".to_string(),
            persona_pieces: Vec::new(),
            piece_tint_colors: Vec::new(),
            is_premium: false,
            is_persona: false,
            persona_cape_on_classic: false,
            is_primary_user: false,
            override_appearance: false,
            is_trusted: true,
        }
    }
}

impl PacketWrite for Skin {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.skin_id.write(writer)?;
        self.play_fab_id.write(writer)?;
        VarUInt(self.resource_patch.len() as u32).write(writer)?;
        writer.write_all(&self.resource_patch)?;
        self.image_width.write(writer)?;
        self.image_height.write(writer)?;
        VarUInt(self.skin_data.len() as u32).write(writer)?;
        writer.write_all(&self.skin_data)?;
        (self.animations.len() as u32).write(writer)?;
        for anim in &self.animations {
            anim.write(writer)?;
        }
        self.cape_width.write(writer)?;
        self.cape_height.write(writer)?;
        VarUInt(self.cape_data.len() as u32).write(writer)?;
        writer.write_all(&self.cape_data)?;
        VarUInt(self.geometry_data.len() as u32).write(writer)?;
        writer.write_all(&self.geometry_data)?;
        VarUInt(self.animation_data.len() as u32).write(writer)?;
        writer.write_all(&self.animation_data)?;
        VarUInt(self.geometry_data_engine_version.len() as u32).write(writer)?;
        writer.write_all(&self.geometry_data_engine_version)?;
        self.cape_id.write(writer)?;
        self.full_id.write(writer)?;
        self.arm_size.write(writer)?;
        self.skin_color.write(writer)?;
        (self.persona_pieces.len() as u32).write(writer)?;
        for piece in &self.persona_pieces {
            piece.write(writer)?;
        }
        (self.piece_tint_colors.len() as u32).write(writer)?;
        for color in &self.piece_tint_colors {
            color.write(writer)?;
        }
        self.is_premium.write(writer)?;
        self.is_persona.write(writer)?;
        self.persona_cape_on_classic.write(writer)?;
        self.is_primary_user.write(writer)?;
        self.override_appearance.write(writer)
    }
}

#[derive(Clone, PacketWrite)]
pub struct SkinAnimation {
    pub image_width: u32,
    pub image_height: u32,
    pub image_data: Vec<u8>,
    pub animation_type: u32,
    pub frames: f32,
    pub expression_type: u32,
}

#[derive(Clone, PacketWrite)]
pub struct PersonaPiece {
    pub piece_id: String,
    pub piece_type: String,
    pub pack_id: String,
    pub is_default: bool,
    pub product_id: String,
}

#[derive(Clone, PacketWrite)]
pub struct PieceTintColor {
    pub piece_type: String,
    pub colors: Vec<String>,
}
