use pumpkin_data::packet::serverbound::PLAY_CHAT_SESSION_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[derive(Debug)]
#[java_packet(PLAY_CHAT_SESSION_UPDATE)]
pub struct SPlayerSession {
    pub session_id: uuid::Uuid,
    pub expires_at: i64,
    pub public_key: Box<[u8]>,
    pub key_signature: Box<[u8]>,
}

impl<'a> ServerPacket<'a> for SPlayerSession {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let session_id = read.get_uuid()?;
        let expires_at = read.get_i64_be()?;

        let public_key_length = read.get_var_int()?.0 as usize;
        let mut public_key = vec![0u8; public_key_length];
        read.read_bytes_to_buf(&mut public_key)?;

        let key_signature_length = read.get_var_int()?.0 as usize;
        let mut key_signature = vec![0u8; key_signature_length];
        read.read_bytes_to_buf(&mut key_signature)?;

        Ok(Self {
            session_id,
            expires_at,
            public_key: public_key.into_boxed_slice(),
            key_signature: key_signature.into_boxed_slice(),
        })
    }
}
