use crate::codec::var_int::VarInt;
use pumpkin_data::packet::clientbound::PLAY_REMOVE_MOB_EFFECT;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_REMOVE_MOB_EFFECT)]
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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.entity_id)?;
        write.write_var_int(&self.effect_id)?;
        Ok(())
    }
}
