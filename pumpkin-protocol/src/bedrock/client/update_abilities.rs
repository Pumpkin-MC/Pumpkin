use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[packet(187)]
#[derive(PacketWrite)]
pub struct CUpdateAbilities {
    pub target_player_raw_id: i64,
    pub player_permission: u8,
    pub command_permission: u8,
    pub layers: Vec<AbilityLayer>,
}

#[derive(PacketWrite)]
pub struct AbilityLayer {
    // https://mojang.github.io/bedrock-protocol-docs/html/SerializedAbilitiesData__SerializedLayer.html
    pub serialized_layer: u16,
    pub abilities_set: u32,
    pub ability_value: u32,
    pub fly_speed: f32,
    pub vertical_fly_speed: f32,
    pub walk_speed: f32,
}

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
