use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::SET_BEACON;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::codec::var_int::VarInt;

#[java_packet(SET_BEACON)]
pub struct SSetBeacon {
    pub primary_effect: Option<VarInt>,
    pub secondary_effect: Option<VarInt>,
}

impl<'a> ServerPacket<'a> for SSetBeacon {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        if *version >= JavaMinecraftVersion::V_1_19 {
            Ok(Self {
                primary_effect: bytebuf.get_option(NetworkReadExt::get_var_int)?,
                secondary_effect: bytebuf.get_option(NetworkReadExt::get_var_int)?,
            })
        } else {
            let p = bytebuf.get_var_int()?;
            let s = bytebuf.get_var_int()?;
            let primary_effect = if p.0 == -1 { None } else { Some(p) };
            let secondary_effect = if s.0 == -1 { None } else { Some(s) };
            Ok(Self {
                primary_effect,
                secondary_effect,
            })
        }
    }
}

impl crate::ClientPacket for SSetBeacon {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_option(
            &self.primary_effect,
            crate::ser::NetworkWriteExt::write_var_int,
        )?;
        write.write_option(
            &self.secondary_effect,
            crate::ser::NetworkWriteExt::write_var_int,
        )?;
        Ok(())
    }
}
