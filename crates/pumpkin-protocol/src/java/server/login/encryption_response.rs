use pumpkin_data::packet::serverbound::LOGIN_KEY;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{ReadingError, ServerPacket, ser::NetworkReadExt};

#[derive(Clone, Debug, PartialEq, Eq)]
#[java_packet(LOGIN_KEY)]
pub struct SEncryptionResponse {
    pub shared_secret: Box<[u8]>,
    pub verify_token: Box<[u8]>,
}

impl<'a> ServerPacket<'a> for SEncryptionResponse {
    fn read(mut read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let shared_secret = read_encryption_buffer(&mut read)?;
        let verify_token = if version >= &JavaMinecraftVersion::V_1_19_3
            && version < &JavaMinecraftVersion::V_1_20_2
        {
            let has_verify_token = read.get_bool()?;
            if has_verify_token {
                read_encryption_buffer(&mut read)?
            } else {
                let _salt = read.get_i64_be()?;
                let _signature = read_encryption_buffer(&mut read)?;
                Box::new([])
            }
        } else {
            read_encryption_buffer(&mut read)?
        };
        Ok(Self {
            shared_secret,
            verify_token,
        })
    }
}

impl crate::ClientPacket for SEncryptionResponse {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_var_int(&crate::VarInt(self.shared_secret.len() as i32))?;
        write.write_all(&self.shared_secret)?;
        if version >= &JavaMinecraftVersion::V_1_19_3 && version < &JavaMinecraftVersion::V_1_20_2 {
            write.write_bool(true)?;
        }
        write.write_var_int(&crate::VarInt(self.verify_token.len() as i32))?;
        write.write_all(&self.verify_token)?;
        Ok(())
    }
}

fn read_encryption_buffer(read: &mut impl NetworkReadExt) -> Result<Box<[u8]>, ReadingError> {
    let length = read.get_var_int()?.0 as usize;
    if length > 256 {
        return Err(ReadingError::Message("Encryption payload too large".into()));
    }
    let mut data = vec![0u8; length];
    read.read_bytes_to_buf(&mut data)?;
    Ok(data.into_boxed_slice())
}
