use pumpkin_data::packet::clientbound::LOGIN_HELLO;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(LOGIN_HELLO)]
pub struct CEncryptionRequest<'a> {
    pub server_id: &'a str, // 20
    pub public_key_length: VarInt,
    pub public_key: &'a [u8],
    pub verify_token_length: VarInt,
    pub verify_token: &'a [u8],
    pub should_authenticate: bool,
}

impl<'a> CEncryptionRequest<'a> {
    pub fn new(
        server_id: &'a str,
        public_key: &'a [u8],
        verify_token: &'a [u8],
        should_authenticate: bool,
    ) -> Self {
        Self {
            server_id,
            public_key_length: public_key.len().into(),
            public_key,
            verify_token_length: verify_token.len().into(),
            verify_token,
            should_authenticate,
        }
    }
}
