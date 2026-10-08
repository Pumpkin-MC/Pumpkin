use pumpkin_data::packet::serverbound::play::KEEP_ALIVE;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(KEEP_ALIVE)]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}

impl<'a> ServerPacket<'a> for SKeepAlive {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let keep_alive_id = bytebuf.get_i64_be()?;
        Ok(Self { keep_alive_id })
    }
}

impl crate::ClientPacket for SKeepAlive {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i64_be(self.keep_alive_id)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ClientPacket;

    #[test]
    fn keep_alive_roundtrip() {
        let packet = SKeepAlive {
            keep_alive_id: 1234567890123456789,
        };
        let mut buf = Vec::new();
        packet.write_packet_data(&mut buf).unwrap();

        let mut slice = buf.as_slice();
        let read_packet = SKeepAlive::read(&mut slice).unwrap();
        assert_eq!(read_packet.keep_alive_id, 1234567890123456789);
    }
}
