use pumpkin_macros::packet;
use std::io::{Error, Read};

use crate::{codec::var_uint::VarUInt, serial::PacketRead};

#[packet(1)]
pub struct SLogin {
    // https://mojang.github.io/bedrock-protocol-docs/html/LoginPacket.html
    pub protocol_version: i32,

    // https://mojang.github.io/bedrock-protocol-docs/html/connectionRequest.html
    pub jwt: Vec<u8>,
    pub raw_token: Vec<u8>,
}

impl PacketRead for SLogin {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let protocol_version = i32::read_be(reader)?;
        let _len = VarUInt::read(reader)?;

        let jwt_len = u32::read(reader)?;
        let mut jwt = vec![0; jwt_len as _];
        reader.read_exact(&mut jwt)?;

        let raw_token_len = u32::read(reader)?;
        let mut raw_token = vec![0; raw_token_len as _];
        reader.read_exact(&mut raw_token)?;

        Ok(Self {
            protocol_version,
            jwt,
            raw_token,
        })
    }
}
