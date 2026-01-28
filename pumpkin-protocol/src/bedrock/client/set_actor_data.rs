use std::{collections::HashMap, io::Write};

use crate::{
    codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt, var_ulong::VarULong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;
use std::io::Error;

#[derive(PacketWrite)]
#[packet(39)] // ProtocolInfo::SET_ACTOR_DATA_PACKET is 39
pub struct CSetActorData {
    /// The unique runtime ID of the entity being updated
    pub actor_runtime_id: VarULong,
    /// A map of entity metadata properties (e.g., flags, name tags, scale)
    pub metadata: EntityMetadata,
    /// Dynamic properties synced between client and server
    pub synced_properties: PropertySyncData,
    /// The server tick at which this update occurred
    pub tick: VarULong,
}

#[derive(Default)]
pub struct EntityMetadata(pub HashMap<u32, MetadataValue>);

impl EntityMetadata {
    pub fn set(&mut self, key: u32, value: MetadataValue) {
        self.0.insert(key, value);
    }

    pub fn set_flag(&mut self, flag_index: u32) {
        let (key, index) = if flag_index < 64 {
            (entity_data_key::FLAGS, flag_index)
        } else {
            (entity_data_key::FLAGS_TWO, flag_index - 64)
        };

        let current_value = match self.0.get(&key) {
            Some(MetadataValue::Long(v)) => *v,
            _ => 0,
        };

        self.0
            .insert(key, MetadataValue::Long(current_value | (1 << index)));
    }
}

impl PacketWrite for EntityMetadata {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.0.len() as u32).write(writer)?;

        for (key, value) in &self.0 {
            VarUInt(*key).write(writer)?;
            VarUInt(value.type_id()).write(writer)?;
            value.write(writer)?;
        }
        Ok(())
    }
}

pub enum MetadataValue {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    CompoundTag,
    BlockPos,
    Long(i64),
    Vec3,
}

impl MetadataValue {
    #[must_use]
    pub fn type_id(&self) -> u32 {
        match self {
            Self::Byte(_) => 0,
            Self::Short(_) => 1,
            Self::Int(_) => 2,
            Self::Float(_) => 3,
            Self::String(_) => 4,
            Self::CompoundTag => 5,
            Self::BlockPos => 6,
            Self::Long(_) => 7,
            Self::Vec3 => 8,
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        match self {
            Self::Byte(v) => v.write(writer),
            Self::Short(v) => v.write(writer),
            // Type 2: Signed VarInt
            Self::Int(v) => VarInt(*v).write(writer),
            // Type 3: LE Float (4 bytes)
            Self::Float(v) => writer.write_all(&v.to_le_bytes()),
            // Type 4: VarInt length + UTF8 String
            Self::String(v) => v.write(writer),
            Self::CompoundTag => todo!(),
            Self::BlockPos => todo!(),
            Self::Long(v) => VarLong(*v).write(writer),
            Self::Vec3 => todo!(),
        }
    }
}

pub struct PropertySyncData {
    pub int_properties: std::collections::HashMap<u32, i32>,
    pub float_properties: std::collections::HashMap<u32, f32>,
}

impl PacketWrite for PropertySyncData {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        // Int Properties
        VarUInt(self.int_properties.len() as u32).write(writer)?;
        for (key, value) in &self.int_properties {
            VarUInt(*key).write(writer)?;
            VarInt(*value).write(writer)?; // Signed VarInt
        }

        // Float Properties
        VarUInt(self.float_properties.len() as u32).write(writer)?;
        for (key, value) in &self.float_properties {
            VarUInt(*key).write(writer)?;
            writer.write_all(&value.to_le_bytes())?; // LE Float
        }
        Ok(())
    }
}
pub mod entity_data_key {
    pub const FLAGS: u32 = 0;
    pub const STRUCTURAL_INTEGRITY: u32 = 1;
    pub const VARIANT: u32 = 2;
    pub const COLOR_INDEX: u32 = 3;
    pub const NAME: u32 = 4;
    pub const OWNER: u32 = 5;
    pub const TARGET: u32 = 6;
    pub const AIR_SUPPLY: u32 = 7;
    pub const EFFECT_COLOR: u32 = 8;
    pub const EFFECT_AMBIENCE: u32 = 9;
    pub const JUMP_DURATION: u32 = 10;
    pub const HURT: u32 = 11;
    pub const HURT_DIRECTION: u32 = 12;
    pub const ROW_TIME_LEFT: u32 = 13;
    pub const ROW_TIME_RIGHT: u32 = 14;
    pub const VALUE: u32 = 15;
    pub const DISPLAY_TILE_RUNTIME_ID: u32 = 16;
    pub const DISPLAY_OFFSET: u32 = 17;
    pub const CUSTOM_DISPLAY: u32 = 18;
    pub const SWELL: u32 = 19;
    pub const OLD_SWELL: u32 = 20;
    pub const SWELL_DIRECTION: u32 = 21;
    pub const CHARGE_AMOUNT: u32 = 22;
    pub const CARRY_BLOCK_RUNTIME_ID: u32 = 23;
    pub const CLIENT_EVENT: u32 = 24;
    pub const USING_ITEM: u32 = 25;
    pub const PLAYER_FLAGS: u32 = 26;
    pub const PLAYER_INDEX: u32 = 27;
    pub const BED_POSITION: u32 = 28;
    pub const POWER_X: u32 = 29;
    pub const POWER_Y: u32 = 30;
    pub const POWER_Z: u32 = 31;
    pub const AUX_POWER: u32 = 32;
    pub const FISH_X: u32 = 33;
    pub const FISH_Z: u32 = 34;
    pub const FISH_ANGLE: u32 = 35;
    pub const AUX_VALUE_DATA: u32 = 36;
    pub const LEASH_HOLDER: u32 = 37;
    pub const SCALE: u32 = 38;
    pub const HAS_NPC: u32 = 39;
    pub const NPC_DATA: u32 = 40;
    pub const ACTIONS: u32 = 41;
    pub const AIR_SUPPLY_MAX: u32 = 42;
    pub const MARK_VARIANT: u32 = 43;
    pub const CONTAINER_TYPE: u32 = 44;
    pub const CONTAINER_SIZE: u32 = 45;
    pub const CONTAINER_STRENGTH_MODIFIER: u32 = 46;
    pub const BLOCK_TARGET: u32 = 47;
    pub const INVULNERABLE_TICKS: u32 = 48;
    pub const TARGET_A: u32 = 49;
    pub const TARGET_B: u32 = 50;
    pub const TARGET_C: u32 = 51;
    pub const AERIAL_ATTACK: u32 = 52;
    pub const WIDTH: u32 = 53;
    pub const HEIGHT: u32 = 54;
    pub const FUSE_TIME: u32 = 55;
    pub const SEAT_OFFSET: u32 = 56;
    pub const SEAT_LOCK_PASSENGER_ROTATION: u32 = 57;
    pub const SEAT_LOCK_PASSENGER_ROTATION_DEGREES: u32 = 58;
    pub const SEAT_ROTATION_OFFSET: u32 = 59;
    pub const SEAT_ROTATION_OFFSET_DEGREES: u32 = 60;
    pub const DATA_RADIUS: u32 = 61;
    pub const DATA_WAITING: u32 = 62;
    pub const DATA_PARTICLE: u32 = 63;
    pub const PEEK_ID: u32 = 64;
    pub const ATTACH_FACE: u32 = 65;
    pub const ATTACHED: u32 = 66;
    pub const ATTACHED_POSITION: u32 = 67;
    pub const TRADE_TARGET: u32 = 68;
    pub const CAREER: u32 = 69;
    pub const HAS_COMMAND_BLOCK: u32 = 70;
    pub const COMMAND_NAME: u32 = 71;
    pub const LAST_COMMAND_OUTPUT: u32 = 72;
    pub const TRACK_COMMAND_OUTPUT: u32 = 73;
    pub const CONTROLLING_SEAT_INDEX: u32 = 74;
    pub const STRENGTH: u32 = 75;
    pub const STRENGTH_MAX: u32 = 76;
    pub const DATA_SPELL_CASTING_COLOR: u32 = 77;
    pub const DATA_LIFETIME_TICKS: u32 = 78;
    pub const POSE_INDEX: u32 = 79;
    pub const DATA_TICK_OFFSET: u32 = 80;
    pub const ALWAYS_SHOW_NAME_TAG: u32 = 81;
    pub const COLOR_TWO_INDEX: u32 = 82;
    pub const NAME_AUTHOR: u32 = 83;
    pub const SCORE: u32 = 84;
    pub const BALLOON_ANCHOR: u32 = 85;
    pub const PUFFED_STATE: u32 = 86;
    pub const BUBBLE_TIME: u32 = 87;
    pub const AGENT: u32 = 88;
    pub const SITTING_AMOUNT: u32 = 89;
    pub const SITTING_AMOUNT_PREVIOUS: u32 = 90;
    pub const EATING_COUNTER: u32 = 91;
    pub const FLAGS_TWO: u32 = 92;
    pub const LAYING_AMOUNT: u32 = 93;
    pub const LAYING_AMOUNT_PREVIOUS: u32 = 94;
    pub const DATA_DURATION: u32 = 95;
    pub const DATA_SPAWN_TIME: u32 = 96;
    pub const DATA_CHANGE_RATE: u32 = 97;
    pub const DATA_CHANGE_ON_PICKUP: u32 = 98;
    pub const DATA_PICKUP_COUNT: u32 = 99;
    pub const INTERACT_TEXT: u32 = 100;
    pub const TRADE_TIER: u32 = 101;
    pub const MAX_TRADE_TIER: u32 = 102;
    pub const TRADE_EXPERIENCE: u32 = 103;
    pub const SKIN_ID: u32 = 104;
    pub const SPAWNING_FRAMES: u32 = 105;
    pub const COMMAND_BLOCK_TICK_DELAY: u32 = 106;
    pub const COMMAND_BLOCK_EXECUTE_ON_FIRST_TICK: u32 = 107;
    pub const AMBIENT_SOUND_INTERVAL: u32 = 108;
    pub const AMBIENT_SOUND_INTERVAL_RANGE: u32 = 109;
    pub const AMBIENT_SOUND_EVENT_NAME: u32 = 110;
    pub const FALL_DAMAGE_MULTIPLIER: u32 = 111;
    pub const NAME_RAW_TEXT: u32 = 112;
    pub const CAN_RIDE_TARGET: u32 = 113;
    pub const LOW_TIER_CURED_TRADE_DISCOUNT: u32 = 114;
    pub const HIGH_TIER_CURED_TRADE_DISCOUNT: u32 = 115;
    pub const NEARBY_CURED_TRADE_DISCOUNT: u32 = 116;
    pub const NEARBY_CURED_DISCOUNT_TIME_STAMP: u32 = 117;
    pub const HIT_BOX: u32 = 118;
    pub const IS_BUOYANT: u32 = 119;
    pub const FREEZING_EFFECT_STRENGTH: u32 = 120;
    pub const BUOYANCY_DATA: u32 = 121;
    pub const GOAT_HORN_COUNT: u32 = 122;
    pub const BASE_RUNTIME_ID: u32 = 123;
    pub const MOVEMENT_SOUND_DISTANCE_OFFSET: u32 = 124;
    pub const HEARTBEAT_INTERVAL_TICKS: u32 = 125;
    pub const HEARTBEAT_SOUND_EVENT: u32 = 126;
    pub const PLAYER_LAST_DEATH_POSITION: u32 = 127;
    pub const PLAYER_LAST_DEATH_DIMENSION: u32 = 128;
    pub const PLAYER_HAS_DIED: u32 = 129;
    pub const COLLISION_BOX: u32 = 130;
    pub const VISIBLE_MOB_EFFECTS: u32 = 131;
    pub const FILTERED_NAME: u32 = 132;
    pub const ENTER_BED_POSITION: u32 = 133;
    pub const SEAT_THIRD_PERSON_CAMERA_RADIUS: u32 = 134;
    pub const SEAT_CAMERA_RELAX_DISTANCE_SMOOTHING: u32 = 135;
}

pub mod entity_data_flag {
    pub const ON_FIRE: u32 = 0;
    pub const SNEAKING: u32 = 1;
    pub const RIDING: u32 = 2;
    pub const SPRINTING: u32 = 3;
    pub const USING_ITEM: u32 = 4;
    pub const INVISIBLE: u32 = 5;
    pub const TEMPTED: u32 = 6;
    pub const IN_LOVE: u32 = 7;
    pub const SADDLED: u32 = 8;
    pub const POWERED: u32 = 9;
    pub const IGNITED: u32 = 10;
    pub const BABY: u32 = 11;
    pub const CONVERTING: u32 = 12;
    pub const CRITICAL: u32 = 13;
    pub const SHOW_NAME: u32 = 14;
    pub const ALWAYS_SHOW_NAME: u32 = 15;
    pub const NO_AI: u32 = 16;
    pub const SILENT: u32 = 17;
    pub const WALL_CLIMBING: u32 = 18;
    pub const CLIMB: u32 = 19;
    pub const SWIM: u32 = 20;
    pub const FLY: u32 = 21;
    pub const WALK: u32 = 22;
    pub const RESTING: u32 = 23;
    pub const SITTING: u32 = 24;
    pub const ANGRY: u32 = 25;
    pub const INTERESTED: u32 = 26;
    pub const CHARGED: u32 = 27;
    pub const TAMED: u32 = 28;
    pub const ORPHANED: u32 = 29;
    pub const LEASHED: u32 = 30;
    pub const SHEARED: u32 = 31;
    pub const GLIDING: u32 = 32;
    pub const ELDER: u32 = 33;
    pub const MOVING: u32 = 34;
    pub const BREATHING: u32 = 35;
    pub const CHESTED: u32 = 36;
    pub const STACKABLE: u32 = 37;
    pub const SHOW_BOTTOM: u32 = 38;
    pub const STANDING: u32 = 39;
    pub const SHAKING: u32 = 40;
    pub const IDLING: u32 = 41;
    pub const CASTING: u32 = 42;
    pub const CHARGING: u32 = 43;
    pub const KEYBOARD_CONTROLLED: u32 = 44;
    pub const POWER_JUMP: u32 = 45;
    pub const DASH: u32 = 46;
    pub const LINGERING: u32 = 47;
    pub const HAS_COLLISION: u32 = 48;
    pub const HAS_GRAVITY: u32 = 49;
    pub const FIRE_IMMUNE: u32 = 50;
    pub const DANCING: u32 = 51;
    pub const ENCHANTED: u32 = 52;
    pub const RETURN_TRIDENT: u32 = 53;
    pub const CONTAINER_PRIVATE: u32 = 54;
    pub const TRANSFORMING: u32 = 55;
    pub const DAMAGE_NEARBY_MOBS: u32 = 56;
    pub const SWIMMING: u32 = 57;
    pub const BRIBED: u32 = 58;
    pub const PREGNANT: u32 = 59;
    pub const LAYING_EGG: u32 = 60;
    pub const PASSENGER_CAN_PICK: u32 = 61;
    pub const TRANSITION_SITTING: u32 = 62;
    pub const EATING: u32 = 63;
    pub const LAYING_DOWN: u32 = 64;
    pub const SNEEZING: u32 = 65;
    pub const TRUSTING: u32 = 66;
    pub const ROLLING: u32 = 67;
    pub const SCARED: u32 = 68;
    pub const IN_SCAFFOLDING: u32 = 69;
    pub const OVER_SCAFFOLDING: u32 = 70;
    pub const DESCEND_THROUGH_BLOCK: u32 = 71;
    pub const BLOCKING: u32 = 72;
    pub const TRANSITION_BLOCKING: u32 = 73;
    pub const BLOCKED_USING_SHIELD: u32 = 74;
    pub const BLOCKED_USING_DAMAGED_SHIELD: u32 = 75;
    pub const SLEEPING: u32 = 76;
    pub const WANTS_TO_WAKE: u32 = 77;
    pub const TRADE_INTEREST: u32 = 78;
    pub const DOOR_BREAKER: u32 = 79;
    pub const BREAKING_OBSTRUCTION: u32 = 80;
    pub const DOOR_OPENER: u32 = 81;
    pub const CAPTAIN: u32 = 82;
    pub const STUNNED: u32 = 83;
    pub const ROARING: u32 = 84;
    pub const DELAYED_ATTACK: u32 = 85;
    pub const AVOIDING_MOBS: u32 = 86;
    pub const AVOIDING_BLOCK: u32 = 87;
    pub const FACING_TARGET_TO_RANGE_ATTACK: u32 = 88;
    pub const HIDDEN_WHEN_INVISIBLE: u32 = 89;
    pub const IN_UI: u32 = 90;
    pub const STALKING: u32 = 91;
    pub const EMOTING: u32 = 92;
    pub const CELEBRATING: u32 = 93;
    pub const ADMIRING: u32 = 94;
    pub const CELEBRATING_SPECIAL: u32 = 95;
    pub const OUT_OF_CONTROL: u32 = 96;
    pub const RAM_ATTACK: u32 = 97;
    pub const PLAYING_DEAD: u32 = 98;
    pub const IN_ASCENDING_BLOCK: u32 = 99;
    pub const OVER_DESCENDING_BLOCK: u32 = 100;
    pub const CROAKING: u32 = 101;
    pub const DIGEST_MOB: u32 = 102;
    pub const JUMP_GOAL: u32 = 103;
    pub const EMERGING: u32 = 104;
    pub const SNIFFING: u32 = 105;
    pub const DIGGING: u32 = 106;
    pub const SONIC_BOOM: u32 = 107;
    pub const HAS_DASH_TIMEOUT: u32 = 108;
    pub const PUSH_TOWARDS_CLOSEST_SPACE: u32 = 109;
    pub const SCENTING: u32 = 110;
    pub const RISING: u32 = 111;
    pub const FEELING_HAPPY: u32 = 112;
    pub const SEARCHING: u32 = 113;
    pub const CRAWLING: u32 = 114;
    pub const TIMER_FLAG_1: u32 = 115;
    pub const TIMER_FLAG_2: u32 = 116;
    pub const TIMER_FLAG_3: u32 = 117;
    pub const BODY_ROTATION_BLOCKED: u32 = 118;
    pub const RENDER_WHEN_INVISIBLE: u32 = 119;
    pub const BODY_ROTATION_AXIS_ALIGNED: u32 = 120;
    pub const COLLIDABLE: u32 = 121;
    pub const WASD_AIR_CONTROLLED: u32 = 122;
    pub const DOES_SERVER_AUTH_ONLY_DISMOUNT: u32 = 123;
    pub const BODY_ROTATION_ALWAYS_FOLLOWS_HEAD: u32 = 124;
    pub const CAN_USE_VERTICAL_MOVEMENT_ACTION: u32 = 125;
    pub const ROTATION_LOCKED_TO_VEHICLE: u32 = 126;
    pub const COUNT: u32 = 127;
}
