/* This file is generated. Do not edit manually. */
use pumpkin_util::version::MinecraftVersion;
pub struct TrackedId {
    pub v1_21: u8,
    pub v1_21_2: u8,
    pub v1_21_4: u8,
    pub v1_21_5: u8,
    pub v1_21_6: u8,
    pub v1_21_7: u8,
    pub v1_21_9: u8,
    pub v1_21_11: u8,
}
impl TrackedId {
    pub fn get(&self, version: &MinecraftVersion) -> u8 {
        match version {
            pumpkin_util::version::MinecraftVersion::V_1_21 => self.v1_21,
            pumpkin_util::version::MinecraftVersion::V_1_21_2 => self.v1_21_2,
            pumpkin_util::version::MinecraftVersion::V_1_21_4 => self.v1_21_4,
            pumpkin_util::version::MinecraftVersion::V_1_21_5 => self.v1_21_5,
            pumpkin_util::version::MinecraftVersion::V_1_21_6 => self.v1_21_6,
            pumpkin_util::version::MinecraftVersion::V_1_21_7 => self.v1_21_7,
            pumpkin_util::version::MinecraftVersion::V_1_21_9 => self.v1_21_9,
            pumpkin_util::version::MinecraftVersion::V_1_21_11 => self.v1_21_11,
            _ => self.v1_21_11,
        }
    }
}
impl From<TrackedId> for u8 {
    fn from(id: TrackedId) -> u8 {
        id.v1_21_11
    }
}
pub struct TrackedData;
impl TrackedData {
    pub const DATA_ACTIVE: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_AIR: TrackedId = TrackedId {
        v1_21: 1u8,
        v1_21_2: 1u8,
        v1_21_4: 1u8,
        v1_21_5: 1u8,
        v1_21_6: 1u8,
        v1_21_7: 1u8,
        v1_21_9: 1u8,
        v1_21_11: 1u8,
    };
    pub const DATA_ANGER: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_ANGER_END_TIME: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 255u8,
        v1_21_7: 21u8,
        v1_21_9: 255u8,
        v1_21_11: 21u8,
    };
    pub const DATA_ANGRY: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_ARMOR_STAND_FLAGS: TrackedId = TrackedId {
        v1_21: 15u8,
        v1_21_2: 15u8,
        v1_21_4: 15u8,
        v1_21_5: 15u8,
        v1_21_6: 15u8,
        v1_21_7: 15u8,
        v1_21_9: 15u8,
        v1_21_11: 15u8,
    };
    pub const DATA_ASK_FOR_BAMBOO_TICKS: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_ATTACHED_FACE: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_BABY: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_BACKGROUND: TrackedId = TrackedId {
        v1_21: 25u8,
        v1_21_2: 25u8,
        v1_21_4: 25u8,
        v1_21_5: 25u8,
        v1_21_6: 25u8,
        v1_21_7: 25u8,
        v1_21_9: 25u8,
        v1_21_11: 25u8,
    };
    pub const DATA_BAT_FLAGS: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_BEAM_TARGET: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 8u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_BEAM_TARGET_ID: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_BEE_FLAGS: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_BEGGING: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 255u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_BILLBOARD: TrackedId = TrackedId {
        v1_21: 15u8,
        v1_21_2: 15u8,
        v1_21_4: 15u8,
        v1_21_5: 15u8,
        v1_21_6: 15u8,
        v1_21_7: 15u8,
        v1_21_9: 15u8,
        v1_21_11: 15u8,
    };
    pub const DATA_BLAZE_FLAGS: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_BLOCK_OFFSET: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 12u8,
        v1_21_6: 12u8,
        v1_21_7: 255u8,
        v1_21_9: 12u8,
        v1_21_11: 12u8,
    };
    pub const DATA_BLOCK_POS: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 255u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_BLOCK_STATE: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 9u8,
        v1_21_7: 8u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_BOOST_TIME: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_BRIGHTNESS: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_BUBBLE_WOBBLE_TICKS: TrackedId = TrackedId {
        v1_21: 14u8,
        v1_21_2: 13u8,
        v1_21_4: 13u8,
        v1_21_5: 13u8,
        v1_21_6: 13u8,
        v1_21_7: 13u8,
        v1_21_9: 13u8,
        v1_21_11: 13u8,
    };
    pub const DATA_CAN_DUPLICATE: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_CARRIED_BLOCK: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_CAT_VARIANT: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 19u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_CAUGHT_FISH: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 9u8,
        v1_21_7: 255u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_CELEBRATING: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_CHARGED: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 19u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_CHARGING: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_CHEST: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_CHILD: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_COLD: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_COLLAR_COLOR: TrackedId = TrackedId {
        v1_21: 20u8,
        v1_21_2: 20u8,
        v1_21_4: 20u8,
        v1_21_5: 20u8,
        v1_21_6: 20u8,
        v1_21_7: 20u8,
        v1_21_9: 20u8,
        v1_21_11: 20u8,
    };
    pub const DATA_COLOR: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_COMMAND: TrackedId = TrackedId {
        v1_21: 14u8,
        v1_21_2: 14u8,
        v1_21_4: 14u8,
        v1_21_5: 13u8,
        v1_21_6: 13u8,
        v1_21_7: 255u8,
        v1_21_9: 13u8,
        v1_21_11: 13u8,
    };
    pub const DATA_CONVERTING: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 19u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_CONVERTING_IN_WATER: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 18u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_COPPER_GOLEM_STATE: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 255u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_CRUMBLING: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_CUSTOM_BLOCK_STATE: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 11u8,
        v1_21_6: 11u8,
        v1_21_7: 255u8,
        v1_21_9: 11u8,
        v1_21_11: 11u8,
    };
    pub const DATA_CUSTOM_NAME: TrackedId = TrackedId {
        v1_21: 2u8,
        v1_21_2: 2u8,
        v1_21_4: 2u8,
        v1_21_5: 2u8,
        v1_21_6: 2u8,
        v1_21_7: 2u8,
        v1_21_9: 2u8,
        v1_21_11: 2u8,
    };
    pub const DATA_DAMAGE_WOBBLE_SIDE: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 9u8,
        v1_21_7: 255u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_DAMAGE_WOBBLE_STRENGTH: TrackedId = TrackedId {
        v1_21: 10u8,
        v1_21_2: 10u8,
        v1_21_4: 10u8,
        v1_21_5: 10u8,
        v1_21_6: 10u8,
        v1_21_7: 255u8,
        v1_21_9: 10u8,
        v1_21_11: 10u8,
    };
    pub const DATA_DAMAGE_WOBBLE_TICKS: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 255u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_DANCING: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 255u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_DARK_TICKS_REMAINING: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_DASHING: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 19u8,
    };
    pub const DATA_DESCRIPTION: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 255u8,
        v1_21_7: 255u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_DIGGING_SAND: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_DRINKING: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_EATING_TICKS: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 19u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_ENCHANTED: TrackedId = TrackedId {
        v1_21: 11u8,
        v1_21_2: 12u8,
        v1_21_4: 12u8,
        v1_21_5: 12u8,
        v1_21_6: 12u8,
        v1_21_7: 255u8,
        v1_21_9: 12u8,
        v1_21_11: 12u8,
    };
    pub const DATA_FACING: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 8u8,
        v1_21_7: 255u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_FINISH_DIG_TIME: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_FLAGS: TrackedId = TrackedId {
        v1_21: 0u8,
        v1_21_2: 0u8,
        v1_21_4: 0u8,
        v1_21_5: 0u8,
        v1_21_6: 0u8,
        v1_21_7: 0u8,
        v1_21_9: 0u8,
        v1_21_11: 0u8,
    };
    pub const DATA_FOX_FLAGS: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 18u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_FROM_BUCKET: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_FROZEN_TICKS: TrackedId = TrackedId {
        v1_21: 7u8,
        v1_21_2: 7u8,
        v1_21_4: 7u8,
        v1_21_5: 7u8,
        v1_21_6: 7u8,
        v1_21_7: 7u8,
        v1_21_9: 7u8,
        v1_21_11: 7u8,
    };
    pub const DATA_FUSE: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 8u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_FUSE_SPEED: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_GLOW_COLOR_OVERRIDE: TrackedId = TrackedId {
        v1_21: 22u8,
        v1_21_2: 22u8,
        v1_21_4: 22u8,
        v1_21_5: 22u8,
        v1_21_6: 22u8,
        v1_21_7: 22u8,
        v1_21_9: 22u8,
        v1_21_11: 22u8,
    };
    pub const DATA_HAS_EGG: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_HAS_FISH: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_HAS_ROPES: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_HEAD_DOWN: TrackedId = TrackedId {
        v1_21: 21u8,
        v1_21_2: 21u8,
        v1_21_4: 21u8,
        v1_21_5: 21u8,
        v1_21_6: 21u8,
        v1_21_7: 255u8,
        v1_21_9: 21u8,
        v1_21_11: 21u8,
    };
    pub const DATA_HEAD_ROLLING_TIME_LEFT: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_HEALTH: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 9u8,
        v1_21_7: 9u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_HEIGHT: TrackedId = TrackedId {
        v1_21: 21u8,
        v1_21_2: 21u8,
        v1_21_4: 21u8,
        v1_21_5: 21u8,
        v1_21_6: 21u8,
        v1_21_7: 21u8,
        v1_21_9: 21u8,
        v1_21_11: 21u8,
    };
    pub const DATA_HIDDEN_GENE: TrackedId = TrackedId {
        v1_21: 21u8,
        v1_21_2: 21u8,
        v1_21_4: 21u8,
        v1_21_5: 21u8,
        v1_21_6: 21u8,
        v1_21_7: 21u8,
        v1_21_9: 21u8,
        v1_21_11: 21u8,
    };
    pub const DATA_HOME_POS: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 255u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_HOOK_ENTITY_ID: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 255u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_HORSE_FLAGS: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_IGNITED: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 18u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_IMMOVABLE: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 255u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_IMMUNE_TO_ZOMBIFICATION: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_INTERPOLATION_DURATION: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 9u8,
        v1_21_7: 9u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_INVUL_TIMER: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 255u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_IN_GROUND: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 10u8,
        v1_21_4: 10u8,
        v1_21_5: 10u8,
        v1_21_6: 10u8,
        v1_21_7: 255u8,
        v1_21_9: 10u8,
        v1_21_11: 10u8,
    };
    pub const DATA_IN_SLEEPING_POSE: TrackedId = TrackedId {
        v1_21: 20u8,
        v1_21_2: 20u8,
        v1_21_4: 20u8,
        v1_21_5: 20u8,
        v1_21_6: 20u8,
        v1_21_7: 255u8,
        v1_21_9: 20u8,
        v1_21_11: 20u8,
    };
    pub const DATA_IRON_GOLEM_FLAGS: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_ITEM: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 255u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_ITEM_DISPLAY: TrackedId = TrackedId {
        v1_21: 24u8,
        v1_21_2: 24u8,
        v1_21_4: 24u8,
        v1_21_5: 24u8,
        v1_21_6: 24u8,
        v1_21_7: 24u8,
        v1_21_9: 24u8,
        v1_21_11: 24u8,
    };
    pub const DATA_ITEM_STACK: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 9u8,
        v1_21_7: 8u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_LAST_OUTPUT: TrackedId = TrackedId {
        v1_21: 15u8,
        v1_21_2: 15u8,
        v1_21_4: 15u8,
        v1_21_5: 14u8,
        v1_21_6: 14u8,
        v1_21_7: 255u8,
        v1_21_9: 14u8,
        v1_21_11: 14u8,
    };
    pub const DATA_LAST_POSE_TICK: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 255u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_LEFT_HORN: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_LEFT_PADDLE_MOVING: TrackedId = TrackedId {
        v1_21: 12u8,
        v1_21_2: 11u8,
        v1_21_4: 11u8,
        v1_21_5: 11u8,
        v1_21_6: 11u8,
        v1_21_7: 11u8,
        v1_21_9: 11u8,
        v1_21_11: 11u8,
    };
    pub const DATA_LEFT_ROTATION: TrackedId = TrackedId {
        v1_21: 13u8,
        v1_21_2: 13u8,
        v1_21_4: 13u8,
        v1_21_5: 13u8,
        v1_21_6: 13u8,
        v1_21_7: 13u8,
        v1_21_9: 13u8,
        v1_21_11: 13u8,
    };
    pub const DATA_LINE_WIDTH: TrackedId = TrackedId {
        v1_21: 24u8,
        v1_21_2: 24u8,
        v1_21_4: 24u8,
        v1_21_5: 24u8,
        v1_21_6: 24u8,
        v1_21_7: 24u8,
        v1_21_9: 24u8,
        v1_21_11: 24u8,
    };
    pub const DATA_LIT: TrackedId = TrackedId {
        v1_21: 14u8,
        v1_21_2: 14u8,
        v1_21_4: 14u8,
        v1_21_5: 13u8,
        v1_21_6: 13u8,
        v1_21_7: 255u8,
        v1_21_9: 13u8,
        v1_21_11: 13u8,
    };
    pub const DATA_LIVING_FLAGS: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 8u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_LOYALTY: TrackedId = TrackedId {
        v1_21: 10u8,
        v1_21_2: 11u8,
        v1_21_4: 11u8,
        v1_21_5: 11u8,
        v1_21_6: 11u8,
        v1_21_7: 255u8,
        v1_21_9: 11u8,
        v1_21_11: 11u8,
    };
    pub const DATA_MAIN_ARM_ID: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 255u8,
        v1_21_7: 18u8,
        v1_21_9: 15u8,
        v1_21_11: 15u8,
    };
    pub const DATA_MAIN_GENE: TrackedId = TrackedId {
        v1_21: 20u8,
        v1_21_2: 20u8,
        v1_21_4: 20u8,
        v1_21_5: 20u8,
        v1_21_6: 20u8,
        v1_21_7: 20u8,
        v1_21_9: 20u8,
        v1_21_11: 20u8,
    };
    pub const DATA_MOB_FLAGS: TrackedId = TrackedId {
        v1_21: 15u8,
        v1_21_2: 15u8,
        v1_21_4: 15u8,
        v1_21_5: 15u8,
        v1_21_6: 15u8,
        v1_21_7: 15u8,
        v1_21_9: 15u8,
        v1_21_11: 15u8,
    };
    pub const DATA_MOISTNESS: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_NAME_VISIBLE: TrackedId = TrackedId {
        v1_21: 3u8,
        v1_21_2: 3u8,
        v1_21_4: 3u8,
        v1_21_5: 3u8,
        v1_21_6: 3u8,
        v1_21_7: 3u8,
        v1_21_9: 3u8,
        v1_21_11: 3u8,
    };
    pub const DATA_NO_GRAVITY: TrackedId = TrackedId {
        v1_21: 5u8,
        v1_21_2: 5u8,
        v1_21_4: 5u8,
        v1_21_5: 5u8,
        v1_21_6: 5u8,
        v1_21_7: 5u8,
        v1_21_9: 5u8,
        v1_21_11: 5u8,
    };
    pub const DATA_OTHER_TRUSTED: TrackedId = TrackedId {
        v1_21: 20u8,
        v1_21_2: 20u8,
        v1_21_4: 20u8,
        v1_21_5: 20u8,
        v1_21_6: 20u8,
        v1_21_7: 255u8,
        v1_21_9: 20u8,
        v1_21_11: 20u8,
    };
    pub const DATA_OWNER: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 255u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_OWNER_UUID: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 18u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_OXIDATION_LEVEL: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 255u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_PANDA_FLAGS: TrackedId = TrackedId {
        v1_21: 22u8,
        v1_21_2: 22u8,
        v1_21_4: 22u8,
        v1_21_5: 22u8,
        v1_21_6: 22u8,
        v1_21_7: 22u8,
        v1_21_9: 22u8,
        v1_21_11: 22u8,
    };
    pub const DATA_PARTICLE: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 10u8,
        v1_21_6: 10u8,
        v1_21_7: 255u8,
        v1_21_9: 10u8,
        v1_21_11: 10u8,
    };
    pub const DATA_PEEK_AMOUNT: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_PHASE_TYPE: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_PIERCE_LEVEL: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 9u8,
        v1_21_7: 255u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_PLAYER_MODE_CUSTOMIZATION_ID: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 255u8,
        v1_21_7: 17u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_PLAYING_DEAD: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_POSE: TrackedId = TrackedId {
        v1_21: 6u8,
        v1_21_2: 6u8,
        v1_21_4: 6u8,
        v1_21_5: 6u8,
        v1_21_6: 6u8,
        v1_21_7: 6u8,
        v1_21_9: 6u8,
        v1_21_11: 6u8,
    };
    pub const DATA_POTION_SWIRLS: TrackedId = TrackedId {
        v1_21: 10u8,
        v1_21_2: 10u8,
        v1_21_4: 10u8,
        v1_21_5: 10u8,
        v1_21_6: 10u8,
        v1_21_7: 10u8,
        v1_21_9: 10u8,
        v1_21_11: 10u8,
    };
    pub const DATA_POTION_SWIRLS_AMBIENT: TrackedId = TrackedId {
        v1_21: 11u8,
        v1_21_2: 11u8,
        v1_21_4: 11u8,
        v1_21_5: 11u8,
        v1_21_6: 11u8,
        v1_21_7: 11u8,
        v1_21_9: 11u8,
        v1_21_11: 11u8,
    };
    pub const DATA_PROFILE: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 255u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_PROJECTILE_FLAGS: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 255u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_PROVOKED: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 18u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_PUFF_STATE: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_RADIUS: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 255u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_RESPONSE: TrackedId = TrackedId {
        v1_21: 10u8,
        v1_21_2: 10u8,
        v1_21_4: 10u8,
        v1_21_5: 10u8,
        v1_21_6: 10u8,
        v1_21_7: 255u8,
        v1_21_9: 10u8,
        v1_21_11: 10u8,
    };
    pub const DATA_RIGHT_HORN: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 255u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_RIGHT_PADDLE_MOVING: TrackedId = TrackedId {
        v1_21: 13u8,
        v1_21_2: 12u8,
        v1_21_4: 12u8,
        v1_21_5: 12u8,
        v1_21_6: 12u8,
        v1_21_7: 12u8,
        v1_21_9: 12u8,
        v1_21_11: 12u8,
    };
    pub const DATA_RIGHT_ROTATION: TrackedId = TrackedId {
        v1_21: 14u8,
        v1_21_2: 14u8,
        v1_21_4: 14u8,
        v1_21_5: 14u8,
        v1_21_6: 14u8,
        v1_21_7: 14u8,
        v1_21_9: 14u8,
        v1_21_11: 14u8,
    };
    pub const DATA_ROTATION: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 10u8,
        v1_21_7: 9u8,
        v1_21_9: 10u8,
        v1_21_11: 10u8,
    };
    pub const DATA_SCALE: TrackedId = TrackedId {
        v1_21: 12u8,
        v1_21_2: 12u8,
        v1_21_4: 12u8,
        v1_21_5: 12u8,
        v1_21_6: 12u8,
        v1_21_7: 12u8,
        v1_21_9: 12u8,
        v1_21_11: 12u8,
    };
    pub const DATA_SCREAMING: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_SHADOW_RADIUS: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 18u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_SHADOW_STRENGTH: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 19u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_SHEARED: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_SHOOTER_ENTITY_ID: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 9u8,
        v1_21_7: 255u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_SHOOTING: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_SHOT_AT_ANGLE: TrackedId = TrackedId {
        v1_21: 10u8,
        v1_21_2: 10u8,
        v1_21_4: 10u8,
        v1_21_5: 10u8,
        v1_21_6: 10u8,
        v1_21_7: 255u8,
        v1_21_9: 10u8,
        v1_21_11: 10u8,
    };
    pub const DATA_SHOW_BOTTOM: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 9u8,
        v1_21_7: 9u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_SILENT: TrackedId = TrackedId {
        v1_21: 4u8,
        v1_21_2: 4u8,
        v1_21_4: 4u8,
        v1_21_5: 4u8,
        v1_21_6: 4u8,
        v1_21_7: 4u8,
        v1_21_9: 4u8,
        v1_21_11: 4u8,
    };
    pub const DATA_SIZE: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_SLEEPING_POSITION: TrackedId = TrackedId {
        v1_21: 14u8,
        v1_21_2: 14u8,
        v1_21_4: 14u8,
        v1_21_5: 14u8,
        v1_21_6: 14u8,
        v1_21_7: 14u8,
        v1_21_9: 14u8,
        v1_21_11: 14u8,
    };
    pub const DATA_SLIME_SIZE: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_SNEEZE_PROGRESS: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 18u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_SNOW_GOLEM_FLAGS: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_SOUND_VARIANT: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 23u8,
        v1_21_6: 23u8,
        v1_21_7: 255u8,
        v1_21_9: 23u8,
        v1_21_11: 23u8,
    };
    pub const DATA_SPELL: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_SPIDER_FLAGS: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_SPIKES_RETRACTED: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_STACK: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 255u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_START_INTERPOLATION: TrackedId = TrackedId {
        v1_21: 8u8,
        v1_21_2: 8u8,
        v1_21_4: 8u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 8u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_STATE: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_STAYING_STILL: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 255u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_STINGER_COUNT: TrackedId = TrackedId {
        v1_21: 13u8,
        v1_21_2: 13u8,
        v1_21_4: 13u8,
        v1_21_5: 13u8,
        v1_21_6: 13u8,
        v1_21_7: 13u8,
        v1_21_9: 13u8,
        v1_21_11: 13u8,
    };
    pub const DATA_STRENGTH: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 19u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_STUCK_ARROW_COUNT: TrackedId = TrackedId {
        v1_21: 12u8,
        v1_21_2: 12u8,
        v1_21_4: 12u8,
        v1_21_5: 12u8,
        v1_21_6: 12u8,
        v1_21_7: 12u8,
        v1_21_9: 12u8,
        v1_21_11: 12u8,
    };
    pub const DATA_TAMEABLE_FLAGS: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_TARGET: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_TELEPORT_DURATION: TrackedId = TrackedId {
        v1_21: 10u8,
        v1_21_2: 10u8,
        v1_21_4: 10u8,
        v1_21_5: 10u8,
        v1_21_6: 10u8,
        v1_21_7: 10u8,
        v1_21_9: 10u8,
        v1_21_11: 10u8,
    };
    pub const DATA_TEXT: TrackedId = TrackedId {
        v1_21: 23u8,
        v1_21_2: 23u8,
        v1_21_4: 23u8,
        v1_21_5: 23u8,
        v1_21_6: 23u8,
        v1_21_7: 23u8,
        v1_21_9: 23u8,
        v1_21_11: 23u8,
    };
    pub const DATA_TEXT_DISPLAY_FLAGS: TrackedId = TrackedId {
        v1_21: 27u8,
        v1_21_2: 27u8,
        v1_21_4: 27u8,
        v1_21_5: 27u8,
        v1_21_6: 27u8,
        v1_21_7: 27u8,
        v1_21_9: 27u8,
        v1_21_11: 27u8,
    };
    pub const DATA_TEXT_OPACITY: TrackedId = TrackedId {
        v1_21: 26u8,
        v1_21_2: 26u8,
        v1_21_4: 26u8,
        v1_21_5: 26u8,
        v1_21_6: 26u8,
        v1_21_7: 26u8,
        v1_21_9: 26u8,
        v1_21_11: 26u8,
    };
    pub const DATA_TRACKED_ENTITY_ID_1: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_TRACKED_ENTITY_ID_2: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_TRACKED_ENTITY_ID_3: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 255u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_TRACKER_BODY_ROTATION: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_TRACKER_HEAD_ROTATION: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_TRACKER_LEFT_ARM_ROTATION: TrackedId = TrackedId {
        v1_21: 18u8,
        v1_21_2: 18u8,
        v1_21_4: 18u8,
        v1_21_5: 18u8,
        v1_21_6: 18u8,
        v1_21_7: 18u8,
        v1_21_9: 18u8,
        v1_21_11: 18u8,
    };
    pub const DATA_TRACKER_LEFT_LEG_ROTATION: TrackedId = TrackedId {
        v1_21: 20u8,
        v1_21_2: 20u8,
        v1_21_4: 20u8,
        v1_21_5: 20u8,
        v1_21_6: 20u8,
        v1_21_7: 20u8,
        v1_21_9: 20u8,
        v1_21_11: 20u8,
    };
    pub const DATA_TRACKER_RIGHT_ARM_ROTATION: TrackedId = TrackedId {
        v1_21: 19u8,
        v1_21_2: 19u8,
        v1_21_4: 19u8,
        v1_21_5: 19u8,
        v1_21_6: 19u8,
        v1_21_7: 19u8,
        v1_21_9: 19u8,
        v1_21_11: 19u8,
    };
    pub const DATA_TRACKER_RIGHT_LEG_ROTATION: TrackedId = TrackedId {
        v1_21: 21u8,
        v1_21_2: 21u8,
        v1_21_4: 21u8,
        v1_21_5: 21u8,
        v1_21_6: 21u8,
        v1_21_7: 21u8,
        v1_21_9: 21u8,
        v1_21_11: 21u8,
    };
    pub const DATA_TRANSLATION: TrackedId = TrackedId {
        v1_21: 11u8,
        v1_21_2: 11u8,
        v1_21_4: 11u8,
        v1_21_5: 11u8,
        v1_21_6: 11u8,
        v1_21_7: 11u8,
        v1_21_9: 11u8,
        v1_21_11: 11u8,
    };
    pub const DATA_TRUSTING: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_UNROOTED: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 255u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_VALUE: TrackedId = TrackedId {
        v1_21: 255u8,
        v1_21_2: 255u8,
        v1_21_4: 255u8,
        v1_21_5: 8u8,
        v1_21_6: 8u8,
        v1_21_7: 255u8,
        v1_21_9: 8u8,
        v1_21_11: 8u8,
    };
    pub const DATA_VARIANT: TrackedId = TrackedId {
        v1_21: 22u8,
        v1_21_2: 22u8,
        v1_21_4: 22u8,
        v1_21_5: 22u8,
        v1_21_6: 22u8,
        v1_21_7: 20u8,
        v1_21_9: 22u8,
        v1_21_11: 20u8,
    };
    pub const DATA_VEX_FLAGS: TrackedId = TrackedId {
        v1_21: 16u8,
        v1_21_2: 16u8,
        v1_21_4: 16u8,
        v1_21_5: 16u8,
        v1_21_6: 16u8,
        v1_21_7: 16u8,
        v1_21_9: 16u8,
        v1_21_11: 16u8,
    };
    pub const DATA_VIEW_RANGE: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_VILLAGER_DATA: TrackedId = TrackedId {
        v1_21: 20u8,
        v1_21_2: 20u8,
        v1_21_4: 20u8,
        v1_21_5: 20u8,
        v1_21_6: 20u8,
        v1_21_7: 18u8,
        v1_21_9: 20u8,
        v1_21_11: 20u8,
    };
    pub const DATA_WAITING: TrackedId = TrackedId {
        v1_21: 9u8,
        v1_21_2: 9u8,
        v1_21_4: 9u8,
        v1_21_5: 9u8,
        v1_21_6: 9u8,
        v1_21_7: 255u8,
        v1_21_9: 9u8,
        v1_21_11: 9u8,
    };
    pub const DATA_WARNING: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 255u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
    pub const DATA_WIDTH: TrackedId = TrackedId {
        v1_21: 20u8,
        v1_21_2: 20u8,
        v1_21_4: 20u8,
        v1_21_5: 20u8,
        v1_21_6: 20u8,
        v1_21_7: 20u8,
        v1_21_9: 20u8,
        v1_21_11: 20u8,
    };
    pub const DATA_ZOMBIE_TYPE: TrackedId = TrackedId {
        v1_21: 17u8,
        v1_21_2: 17u8,
        v1_21_4: 17u8,
        v1_21_5: 17u8,
        v1_21_6: 17u8,
        v1_21_7: 17u8,
        v1_21_9: 17u8,
        v1_21_11: 17u8,
    };
}
