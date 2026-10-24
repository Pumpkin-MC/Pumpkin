// Last verified for v2169

use pumpkin_macros::packet;

use crate::{bedrock::client::SerializedAbilitiesData, serial::PacketWrite};

/// Updates player movement and interaction abilities such as flight and build permissions.
#[packet(187)]
#[derive(PacketWrite)]
pub struct CUpdateAbilities {
    /// Serialized ability layers and permission parameters.
    pub data: SerializedAbilitiesData,
}

// TODO: confirm these
/// Bit flags and identifiers for individual player abilities and privileges.
#[repr(u32)]
pub enum Ability {
    Build = 0,
    Mine = 1,
    DoorsAndSwitches = 2,
    OpenContainers = 3,
    AttackPlayers = 4,
    AttackMobs = 5,
    OperatorCommands = 6,
    Teleport = 7,
    Invulnerable = 8,
    Flying = 9,
    MayFly = 10,
    Instabuild = 11,
    Lightning = 12,
    FlySpeed = 13,
    WalkSpeed = 14,
    Muted = 15,
    WorldBuilder = 16,
    NoClip = 17,
    PrivilegedBuilder = 18,
    VerticalFlySpeed = 19,
    AbilityCount = 20,
}
