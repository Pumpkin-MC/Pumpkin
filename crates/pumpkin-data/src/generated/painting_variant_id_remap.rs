/* This file is generated. Do not edit manually. */
use pumpkin_util::version::JavaMinecraftVersion;
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_7_6: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_8: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_9: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_10: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_11: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_12: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_13: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_13_2: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_14: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_15: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_16: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_16_2: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_17: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
pub static PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_18: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 17,
    17, 17, 17, 25,
];
#[must_use]
pub fn remap_painting_variant_id_for_version(
    painting_variant_id: u32,
    version: JavaMinecraftVersion,
) -> u32 {
    match version {
        pumpkin_util::version::JavaMinecraftVersion::V_1_7_2
        | pumpkin_util::version::JavaMinecraftVersion::V_1_7_6 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_7_6
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_8 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_8
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_9
        | pumpkin_util::version::JavaMinecraftVersion::V_1_9_1
        | pumpkin_util::version::JavaMinecraftVersion::V_1_9_2
        | pumpkin_util::version::JavaMinecraftVersion::V_1_9_3 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_9
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_10 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_10
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_11
        | pumpkin_util::version::JavaMinecraftVersion::V_1_11_1 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_11
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_12
        | pumpkin_util::version::JavaMinecraftVersion::V_1_12_1
        | pumpkin_util::version::JavaMinecraftVersion::V_1_12_2 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_12
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_13
        | pumpkin_util::version::JavaMinecraftVersion::V_1_13_1 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_13
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_13_2 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_13_2
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_14
        | pumpkin_util::version::JavaMinecraftVersion::V_1_14_1
        | pumpkin_util::version::JavaMinecraftVersion::V_1_14_2
        | pumpkin_util::version::JavaMinecraftVersion::V_1_14_3
        | pumpkin_util::version::JavaMinecraftVersion::V_1_14_4 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_14
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_15
        | pumpkin_util::version::JavaMinecraftVersion::V_1_15_1
        | pumpkin_util::version::JavaMinecraftVersion::V_1_15_2 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_15
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_16
        | pumpkin_util::version::JavaMinecraftVersion::V_1_16_1 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_16
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_16_2
        | pumpkin_util::version::JavaMinecraftVersion::V_1_16_3
        | pumpkin_util::version::JavaMinecraftVersion::V_1_16_4 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_16_2
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_17
        | pumpkin_util::version::JavaMinecraftVersion::V_1_17_1 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_17
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_18
        | pumpkin_util::version::JavaMinecraftVersion::V_1_18_2 => {
            PAINTING_VARIANT_ID_REMAP_V_26_2_TO_V_1_18
                .get(painting_variant_id as usize)
                .copied()
                .unwrap_or(painting_variant_id)
        }
        _ => painting_variant_id,
    }
}
#[must_use]
pub fn remap_motive_id_for_version(motive_id: u32, version: JavaMinecraftVersion) -> u32 {
    remap_painting_variant_id_for_version(motive_id, version)
}
