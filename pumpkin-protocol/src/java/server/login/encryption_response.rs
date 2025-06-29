use std::io::Read;

use pumpkin_data::packet::serverbound::LOGIN_KEY;
use pumpkin_macros::packet;

use crate::{ReadingError, ServerPacket, ser::NetworkReadExt};

#[packet(LOGIN_KEY)]
pub struct SEncryptionResponse {
    pub shared_secret: Box<[u8]>,
    pub verify_token: Box<[u8]>,
}

impl ServerPacket for SEncryptionResponse {
    fn read(read: impl Read) -> Result<Self, ReadingError> {
        let mut read = read;

        let shared_secret_length = read.get_var_int()?.0 as usize;
        let shared_secret = read.read_boxed_slice(shared_secret_length)?;
        let verify_token_length = read.get_var_int()?.0 as usize;
        let verify_token = read.read_boxed_slice(verify_token_length)?;
        Ok(Self {
            shared_secret,
            verify_token,
        })
    }
}
