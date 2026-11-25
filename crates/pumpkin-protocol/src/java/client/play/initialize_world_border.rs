use pumpkin_data::packet::clientbound::play::INITIALIZE_BORDER;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use crate::{VarInt, codec::var_long::VarLong};
use pumpkin_util::version::JavaMinecraftVersion;

/// Fully initializes the world border for the client.
///
/// This packet is sent when a player joins the world or changes dimensions.
/// It synchronizes the current position, size, and all warning parameters
/// to ensure the client-side visual barrier matches the server's authority.
#[java_packet(INITIALIZE_BORDER)]
pub struct CInitializeWorldBorder {
    /// The X coordinate of the center of the world border.
    pub x: f64,
    /// The Z coordinate of the center of the world border.
    pub z: f64,
    /// The diameter the border is moving from.
    pub old_diameter: f64,
    /// The diameter the border is moving toward.
    pub new_diameter: f64,
    /// The time (in milliseconds) it will take to reach `new_diameter`.
    pub speed: VarLong,
    /// The maximum distance a player can be teleported by a portal
    /// before the border prevents the teleport.
    pub portal_teleport_boundary: VarInt,
    /// Distance in blocks from the border where the screen starts to tint red.
    pub warning_blocks: VarInt,
    /// Time in seconds that a player must be on a collision course with
    /// the border before the warning tint appears.
    pub warning_time: VarInt,
}

impl CInitializeWorldBorder {
    #[expect(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        x: f64,
        z: f64,
        old_diameter: f64,
        new_diameter: f64,
        speed: VarLong,
        portal_teleport_boundary: VarInt,
        warning_blocks: VarInt,
        warning_time: VarInt,
    ) -> Self {
        Self {
            x,
            z,
            old_diameter,
            new_diameter,
            speed,
            portal_teleport_boundary,
            warning_blocks,
            warning_time,
        }
    }
}

impl ClientPacket for CInitializeWorldBorder {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f64_be(self.x)?;
        write.write_f64_be(self.z)?;
        write.write_f64_be(self.old_diameter)?;
        write.write_f64_be(self.new_diameter)?;
        let speed = if *version >= JavaMinecraftVersion::V_1_21_11 {
            VarLong(self.speed.0 / 50)
        } else {
            self.speed
        };
        write.write_var_long(&speed)?;
        write.write_var_int(&self.portal_teleport_boundary)?;
        write.write_var_int(&self.warning_blocks)?;
        write.write_var_int(&self.warning_time)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::java::client::play::CSetBorderLerpSize;
    use crate::ser::NetworkReadExt;

    #[test]
    fn border_duration_uses_ticks_since_java_1_21_11() -> Result<(), Box<dyn std::error::Error>> {
        let lerp = CSetBorderLerpSize::new(32.0, 16.0, VarLong(7500));
        let init = CInitializeWorldBorder::new(
            0.0,
            0.0,
            32.0,
            16.0,
            VarLong(7500),
            VarInt(8),
            VarInt(5),
            VarInt(15),
        );
        for (version, duration, wire) in [
            (JavaMinecraftVersion::V_1_21_9, 7500, [0xcc, 0x3a]),
            (JavaMinecraftVersion::V_1_21_11, 150, [0x96, 0x01]),
            (JavaMinecraftVersion::V_26_3, 150, [0x96, 0x01]),
        ] {
            let mut lerp_bytes = Vec::new();
            lerp.write_packet_data(&mut lerp_bytes, &version)?;
            let mut lerp_fields = lerp_bytes.as_slice();
            assert_eq!(lerp_fields.get_f64_be()?, 32.0);
            assert_eq!(lerp_fields.get_f64_be()?, 16.0);
            assert_eq!(lerp_fields, wire);
            assert_eq!(lerp_fields.get_var_long()?.0, duration);
            assert!(lerp_fields.is_empty());

            let mut init_bytes = Vec::new();
            init.write_packet_data(&mut init_bytes, &version)?;
            let mut init_fields = init_bytes.as_slice();
            for expected in [0.0, 0.0, 32.0, 16.0] {
                assert_eq!(init_fields.get_f64_be()?, expected);
            }
            assert!(init_fields.starts_with(&wire));
            assert_eq!(init_fields.get_var_long()?.0, duration);
            assert_eq!(init_fields, [8, 5, 15]);
        }
        Ok(())
    }
}
