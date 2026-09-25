use std::io::{Error, Read, Write};

use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    BClientPacket, BServerPacket,
    codec::var_int::VarIntType,
    serial::{PacketRead, PacketWrite},
};

pub trait Packet {
    const PACKET_ID: VarIntType;
}

pub trait MultiVersionJavaPacket {
    #[must_use]
    fn to_id(version: JavaMinecraftVersion) -> i32;

    #[must_use]
    fn state() -> crate::ConnectionState {
        crate::ConnectionState::Play
    }
}

impl<P: Packet + PacketWrite> BClientPacket for P {
    fn write_packet(&self, mut writer: impl Write) -> Result<(), Error> {
        self.write(&mut writer)
    }
}

impl<P: Packet + PacketRead> BServerPacket for P {
    fn read(mut read: impl Read) -> Result<Self, Error> {
        P::read(&mut read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ConnectionState,
        java::{client, server},
    };

    struct ExternalPacket;

    impl MultiVersionJavaPacket for ExternalPacket {
        fn to_id(_: JavaMinecraftVersion) -> i32 {
            0
        }
    }

    #[test]
    fn java_packet_state_matches_protocol_group() {
        assert_eq!(
            server::handshake::SHandShake::state(),
            ConnectionState::HandShake
        );
        assert_eq!(
            server::status::SStatusRequest::state(),
            ConnectionState::Status
        );
        assert_eq!(
            client::status::CPingResponse::state(),
            ConnectionState::Status
        );
        assert_eq!(
            client::login::CSetCompression::state(),
            ConnectionState::Login
        );
        assert_eq!(
            client::login::CEncryptionRequest::state(),
            ConnectionState::Login
        );
        assert_eq!(
            client::login::CLoginSuccess::state(),
            ConnectionState::Login
        );
        assert_eq!(
            server::login::SEncryptionResponse::state(),
            ConnectionState::Login
        );
        assert_eq!(
            client::config::CConfigKeepAlive::state(),
            ConnectionState::Config
        );
        assert_eq!(
            client::config::CFinishConfig::state(),
            ConnectionState::Config
        );
        assert_eq!(client::play::CKeepAlive::state(), ConnectionState::Play);
        assert_eq!(
            client::play::CStartConfiguration::state(),
            ConnectionState::Play
        );

        assert_eq!(ExternalPacket::state(), ConnectionState::Play);
    }
}
