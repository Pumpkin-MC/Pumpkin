/* This file is generated. Do not edit manually. */
use pumpkin_util::version::JavaMinecraftVersion;
#[must_use]
pub fn remap_environment_attribute_id_for_version(
    environment_attribute_id: u32,
    version: JavaMinecraftVersion,
) -> u32 {
    match version {
        _ => environment_attribute_id,
    }
}
