use std::io::{Error, Write};

use pumpkin_util::GameMode;

use crate::{
    codec::{var_int::VarInt, var_long::VarLong},
    serial::PacketWrite,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BuildPlatform {
    Unknown = -1,
    Google = 1,
    Ios = 2,
    Osx = 3,
    Amazon = 4,
    GearVr = 5,
    Uwp = 7,
    Win32 = 8,
    Dedicated = 9,
    TvOs = 10,
    Sony = 11,
    Nx = 12,
    Xbox = 13,
    WindowsPhone = 14,
    Linux = 15,
}

impl PacketWrite for BuildPlatform {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as i32).write(writer)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum GameType {
    Unknown = -1,
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Default = 5,
    Spectator = 6,
    //WorldDefault = 0,
}

impl PacketWrite for GameType {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(*self as i32).write(writer)
    }
}

impl From<GameMode> for GameType {
    fn from(value: GameMode) -> Self {
        match value {
            GameMode::Survival => Self::Survival,
            GameMode::Creative => Self::Creative,
            GameMode::Adventure => Self::Adventure,
            GameMode::Spectator => Self::Spectator,
        }
    }
}

#[derive(Clone, PacketWrite)]
pub struct SerializedAbilitiesData {
    pub target_player_raw_id: i64,
    pub player_permissions: PlayerPermissionLevel,
    pub command_permissions: CommandPermissionLevel,
    pub layers: Vec<SerializedAbilitiesDataSerializedLayer>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i8)]
pub enum PlayerPermissionLevel {
    Visitor = 0,
    Member = 1,
    Operator = 2,
    Custom = 3,
}

impl PacketWrite for PlayerPermissionLevel {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as i8).write(writer)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandPermissionLevel {
    Any = 0,
    GameDirectors = 1,
    Admin = 2,
    Host = 3,
    Owner = 4,
    Internal = 5,
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for CommandPermissionLevel {
    fn to_string(&self) -> String {
        match self {
            Self::Any => "any",
            Self::GameDirectors => "gamedirectors",
            Self::Admin => "admin",
            Self::Host => "host",
            Self::Owner => "owner",
            Self::Internal => "internal",
        }
        .into()
    }
}

impl PacketWrite for CommandPermissionLevel {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)
    }
}

#[derive(Default, Clone, PacketWrite)]
pub struct SerializedAbilitiesDataSerializedLayer {
    pub serialized_layer: u16,
    pub abilities_set: u32,
    pub ability_value: u32,
    pub fly_speed: f32,
    pub vertical_fly_speed: f32,
    pub walk_speed: f32,
}

#[derive(Default, Clone, PacketWrite)]
pub struct ActorLink {
    pub ridden_unique_id: VarLong,
    pub rider_unique_id: VarLong,
    pub link_type: u8,
    pub immediate: bool,
    pub rider_initiated: bool,
    pub vehicle_angular_velocity: f32,
}
