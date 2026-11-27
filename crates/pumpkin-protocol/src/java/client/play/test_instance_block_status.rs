use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::TEST_INSTANCE_BLOCK_STATUS;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};

#[java_packet(TEST_INSTANCE_BLOCK_STATUS)]
pub struct CTestInstanceBlockStatus {
    pub status: TextComponent,
    pub size: Option<[i32; 3]>,
}

impl CTestInstanceBlockStatus {
    #[must_use]
    pub const fn new(status: TextComponent, size: Option<[i32; 3]>) -> Self {
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
        write.write_bool(self.size.is_some())?;
        if let Some([x, y, z]) = self.size {
            write.write_var_int(&VarInt(x))?;
            write.write_var_int(&VarInt(y))?;
            write.write_var_int(&VarInt(z))?;
        }
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CTestInstanceBlockStatus {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let status = bytebuf.get_component(version)?;
        let size = if bytebuf.get_bool()? {
            Some([
                bytebuf.get_var_int()?.0,
                bytebuf.get_var_int()?.0,
                bytebuf.get_var_int()?.0,
            ])
        } else {
            None
        };
        Ok(Self { status, size })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_packet_uses_component_then_optional_varint_size()
    -> Result<(), Box<dyn std::error::Error>> {
        let packet = CTestInstanceBlockStatus::new(TextComponent::text("ready"), Some([1, 2, 3]));
        let version = JavaMinecraftVersion::V_26_3;
        let mut bytes = Vec::new();
        packet.write_packet_data(&mut bytes, &version)?;
        assert!(bytes.ends_with(&[1, 1, 2, 3]));

        let decoded = CTestInstanceBlockStatus::read(&mut bytes.as_slice(), &version)?;
        assert_eq!(decoded.status, packet.status);
        assert_eq!(decoded.size, packet.size);
        Ok(())
    }
}
