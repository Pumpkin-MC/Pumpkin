/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageType {
    pub death_message_type: DeathMessageType,
    pub exhaustion: f32,
    pub effects: Option<DamageEffects>,
    pub message_id: &'static str,
    pub scaling: DamageScaling,
    pub id: u8,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DeathMessageType {
    Default,
    FallVariants,
    IntentionalGameDesign,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DamageEffects {
    Hurt,
    Thorns,
    Drowning,
    Burning,
    Poking,
    Freezing,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DamageScaling {
    Never,
    WhenCausedByLivingNonPlayer,
    Always,
}
impl DamageType {
    pub const ARROW: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "arrow",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 0,
    };
    pub const BAD_RESPAWN_POINT: DamageType = DamageType {
        death_message_type: DeathMessageType::IntentionalGameDesign,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "badRespawnPoint",
        scaling: DamageScaling::Always,
        id: 1,
    };
    pub const CACTUS: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "cactus",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 2,
    };
    pub const CAMPFIRE: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: Some(DamageEffects::Burning),
        message_id: "inFire",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 3,
    };
    pub const CRAMMING: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "cramming",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 4,
    };
    pub const DRAGON_BREATH: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "dragonBreath",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 5,
    };
    pub const DROWN: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: Some(DamageEffects::Drowning),
        message_id: "drown",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 6,
    };
    pub const DRY_OUT: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "dryout",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 7,
    };
    pub const ENDER_PEARL: DamageType = DamageType {
        death_message_type: DeathMessageType::FallVariants,
        exhaustion: 0f32,
        effects: None,
        message_id: "fall",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 8,
    };
    pub const EXPLOSION: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "explosion",
        scaling: DamageScaling::Always,
        id: 9,
    };
    pub const FALL: DamageType = DamageType {
        death_message_type: DeathMessageType::FallVariants,
        exhaustion: 0f32,
        effects: None,
        message_id: "fall",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 10,
    };
    pub const FALLING_ANVIL: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "anvil",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 11,
    };
    pub const FALLING_BLOCK: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "fallingBlock",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 12,
    };
    pub const FALLING_STALACTITE: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "fallingStalactite",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 13,
    };
    pub const FIREBALL: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: Some(DamageEffects::Burning),
        message_id: "fireball",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 14,
    };
    pub const FIREWORKS: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "fireworks",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 15,
    };
    pub const FLY_INTO_WALL: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "flyIntoWall",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 16,
    };
    pub const FREEZE: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: Some(DamageEffects::Freezing),
        message_id: "freeze",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 17,
    };
    pub const GENERIC: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "generic",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 18,
    };
    pub const GENERIC_KILL: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "genericKill",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 19,
    };
    pub const HOT_FLOOR: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: Some(DamageEffects::Burning),
        message_id: "hotFloor",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 20,
    };
    pub const IN_FIRE: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: Some(DamageEffects::Burning),
        message_id: "inFire",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 21,
    };
    pub const IN_WALL: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "inWall",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 22,
    };
    pub const INDIRECT_MAGIC: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "indirectMagic",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 23,
    };
    pub const LAVA: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: Some(DamageEffects::Burning),
        message_id: "lava",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 24,
    };
    pub const LIGHTNING_BOLT: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "lightningBolt",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 25,
    };
    pub const MACE_SMASH: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "mace_smash",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 26,
    };
    pub const MAGIC: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "magic",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 27,
    };
    pub const MOB_ATTACK: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "mob",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 28,
    };
    pub const MOB_ATTACK_NO_AGGRO: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "mob",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 29,
    };
    pub const MOB_PROJECTILE: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "mob",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 30,
    };
    pub const ON_FIRE: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: Some(DamageEffects::Burning),
        message_id: "onFire",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 31,
    };
    pub const OUT_OF_WORLD: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "outOfWorld",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 32,
    };
    pub const OUTSIDE_BORDER: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "outsideBorder",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 33,
    };
    pub const PLAYER_ATTACK: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "player",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 34,
    };
    pub const PLAYER_EXPLOSION: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "explosion.player",
        scaling: DamageScaling::Always,
        id: 35,
    };
    pub const SONIC_BOOM: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "sonic_boom",
        scaling: DamageScaling::Always,
        id: 36,
    };
    pub const SPEAR: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "spear",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 37,
    };
    pub const SPIT: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "mob",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 38,
    };
    pub const STALAGMITE: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "stalagmite",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 39,
    };
    pub const STARVE: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "starve",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 40,
    };
    pub const STING: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "sting",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 41,
    };
    pub const SWEET_BERRY_BUSH: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: Some(DamageEffects::Poking),
        message_id: "sweetBerryBush",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 42,
    };
    pub const THORNS: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: Some(DamageEffects::Thorns),
        message_id: "thorns",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 43,
    };
    pub const THROWN: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "thrown",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 44,
    };
    pub const TRIDENT: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "trident",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 45,
    };
    pub const UNATTRIBUTED_FIREBALL: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: Some(DamageEffects::Burning),
        message_id: "onFire",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 46,
    };
    pub const WIND_CHARGE: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "mob",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 47,
    };
    pub const WITHER: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0f32,
        effects: None,
        message_id: "wither",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 48,
    };
    pub const WITHER_SKULL: DamageType = DamageType {
        death_message_type: DeathMessageType::Default,
        exhaustion: 0.1f32,
        effects: None,
        message_id: "witherSkull",
        scaling: DamageScaling::WhenCausedByLivingNonPlayer,
        id: 49,
    };
    #[doc = r" Try to parse a damage type from a resource location string."]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "arrow" => Some(Self::ARROW),
            "bad_respawn_point" => Some(Self::BAD_RESPAWN_POINT),
            "cactus" => Some(Self::CACTUS),
            "campfire" => Some(Self::CAMPFIRE),
            "cramming" => Some(Self::CRAMMING),
            "dragon_breath" => Some(Self::DRAGON_BREATH),
            "drown" => Some(Self::DROWN),
            "dry_out" => Some(Self::DRY_OUT),
            "ender_pearl" => Some(Self::ENDER_PEARL),
            "explosion" => Some(Self::EXPLOSION),
            "fall" => Some(Self::FALL),
            "falling_anvil" => Some(Self::FALLING_ANVIL),
            "falling_block" => Some(Self::FALLING_BLOCK),
            "falling_stalactite" => Some(Self::FALLING_STALACTITE),
            "fireball" => Some(Self::FIREBALL),
            "fireworks" => Some(Self::FIREWORKS),
            "fly_into_wall" => Some(Self::FLY_INTO_WALL),
            "freeze" => Some(Self::FREEZE),
            "generic" => Some(Self::GENERIC),
            "generic_kill" => Some(Self::GENERIC_KILL),
            "hot_floor" => Some(Self::HOT_FLOOR),
            "in_fire" => Some(Self::IN_FIRE),
            "in_wall" => Some(Self::IN_WALL),
            "indirect_magic" => Some(Self::INDIRECT_MAGIC),
            "lava" => Some(Self::LAVA),
            "lightning_bolt" => Some(Self::LIGHTNING_BOLT),
            "mace_smash" => Some(Self::MACE_SMASH),
            "magic" => Some(Self::MAGIC),
            "mob_attack" => Some(Self::MOB_ATTACK),
            "mob_attack_no_aggro" => Some(Self::MOB_ATTACK_NO_AGGRO),
            "mob_projectile" => Some(Self::MOB_PROJECTILE),
            "on_fire" => Some(Self::ON_FIRE),
            "out_of_world" => Some(Self::OUT_OF_WORLD),
            "outside_border" => Some(Self::OUTSIDE_BORDER),
            "player_attack" => Some(Self::PLAYER_ATTACK),
            "player_explosion" => Some(Self::PLAYER_EXPLOSION),
            "sonic_boom" => Some(Self::SONIC_BOOM),
            "spear" => Some(Self::SPEAR),
            "spit" => Some(Self::SPIT),
            "stalagmite" => Some(Self::STALAGMITE),
            "starve" => Some(Self::STARVE),
            "sting" => Some(Self::STING),
            "sweet_berry_bush" => Some(Self::SWEET_BERRY_BUSH),
            "thorns" => Some(Self::THORNS),
            "thrown" => Some(Self::THROWN),
            "trident" => Some(Self::TRIDENT),
            "unattributed_fireball" => Some(Self::UNATTRIBUTED_FIREBALL),
            "wind_charge" => Some(Self::WIND_CHARGE),
            "wither" => Some(Self::WITHER),
            "wither_skull" => Some(Self::WITHER_SKULL),
            _ => None,
        }
    }
}
