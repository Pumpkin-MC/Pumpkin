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
    fn read(
        mut read: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        let shared_secret = read_encryption_buffer(&mut read)?;
        let verify_token = read_encryption_buffer(&mut read)?;
        Ok(Self {
            shared_secret,
            verify_token,
        })
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
