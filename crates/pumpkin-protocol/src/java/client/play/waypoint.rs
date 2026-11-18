use std::io::Write;

use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, VarInt, WritingError};
use pumpkin_data::packet::clientbound::play::WAYPOINT;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::text::color::RGBColor;
use pumpkin_util::version::JavaMinecraftVersion;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum WaypointOperation {
    Track = 0,
    Untrack = 1,
    Update = 2,
}

/// Where the waypoint points. Variant order is the wire id (vanilla `TrackedWaypoint.Type`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WaypointTarget {
    Empty,
    Position(BlockPos),
    Chunk(Vector2<i32>),
    Azimuth(f32),
}

impl WaypointTarget {
    const fn type_id(&self) -> i32 {
        match self {
            Self::Empty => 0,
            Self::Position(_) => 1,
            Self::Chunk(_) => 2,
            Self::Azimuth(_) => 3,
        }
    }
}

/// Vanilla `Waypoint.Icon`: a style asset key plus an optional RGB color.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WaypointIcon<'a> {
    pub style: &'a str,
    pub color: Option<RGBColor>,
}

impl WaypointIcon<'_> {
    /// Vanilla `WaypointStyleAssets.DEFAULT`.
    pub const DEFAULT_STYLE: &'static str = "minecraft:default";

    /// Vanilla `Waypoint.Icon.NULL`, the client picks a color from the waypoint's UUID.
    pub const NULL: WaypointIcon<'static> = WaypointIcon {
        style: Self::DEFAULT_STYLE,
        color: None,
    };
}

#[derive(Clone, Copy, Debug)]
pub struct TrackedWaypoint<'a> {
    pub identifier: Uuid,
    pub icon: WaypointIcon<'a>,
    pub target: WaypointTarget,
}

/// Syncs tracked waypoints (`ClientboundTrackedWaypointPacket`) to client.
#[java_packet(WAYPOINT)]
pub struct CWaypoint<'a> {
    pub operation: WaypointOperation,
    pub waypoint: TrackedWaypoint<'a>,
}

impl<'a> CWaypoint<'a> {
    #[must_use]
    pub const fn new(
        operation: WaypointOperation,
        identifier: Uuid,
        icon: WaypointIcon<'a>,
        target: WaypointTarget,
    ) -> Self {
        Self {
            operation,
            waypoint: TrackedWaypoint {
                identifier,
                icon,
                target,
            },
        }
    }

    /// Vanilla `removeWaypoint`, the client only looks at the identifier here.
    #[must_use]
    pub const fn remove(identifier: Uuid) -> Self {
        Self::new(
            WaypointOperation::Untrack,
            identifier,
            WaypointIcon::NULL,
            WaypointTarget::Empty,
        )
    }

    #[must_use]
    pub const fn add_position(
        identifier: Uuid,
        icon: WaypointIcon<'a>,
        position: BlockPos,
    ) -> Self {
        Self::new(
            WaypointOperation::Track,
            identifier,
            icon,
            WaypointTarget::Position(position),
        )
    }

    #[must_use]
    pub const fn update_position(
        identifier: Uuid,
        icon: WaypointIcon<'a>,
        position: BlockPos,
    ) -> Self {
        Self::new(
            WaypointOperation::Update,
            identifier,
            icon,
            WaypointTarget::Position(position),
        )
    }
}

impl ClientPacket for CWaypoint<'_> {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&VarInt(self.operation as i32))?;

        // hmm, the id is an Either<UUID, String>, true picks the UUID side.
        // Servers only ever send UUIDs, the string side is for client made waypoints.
        write.write_bool(true)?;
        write.write_uuid(&self.waypoint.identifier)?;

        let icon = &self.waypoint.icon;
        write.write_string(icon.style)?;
        // yeah, RGB_COLOR is three raw bytes, not an int
        write.write_option(&icon.color, |w, color| {
            w.write_u8(color.red)?;
            w.write_u8(color.green)?;
            w.write_u8(color.blue)
        })?;

        write.write_var_int(&VarInt(self.waypoint.target.type_id()))?;
        match self.waypoint.target {
            WaypointTarget::Empty => {}
            WaypointTarget::Position(pos) => {
                write.write_var_int(&VarInt(pos.0.x))?;
                write.write_var_int(&VarInt(pos.0.y))?;
                write.write_var_int(&VarInt(pos.0.z))?;
            }
            WaypointTarget::Chunk(chunk) => {
                write.write_var_int(&VarInt(chunk.x))?;
                write.write_var_int(&VarInt(chunk.y))?;
            }
            WaypointTarget::Azimuth(angle) => write.write_f32_be(angle)?,
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;
    use pumpkin_util::text::color::RGBColor;
    use pumpkin_util::version::JavaMinecraftVersion;
    use uuid::Uuid;

    use super::{CWaypoint, WaypointIcon, WaypointOperation, WaypointTarget};
    use crate::ClientPacket;

    const ID: Uuid = Uuid::from_u128(0x0011_2233_4455_6677_8899_aabb_ccdd_eeff);

    fn encode(packet: &CWaypoint<'_>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut out = Vec::new();
        packet.write_packet_data(&mut out, &JavaMinecraftVersion::V_26_3)?;
        Ok(out)
    }

    // Layout of ClientboundTrackedWaypointPacket.STREAM_CODEC in 26.3: operation, either uuid,
    // icon style + optional rgb, type ordinal, contents.
    fn header(operation: u8, color: Option<[u8; 3]>, type_id: u8) -> Vec<u8> {
        let mut out = vec![operation, 1];
        out.extend_from_slice(ID.as_bytes());
        out.push(WaypointIcon::DEFAULT_STYLE.len() as u8);
        out.extend_from_slice(WaypointIcon::DEFAULT_STYLE.as_bytes());
        match color {
            Some(rgb) => {
                out.push(1);
                out.extend_from_slice(&rgb);
            }
            None => out.push(0),
        }
        out.push(type_id);
        out
    }

    #[test]
    fn position_target_is_three_varints() -> Result<(), Box<dyn std::error::Error>> {
        let icon = WaypointIcon {
            style: WaypointIcon::DEFAULT_STYLE,
            color: Some(RGBColor::new(0x12, 0x34, 0x56)),
        };
        let encoded = encode(&CWaypoint::add_position(
            ID,
            icon,
            BlockPos::new(1, -1, 300),
        ))?;
        let mut expected = header(0, Some([0x12, 0x34, 0x56]), 1);
        // 1, -1 and 300 as VarInts
        expected.extend_from_slice(&[0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F, 0xAC, 0x02]);
        assert_eq!(encoded, expected);
        Ok(())
    }

    #[test]
    fn chunk_and_azimuth_targets() -> Result<(), Box<dyn std::error::Error>> {
        let chunk = CWaypoint::new(
            WaypointOperation::Update,
            ID,
            WaypointIcon::NULL,
            WaypointTarget::Chunk(Vector2::new(2, -2)),
        );
        let mut expected = header(2, None, 2);
        expected.extend_from_slice(&[0x02, 0xFE, 0xFF, 0xFF, 0xFF, 0x0F]);
        assert_eq!(encode(&chunk)?, expected);

        let azimuth = CWaypoint::new(
            WaypointOperation::Track,
            ID,
            WaypointIcon::NULL,
            WaypointTarget::Azimuth(1.5),
        );
        let mut expected = header(0, None, 3);
        expected.extend_from_slice(&1.5f32.to_be_bytes());
        assert_eq!(encode(&azimuth)?, expected);
        Ok(())
    }

    #[test]
    fn remove_sends_empty_target() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(encode(&CWaypoint::remove(ID))?, header(1, None, 0));
        Ok(())
    }
}
