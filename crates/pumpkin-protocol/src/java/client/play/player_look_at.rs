use pumpkin_data::packet::clientbound::play::PLAYER_LOOK_AT;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAYER_LOOK_AT)]
pub struct CPlayerLookAt {
    pub from_anchor: VarInt,
    pub target_x: f64,
    pub target_y: f64,
    pub target_z: f64,
    pub entity: Option<(VarInt, VarInt)>,
}

impl CPlayerLookAt {
    #[must_use]
    pub const fn at_position(
        from_anchor: VarInt,
        target_x: f64,
        target_y: f64,
        target_z: f64,
    ) -> Self {
        Self {
            from_anchor,
            target_x,
            target_y,
            target_z,
            entity: None,
        }
    }

    #[must_use]
    pub const fn at_entity(
        from_anchor: VarInt,
        target_x: f64,
        target_y: f64,
        target_z: f64,
        entity_id: VarInt,
        to_anchor: VarInt,
    ) -> Self {
        Self {
            from_anchor,
            target_x,
            target_y,
            target_z,
            entity: Some((entity_id, to_anchor)),
        }
    }
}

impl ClientPacket for CPlayerLookAt {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.from_anchor)?;
        write.write_f64_be(self.target_x)?;
        write.write_f64_be(self.target_y)?;
        write.write_f64_be(self.target_z)?;
        if let Some((entity_id, to_anchor)) = self.entity {
            write.write_bool(true)?;
            write.write_var_int(&entity_id)?;
            write.write_var_int(&to_anchor)?;
        } else {
            write.write_bool(false)?;
        }
        Ok(())
    }
}
