use std::io::{Error, Read};

use pumpkin_macros::packet;

use crate::{bedrock::server::login::read_connection_request, serial::PacketRead};

/// Sent by a split-screen sub-client over an existing Bedrock connection to authenticate and join.
#[packet(94)]
pub struct SSubClientLogin {
    /// Identity and authentication JSON/JWT payload.
    pub jwt: Vec<u8>,
    /// Raw client data JWT carrying skin parameters, device details, and locale settings.
    pub raw_token: Vec<u8>,
}

impl PacketRead for SSubClientLogin {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let (jwt, raw_token) = read_connection_request(reader)?;
        Ok(Self { jwt, raw_token })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{
        MAX_PACKET_DATA_SIZE,
        codec::var_uint::VarUInt,
        serial::{PacketRead, PacketWrite},
    };

    use super::*;

    #[test]
    fn reads_sub_client_login_connection_request() {
        let jwt = br#"{"AuthenticationType":0,"Token":"abc"}"#;
        let client_token = b"header.payload.sig";
        let connection_request_len = 4 + jwt.len() + 4 + client_token.len();
        let mut input = Vec::new();

        VarUInt(connection_request_len as u32)
            .write(&mut input)
            .expect("write connection request length");
        (jwt.len() as u32)
            .write(&mut input)
            .expect("write JWT length");
        input.extend_from_slice(jwt);
        (client_token.len() as u32)
            .write(&mut input)
            .expect("write raw token length");
        input.extend_from_slice(client_token);

        let packet = SSubClientLogin::read(&mut Cursor::new(input)).expect("read sub-client login");
        assert_eq!(packet.jwt, jwt);
        assert_eq!(packet.raw_token, client_token);
    }

    #[test]
    fn rejects_oversized_connection_request() {
        let mut input = Vec::new();
        VarUInt((MAX_PACKET_DATA_SIZE + 1) as u32)
            .write(&mut input)
            .expect("write oversized length");

        assert!(SSubClientLogin::read(&mut Cursor::new(input)).is_err());
    }
}
