use pumpkin_macros::packet;
use std::io::{Error, ErrorKind, Read};

use crate::{codec::var_uint::VarUInt, ser::NetworkReadExt, serial::PacketRead};

#[packet(1)]
pub struct SLogin {
    // https://mojang.github.io/bedrock-protocol-docs/html/LoginPacket.html
    pub protocol_version: i32,

    // https://mojang.github.io/bedrock-protocol-docs/html/connectionRequest.html
    pub jwt: String,
    pub raw_token: String,
}

impl PacketRead for SLogin {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let protocol_version = reader
            .get_i32_be()
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;
        let _len = VarUInt::read(reader)?;

        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;

        let mut buf2 = vec![0; i32::from_le_bytes(buf) as _];
        reader.read_exact(&mut buf2)?;
        let jwt = unsafe { String::from_utf8_unchecked(buf2) };

        let mut buf2 = vec![0; i32::from_le_bytes(buf) as _];
        reader.read_exact(&mut buf2)?;
        let raw_token = unsafe { String::from_utf8_unchecked(buf2) };

        Ok(Self {
            protocol_version,
            jwt,
            raw_token,
        })
    }
}
