/* This file is generated. Do not edit manually. */
use pumpkin_util::version::JavaMinecraftVersion;
pub static ENVIRONMENT_ATTRIBUTE_ID_REMAP_V_26_2_TO_V_26_3: &[u32] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 49, 50,
];
#[must_use]
pub fn remap_environment_attribute_id_for_version(
    environment_attribute_id: u32,
    version: JavaMinecraftVersion,
) -> u32 {
    match version {
        pumpkin_util::version::JavaMinecraftVersion::V_26_3 => {
            ENVIRONMENT_ATTRIBUTE_ID_REMAP_V_26_2_TO_V_26_3
                .get(environment_attribute_id as usize)
                .copied()
                .unwrap_or(environment_attribute_id)
        }
        _ => environment_attribute_id,
    }
}
