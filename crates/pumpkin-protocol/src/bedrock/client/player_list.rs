use crate::{
    codec::{var_long::VarLong, var_uint::VarUInt},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use std::io::{Error, Write};
use uuid::Uuid;

use super::common::BuildPlatform;

const WIDE_SKIN_RESOURCE_PATCH: &[u8] = br#"{"geometry":{"default":"geometry.humanoid.custom"}}"#;
const SLIM_SKIN_RESOURCE_PATCH: &[u8] =
    br#"{"geometry":{"default":"geometry.humanoid.customSlim"}}"#;
const DEFAULT_SKIN_GEOMETRY: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/bedrock/player_geometry.json"
));

#[packet(63)]
pub struct CPlayerList {
    pub action: u8,
    pub entries: Vec<PlayerListEntry>,
}

impl PacketWrite for CPlayerList {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.entries.len() as u32).write(writer)?;
        for entry in &self.entries {
            match self.action {
                Self::ACTION_ADD => {
                    VarUInt(1).write(writer)?;
                    Self::ACTION_ADD.write(writer)?;
                    entry.write(writer)?;
                }
                Self::ACTION_REMOVE => {
                    VarUInt(0).write(writer)?;
                    Self::ACTION_REMOVE.write(writer)?;
                    entry.uuid.write(writer)?;
                }
                _ => return Err(Error::other("Invalid PlayerList action")),
            }
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
    pub build_platform: BuildPlatform,
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
        u32::from_be_bytes(self.player_color).write(writer)
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
    pub profile_hash: String,
}

impl Skin {
    #[must_use]
    pub fn steve() -> Self {
        Self {
            skin_id: "Standard_Custom".to_string(),
            play_fab_id: String::new(),
            resource_patch: WIDE_SKIN_RESOURCE_PATCH.to_vec(),
            image_width: 64,
            image_height: 64,
            // 64 * 64 * 4 = 16384 bytes of raw RGBA data
            // Fill with 255 so the skin is visible (solid white) instead of invisible (transparent)
            skin_data: vec![255; 16384],
            animations: Vec::new(),
            cape_width: 0,
            cape_height: 0,
            cape_data: Vec::new(),
            geometry_data: DEFAULT_SKIN_GEOMETRY.to_vec(),
            animation_data: Vec::new(),
            geometry_data_engine_version: b"1.26.40".to_vec(),
            cape_id: String::new(),
            full_id: "Standard_Custom".to_string(),
            arm_size: "wide".to_string(),
            skin_color: "#0".to_string(),
            persona_pieces: Vec::new(),
            piece_tint_colors: Vec::new(),
            is_premium: true,
            is_persona: false,
            persona_cape_on_classic: false,
            is_primary_user: false,
            override_appearance: true,
            is_trusted: true,
            profile_hash: String::new(),
        }
    }

    /// Selects the standard wide or slim player geometry while preserving the
    /// rest of the serialized skin.
    pub fn set_slim(&mut self, slim: bool) {
        self.arm_size = if slim { "slim" } else { "wide" }.to_string();
        self.resource_patch = if slim {
            SLIM_SKIN_RESOURCE_PATCH
        } else {
            WIDE_SKIN_RESOURCE_PATCH
        }
        .to_vec();
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
        VarUInt(self.animations.len() as u32).write(writer)?;
        for anim in &self.animations {
            anim.write(writer)?;
        }
        self.cape_width.write(writer)?;
        self.cape_height.write(writer)?;
        VarUInt(self.cape_data.len() as u32).write(writer)?;
        writer.write_all(&self.cape_data)?;
        VarUInt(self.geometry_data.len() as u32).write(writer)?;
        writer.write_all(&self.geometry_data)?;
        VarUInt(self.geometry_data_engine_version.len() as u32).write(writer)?;
        writer.write_all(&self.geometry_data_engine_version)?;
        VarUInt(self.animation_data.len() as u32).write(writer)?;
        writer.write_all(&self.animation_data)?;
        self.cape_id.write(writer)?;
        self.full_id.write(writer)?;
        u8::from(!self.arm_size.eq_ignore_ascii_case("slim")).write(writer)?;
        parse_color(&self.skin_color).write(writer)?;
        VarUInt(self.persona_pieces.len() as u32).write(writer)?;
        for piece in &self.persona_pieces {
            piece.write(writer)?;
        }
        VarUInt(self.piece_tint_colors.len() as u32).write(writer)?;
        for color in &self.piece_tint_colors {
            color.write(writer)?;
        }
        self.is_premium.write(writer)?;
        self.is_persona.write(writer)?;
        self.persona_cape_on_classic.write(writer)?;
        self.is_primary_user.write(writer)?;
        self.override_appearance.write(writer)?;
        self.is_trusted.to_string().write(writer)?;
        self.profile_hash.write(writer)
    }
}

#[derive(Clone)]
pub struct SkinAnimation {
    pub image_width: u32,
    pub image_height: u32,
    pub image_data: Vec<u8>,
    pub animation_type: u32,
    pub frames: f32,
    pub expression_type: u32,
}

impl PacketWrite for SkinAnimation {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.image_width.write(writer)?;
        self.image_height.write(writer)?;
        VarUInt(self.image_data.len() as u32).write(writer)?;
        writer.write_all(&self.image_data)?;
        VarUInt(self.animation_type).write(writer)?;
        self.frames.write(writer)?;
        VarUInt(self.expression_type).write(writer)
    }
}

#[derive(Clone, PacketWrite)]
pub struct PersonaPiece {
    pub piece_id: String,
    pub piece_type: i32,
    pub pack_id: Uuid,
    pub is_default: bool,
    pub product_id: String,
}

#[derive(Clone)]
pub struct PieceTintColor {
    pub piece_type: String,
    pub colors: [i32; 4],
}

impl PacketWrite for PieceTintColor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let piece_type = if self.piece_type == "persona_hand" {
            "hands"
        } else {
            self.piece_type
                .strip_prefix("persona_")
                .unwrap_or(&self.piece_type)
        };
        piece_type.write(writer)?;
        for color in self.colors {
            color.write(writer)?;
        }
        Ok(())
    }
}

fn parse_color(color: &str) -> i32 {
    let value = color.trim_start_matches('#');
    u32::from_str_radix(value, 16).unwrap_or_default() as i32
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_SKIN_GEOMETRY, SLIM_SKIN_RESOURCE_PATCH, Skin, WIDE_SKIN_RESOURCE_PATCH};

    #[test]
    fn fallback_skin_contains_the_geometry_it_references() {
        let skin = Skin::steve();

        assert_eq!(skin.resource_patch, WIDE_SKIN_RESOURCE_PATCH);
        assert_eq!(skin.geometry_data, DEFAULT_SKIN_GEOMETRY);
        assert!(
            String::from_utf8_lossy(&skin.geometry_data)
                .contains(r#""identifier":"geometry.humanoid.custom""#)
        );
        assert!(skin.override_appearance);
        assert!(skin.is_trusted);
    }

    #[test]
    fn slim_fallback_skin_references_the_slim_geometry() {
        let mut skin = Skin::steve();
        skin.set_slim(true);

        assert_eq!(skin.arm_size, "slim");
        assert_eq!(skin.resource_patch, SLIM_SKIN_RESOURCE_PATCH);
        assert!(
            String::from_utf8_lossy(&skin.geometry_data)
                .contains(r#""identifier":"geometry.humanoid.customSlim""#)
        );
    }
}
