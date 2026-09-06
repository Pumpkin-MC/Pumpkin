use pumpkin_data::packet::clientbound::play::UPDATE_MOB_EFFECT;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::codec::var_int::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(UPDATE_MOB_EFFECT)]
pub struct CUpdateMobEffect {
    pub entity_id: VarInt,
    pub effect_id: VarInt,
    pub amplifier: VarInt,
    pub duration: VarInt,
    pub flags: i8,
}

impl CUpdateMobEffect {
    #[must_use]
    pub const fn new(
        entity_id: VarInt,
        effect_id: VarInt,
        amplifier: VarInt,
        duration: VarInt,
        flags: i8,
    ) -> Self {
        Self {
            entity_id,
            effect_id,
            amplifier,
            duration,
            flags,
        }
    }
}

impl ClientPacket for CUpdateMobEffect {
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

        if *version >= JavaMinecraftVersion::V_1_20_5 {
            write.write_var_int(&self.amplifier)?;
        } else {
            write.write_i8(self.amplifier.0 as i8)?;
        }

        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i16_be(self.duration.0 as i16)?;
        } else {
            write.write_var_int(&self.duration)?;
        }

        if *version > JavaMinecraftVersion::V_1_7_6 {
            write.write_i8(self.flags)?;
        }

        if *version >= JavaMinecraftVersion::V_1_19 && *version < JavaMinecraftVersion::V_1_20_5 {
            write.write_bool(false)?;
        }

        Ok(())
    }
}
