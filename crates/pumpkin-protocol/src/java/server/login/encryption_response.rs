use pumpkin_util::version::JavaMinecraftVersion;

use crate::{MultiVersionJavaPacket, ReadingError, ServerPacket, ser::NetworkReadExt};

#[derive(Clone, Debug, PartialEq, Eq)]
//#[java_packet(KEY)]
pub struct SEncryptionResponse {
    pub shared_secret: Box<[u8]>,
    pub verify_token: Box<[u8]>,
}

impl MultiVersionJavaPacket for SEncryptionResponse {
    fn to_id(_version: JavaMinecraftVersion) -> i32 {
        1
    }
}

impl<'a> ServerPacket<'a> for SEncryptionResponse {
    fn read(mut read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let shared_secret = read_encryption_buffer(&mut read, *version)?;
        let verify_token = if version >= &JavaMinecraftVersion::V_1_19_3
            && version < &JavaMinecraftVersion::V_1_20_2
        {
            let has_verify_token = read.get_bool()?;
            if has_verify_token {
                read_encryption_buffer(&mut read, *version)?
            } else {
                let _salt = read.get_i64_be()?;
                let _signature = read_encryption_buffer(&mut read, *version)?;
                Box::new([])
            }
        } else {
            read_encryption_buffer(&mut read, *version)?
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
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i16_be(self.shared_secret.len() as i16)?;
            write.write_all(&self.shared_secret)?;
            write.write_i16_be(self.verify_token.len() as i16)?;
            write.write_all(&self.verify_token)?;
            return Ok(());
        }
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

fn read_encryption_buffer(
    read: &mut impl NetworkReadExt,
    version: JavaMinecraftVersion,
) -> Result<Box<[u8]>, ReadingError> {
    let length = if version <= JavaMinecraftVersion::V_1_7_6 {
        let len = read.get_i16_be()?;
        if len < 0 {
            return Err(ReadingError::Message(
                "Key was smaller than nothing! Weird key!".into(),
            ));
        }
        len as usize
    } else {
        read.get_var_int()?.0 as usize
    };
    if length > 256 {
        return Err(ReadingError::Message("Encryption payload too large".into()));
    }
    let mut data = vec![0u8; length];
    read.read_bytes_to_buf(&mut data)?;
    Ok(data.into_boxed_slice())
}
