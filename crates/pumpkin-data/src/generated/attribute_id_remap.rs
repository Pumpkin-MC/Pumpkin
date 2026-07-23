/* This file is generated. Do not edit manually. */
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_20_5: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 65535, 65535, 65535, 7, 8, 9, 10, 65535, 11, 12, 13,
    14, 15, 16, 65535, 65535, 17, 65535, 65535, 18, 19, 65535, 20, 21, 65535, 65535, 65535, 65535,
    65535, 65535,
];
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 7, 65535, 8, 9, 10, 11, 12, 65535, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 65535, 22, 23, 24, 25, 26, 27, 28, 29, 65535, 30, 65535, 65535,
];
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_2: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 7, 65535, 8, 9, 10, 11, 12, 65535, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 65535, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 65535, 65535,
];
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_4: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 7, 65535, 8, 9, 10, 11, 12, 65535, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 65535, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 65535, 65535,
];
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_5: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 7, 65535, 8, 9, 10, 11, 12, 65535, 13, 14, 15, 16,
    17, 18, 19, 20, 21, 65535, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 65535, 65535,
];
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_6: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 7, 8, 9, 10, 11, 12, 13, 65535, 14, 15, 16, 17, 18,
    19, 20, 21, 22, 65535, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
];
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_7: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 7, 8, 9, 10, 11, 12, 13, 65535, 14, 15, 16, 17, 18,
    19, 20, 21, 22, 65535, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
];
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_9: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 7, 8, 9, 10, 11, 12, 13, 65535, 14, 15, 16, 17, 18,
    19, 20, 21, 22, 65535, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
];
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_11: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 7, 8, 9, 10, 11, 12, 13, 65535, 14, 15, 16, 17, 18,
    19, 20, 21, 22, 65535, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
];
pub static ATTRIBUTE_ID_REMAP_V_26_2_TO_V_26_1: &[u16] = &[
    65535, 0, 1, 2, 3, 4, 65535, 5, 6, 65535, 7, 8, 9, 10, 11, 12, 13, 65535, 14, 15, 16, 17, 18,
    19, 20, 21, 22, 65535, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
];
#[must_use]
pub fn remap_attribute_id_for_version(
    attribute_id: u16,
    version: pumpkin_util::version::JavaMinecraftVersion,
) -> Option<u16> {
    match version {
        pumpkin_util::version::JavaMinecraftVersion::V_1_20_5 => {
            ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_20_5
                .get(usize::from(attribute_id))
                .copied()
                .filter(|id| *id != u16::MAX)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_21 => ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21
            .get(usize::from(attribute_id))
            .copied()
            .filter(|id| *id != u16::MAX),
        pumpkin_util::version::JavaMinecraftVersion::V_1_21_2 => {
            ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_2
                .get(usize::from(attribute_id))
                .copied()
                .filter(|id| *id != u16::MAX)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_21_4 => {
            ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_4
                .get(usize::from(attribute_id))
                .copied()
                .filter(|id| *id != u16::MAX)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_21_5 => {
            ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_5
                .get(usize::from(attribute_id))
                .copied()
                .filter(|id| *id != u16::MAX)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_21_6 => {
            ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_6
                .get(usize::from(attribute_id))
                .copied()
                .filter(|id| *id != u16::MAX)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_21_7 => {
            ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_7
                .get(usize::from(attribute_id))
                .copied()
                .filter(|id| *id != u16::MAX)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_21_9 => {
            ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_9
                .get(usize::from(attribute_id))
                .copied()
                .filter(|id| *id != u16::MAX)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_1_21_11 => {
            ATTRIBUTE_ID_REMAP_V_26_2_TO_V_1_21_11
                .get(usize::from(attribute_id))
                .copied()
                .filter(|id| *id != u16::MAX)
        }
        pumpkin_util::version::JavaMinecraftVersion::V_26_1 => ATTRIBUTE_ID_REMAP_V_26_2_TO_V_26_1
            .get(usize::from(attribute_id))
            .copied()
            .filter(|id| *id != u16::MAX),
        _ => Some(attribute_id),
    }
}
