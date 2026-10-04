//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/server/login/encryption_response.rs`.

use crate::java::legacy::LegacyRead;
use crate::java::server::login::SEncryptionResponse;
use crate::{ser::NetworkReadExt, ser::ReadingError};
use pumpkin_util::version::JavaMinecraftVersion;

impl<'a> LegacyRead<'a> for SEncryptionResponse {
    fn read_legacy(
        mut read: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        let shared_secret = read_encryption_buffer(&mut read, *version)?;
        let verify_token = if version >= &JavaMinecraftVersion::V_1_19
            && version < &JavaMinecraftVersion::V_1_19_3
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
