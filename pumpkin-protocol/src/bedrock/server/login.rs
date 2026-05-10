use pumpkin_macros::packet;
use serde::Deserialize;
use std::io::{Error, ErrorKind, Read};

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

#[packet(1)]
pub struct SLogin {
    // https://mojang.github.io/bedrock-protocol-docs/html/LoginPacket.html
    pub protocol_version: i32,

    // https://mojang.github.io/bedrock-protocol-docs/html/connectionRequest.html
    pub jwt: Vec<u8>,
    pub raw_token: Vec<u8>,
}

impl PacketRead for SLogin {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        const MAX_TOKEN_SIZE: usize = 2000 * 1024; // 2MB limit
        let protocol_version = i32::read_be(reader)?;
        let _len = VarUInt::read(reader)?;

        let jwt_len = u32::read(reader)?;
        if jwt_len as usize > MAX_TOKEN_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "JWT length exceeds limit",
            ));
        }
        let mut jwt = vec![0; jwt_len as _];
        reader.read_exact(&mut jwt)?;

        let raw_token_len = u32::read(reader)?;
        if raw_token_len as usize > MAX_TOKEN_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Raw token length exceeds limit",
            ));
        }
        let mut raw_token = vec![0; raw_token_len as _];
        reader.read_exact(&mut raw_token)?;

        Ok(Self {
            protocol_version,
            jwt,
            raw_token,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SkinAnimation {
    pub frames: f64,
    pub image: String,
    pub image_height: i32,
    pub image_width: i32,
    #[serde(rename = "Type")]
    pub animation_type: i32, // 'type' is a reserved keyword in Rust
    pub animation_expression: i32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PersonaPiece {
    #[serde(rename = "IsDefault")]
    pub is_default: bool,
    #[serde(rename = "PackId")]
    pub pack_id: String,
    #[serde(rename = "PieceId")]
    pub piece_id: String,
    pub piece_type: String,
    #[serde(rename = "ProductId")]
    pub product_id: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct PersonaPieceTintColour {
    #[serde(rename = "Colors")]
    pub colours: [String; 4],
    pub piece_type: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ClientData {
    pub client_random_id: i64,
    #[serde(rename = "DeviceOS")]
    pub device_os: i32,
    #[serde(rename = "DeviceId")]
    pub device_id: String,
    pub game_version: String,
    pub language_code: String,
    pub current_input_mode: i32,
    pub default_input_mode: i32,
    #[serde(rename = "UIProfile")]
    pub ui_profile: i32,
    pub server_address: String,

    #[serde(default)]
    pub device_model: String,
    #[serde(rename = "GuiScale", default)]
    pub gui_scale: i32,
    #[serde(default)]
    pub is_editor_mode: bool,
    #[serde(default)]
    pub max_view_distance: i32,
    #[serde(default)]
    pub memory_tier: i32,
    #[serde(default)]
    pub platform_type: i32,
    #[serde(default)]
    pub graphics_mode: i32,
    #[serde(default)]
    pub compatible_with_client_side_chunk_gen: bool,

    #[serde(rename = "PlatformOfflineId", default)]
    pub platform_offline_id: String,
    #[serde(rename = "PlatformOnlineId", default)]
    pub platform_online_id: String,
    #[serde(rename = "PlatformUserId", default)]
    pub platform_user_id: String,
    #[serde(rename = "SelfSignedId", default)]
    pub self_signed_id: String,
    #[serde(rename = "PlayFabId", default)]
    pub play_fab_id: String,
    #[serde(default)]
    pub third_party_name: String,
    #[serde(default)]
    pub third_party_name_only: bool,

    #[serde(rename = "SkinId", default)]
    pub skin_id: String,
    #[serde(default)]
    pub skin_data: String,
    #[serde(default)]
    pub skin_image_height: i32,
    #[serde(default)]
    pub skin_image_width: i32,
    #[serde(rename = "SkinColor", default)]
    pub skin_colour: String,
    #[serde(default)]
    pub arm_size: String,
    #[serde(default)]
    pub persona_skin: bool,
    #[serde(default)]
    pub premium_skin: bool,
    #[serde(default)]
    pub trusted_skin: bool,
    #[serde(default)]
    pub override_skin: bool,

    #[serde(default)]
    pub cape_data: String,
    #[serde(rename = "CapeId", default)]
    pub cape_id: String,
    #[serde(default)]
    pub cape_image_height: i32,
    #[serde(default)]
    pub cape_image_width: i32,
    #[serde(default)]
    pub cape_on_classic_skin: bool,
    #[serde(rename = "SkinGeometryData", default)]
    pub skin_geometry: String,
    #[serde(rename = "SkinGeometryDataEngineVersion", default)]
    pub skin_geometry_version: String,
    #[serde(default)]
    pub skin_resource_patch: String,

    #[serde(default)]
    pub animated_image_data: Vec<SkinAnimation>,
    #[serde(default)]
    pub skin_animation_data: String,
    #[serde(default)]
    pub persona_pieces: Vec<PersonaPiece>,
    #[serde(rename = "PieceTintColors", default)]
    pub piece_tint_colours: Vec<PersonaPieceTintColour>,
}
