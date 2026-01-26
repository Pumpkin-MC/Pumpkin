use pumpkin_data::packet::clientbound::LOGIN_HELLO;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

/// Sent by the server to initiate the encryption handshake.
///
/// This packet provides the client with the server's public key and a
/// verification token, allowing the client to generate a shared secret
/// for secure communication.
#[derive(Serialize, Deserialize)]
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
    pub fn new(
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
