use pumpkin_data::packet::serverbound::play::KEEP_ALIVE;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket, VarInt,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(KEEP_ALIVE)]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}

impl<'a> ServerPacket<'a> for SKeepAlive {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let keep_alive_id = if version >= &JavaMinecraftVersion::V_1_12_2 {
            bytebuf.get_i64_be()?
        } else if version >= &JavaMinecraftVersion::V_1_8 {
            i64::from(bytebuf.get_var_int()?.0)
        } else {
            i64::from(bytebuf.get_i32_be()?)
        };
        Ok(Self { keep_alive_id })
    }
}

impl crate::ClientPacket for SKeepAlive {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        if version >= &JavaMinecraftVersion::V_1_12_2 {
            write.write_i64_be(self.keep_alive_id)?;
        } else if version >= &JavaMinecraftVersion::V_1_8 {
            write.write_var_int(&VarInt(self.keep_alive_id as i32))?;
        } else {
            write.write_i32_be(self.keep_alive_id as i32)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientPacket;

    #[test]
    fn keep_alive_roundtrip_modern() {
        let packet = SKeepAlive {
            keep_alive_id: 1234567890123456789,
        };
        let mut buf = Vec::new();
        let version = JavaMinecraftVersion::V_1_21_4;
        packet.write_packet_data(&mut buf, &version).unwrap();

        let mut slice = buf.as_slice();
        let read_packet = SKeepAlive::read(&mut slice, &version).unwrap();
        assert_eq!(read_packet.keep_alive_id, 1234567890123456789);
    }

    #[test]
    fn keep_alive_roundtrip_1_8() {
        let packet = SKeepAlive {
            keep_alive_id: 12345,
        };
        let mut buf = Vec::new();
        let version = JavaMinecraftVersion::V_1_8;
        packet.write_packet_data(&mut buf, &version).unwrap();

        let mut slice = buf.as_slice();
        let read_packet = SKeepAlive::read(&mut slice, &version).unwrap();
        assert_eq!(read_packet.keep_alive_id, 12345);
    }

    #[test]
    fn keep_alive_roundtrip_1_7() {
        let packet = SKeepAlive {
            keep_alive_id: 12345,
        };
        let mut buf = Vec::new();
        let version = JavaMinecraftVersion::V_1_7_2;
        packet.write_packet_data(&mut buf, &version).unwrap();

        let mut slice = buf.as_slice();
        let read_packet = SKeepAlive::read(&mut slice, &version).unwrap();
        assert_eq!(read_packet.keep_alive_id, 12345);
    }
}
