use crate::{
    ClientPacket, ConnectionState, ReadingError, ServerPacket, VarInt, ser::NetworkReadExt,
    ser::NetworkWriteExt,
};
use pumpkin_data::packet::serverbound::handshake::INTENTION;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// The very first packet sent by the client to initiate a connection
///
/// It determines whether the client wants to check the server status (SLP)
/// or actually login to play.
#[java_packet(INTENTION)]
pub struct SHandShake {
    /// The protocol version of the client (e.g., 767 for 1.21).
    pub protocol_version: VarInt,
    /// The hostname or IP used by the client to connect
    pub server_address: Box<str>,
    /// The port number used by the client to connect
    pub server_port: u16,
    /// The state the client wants to transition to (1 for Status, 2 for Login)
    pub next_state: ConnectionState,
}

impl<'a> ServerPacket<'a> for SHandShake {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            protocol_version: read.get_var_int()?,
            // A vanilla client never sends more than 255 characters here, but
            // `BungeeCord`-style IP forwarding appends the client's IP, its UUID
            // and its signed profile properties to this field, which easily
            // exceeds that. Read it with the same bound `write_string` already
            // writes it with, as Spigot does.
            server_address: read.get_str_bounded(i16::MAX as usize)?,
            server_port: read.get_u16_be()?,
            next_state: read
                .get_var_int()?
                .try_into()
                .map_err(|_| ReadingError::Message("Invalid status".to_string()))?,
        })
    }
}

impl ClientPacket for SHandShake {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.protocol_version)?;
        write.write_string(&self.server_address)?;
        write.write_u16_be(self.server_port)?;
        write.write_var_int(&VarInt(self.next_state as i32))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Encodes the body of a handshake packet, as a client or a proxy sends it.
    fn encode_handshake(server_address: &str, next_state: i32) -> Vec<u8> {
        let mut buf = Vec::new();
        let protocol_version = JavaMinecraftVersion::V_1_21_11.protocol_version();
        buf.write_var_int(&VarInt(protocol_version))
            .expect("write protocol version");
        // Bound by the input itself so that this helper can also build the
        // oversized address used by `rejects_oversized_server_address`.
        buf.write_string_bounded(server_address, server_address.len())
            .expect("write server address");
        buf.write_u16_be(25565).expect("write server port");
        buf.write_var_int(&VarInt(next_state))
            .expect("write next state");
        buf
    }

    /// The address `BungeeCord` sends downstream when `ip_forward` is enabled:
    /// the host, the client's IP, its UUID and the signed profile properties,
    /// separated by NUL bytes. `bungeecord_login` in the `pumpkin` crate splits
    /// this exact value back into those four parts.
    fn bungeecord_forwarded_address() -> String {
        let textures = "e".repeat(432);
        let signature = "s".repeat(684);
        format!(
            "mc.example.com\0192.0.2.10\0d8f4a1e0-0f1b-4c3a-9f2e-1a2b3c4d5e6f\0\
             [{{\"name\":\"textures\",\"value\":\"{textures}\",\"signature\":\"{signature}\"}}]"
        )
    }

    #[test]
    fn reads_bungeecord_forwarded_server_address() {
        let address = bungeecord_forwarded_address();
        let buf = encode_handshake(&address, 2);

        let packet = SHandShake::read(&mut &buf[..], &JavaMinecraftVersion::V_1_21_11)
            .expect("a BungeeCord-forwarded address should be readable");

        assert_eq!(&*packet.server_address, address.as_str());
        assert_eq!(packet.server_address.split('\0').count(), 4);
        assert_eq!(packet.next_state, ConnectionState::Login);
    }

    #[test]
    fn reads_plain_server_address() {
        let buf = encode_handshake("localhost", 1);

        let packet = SHandShake::read(&mut &buf[..], &JavaMinecraftVersion::V_1_21_11)
            .expect("a plain address should be readable");

        assert_eq!(&*packet.server_address, "localhost");
        assert_eq!(packet.next_state, ConnectionState::Status);
    }

    #[test]
    fn rejects_oversized_server_address() {
        let address = "a".repeat(i16::MAX as usize + 1);
        let buf = encode_handshake(&address, 2);

        let result = SHandShake::read(&mut &buf[..], &JavaMinecraftVersion::V_1_21_11);

        assert!(matches!(result, Err(ReadingError::TooLarge(_))));
    }
}
