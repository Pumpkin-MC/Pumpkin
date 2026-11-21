use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::TEST_INSTANCE_BLOCK_STATUS;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};

#[java_packet(TEST_INSTANCE_BLOCK_STATUS)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CTestInstanceBlockStatus {
    pub status: TextComponent,
    pub size: Option<[VarInt; 3]>,
}

impl CTestInstanceBlockStatus {
    #[must_use]
    pub const fn new(status: TextComponent, size: Option<[VarInt; 3]>) -> Self {
        Self { status, size }
    }
}

impl ClientPacket for CTestInstanceBlockStatus {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_component(&self.status, version)?;
        write.write_option(&self.size, |w, s| {
            w.write_var_int(&s[0])?;
            w.write_var_int(&s[1])?;
            w.write_var_int(&s[2])
        })?;
        Ok(())
    }
}

impl ServerPacket<'_> for CTestInstanceBlockStatus {
    fn read(bytebuf: &mut &[u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            status: bytebuf.get_component_borrowed(version)?,
            size: bytebuf
                .get_option(|b| Ok([b.get_var_int()?, b.get_var_int()?, b.get_var_int()?]))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_test_instance_block_status() {
        let packet = CTestInstanceBlockStatus::new(
            TextComponent::text("minecraft:always_pass"),
            Some([VarInt(5), VarInt(6), VarInt(7)]),
        );
        let mut buf = Vec::new();
        packet
            .write_packet_data(&mut buf, &JavaMinecraftVersion::V_26_3)
            .unwrap();

        let mut slice = buf.as_slice();
        let decoded =
            CTestInstanceBlockStatus::read(&mut slice, &JavaMinecraftVersion::V_26_3).unwrap();
        assert!(slice.is_empty());
        assert_eq!(decoded, packet);
    }
}
