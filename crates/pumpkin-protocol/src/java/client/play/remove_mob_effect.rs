use crate::codec::var_int::VarInt;
use pumpkin_data::packet::clientbound::play::REMOVE_MOB_EFFECT;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(REMOVE_MOB_EFFECT)]
pub struct CRemoveMobEffect {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
}

impl CRemoveMobEffect {
    #[must_use]
    pub const fn new(entity_id: VarInt, effect_id: VarInt) -> Self {
        Self {
            entity_id,
            effect_id,
        }
    }
}

impl ClientPacket for CRemoveMobEffect {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.entity_id.0)?;
        } else {
            write.write_var_int(&self.entity_id)?;
        }

        if *version >= JavaMinecraftVersion::V_1_18_2 {
            write.write_var_int(&self.effect_id)?;
        } else {
            write.write_u8(self.effect_id.0 as u8)?;
        }
        Ok(())
    }
}
