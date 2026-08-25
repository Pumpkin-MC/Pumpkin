/* This file is generated. Do not edit manually. */
use crate::meta_data_type::MetaDataType;
use pumpkin_util::version::JavaMinecraftVersion;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TrackedId {
    pub v26_2: u8,
}
impl TrackedId {
    #[must_use]
    pub const fn get(&self, version: &JavaMinecraftVersion) -> u8 {
        match version {
            pumpkin_util::version::JavaMinecraftVersion::V_26_2 => self.v26_2,
            _ => self.v26_2,
        }
    }
}
impl From<TrackedId> for u8 {
    fn from(id: TrackedId) -> u8 {
        id.v26_2
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TrackedData {
    pub id: TrackedId,
    pub r#type: MetaDataType,
}
impl TrackedData {
    #[must_use]
    pub const fn new(id: TrackedId, r#type: MetaDataType) -> Self {
        Self { id, r#type }
    }
    #[must_use]
    pub const fn get(&self, version: &JavaMinecraftVersion) -> u8 {
        self.id.get(version)
    }
}
pub mod abstract_arrow {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const IN_GROUND: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const PIERCE_LEVEL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod abstract_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod abstract_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod abstract_chested_horse {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_CHEST: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_CHEST: TrackedData = DATA_ID_CHEST;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_cow {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_cube_mob {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_SIZE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_fish {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const FROM_BUCKET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_golem {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_horse {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_hurting_projectile {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod abstract_illager {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const IS_CELEBRATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_minecart {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod abstract_minecart_container {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod abstract_nautilus {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DASH: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_OWNERUUID_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const OWNERUUID_ID: TrackedData = DATA_OWNERUUID_ID;
    pub const DATA_OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNER_UUID: TrackedData = DATA_OWNERUUID_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_piglin {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_IMMUNE_TO_ZOMBIFICATION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const IMMUNE_TO_ZOMBIFICATION: TrackedData = DATA_IMMUNE_TO_ZOMBIFICATION;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_schooling_fish {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const FROM_BUCKET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_skeleton {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_thrown_potion {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod abstract_villager {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_UNHAPPY_COUNTER: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const UNHAPPY_COUNTER: TrackedData = DATA_UNHAPPY_COUNTER;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod abstract_wind_charge {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod acacia_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod acacia_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod ageable_mob {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod ageable_water_creature {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod allay {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CAN_DUPLICATE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DANCING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CAN_DUPLICATE: TrackedData = DATA_CAN_DUPLICATE;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DANCING: TrackedData = DATA_DANCING;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod ambient_creature {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod animal {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod area_effect_cloud {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_PARTICLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_RADIUS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_WAITING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const PARTICLE: TrackedData = DATA_PARTICLE;
    pub const POSE: TrackedData = DATA_POSE;
    pub const RADIUS: TrackedData = DATA_RADIUS;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const WAITING: TrackedData = DATA_WAITING;
}
pub mod armadillo {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const ARMADILLO_STATE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::ARMADILLO_STATE,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod armor_stand {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BODY_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::ROTATIONS,
    };
    pub const DATA_CLIENT_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEAD_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::ROTATIONS,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LEFT_ARM_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::ROTATIONS,
    };
    pub const DATA_LEFT_LEG_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::ROTATIONS,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_RIGHT_ARM_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::ROTATIONS,
    };
    pub const DATA_RIGHT_LEG_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::ROTATIONS,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BODY_POSE: TrackedData = DATA_BODY_POSE;
    pub const CLIENT_FLAGS: TrackedData = DATA_CLIENT_FLAGS;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEAD_POSE: TrackedData = DATA_HEAD_POSE;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LEFT_ARM_POSE: TrackedData = DATA_LEFT_ARM_POSE;
    pub const LEFT_LEG_POSE: TrackedData = DATA_LEFT_LEG_POSE;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const RIGHT_ARM_POSE: TrackedData = DATA_RIGHT_ARM_POSE;
    pub const RIGHT_LEG_POSE: TrackedData = DATA_RIGHT_LEG_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod arrow {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_EFFECT_COLOR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const IN_GROUND: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const PIERCE_LEVEL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod avatar {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_PLAYER_MAIN_HAND: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::HUMANOID_ARM,
    };
    pub const DATA_PLAYER_MODE_CUSTOMISATION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const PLAYER_MAIN_HAND: TrackedData = DATA_PLAYER_MAIN_HAND;
    pub const MAIN_ARM_ID: TrackedData = DATA_PLAYER_MAIN_HAND;
    pub const PLAYER_MODE_CUSTOMISATION: TrackedData = DATA_PLAYER_MODE_CUSTOMISATION;
    pub const PLAYER_MODE_CUSTOMIZATION_ID: TrackedData = DATA_PLAYER_MODE_CUSTOMISATION;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod axolotl {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_PLAYING_DEAD: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const FROM_BUCKET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const PLAYING_DEAD: TrackedData = DATA_PLAYING_DEAD;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT: TrackedData = DATA_VARIANT;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod bamboo_chest_raft {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod bamboo_raft {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod bat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod bee {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ANGER_END_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::LONG,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ANGER_END_TIME: TrackedData = DATA_ANGER_END_TIME;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod birch_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod birch_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod blaze {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod block_attached_entity {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod block_display {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BILLBOARD_RENDER_CONSTRAINTS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_BLOCK_STATE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 23u8 },
        r#type: MetaDataType::BLOCK_STATE,
    };
    pub const DATA_BRIGHTNESS_OVERRIDE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_GLOW_COLOR_OVERRIDE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 22u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_HEIGHT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LEFT_ROTATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::QUATERNION,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_POS_ROT_INTERPOLATION_DURATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_RIGHT_ROTATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::QUATERNION,
    };
    pub const DATA_SCALE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::VECTOR3,
    };
    pub const DATA_SHADOW_RADIUS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_SHADOW_STRENGTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSLATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::VECTOR3,
    };
    pub const DATA_VIEW_RANGE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_WIDTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const BILLBOARD_RENDER_CONSTRAINTS_ID: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const DATA_BILLBOARD_RENDER_CONSTRAINTS: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BILLBOARD_RENDER_CONSTRAINTS: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BILLBOARD: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BLOCK_STATE_ID: TrackedData = DATA_BLOCK_STATE_ID;
    pub const DATA_BLOCK_STATE: TrackedData = DATA_BLOCK_STATE_ID;
    pub const BLOCK_STATE: TrackedData = DATA_BLOCK_STATE_ID;
    pub const BRIGHTNESS_OVERRIDE_ID: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const DATA_BRIGHTNESS_OVERRIDE: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const BRIGHTNESS_OVERRIDE: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const BRIGHTNESS: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const GLOW_COLOR_OVERRIDE_ID: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const DATA_GLOW_COLOR_OVERRIDE: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const GLOW_COLOR_OVERRIDE: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const HEIGHT_ID: TrackedData = DATA_HEIGHT_ID;
    pub const DATA_HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const LEFT_ROTATION_ID: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const DATA_LEFT_ROTATION: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const LEFT_ROTATION: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const POS_ROT_INTERPOLATION_DURATION_ID: TrackedData =
        DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const DATA_POS_ROT_INTERPOLATION_DURATION: TrackedData =
        DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const POS_ROT_INTERPOLATION_DURATION: TrackedData = DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const TELEPORT_DURATION: TrackedData = DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const RIGHT_ROTATION_ID: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const DATA_RIGHT_ROTATION: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const RIGHT_ROTATION: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const SCALE_ID: TrackedData = DATA_SCALE_ID;
    pub const DATA_SCALE: TrackedData = DATA_SCALE_ID;
    pub const SCALE: TrackedData = DATA_SCALE_ID;
    pub const SHADOW_RADIUS_ID: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const DATA_SHADOW_RADIUS: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const SHADOW_RADIUS: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const SHADOW_STRENGTH_ID: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const DATA_SHADOW_STRENGTH: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const SHADOW_STRENGTH: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TRANSFORMATION_INTERPOLATION_DURATION_ID: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const DATA_TRANSFORMATION_INTERPOLATION_DURATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const TRANSFORMATION_INTERPOLATION_DURATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const INTERPOLATION_DURATION: TrackedData = DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const START_INTERPOLATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const TRANSLATION_ID: TrackedData = DATA_TRANSLATION_ID;
    pub const DATA_TRANSLATION: TrackedData = DATA_TRANSLATION_ID;
    pub const TRANSLATION: TrackedData = DATA_TRANSLATION_ID;
    pub const VIEW_RANGE_ID: TrackedData = DATA_VIEW_RANGE_ID;
    pub const DATA_VIEW_RANGE: TrackedData = DATA_VIEW_RANGE_ID;
    pub const VIEW_RANGE: TrackedData = DATA_VIEW_RANGE_ID;
    pub const WIDTH_ID: TrackedData = DATA_WIDTH_ID;
    pub const DATA_WIDTH: TrackedData = DATA_WIDTH_ID;
    pub const WIDTH: TrackedData = DATA_WIDTH_ID;
}
pub mod boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod bogged {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SHEARED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHEARED: TrackedData = DATA_SHEARED;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod breeze {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod breeze_wind_charge {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod camel {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DASH: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const LAST_POSE_CHANGE_TICK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::LONG,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod camel_husk {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DASH: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const LAST_POSE_CHANGE_TICK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::LONG,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod cat {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_COLLAR_COLOR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 23u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_OWNERUUID_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SOUND_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 24u8 },
        r#type: MetaDataType::CAT_SOUND_VARIANT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::CAT_VARIANT,
    };
    pub const IS_LYING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const RELAX_STATE_ONE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 22u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const COLLAR_COLOR: TrackedData = DATA_COLLAR_COLOR;
    pub const CAT_COLLAR_COLOR: TrackedData = DATA_COLLAR_COLOR;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const OWNERUUID_ID: TrackedData = DATA_OWNERUUID_ID;
    pub const DATA_OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNER_UUID: TrackedData = DATA_OWNERUUID_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SOUND_VARIANT_ID: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const DATA_SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const CAT_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const CAT_VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const IN_SLEEPING_POSE: TrackedData = IS_LYING;
    pub const HEAD_DOWN: TrackedData = RELAX_STATE_ONE;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod cave_spider {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod cherry_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod cherry_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod chest_minecart {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod chest_raft {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod chicken {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SOUND_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::CHICKEN_SOUND_VARIANT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::CHICKEN_VARIANT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SOUND_VARIANT_ID: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const DATA_SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod cod {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const FROM_BUCKET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod command_block_minecart {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_COMMAND_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::STRING,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_LAST_OUTPUT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::COMPONENT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_COMMAND_NAME: TrackedData = DATA_ID_COMMAND_NAME;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_LAST_OUTPUT: TrackedData = DATA_ID_LAST_OUTPUT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod copper_golem {
    use super::*;
    pub const COPPER_GOLEM_STATE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::COPPER_GOLEM_STATE,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_WEATHER_STATE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::WEATHERING_COPPER_STATE,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const WEATHER_STATE: TrackedData = DATA_WEATHER_STATE;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod cow {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SOUND_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::COW_SOUND_VARIANT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::COW_VARIANT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SOUND_VARIANT_ID: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const DATA_SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod creaking {
    use super::*;
    pub const CAN_MOVE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const HOME_POS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const IS_ACTIVE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const IS_TEARING_DOWN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod creeper {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_IS_IGNITED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_IS_POWERED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_SWELL_DIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const IS_IGNITED: TrackedData = DATA_IS_IGNITED;
    pub const IS_POWERED: TrackedData = DATA_IS_POWERED;
    pub const CHARGED: TrackedData = DATA_IS_POWERED;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const SWELL_DIR: TrackedData = DATA_SWELL_DIR;
    pub const FUSE_ID: TrackedData = DATA_SWELL_DIR;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod dark_oak_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod dark_oak_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod display {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BILLBOARD_RENDER_CONSTRAINTS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_BRIGHTNESS_OVERRIDE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_GLOW_COLOR_OVERRIDE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 22u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_HEIGHT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LEFT_ROTATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::QUATERNION,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_POS_ROT_INTERPOLATION_DURATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_RIGHT_ROTATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::QUATERNION,
    };
    pub const DATA_SCALE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::VECTOR3,
    };
    pub const DATA_SHADOW_RADIUS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_SHADOW_STRENGTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSLATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::VECTOR3,
    };
    pub const DATA_VIEW_RANGE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_WIDTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const BILLBOARD_RENDER_CONSTRAINTS_ID: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const DATA_BILLBOARD_RENDER_CONSTRAINTS: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BILLBOARD_RENDER_CONSTRAINTS: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BILLBOARD: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BRIGHTNESS_OVERRIDE_ID: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const DATA_BRIGHTNESS_OVERRIDE: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const BRIGHTNESS_OVERRIDE: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const BRIGHTNESS: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const GLOW_COLOR_OVERRIDE_ID: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const DATA_GLOW_COLOR_OVERRIDE: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const GLOW_COLOR_OVERRIDE: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const HEIGHT_ID: TrackedData = DATA_HEIGHT_ID;
    pub const DATA_HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const LEFT_ROTATION_ID: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const DATA_LEFT_ROTATION: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const LEFT_ROTATION: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const POS_ROT_INTERPOLATION_DURATION_ID: TrackedData =
        DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const DATA_POS_ROT_INTERPOLATION_DURATION: TrackedData =
        DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const POS_ROT_INTERPOLATION_DURATION: TrackedData = DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const TELEPORT_DURATION: TrackedData = DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const RIGHT_ROTATION_ID: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const DATA_RIGHT_ROTATION: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const RIGHT_ROTATION: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const SCALE_ID: TrackedData = DATA_SCALE_ID;
    pub const DATA_SCALE: TrackedData = DATA_SCALE_ID;
    pub const SCALE: TrackedData = DATA_SCALE_ID;
    pub const SHADOW_RADIUS_ID: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const DATA_SHADOW_RADIUS: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const SHADOW_RADIUS: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const SHADOW_STRENGTH_ID: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const DATA_SHADOW_STRENGTH: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const SHADOW_STRENGTH: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TRANSFORMATION_INTERPOLATION_DURATION_ID: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const DATA_TRANSFORMATION_INTERPOLATION_DURATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const TRANSFORMATION_INTERPOLATION_DURATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const INTERPOLATION_DURATION: TrackedData = DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const START_INTERPOLATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const TRANSLATION_ID: TrackedData = DATA_TRANSLATION_ID;
    pub const DATA_TRANSLATION: TrackedData = DATA_TRANSLATION_ID;
    pub const TRANSLATION: TrackedData = DATA_TRANSLATION_ID;
    pub const VIEW_RANGE_ID: TrackedData = DATA_VIEW_RANGE_ID;
    pub const DATA_VIEW_RANGE: TrackedData = DATA_VIEW_RANGE_ID;
    pub const VIEW_RANGE: TrackedData = DATA_VIEW_RANGE_ID;
    pub const WIDTH_ID: TrackedData = DATA_WIDTH_ID;
    pub const DATA_WIDTH: TrackedData = DATA_WIDTH_ID;
    pub const WIDTH: TrackedData = DATA_WIDTH_ID;
}
pub mod dolphin {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const GOT_FISH: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const MOISTNESS_LEVEL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod donkey {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_CHEST: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_CHEST: TrackedData = DATA_ID_CHEST;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod dragon_fireball {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod drowned {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DROWNED_CONVERSION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SPECIAL_TYPE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DROWNED_CONVERSION_ID: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DATA_DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SPECIAL_TYPE_ID: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const DATA_SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod egg {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod elder_guardian {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_ATTACK_TARGET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_MOVING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_ATTACK_TARGET: TrackedData = DATA_ID_ATTACK_TARGET;
    pub const ID_MOVING: TrackedData = DATA_ID_MOVING;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod end_crystal {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BEAM_TARGET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SHOW_BOTTOM: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const BEAM_TARGET: TrackedData = DATA_BEAM_TARGET;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHOW_BOTTOM: TrackedData = DATA_SHOW_BOTTOM;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod ender_dragon {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_PHASE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const PHASE: TrackedData = DATA_PHASE;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod ender_man {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CARRY_STATE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_CREEPY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STARED_AT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CARRY_STATE: TrackedData = DATA_CARRY_STATE;
    pub const CREEPY: TrackedData = DATA_CREEPY;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STARED_AT: TrackedData = DATA_STARED_AT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod ender_pearl {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod enderman {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CARRY_STATE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_CREEPY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STARED_AT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CARRY_STATE: TrackedData = DATA_CARRY_STATE;
    pub const CREEPY: TrackedData = DATA_CREEPY;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STARED_AT: TrackedData = DATA_STARED_AT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod endermite {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod entity {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod evoker {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SPELL_CASTING_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const IS_CELEBRATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SPELL_CASTING_ID: TrackedData = DATA_SPELL_CASTING_ID;
    pub const DATA_SPELL_CASTING: TrackedData = DATA_SPELL_CASTING_ID;
    pub const SPELL_CASTING: TrackedData = DATA_SPELL_CASTING_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod evoker_fangs {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod experience_bottle {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod experience_orb {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VALUE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VALUE: TrackedData = DATA_VALUE;
}
pub mod eye_of_ender {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod falling_block {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_START_POS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BLOCK_POS,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const START_POS: TrackedData = DATA_START_POS;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod falling_block_entity {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_START_POS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BLOCK_POS,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const START_POS: TrackedData = DATA_START_POS;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod fireball {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod firework_rocket {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ATTACHED_TO_TARGET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::OPTIONAL_UNSIGNED_INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_FIREWORKS_ITEM: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SHOT_AT_ANGLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ATTACHED_TO_TARGET: TrackedData = DATA_ATTACHED_TO_TARGET;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_FIREWORKS_ITEM: TrackedData = DATA_ID_FIREWORKS_ITEM;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHOT_AT_ANGLE: TrackedData = DATA_SHOT_AT_ANGLE;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod firework_rocket_entity {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ATTACHED_TO_TARGET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::OPTIONAL_UNSIGNED_INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_FIREWORKS_ITEM: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SHOT_AT_ANGLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ATTACHED_TO_TARGET: TrackedData = DATA_ATTACHED_TO_TARGET;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_FIREWORKS_ITEM: TrackedData = DATA_ID_FIREWORKS_ITEM;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHOT_AT_ANGLE: TrackedData = DATA_SHOT_AT_ANGLE;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod fishing_bobber {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BITING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_HOOKED_ENTITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const BITING: TrackedData = DATA_BITING;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const HOOKED_ENTITY: TrackedData = DATA_HOOKED_ENTITY;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod fishing_hook {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BITING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_HOOKED_ENTITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const BITING: TrackedData = DATA_BITING;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const HOOKED_ENTITY: TrackedData = DATA_HOOKED_ENTITY;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod fox {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRUSTED_ID_0: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_TRUSTED_ID_1: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_TYPE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TRUSTED_ID_0: TrackedData = DATA_TRUSTED_ID_0;
    pub const TRUSTED_ID_1: TrackedData = DATA_TRUSTED_ID_1;
    pub const TYPE_ID: TrackedData = DATA_TYPE_ID;
    pub const DATA_TYPE: TrackedData = DATA_TYPE_ID;
    pub const TYPE: TrackedData = DATA_TYPE_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod frog {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TONGUE_TARGET_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_UNSIGNED_INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::FROG_VARIANT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TONGUE_TARGET_ID: TrackedData = DATA_TONGUE_TARGET_ID;
    pub const DATA_TONGUE_TARGET: TrackedData = DATA_TONGUE_TARGET_ID;
    pub const TONGUE_TARGET: TrackedData = DATA_TONGUE_TARGET_ID;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod furnace_minecart {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_FUEL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_FUEL: TrackedData = DATA_ID_FUEL;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod ghast {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_IS_CHARGING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const IS_CHARGING: TrackedData = DATA_IS_CHARGING;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod giant {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod glow_item_frame {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DIRECTION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::DIRECTION,
    };
    pub const DATA_ITEM: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_ROTATION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DIRECTION: TrackedData = DATA_DIRECTION;
    pub const ITEM: TrackedData = DATA_ITEM;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const ROTATION: TrackedData = DATA_ROTATION;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod glow_squid {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DARK_TICKS_REMAINING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DARK_TICKS_REMAINING: TrackedData = DATA_DARK_TICKS_REMAINING;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod goat {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HAS_LEFT_HORN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_HAS_RIGHT_HORN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_IS_SCREAMING_GOAT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HAS_LEFT_HORN: TrackedData = DATA_HAS_LEFT_HORN;
    pub const HAS_RIGHT_HORN: TrackedData = DATA_HAS_RIGHT_HORN;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const IS_SCREAMING_GOAT: TrackedData = DATA_IS_SCREAMING_GOAT;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod guardian {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_ATTACK_TARGET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_MOVING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_ATTACK_TARGET: TrackedData = DATA_ID_ATTACK_TARGET;
    pub const ID_MOVING: TrackedData = DATA_ID_MOVING;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod hanging_entity {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DIRECTION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::DIRECTION,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DIRECTION: TrackedData = DATA_DIRECTION;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod happy_ghast {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const IS_LEASH_HOLDER: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const STAYS_STILL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod hoglin {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_IMMUNE_TO_ZOMBIFICATION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const IMMUNE_TO_ZOMBIFICATION: TrackedData = DATA_IMMUNE_TO_ZOMBIFICATION;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod hopper_minecart {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod horse {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_ID_TYPE_VARIANT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const ID_TYPE_VARIANT: TrackedData = DATA_ID_TYPE_VARIANT;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod husk {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DROWNED_CONVERSION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SPECIAL_TYPE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DROWNED_CONVERSION_ID: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DATA_DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SPECIAL_TYPE_ID: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const DATA_SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod illusioner {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SPELL_CASTING_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const IS_CELEBRATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SPELL_CASTING_ID: TrackedData = DATA_SPELL_CASTING_ID;
    pub const DATA_SPELL_CASTING: TrackedData = DATA_SPELL_CASTING_ID;
    pub const SPELL_CASTING: TrackedData = DATA_SPELL_CASTING_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod interaction {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_HEIGHT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_RESPONSE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_WIDTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const HEIGHT_ID: TrackedData = DATA_HEIGHT_ID;
    pub const DATA_HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const RESPONSE_ID: TrackedData = DATA_RESPONSE_ID;
    pub const DATA_RESPONSE: TrackedData = DATA_RESPONSE_ID;
    pub const RESPONSE: TrackedData = DATA_RESPONSE_ID;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const WIDTH_ID: TrackedData = DATA_WIDTH_ID;
    pub const DATA_WIDTH: TrackedData = DATA_WIDTH_ID;
    pub const WIDTH: TrackedData = DATA_WIDTH_ID;
}
pub mod iron_golem {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod item {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM: TrackedData = DATA_ITEM;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod item_display {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BILLBOARD_RENDER_CONSTRAINTS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_BRIGHTNESS_OVERRIDE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_GLOW_COLOR_OVERRIDE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 22u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_HEIGHT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ITEM_DISPLAY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 24u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_ITEM_STACK_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 23u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_LEFT_ROTATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::QUATERNION,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_POS_ROT_INTERPOLATION_DURATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_RIGHT_ROTATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::QUATERNION,
    };
    pub const DATA_SCALE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::VECTOR3,
    };
    pub const DATA_SHADOW_RADIUS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_SHADOW_STRENGTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSLATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::VECTOR3,
    };
    pub const DATA_VIEW_RANGE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_WIDTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const BILLBOARD_RENDER_CONSTRAINTS_ID: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const DATA_BILLBOARD_RENDER_CONSTRAINTS: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BILLBOARD_RENDER_CONSTRAINTS: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BILLBOARD: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BRIGHTNESS_OVERRIDE_ID: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const DATA_BRIGHTNESS_OVERRIDE: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const BRIGHTNESS_OVERRIDE: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const BRIGHTNESS: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const GLOW_COLOR_OVERRIDE_ID: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const DATA_GLOW_COLOR_OVERRIDE: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const GLOW_COLOR_OVERRIDE: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const HEIGHT_ID: TrackedData = DATA_HEIGHT_ID;
    pub const DATA_HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const ITEM_DISPLAY_ID: TrackedData = DATA_ITEM_DISPLAY_ID;
    pub const DATA_ITEM_DISPLAY: TrackedData = DATA_ITEM_DISPLAY_ID;
    pub const ITEM_DISPLAY: TrackedData = DATA_ITEM_DISPLAY_ID;
    pub const ITEM_STACK_ID: TrackedData = DATA_ITEM_STACK_ID;
    pub const DATA_ITEM_STACK: TrackedData = DATA_ITEM_STACK_ID;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK_ID;
    pub const ITEM: TrackedData = DATA_ITEM_STACK_ID;
    pub const LEFT_ROTATION_ID: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const DATA_LEFT_ROTATION: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const LEFT_ROTATION: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const POS_ROT_INTERPOLATION_DURATION_ID: TrackedData =
        DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const DATA_POS_ROT_INTERPOLATION_DURATION: TrackedData =
        DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const POS_ROT_INTERPOLATION_DURATION: TrackedData = DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const TELEPORT_DURATION: TrackedData = DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const RIGHT_ROTATION_ID: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const DATA_RIGHT_ROTATION: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const RIGHT_ROTATION: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const SCALE_ID: TrackedData = DATA_SCALE_ID;
    pub const DATA_SCALE: TrackedData = DATA_SCALE_ID;
    pub const SCALE: TrackedData = DATA_SCALE_ID;
    pub const SHADOW_RADIUS_ID: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const DATA_SHADOW_RADIUS: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const SHADOW_RADIUS: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const SHADOW_STRENGTH_ID: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const DATA_SHADOW_STRENGTH: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const SHADOW_STRENGTH: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TRANSFORMATION_INTERPOLATION_DURATION_ID: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const DATA_TRANSFORMATION_INTERPOLATION_DURATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const TRANSFORMATION_INTERPOLATION_DURATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const INTERPOLATION_DURATION: TrackedData = DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const START_INTERPOLATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const TRANSLATION_ID: TrackedData = DATA_TRANSLATION_ID;
    pub const DATA_TRANSLATION: TrackedData = DATA_TRANSLATION_ID;
    pub const TRANSLATION: TrackedData = DATA_TRANSLATION_ID;
    pub const VIEW_RANGE_ID: TrackedData = DATA_VIEW_RANGE_ID;
    pub const DATA_VIEW_RANGE: TrackedData = DATA_VIEW_RANGE_ID;
    pub const VIEW_RANGE: TrackedData = DATA_VIEW_RANGE_ID;
    pub const WIDTH_ID: TrackedData = DATA_WIDTH_ID;
    pub const DATA_WIDTH: TrackedData = DATA_WIDTH_ID;
    pub const WIDTH: TrackedData = DATA_WIDTH_ID;
}
pub mod item_entity {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM: TrackedData = DATA_ITEM;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod item_frame {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DIRECTION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::DIRECTION,
    };
    pub const DATA_ITEM: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_ROTATION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DIRECTION: TrackedData = DATA_DIRECTION;
    pub const ITEM: TrackedData = DATA_ITEM;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const ROTATION: TrackedData = DATA_ROTATION;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod jungle_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod jungle_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod large_fireball {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod leash_fence_knot_entity {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod leash_knot {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod lightning_bolt {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod lingering_potion {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod living_entity {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod llama {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_CHEST: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_STRENGTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_CHEST: TrackedData = DATA_ID_CHEST;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STRENGTH_ID: TrackedData = DATA_STRENGTH_ID;
    pub const DATA_STRENGTH: TrackedData = DATA_STRENGTH_ID;
    pub const STRENGTH: TrackedData = DATA_STRENGTH_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod llama_spit {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod magma_cube {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_SIZE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod mangrove_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod mangrove_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod mannequin {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DESCRIPTION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_IMMOVABLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_PLAYER_MAIN_HAND: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::HUMANOID_ARM,
    };
    pub const DATA_PLAYER_MODE_CUSTOMISATION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_PROFILE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::RESOLVABLE_PROFILE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DESCRIPTION: TrackedData = DATA_DESCRIPTION;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const IMMOVABLE: TrackedData = DATA_IMMOVABLE;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const PLAYER_MAIN_HAND: TrackedData = DATA_PLAYER_MAIN_HAND;
    pub const MAIN_ARM_ID: TrackedData = DATA_PLAYER_MAIN_HAND;
    pub const PLAYER_MODE_CUSTOMISATION: TrackedData = DATA_PLAYER_MODE_CUSTOMISATION;
    pub const PLAYER_MODE_CUSTOMIZATION_ID: TrackedData = DATA_PLAYER_MODE_CUSTOMISATION;
    pub const POSE: TrackedData = DATA_POSE;
    pub const PROFILE: TrackedData = DATA_PROFILE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod marker {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod minecart {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod minecart_chest {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod minecart_command_block {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_COMMAND_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::STRING,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_LAST_OUTPUT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::COMPONENT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_COMMAND_NAME: TrackedData = DATA_ID_COMMAND_NAME;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_LAST_OUTPUT: TrackedData = DATA_ID_LAST_OUTPUT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod minecart_furnace {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_FUEL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_FUEL: TrackedData = DATA_ID_FUEL;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod minecart_hopper {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod minecart_spawner {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod minecart_tnt {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod mob {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod monster {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod mooshroom {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TYPE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TYPE: TrackedData = DATA_TYPE;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod mule {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_CHEST: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_CHEST: TrackedData = DATA_ID_CHEST;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod mushroom_cow {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TYPE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TYPE: TrackedData = DATA_TYPE;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod nautilus {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DASH: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_OWNERUUID_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const OWNERUUID_ID: TrackedData = DATA_OWNERUUID_ID;
    pub const DATA_OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNER_UUID: TrackedData = DATA_OWNERUUID_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod oak_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod oak_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod ocelot {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRUSTING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TRUSTING: TrackedData = DATA_TRUSTING;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod ominous_item_spawner {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM: TrackedData = DATA_ITEM;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod painting {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DIRECTION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::DIRECTION,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_PAINTING_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::PAINTING_VARIANT,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DIRECTION: TrackedData = DATA_DIRECTION;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const PAINTING_VARIANT_ID: TrackedData = DATA_PAINTING_VARIANT_ID;
    pub const DATA_PAINTING_VARIANT: TrackedData = DATA_PAINTING_VARIANT_ID;
    pub const PAINTING_VARIANT: TrackedData = DATA_PAINTING_VARIANT_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod pale_oak_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod pale_oak_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod panda {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 23u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const EAT_COUNTER: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::INT,
    };
    pub const HIDDEN_GENE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 22u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const MAIN_GENE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const SNEEZE_COUNTER: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::INT,
    };
    pub const UNHAPPY_COUNTER: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const HIDDEN_GENE: TrackedData = HIDDEN_GENE_ID;
    pub const MAIN_GENE: TrackedData = MAIN_GENE_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod parched {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod parrot {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_OWNERUUID_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const OWNERUUID_ID: TrackedData = DATA_OWNERUUID_ID;
    pub const DATA_OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNER_UUID: TrackedData = DATA_OWNERUUID_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod pathfinder_mob {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod patrolling_monster {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod phantom {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_SIZE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod pig {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_BOOST_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SOUND_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::PIG_SOUND_VARIANT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::PIG_VARIANT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const BOOST_TIME: TrackedData = DATA_BOOST_TIME;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SOUND_VARIANT_ID: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const DATA_SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod piglin {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_IMMUNE_TO_ZOMBIFICATION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_IS_CHARGING_CROSSBOW: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_IS_DANCING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const IMMUNE_TO_ZOMBIFICATION: TrackedData = DATA_IMMUNE_TO_ZOMBIFICATION;
    pub const IS_CHARGING_CROSSBOW: TrackedData = DATA_IS_CHARGING_CROSSBOW;
    pub const IS_DANCING: TrackedData = DATA_IS_DANCING;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod piglin_brute {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_IMMUNE_TO_ZOMBIFICATION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const IMMUNE_TO_ZOMBIFICATION: TrackedData = DATA_IMMUNE_TO_ZOMBIFICATION;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod pillager {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const IS_CELEBRATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const IS_CHARGING_CROSSBOW: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod player {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_PLAYER_ABSORPTION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_PLAYER_MAIN_HAND: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::HUMANOID_ARM,
    };
    pub const DATA_PLAYER_MODE_CUSTOMISATION: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SCORE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SHOULDER_PARROT_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_UNSIGNED_INT,
    };
    pub const DATA_SHOULDER_PARROT_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::OPTIONAL_UNSIGNED_INT,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const PLAYER_ABSORPTION_ID: TrackedData = DATA_PLAYER_ABSORPTION_ID;
    pub const DATA_PLAYER_ABSORPTION: TrackedData = DATA_PLAYER_ABSORPTION_ID;
    pub const PLAYER_ABSORPTION: TrackedData = DATA_PLAYER_ABSORPTION_ID;
    pub const PLAYER_MAIN_HAND: TrackedData = DATA_PLAYER_MAIN_HAND;
    pub const MAIN_ARM_ID: TrackedData = DATA_PLAYER_MAIN_HAND;
    pub const PLAYER_MODE_CUSTOMISATION: TrackedData = DATA_PLAYER_MODE_CUSTOMISATION;
    pub const PLAYER_MODE_CUSTOMIZATION_ID: TrackedData = DATA_PLAYER_MODE_CUSTOMISATION;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SCORE_ID: TrackedData = DATA_SCORE_ID;
    pub const DATA_SCORE: TrackedData = DATA_SCORE_ID;
    pub const SCORE: TrackedData = DATA_SCORE_ID;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHOULDER_PARROT_LEFT: TrackedData = DATA_SHOULDER_PARROT_LEFT;
    pub const SHOULDER_PARROT_RIGHT: TrackedData = DATA_SHOULDER_PARROT_RIGHT;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod polar_bear {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STANDING_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STANDING_ID: TrackedData = DATA_STANDING_ID;
    pub const DATA_STANDING: TrackedData = DATA_STANDING_ID;
    pub const STANDING: TrackedData = DATA_STANDING_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod primed_tnt {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BLOCK_STATE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BLOCK_STATE,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_FUSE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const BLOCK_STATE_ID: TrackedData = DATA_BLOCK_STATE_ID;
    pub const DATA_BLOCK_STATE: TrackedData = DATA_BLOCK_STATE_ID;
    pub const BLOCK_STATE: TrackedData = DATA_BLOCK_STATE_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const FUSE_ID: TrackedData = DATA_FUSE_ID;
    pub const DATA_FUSE: TrackedData = DATA_FUSE_ID;
    pub const FUSE: TrackedData = DATA_FUSE_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod projectile {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod pufferfish {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const FROM_BUCKET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const PUFF_STATE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod rabbit {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TYPE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TYPE_ID: TrackedData = DATA_TYPE_ID;
    pub const DATA_TYPE: TrackedData = DATA_TYPE_ID;
    pub const TYPE: TrackedData = DATA_TYPE_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod raft {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod raider {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const IS_CELEBRATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod ravager {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const IS_CELEBRATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod salmon {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TYPE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const FROM_BUCKET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TYPE: TrackedData = DATA_TYPE;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod sheep {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_WOOL_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const WOOL_ID: TrackedData = DATA_WOOL_ID;
    pub const DATA_WOOL: TrackedData = DATA_WOOL_ID;
    pub const WOOL: TrackedData = DATA_WOOL_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod shoulder_riding_entity {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_OWNERUUID_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const OWNERUUID_ID: TrackedData = DATA_OWNERUUID_ID;
    pub const DATA_OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNER_UUID: TrackedData = DATA_OWNERUUID_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod shulker {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ATTACH_FACE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::DIRECTION,
    };
    pub const DATA_COLOR_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_PEEK_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ATTACH_FACE_ID: TrackedData = DATA_ATTACH_FACE_ID;
    pub const DATA_ATTACH_FACE: TrackedData = DATA_ATTACH_FACE_ID;
    pub const ATTACH_FACE: TrackedData = DATA_ATTACH_FACE_ID;
    pub const COLOR_ID: TrackedData = DATA_COLOR_ID;
    pub const DATA_COLOR: TrackedData = DATA_COLOR_ID;
    pub const COLOR: TrackedData = DATA_COLOR_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const PEEK_ID: TrackedData = DATA_PEEK_ID;
    pub const DATA_PEEK: TrackedData = DATA_PEEK_ID;
    pub const PEEK: TrackedData = DATA_PEEK_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod shulker_bullet {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod silverfish {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod skeleton {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_STRAY_CONVERSION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STRAY_CONVERSION_ID: TrackedData = DATA_STRAY_CONVERSION_ID;
    pub const DATA_STRAY_CONVERSION: TrackedData = DATA_STRAY_CONVERSION_ID;
    pub const STRAY_CONVERSION: TrackedData = DATA_STRAY_CONVERSION_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod skeleton_horse {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod slime {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_SIZE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod small_fireball {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod sniffer {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DROP_SEED_AT_TICK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STATE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::SNIFFER_STATE,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DROP_SEED_AT_TICK: TrackedData = DATA_DROP_SEED_AT_TICK;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STATE: TrackedData = DATA_STATE;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod snow_golem {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_PUMPKIN_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const PUMPKIN_ID: TrackedData = DATA_PUMPKIN_ID;
    pub const DATA_PUMPKIN: TrackedData = DATA_PUMPKIN_ID;
    pub const PUMPKIN: TrackedData = DATA_PUMPKIN_ID;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod snowball {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod spawner_minecart {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod spectral_arrow {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const IN_GROUND: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const PIERCE_LEVEL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod spellcaster_illager {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SPELL_CASTING_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const IS_CELEBRATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SPELL_CASTING_ID: TrackedData = DATA_SPELL_CASTING_ID;
    pub const DATA_SPELL_CASTING: TrackedData = DATA_SPELL_CASTING_ID;
    pub const SPELL_CASTING: TrackedData = DATA_SPELL_CASTING_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod spider {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod splash_potion {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod spruce_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod spruce_chest_boat {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_BUBBLE_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_PADDLE_LEFT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_PADDLE_RIGHT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_BUBBLE_TIME: TrackedData = DATA_ID_BUBBLE_TIME;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const ID_PADDLE_LEFT: TrackedData = DATA_ID_PADDLE_LEFT;
    pub const ID_PADDLE_RIGHT: TrackedData = DATA_ID_PADDLE_RIGHT;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod squid {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod stray {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod strider {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_BOOST_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_SUFFOCATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const BOOST_TIME: TrackedData = DATA_BOOST_TIME;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const SUFFOCATING: TrackedData = DATA_SUFFOCATING;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod sulfur_cube {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const FROM_BUCKET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const ID_SIZE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const MAX_FUSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod tadpole {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const FROM_BUCKET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod tamable_animal {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_OWNERUUID_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const OWNERUUID_ID: TrackedData = DATA_OWNERUUID_ID;
    pub const DATA_OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNER_UUID: TrackedData = DATA_OWNERUUID_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod text_display {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BACKGROUND_COLOR_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 25u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BILLBOARD_RENDER_CONSTRAINTS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_BRIGHTNESS_OVERRIDE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_GLOW_COLOR_OVERRIDE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 22u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_HEIGHT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LEFT_ROTATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::QUATERNION,
    };
    pub const DATA_LINE_WIDTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 24u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_POS_ROT_INTERPOLATION_DURATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_RIGHT_ROTATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::QUATERNION,
    };
    pub const DATA_SCALE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::VECTOR3,
    };
    pub const DATA_SHADOW_RADIUS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_SHADOW_STRENGTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STYLE_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 27u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_TEXT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 23u8 },
        r#type: MetaDataType::COMPONENT,
    };
    pub const DATA_TEXT_OPACITY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 26u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TRANSLATION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::VECTOR3,
    };
    pub const DATA_VIEW_RANGE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_WIDTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const BACKGROUND_COLOR_ID: TrackedData = DATA_BACKGROUND_COLOR_ID;
    pub const DATA_BACKGROUND_COLOR: TrackedData = DATA_BACKGROUND_COLOR_ID;
    pub const BACKGROUND_COLOR: TrackedData = DATA_BACKGROUND_COLOR_ID;
    pub const BACKGROUND: TrackedData = DATA_BACKGROUND_COLOR_ID;
    pub const BILLBOARD_RENDER_CONSTRAINTS_ID: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const DATA_BILLBOARD_RENDER_CONSTRAINTS: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BILLBOARD_RENDER_CONSTRAINTS: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BILLBOARD: TrackedData = DATA_BILLBOARD_RENDER_CONSTRAINTS_ID;
    pub const BRIGHTNESS_OVERRIDE_ID: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const DATA_BRIGHTNESS_OVERRIDE: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const BRIGHTNESS_OVERRIDE: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const BRIGHTNESS: TrackedData = DATA_BRIGHTNESS_OVERRIDE_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const GLOW_COLOR_OVERRIDE_ID: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const DATA_GLOW_COLOR_OVERRIDE: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const GLOW_COLOR_OVERRIDE: TrackedData = DATA_GLOW_COLOR_OVERRIDE_ID;
    pub const HEIGHT_ID: TrackedData = DATA_HEIGHT_ID;
    pub const DATA_HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const HEIGHT: TrackedData = DATA_HEIGHT_ID;
    pub const LEFT_ROTATION_ID: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const DATA_LEFT_ROTATION: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const LEFT_ROTATION: TrackedData = DATA_LEFT_ROTATION_ID;
    pub const LINE_WIDTH_ID: TrackedData = DATA_LINE_WIDTH_ID;
    pub const DATA_LINE_WIDTH: TrackedData = DATA_LINE_WIDTH_ID;
    pub const LINE_WIDTH: TrackedData = DATA_LINE_WIDTH_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const POS_ROT_INTERPOLATION_DURATION_ID: TrackedData =
        DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const DATA_POS_ROT_INTERPOLATION_DURATION: TrackedData =
        DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const POS_ROT_INTERPOLATION_DURATION: TrackedData = DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const TELEPORT_DURATION: TrackedData = DATA_POS_ROT_INTERPOLATION_DURATION_ID;
    pub const RIGHT_ROTATION_ID: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const DATA_RIGHT_ROTATION: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const RIGHT_ROTATION: TrackedData = DATA_RIGHT_ROTATION_ID;
    pub const SCALE_ID: TrackedData = DATA_SCALE_ID;
    pub const DATA_SCALE: TrackedData = DATA_SCALE_ID;
    pub const SCALE: TrackedData = DATA_SCALE_ID;
    pub const SHADOW_RADIUS_ID: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const DATA_SHADOW_RADIUS: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const SHADOW_RADIUS: TrackedData = DATA_SHADOW_RADIUS_ID;
    pub const SHADOW_STRENGTH_ID: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const DATA_SHADOW_STRENGTH: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const SHADOW_STRENGTH: TrackedData = DATA_SHADOW_STRENGTH_ID;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STYLE_FLAGS_ID: TrackedData = DATA_STYLE_FLAGS_ID;
    pub const DATA_STYLE_FLAGS: TrackedData = DATA_STYLE_FLAGS_ID;
    pub const STYLE_FLAGS: TrackedData = DATA_STYLE_FLAGS_ID;
    pub const TEXT_DISPLAY_FLAGS: TrackedData = DATA_STYLE_FLAGS_ID;
    pub const TEXT_ID: TrackedData = DATA_TEXT_ID;
    pub const DATA_TEXT: TrackedData = DATA_TEXT_ID;
    pub const TEXT: TrackedData = DATA_TEXT_ID;
    pub const TEXT_OPACITY_ID: TrackedData = DATA_TEXT_OPACITY_ID;
    pub const DATA_TEXT_OPACITY: TrackedData = DATA_TEXT_OPACITY_ID;
    pub const TEXT_OPACITY: TrackedData = DATA_TEXT_OPACITY_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const TRANSFORMATION_INTERPOLATION_DURATION_ID: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const DATA_TRANSFORMATION_INTERPOLATION_DURATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const TRANSFORMATION_INTERPOLATION_DURATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const INTERPOLATION_DURATION: TrackedData = DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID;
    pub const TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const START_INTERPOLATION: TrackedData =
        DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID;
    pub const TRANSLATION_ID: TrackedData = DATA_TRANSLATION_ID;
    pub const DATA_TRANSLATION: TrackedData = DATA_TRANSLATION_ID;
    pub const TRANSLATION: TrackedData = DATA_TRANSLATION_ID;
    pub const VIEW_RANGE_ID: TrackedData = DATA_VIEW_RANGE_ID;
    pub const DATA_VIEW_RANGE: TrackedData = DATA_VIEW_RANGE_ID;
    pub const VIEW_RANGE: TrackedData = DATA_VIEW_RANGE_ID;
    pub const WIDTH_ID: TrackedData = DATA_WIDTH_ID;
    pub const DATA_WIDTH: TrackedData = DATA_WIDTH_ID;
    pub const WIDTH: TrackedData = DATA_WIDTH_ID;
}
pub mod throwable_item_projectile {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod throwable_projectile {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod thrown_egg {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod thrown_enderpearl {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod thrown_experience_bottle {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod thrown_lingering_potion {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod thrown_splash_potion {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ITEM_STACK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::ITEM_STACK,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ITEM_STACK: TrackedData = DATA_ITEM_STACK;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod thrown_trident {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const ID_FOIL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const ID_LOYALTY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const IN_GROUND: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const PIERCE_LEVEL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod tnt {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BLOCK_STATE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BLOCK_STATE,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_FUSE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const BLOCK_STATE_ID: TrackedData = DATA_BLOCK_STATE_ID;
    pub const DATA_BLOCK_STATE: TrackedData = DATA_BLOCK_STATE_ID;
    pub const BLOCK_STATE: TrackedData = DATA_BLOCK_STATE_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const FUSE_ID: TrackedData = DATA_FUSE_ID;
    pub const DATA_FUSE: TrackedData = DATA_FUSE_ID;
    pub const FUSE: TrackedData = DATA_FUSE_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod tnt_minecart {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_CUSTOM_DISPLAY_BLOCK: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_STATE,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_DISPLAY_OFFSET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_CUSTOM_DISPLAY_BLOCK: TrackedData = DATA_ID_CUSTOM_DISPLAY_BLOCK;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_DISPLAY_OFFSET: TrackedData = DATA_ID_DISPLAY_OFFSET;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod trader_llama {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_CHEST: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_STRENGTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_CHEST: TrackedData = DATA_ID_CHEST;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STRENGTH_ID: TrackedData = DATA_STRENGTH_ID;
    pub const DATA_STRENGTH: TrackedData = DATA_STRENGTH_ID;
    pub const STRENGTH: TrackedData = DATA_STRENGTH_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod trident {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const ID_FOIL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const ID_LOYALTY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const IN_GROUND: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const PIERCE_LEVEL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod tropical_fish {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_TYPE_VARIANT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const FROM_BUCKET: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_TYPE_VARIANT: TrackedData = DATA_ID_TYPE_VARIANT;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod turtle {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const HAS_EGG: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const LAYING_EGG: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod vehicle_entity {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_ID_DAMAGE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_HURT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ID_HURTDIR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const ID_DAMAGE: TrackedData = DATA_ID_DAMAGE;
    pub const ID_HURT: TrackedData = DATA_ID_HURT;
    pub const ID_HURTDIR: TrackedData = DATA_ID_HURTDIR;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod vex {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod villager {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_UNHAPPY_COUNTER: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VILLAGER_DATA: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::VILLAGER_DATA,
    };
    pub const DATA_VILLAGER_DATA_FINALIZED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const UNHAPPY_COUNTER: TrackedData = DATA_UNHAPPY_COUNTER;
    pub const VILLAGER_DATA: TrackedData = DATA_VILLAGER_DATA;
    pub const VILLAGER_DATA_FINALIZED: TrackedData = DATA_VILLAGER_DATA_FINALIZED;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod vindicator {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const IS_CELEBRATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod wandering_trader {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_UNHAPPY_COUNTER: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const UNHAPPY_COUNTER: TrackedData = DATA_UNHAPPY_COUNTER;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod warden {
    use super::*;
    pub const CLIENT_ANGER_LEVEL: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod water_animal {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod wind_charge {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod witch {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_USING_ITEM: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const IS_CELEBRATING: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const USING_ITEM: TrackedData = DATA_USING_ITEM;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod wither {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_INV: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TARGET_A: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TARGET_B: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TARGET_C: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_INV: TrackedData = DATA_ID_INV;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TARGET_A: TrackedData = DATA_TARGET_A;
    pub const TARGET_B: TrackedData = DATA_TARGET_B;
    pub const TARGET_C: TrackedData = DATA_TARGET_C;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod wither_boss {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_INV: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TARGET_A: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TARGET_B: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TARGET_C: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_INV: TrackedData = DATA_ID_INV;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TARGET_A: TrackedData = DATA_TARGET_A;
    pub const TARGET_B: TrackedData = DATA_TARGET_B;
    pub const TARGET_C: TrackedData = DATA_TARGET_C;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod wither_skeleton {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod wither_skull {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DANGEROUS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DANGEROUS: TrackedData = DATA_DANGEROUS;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
}
pub mod wolf {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ANGER_END_TIME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 22u8 },
        r#type: MetaDataType::LONG,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_COLLAR_COLOR: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_INTERESTED_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_OWNERUUID_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SOUND_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 24u8 },
        r#type: MetaDataType::WOLF_SOUND_VARIANT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 23u8 },
        r#type: MetaDataType::WOLF_VARIANT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ANGER_END_TIME: TrackedData = DATA_ANGER_END_TIME;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const COLLAR_COLOR: TrackedData = DATA_COLLAR_COLOR;
    pub const WOLF_COLLAR_COLOR: TrackedData = DATA_COLLAR_COLOR;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const INTERESTED_ID: TrackedData = DATA_INTERESTED_ID;
    pub const DATA_INTERESTED: TrackedData = DATA_INTERESTED_ID;
    pub const INTERESTED: TrackedData = DATA_INTERESTED_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const OWNERUUID_ID: TrackedData = DATA_OWNERUUID_ID;
    pub const DATA_OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNER_UUID: TrackedData = DATA_OWNERUUID_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SOUND_VARIANT_ID: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const DATA_SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const SOUND_VARIANT: TrackedData = DATA_SOUND_VARIANT_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const WOLF_VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod zoglin {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod zombie {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DROWNED_CONVERSION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SPECIAL_TYPE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DROWNED_CONVERSION_ID: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DATA_DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SPECIAL_TYPE_ID: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const DATA_SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod zombie_horse {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_ID_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const ID_FLAGS: TrackedData = DATA_ID_FLAGS;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod zombie_nautilus {
    use super::*;
    pub const AGE_LOCKED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DASH: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_OWNERUUID_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::OPTIONAL_LIVING_ENTITY_REFERENCE,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VARIANT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::ZOMBIE_NAUTILUS_VARIANT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const FLAGS_ID: TrackedData = DATA_FLAGS_ID;
    pub const DATA_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const TAMEABLE_FLAGS: TrackedData = DATA_FLAGS_ID;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const OWNERUUID_ID: TrackedData = DATA_OWNERUUID_ID;
    pub const DATA_OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNERUUID: TrackedData = DATA_OWNERUUID_ID;
    pub const OWNER_UUID: TrackedData = DATA_OWNERUUID_ID;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VARIANT_ID: TrackedData = DATA_VARIANT_ID;
    pub const DATA_VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const VARIANT: TrackedData = DATA_VARIANT_ID;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod zombie_villager {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CONVERTING_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 19u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DROWNED_CONVERSION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SPECIAL_TYPE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_VILLAGER_DATA: TrackedData = TrackedData {
        id: TrackedId { v26_2: 20u8 },
        r#type: MetaDataType::VILLAGER_DATA,
    };
    pub const DATA_VILLAGER_DATA_FINALIZED: TrackedData = TrackedData {
        id: TrackedId { v26_2: 21u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CONVERTING_ID: TrackedData = DATA_CONVERTING_ID;
    pub const DATA_CONVERTING: TrackedData = DATA_CONVERTING_ID;
    pub const CONVERTING: TrackedData = DATA_CONVERTING_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DROWNED_CONVERSION_ID: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DATA_DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SPECIAL_TYPE_ID: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const DATA_SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const VILLAGER_DATA: TrackedData = DATA_VILLAGER_DATA;
    pub const VILLAGER_DATA_FINALIZED: TrackedData = DATA_VILLAGER_DATA_FINALIZED;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
pub mod zombified_piglin {
    use super::*;
    pub const DATA_AIR_SUPPLY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 1u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_ARROW_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 12u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_BABY_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 16u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_CUSTOM_NAME: TrackedData = TrackedData {
        id: TrackedId { v26_2: 2u8 },
        r#type: MetaDataType::OPTIONAL_COMPONENT,
    };
    pub const DATA_CUSTOM_NAME_VISIBLE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 3u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_DROWNED_CONVERSION_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 18u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_AMBIENCE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 11u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_EFFECT_PARTICLES: TrackedData = TrackedData {
        id: TrackedId { v26_2: 10u8 },
        r#type: MetaDataType::PARTICLES,
    };
    pub const DATA_HEALTH_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 9u8 },
        r#type: MetaDataType::FLOAT,
    };
    pub const DATA_LIVING_ENTITY_FLAGS: TrackedData = TrackedData {
        id: TrackedId { v26_2: 8u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_MOB_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 15u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_NO_GRAVITY: TrackedData = TrackedData {
        id: TrackedId { v26_2: 5u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_POSE: TrackedData = TrackedData {
        id: TrackedId { v26_2: 6u8 },
        r#type: MetaDataType::POSE,
    };
    pub const DATA_SHARED_FLAGS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 0u8 },
        r#type: MetaDataType::BYTE,
    };
    pub const DATA_SILENT: TrackedData = TrackedData {
        id: TrackedId { v26_2: 4u8 },
        r#type: MetaDataType::BOOLEAN,
    };
    pub const DATA_SPECIAL_TYPE_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 17u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_STINGER_COUNT_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 13u8 },
        r#type: MetaDataType::INT,
    };
    pub const DATA_TICKS_FROZEN: TrackedData = TrackedData {
        id: TrackedId { v26_2: 7u8 },
        r#type: MetaDataType::INT,
    };
    pub const SLEEPING_POS_ID: TrackedData = TrackedData {
        id: TrackedId { v26_2: 14u8 },
        r#type: MetaDataType::OPTIONAL_BLOCK_POS,
    };
    pub const AIR_SUPPLY_ID: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const DATA_AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const AIR_SUPPLY: TrackedData = DATA_AIR_SUPPLY_ID;
    pub const ARROW_COUNT_ID: TrackedData = DATA_ARROW_COUNT_ID;
    pub const DATA_ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const ARROW_COUNT: TrackedData = DATA_ARROW_COUNT_ID;
    pub const BABY_ID: TrackedData = DATA_BABY_ID;
    pub const DATA_BABY: TrackedData = DATA_BABY_ID;
    pub const BABY: TrackedData = DATA_BABY_ID;
    pub const CUSTOM_NAME: TrackedData = DATA_CUSTOM_NAME;
    pub const CUSTOM_NAME_VISIBLE: TrackedData = DATA_CUSTOM_NAME_VISIBLE;
    pub const DROWNED_CONVERSION_ID: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DATA_DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const DROWNED_CONVERSION: TrackedData = DATA_DROWNED_CONVERSION_ID;
    pub const EFFECT_AMBIENCE_ID: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const DATA_EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_AMBIENCE: TrackedData = DATA_EFFECT_AMBIENCE_ID;
    pub const EFFECT_PARTICLES: TrackedData = DATA_EFFECT_PARTICLES;
    pub const HEALTH_ID: TrackedData = DATA_HEALTH_ID;
    pub const DATA_HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const HEALTH: TrackedData = DATA_HEALTH_ID;
    pub const LIVING_ENTITY_FLAGS: TrackedData = DATA_LIVING_ENTITY_FLAGS;
    pub const MOB_FLAGS_ID: TrackedData = DATA_MOB_FLAGS_ID;
    pub const DATA_MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const MOB_FLAGS: TrackedData = DATA_MOB_FLAGS_ID;
    pub const NO_GRAVITY: TrackedData = DATA_NO_GRAVITY;
    pub const POSE: TrackedData = DATA_POSE;
    pub const SHARED_FLAGS_ID: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const DATA_SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SHARED_FLAGS: TrackedData = DATA_SHARED_FLAGS_ID;
    pub const SILENT: TrackedData = DATA_SILENT;
    pub const SPECIAL_TYPE_ID: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const DATA_SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const SPECIAL_TYPE: TrackedData = DATA_SPECIAL_TYPE_ID;
    pub const STINGER_COUNT_ID: TrackedData = DATA_STINGER_COUNT_ID;
    pub const DATA_STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const STINGER_COUNT: TrackedData = DATA_STINGER_COUNT_ID;
    pub const TICKS_FROZEN: TrackedData = DATA_TICKS_FROZEN;
    pub const SLEEPING_POS: TrackedData = SLEEPING_POS_ID;
}
