use pumpkin_macros::packet;
use std::io::{Error, ErrorKind, Read};

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
        const MAX_TOKEN_SIZE: usize = 256 * 1024; // 256KB limit

        let protocol_version = i32::read_be(reader)?;

        let _len = VarUInt::read(reader)?;

        let jwt_len = u32::read_be(reader)? as usize;
        if jwt_len > MAX_TOKEN_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "JWT length exceeds limit",
            ));
        }
        let mut jwt = vec![0u8; jwt_len];
        reader.read_exact(&mut jwt)?;

        let raw_token_len = u32::read_be(reader)? as usize;
        if raw_token_len > MAX_TOKEN_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Raw token length exceeds limit",
            ));
        }
        let mut raw_token = vec![0u8; raw_token_len];
        reader.read_exact(&mut raw_token)?;

        Ok(Self {
            protocol_version,
            jwt,
            raw_token,
        })
    }
}
