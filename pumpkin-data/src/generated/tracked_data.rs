/* This file is generated. Do not edit manually. */
use pumpkin_util::version::MinecraftVersion;
#[derive(Copy, Clone, Debug)]
pub struct TrackedId {
    pub latest: u8,
    pub v1_21_7: u8,
}
impl TrackedId {
    pub fn get(&self, version: &MinecraftVersion) -> u8 {
        match version {
            MinecraftVersion::V_1_21_7 | MinecraftVersion::V_1_21_9 => self.v1_21_7,
            _ => self.latest,
        }
    }
}
impl From<TrackedId> for u8 {
    fn from(id: TrackedId) -> u8 {
        id.latest
    }
}
pub struct TrackedData;
impl TrackedData {
    pub const DATA_ACTIVE: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_AIR: TrackedId = TrackedId {
        latest: 1u8,
        v1_21_7: 1u8,
    };
    pub const DATA_ANGER: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_ANGER_END_TIME: TrackedId = TrackedId {
        latest: 21u8,
        v1_21_7: 21u8,
    };
    pub const DATA_ANGRY: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
    pub const DATA_ARMOR_STAND_FLAGS: TrackedId = TrackedId {
        latest: 15u8,
        v1_21_7: 15u8,
    };
    pub const DATA_ASK_FOR_BAMBOO_TICKS: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_ATTACHED_FACE: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_BABY: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_BACKGROUND: TrackedId = TrackedId {
        latest: 25u8,
        v1_21_7: 25u8,
    };
    pub const DATA_BAT_FLAGS: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_BEAM_TARGET: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 8u8,
    };
    pub const DATA_BEAM_TARGET_ID: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
    pub const DATA_BEE_FLAGS: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
    pub const DATA_BEGGING: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 255u8,
    };
    pub const DATA_BILLBOARD: TrackedId = TrackedId {
        latest: 15u8,
        v1_21_7: 15u8,
    };
    pub const DATA_BLAZE_FLAGS: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_BLOCK_OFFSET: TrackedId = TrackedId {
        latest: 12u8,
        v1_21_7: 255u8,
    };
    pub const DATA_BLOCK_POS: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 255u8,
    };
    pub const DATA_BLOCK_STATE: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 8u8,
    };
    pub const DATA_BOOST_TIME: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_BRIGHTNESS: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_BUBBLE_WOBBLE_TICKS: TrackedId = TrackedId {
        latest: 13u8,
        v1_21_7: 13u8,
    };
    pub const DATA_CAN_DUPLICATE: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_CARRIED_BLOCK: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_CAT_VARIANT: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 19u8,
    };
    pub const DATA_CAUGHT_FISH: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 255u8,
    };
    pub const DATA_CELEBRATING: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_CHARGED: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 19u8,
    };
    pub const DATA_CHARGING: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_CHEST: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_CHILD: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_COLD: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_COLLAR_COLOR: TrackedId = TrackedId {
        latest: 20u8,
        v1_21_7: 20u8,
    };
    pub const DATA_COLOR: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_COMMAND: TrackedId = TrackedId {
        latest: 13u8,
        v1_21_7: 255u8,
    };
    pub const DATA_CONVERTING: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 19u8,
    };
    pub const DATA_CONVERTING_IN_WATER: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 18u8,
    };
    pub const DATA_COPPER_GOLEM_STATE: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_CRUMBLING: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_CUSTOM_BLOCK_STATE: TrackedId = TrackedId {
        latest: 11u8,
        v1_21_7: 255u8,
    };
    pub const DATA_CUSTOM_NAME: TrackedId = TrackedId {
        latest: 2u8,
        v1_21_7: 2u8,
    };
    pub const DATA_DAMAGE_WOBBLE_SIDE: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 255u8,
    };
    pub const DATA_DAMAGE_WOBBLE_STRENGTH: TrackedId = TrackedId {
        latest: 10u8,
        v1_21_7: 255u8,
    };
    pub const DATA_DAMAGE_WOBBLE_TICKS: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 255u8,
    };
    pub const DATA_DANCING: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 255u8,
    };
    pub const DATA_DARK_TICKS_REMAINING: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_DASHING: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 255u8,
    };
    pub const DATA_DESCRIPTION: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 255u8,
    };
    pub const DATA_DIGGING_SAND: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_DRINKING: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_EATING_TICKS: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 19u8,
    };
    pub const DATA_ENCHANTED: TrackedId = TrackedId {
        latest: 12u8,
        v1_21_7: 255u8,
    };
    pub const DATA_FACING: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 255u8,
    };
    pub const DATA_FINISH_DIG_TIME: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_FLAGS: TrackedId = TrackedId {
        latest: 0u8,
        v1_21_7: 0u8,
    };
    pub const DATA_FOX_FLAGS: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 18u8,
    };
    pub const DATA_FROM_BUCKET: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_FROZEN_TICKS: TrackedId = TrackedId {
        latest: 7u8,
        v1_21_7: 7u8,
    };
    pub const DATA_FUSE: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 8u8,
    };
    pub const DATA_FUSE_SPEED: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_GLOW_COLOR_OVERRIDE: TrackedId = TrackedId {
        latest: 22u8,
        v1_21_7: 22u8,
    };
    pub const DATA_HAS_EGG: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_HAS_FISH: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_HAS_ROPES: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_HEAD_DOWN: TrackedId = TrackedId {
        latest: 21u8,
        v1_21_7: 255u8,
    };
    pub const DATA_HEAD_ROLLING_TIME_LEFT: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_HEALTH: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 9u8,
    };
    pub const DATA_HEIGHT: TrackedId = TrackedId {
        latest: 21u8,
        v1_21_7: 21u8,
    };
    pub const DATA_HIDDEN_GENE: TrackedId = TrackedId {
        latest: 21u8,
        v1_21_7: 21u8,
    };
    pub const DATA_HOME_POS: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 255u8,
    };
    pub const DATA_HOOK_ENTITY_ID: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 255u8,
    };
    pub const DATA_HORSE_FLAGS: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
    pub const DATA_IGNITED: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 18u8,
    };
    pub const DATA_IMMOVABLE: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_IMMUNE_TO_ZOMBIFICATION: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_INTERPOLATION_DURATION: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 9u8,
    };
    pub const DATA_INVUL_TIMER: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 255u8,
    };
    pub const DATA_IN_GROUND: TrackedId = TrackedId {
        latest: 10u8,
        v1_21_7: 255u8,
    };
    pub const DATA_IN_SLEEPING_POSE: TrackedId = TrackedId {
        latest: 20u8,
        v1_21_7: 255u8,
    };
    pub const DATA_IRON_GOLEM_FLAGS: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_ITEM: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 255u8,
    };
    pub const DATA_ITEM_DISPLAY: TrackedId = TrackedId {
        latest: 24u8,
        v1_21_7: 24u8,
    };
    pub const DATA_ITEM_STACK: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 8u8,
    };
    pub const DATA_LAST_OUTPUT: TrackedId = TrackedId {
        latest: 14u8,
        v1_21_7: 255u8,
    };
    pub const DATA_LAST_POSE_TICK: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 255u8,
    };
    pub const DATA_LEFT_HORN: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_LEFT_PADDLE_MOVING: TrackedId = TrackedId {
        latest: 11u8,
        v1_21_7: 11u8,
    };
    pub const DATA_LEFT_ROTATION: TrackedId = TrackedId {
        latest: 13u8,
        v1_21_7: 13u8,
    };
    pub const DATA_LINE_WIDTH: TrackedId = TrackedId {
        latest: 24u8,
        v1_21_7: 24u8,
    };
    pub const DATA_LIT: TrackedId = TrackedId {
        latest: 13u8,
        v1_21_7: 255u8,
    };
    pub const DATA_LIVING_FLAGS: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 8u8,
    };
    pub const DATA_LOYALTY: TrackedId = TrackedId {
        latest: 11u8,
        v1_21_7: 255u8,
    };
    pub const DATA_MAIN_ARM_ID: TrackedId = TrackedId {
        latest: 15u8,
        v1_21_7: 18u8,
    };
    pub const DATA_MAIN_GENE: TrackedId = TrackedId {
        latest: 20u8,
        v1_21_7: 20u8,
    };
    pub const DATA_MOB_FLAGS: TrackedId = TrackedId {
        latest: 15u8,
        v1_21_7: 15u8,
    };
    pub const DATA_MOISTNESS: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_NAME_VISIBLE: TrackedId = TrackedId {
        latest: 3u8,
        v1_21_7: 3u8,
    };
    pub const DATA_NO_GRAVITY: TrackedId = TrackedId {
        latest: 5u8,
        v1_21_7: 5u8,
    };
    pub const DATA_OTHER_TRUSTED: TrackedId = TrackedId {
        latest: 20u8,
        v1_21_7: 255u8,
    };
    pub const DATA_OWNER: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 255u8,
    };
    pub const DATA_OWNER_UUID: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 18u8,
    };
    pub const DATA_OXIDATION_LEVEL: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_PANDA_FLAGS: TrackedId = TrackedId {
        latest: 22u8,
        v1_21_7: 22u8,
    };
    pub const DATA_PARTICLE: TrackedId = TrackedId {
        latest: 10u8,
        v1_21_7: 255u8,
    };
    pub const DATA_PEEK_AMOUNT: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
    pub const DATA_PHASE_TYPE: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_PIERCE_LEVEL: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 255u8,
    };
    pub const DATA_PLAYER_MODE_CUSTOMIZATION_ID: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 17u8,
    };
    pub const DATA_PLAYING_DEAD: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_POSE: TrackedId = TrackedId {
        latest: 6u8,
        v1_21_7: 6u8,
    };
    pub const DATA_POTION_SWIRLS: TrackedId = TrackedId {
        latest: 10u8,
        v1_21_7: 10u8,
    };
    pub const DATA_POTION_SWIRLS_AMBIENT: TrackedId = TrackedId {
        latest: 11u8,
        v1_21_7: 11u8,
    };
    pub const DATA_PROFILE: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_PROJECTILE_FLAGS: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 255u8,
    };
    pub const DATA_PROVOKED: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 18u8,
    };
    pub const DATA_PUFF_STATE: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
    pub const DATA_RADIUS: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 255u8,
    };
    pub const DATA_RESPONSE: TrackedId = TrackedId {
        latest: 10u8,
        v1_21_7: 255u8,
    };
    pub const DATA_RIGHT_HORN: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 255u8,
    };
    pub const DATA_RIGHT_PADDLE_MOVING: TrackedId = TrackedId {
        latest: 12u8,
        v1_21_7: 12u8,
    };
    pub const DATA_RIGHT_ROTATION: TrackedId = TrackedId {
        latest: 14u8,
        v1_21_7: 14u8,
    };
    pub const DATA_ROTATION: TrackedId = TrackedId {
        latest: 10u8,
        v1_21_7: 9u8,
    };
    pub const DATA_SCALE: TrackedId = TrackedId {
        latest: 12u8,
        v1_21_7: 12u8,
    };
    pub const DATA_SCREAMING: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_SHADOW_RADIUS: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 18u8,
    };
    pub const DATA_SHADOW_STRENGTH: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 19u8,
    };
    pub const DATA_SHEARED: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_SHOOTER_ENTITY_ID: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 255u8,
    };
    pub const DATA_SHOOTING: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_SHOT_AT_ANGLE: TrackedId = TrackedId {
        latest: 10u8,
        v1_21_7: 255u8,
    };
    pub const DATA_SHOW_BOTTOM: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 9u8,
    };
    pub const DATA_SILENT: TrackedId = TrackedId {
        latest: 4u8,
        v1_21_7: 4u8,
    };
    pub const DATA_SIZE: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_SLEEPING_POSITION: TrackedId = TrackedId {
        latest: 14u8,
        v1_21_7: 14u8,
    };
    pub const DATA_SLIME_SIZE: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_SNEEZE_PROGRESS: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 18u8,
    };
    pub const DATA_SNOW_GOLEM_FLAGS: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_SOUND_VARIANT: TrackedId = TrackedId {
        latest: 23u8,
        v1_21_7: 255u8,
    };
    pub const DATA_SPELL: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_SPIDER_FLAGS: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_SPIKES_RETRACTED: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_STACK: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 255u8,
    };
    pub const DATA_START_INTERPOLATION: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 8u8,
    };
    pub const DATA_STATE: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_STAYING_STILL: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_STINGER_COUNT: TrackedId = TrackedId {
        latest: 13u8,
        v1_21_7: 13u8,
    };
    pub const DATA_STRENGTH: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 19u8,
    };
    pub const DATA_STUCK_ARROW_COUNT: TrackedId = TrackedId {
        latest: 12u8,
        v1_21_7: 12u8,
    };
    pub const DATA_TAMEABLE_FLAGS: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
    pub const DATA_TARGET: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_TELEPORT_DURATION: TrackedId = TrackedId {
        latest: 10u8,
        v1_21_7: 10u8,
    };
    pub const DATA_TEXT: TrackedId = TrackedId {
        latest: 23u8,
        v1_21_7: 23u8,
    };
    pub const DATA_TEXT_DISPLAY_FLAGS: TrackedId = TrackedId {
        latest: 27u8,
        v1_21_7: 27u8,
    };
    pub const DATA_TEXT_OPACITY: TrackedId = TrackedId {
        latest: 26u8,
        v1_21_7: 26u8,
    };
    pub const DATA_TRACKED_ENTITY_ID_1: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_TRACKED_ENTITY_ID_2: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_TRACKED_ENTITY_ID_3: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 255u8,
    };
    pub const DATA_TRACKER_BODY_ROTATION: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
    pub const DATA_TRACKER_HEAD_ROTATION: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_TRACKER_LEFT_ARM_ROTATION: TrackedId = TrackedId {
        latest: 18u8,
        v1_21_7: 18u8,
    };
    pub const DATA_TRACKER_LEFT_LEG_ROTATION: TrackedId = TrackedId {
        latest: 20u8,
        v1_21_7: 20u8,
    };
    pub const DATA_TRACKER_RIGHT_ARM_ROTATION: TrackedId = TrackedId {
        latest: 19u8,
        v1_21_7: 19u8,
    };
    pub const DATA_TRACKER_RIGHT_LEG_ROTATION: TrackedId = TrackedId {
        latest: 21u8,
        v1_21_7: 21u8,
    };
    pub const DATA_TRANSLATION: TrackedId = TrackedId {
        latest: 11u8,
        v1_21_7: 11u8,
    };
    pub const DATA_TRUSTING: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_UNROOTED: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 255u8,
    };
    pub const DATA_VALUE: TrackedId = TrackedId {
        latest: 8u8,
        v1_21_7: 255u8,
    };
    pub const DATA_VARIANT: TrackedId = TrackedId {
        latest: 20u8,
        v1_21_7: 20u8,
    };
    pub const DATA_VEX_FLAGS: TrackedId = TrackedId {
        latest: 16u8,
        v1_21_7: 16u8,
    };
    pub const DATA_VIEW_RANGE: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
    pub const DATA_VILLAGER_DATA: TrackedId = TrackedId {
        latest: 20u8,
        v1_21_7: 18u8,
    };
    pub const DATA_WAITING: TrackedId = TrackedId {
        latest: 9u8,
        v1_21_7: 255u8,
    };
    pub const DATA_WARNING: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 255u8,
    };
    pub const DATA_WIDTH: TrackedId = TrackedId {
        latest: 20u8,
        v1_21_7: 20u8,
    };
    pub const DATA_ZOMBIE_TYPE: TrackedId = TrackedId {
        latest: 17u8,
        v1_21_7: 17u8,
    };
}
