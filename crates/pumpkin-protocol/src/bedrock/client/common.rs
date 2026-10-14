use pumpkin_util::GameMode;

use crate::{codec::var_long::VarLong, serial::PacketWrite};

/// Operating system or client platform identifier reported by the client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
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
    Nintendo = 12,
    Xbox = 13,
    WindowsPhone = 14,
    Linux = 15,
}

/// Game mode identifier used in Bedrock protocol packets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
#[repr(i32)]
#[serial(varint)]
pub enum GameType {
    Unknown = -1,
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Default = 5,
    Spectator = 6,
    //WorldDefault = 0,
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

/// Serialized abilities data containing permissions and ability bitmasks for a player.
#[derive(Clone, PacketWrite)]
pub struct SerializedAbilitiesData {
    /// Unique entity ID of the target player.
    pub target_player_raw_id: i64,
    /// Permission level shown in the player list.
    pub player_permissions: PlayerPermissionLevel,
    /// Level determining what command categories the player can invoke.
    pub command_permissions: CommandPermissionLevel,
    /// Ability layers specifying capabilities (flight, invulnerability, etc.).
    pub layers: Vec<SerializedAbilitiesDataSerializedLayer>,
}

/// Player permission level representing their world access rights.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
#[repr(i8)]
pub enum PlayerPermissionLevel {
    Visitor = 0,
    Member = 1,
    Operator = 2,
    Custom = 3,
}

/// Command permission level determining allowed command execution scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PacketWrite)]
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

/// A single ability layer defining movement speeds and allowed action bitflags.
#[derive(Default, Clone, PacketWrite)]
pub struct SerializedAbilitiesDataSerializedLayer {
    /// Layer type index (such as base, spectator, or creative).
    pub serialized_layer: u16,
    /// Bitmask of abilities configured for this layer.
    pub abilities_set: u32,
    /// Current boolean values for the configured abilities bitmask.
    pub ability_value: u32,
    /// Flying speed multiplier.
    pub fly_speed: f32,
    /// Vertical flying speed multiplier.
    pub vertical_fly_speed: f32,
    /// Walking speed multiplier.
    pub walk_speed: f32,
}

/// Describes an attachment link between two entities, such as a player riding a vehicle.
#[derive(Default, Clone, PacketWrite)]
pub struct ActorLink {
    /// Unique entity ID of the vehicle / entity being ridden.
    pub ridden_unique_id: VarLong,
    /// Unique entity ID of the passenger / rider.
    pub rider_unique_id: VarLong,
    /// Type of link (e.g. 0 for passenger, 1 for vehicle).
    pub link_type: u8,
    /// Whether the link takes effect immediately without smooth transition.
    pub immediate: bool,
    /// Whether the link was initiated by the passenger entity.
    pub rider_initiated: bool,
    /// Rotational velocity applied to the vehicle.
    pub vehicle_angular_velocity: f32,
}
