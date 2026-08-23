use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, MultiVersionJavaPacket};
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the server to initiate the encryption handshake.
///
/// This packet provides the client with the server's public key and a
/// verification token, allowing the client to generate a shared secret
/// for secure communication.
//#[java_packet(HELLO)]
pub struct CEncryptionRequest<'a> {
    /// The server's ID string. In modern Minecraft, this is usually
    /// an empty string unless the server is using legacy authentication.
    pub server_id: &'a str,
    /// The server's DER-encoded RSA public key.
    pub public_key: &'a [u8],
    /// A random bitstring used to verify that the client can correctly
    /// encrypt data with the server's public key.
    pub verify_token: &'a [u8],
    /// Indicates whether the server is in "online mode" and requires
    /// Mojang authentication.
    pub should_authenticate: bool,
}

impl MultiVersionJavaPacket for CEncryptionRequest<'_> {
    fn to_id(_version: JavaMinecraftVersion) -> i32 {
        1
    }
}

impl<'a> CEncryptionRequest<'a> {
    #[must_use]
    pub const fn new(
        server_id: &'a str,
        public_key: &'a [u8],
        verify_token: &'a [u8],
        should_authenticate: bool,
    ) -> Self {
        Self {
            server_id,
            public_key,
            verify_token,
            should_authenticate,
        }
    }
}

impl ClientPacket for CEncryptionRequest<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(self.server_id)?;
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i16_be(self.public_key.len() as i16)?;
        } else {
            write.write_var_int(&crate::VarInt(self.public_key.len() as i32))?;
        }
        write.write_all(self.public_key)?;
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i16_be(self.verify_token.len() as i16)?;
        } else {
            write.write_var_int(&crate::VarInt(self.verify_token.len() as i32))?;
        }
        write.write_all(self.verify_token)?;
        if version >= &JavaMinecraftVersion::V_1_20_5 {
            write.write_bool(self.should_authenticate)?;
        }
        Ok(())
    }
}

impl<'a> crate::ServerPacket<'a> for CEncryptionRequest<'a> {
    fn read(
        read: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, crate::ReadingError> {
        use crate::ser::{NetworkReadExt, NetworkReadSliceExt};
        let server_id = read.get_str_bounded_borrowed(20)?;
        let public_key_len = if *version <= JavaMinecraftVersion::V_1_7_6 {
            let pkl = read.get_i16_be()?;
            if pkl < 0 {
                return Err(crate::ReadingError::Message(
                    "Key was smaller than nothing! Weird key!".into(),
                ));
            }
            pkl as usize
        } else {
            read.get_var_int()?.0 as usize
        };
        let public_key = read.read_slice_borrowed(public_key_len)?;

        let verify_token_len = if *version <= JavaMinecraftVersion::V_1_7_6 {
            let vtl = read.get_i16_be()?;
            if vtl < 0 {
                return Err(crate::ReadingError::Message(
                    "Key was smaller than nothing! Weird key!".into(),
                ));
            }
            vtl as usize
        } else {
            read.get_var_int()?.0 as usize
        };
        let verify_token = read.read_slice_borrowed(verify_token_len)?;

        let should_authenticate = if version >= &JavaMinecraftVersion::V_1_20_5 {
            read.get_bool()?
        } else {
            true
        };
        Ok(Self {
            server_id,
            public_key,
            verify_token,
            should_authenticate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ServerPacket;

    #[test]
    fn encryption_request_roundtrip() {
        let packet =
            CEncryptionRequest::new("test_server", b"public_key_bytes", b"verify_1234", true);
        let mut buf = Vec::new();
        let version = JavaMinecraftVersion::V_1_21_4;
        packet.write_packet_data(&mut buf, &version).unwrap();

        let mut slice = buf.as_slice();
        let read_packet = CEncryptionRequest::read(&mut slice, &version).unwrap();
        assert_eq!(read_packet.server_id, packet.server_id);
        assert_eq!(read_packet.public_key, packet.public_key);
        assert_eq!(read_packet.verify_token, packet.verify_token);
        assert_eq!(read_packet.should_authenticate, packet.should_authenticate);
    }
}
