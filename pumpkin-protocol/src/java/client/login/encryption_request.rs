use pumpkin_data::packet::clientbound::LOGIN_HELLO;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the server to initiate the encryption handshake.
///
/// This packet provides the client with the server's public key and a
/// verification token, allowing the client to generate a shared secret
/// for secure communication.
#[java_packet(LOGIN_HELLO)]
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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(self.server_id)?;
        write.write_var_int(&crate::VarInt(self.public_key.len() as i32))?;
        write.write_all(self.public_key)?;
        write.write_var_int(&crate::VarInt(self.verify_token.len() as i32))?;
        write.write_all(self.verify_token)?;
        write.write_bool(self.should_authenticate)?;
        Ok(())
    }
}
