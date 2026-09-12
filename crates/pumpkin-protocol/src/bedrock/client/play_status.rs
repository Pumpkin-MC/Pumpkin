// Last verified for v2169

use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[derive(Clone, Copy)]
#[repr(i32)]
#[packet(2)]
pub enum CPlayStatus {
    LoginSuccess = 0,
    OutdatedClient = 1,
    OutdatedServer = 2,
    PlayerSpawn = 3,
    InvalidTenant = 4,
    EditionMismatchEduToVanilla = 5,
    EditionMismatchVanillaToEdu = 6,
    ServerFullSubClient = 7,
    EditorMismatchEditorToVanilla = 8,
    EditorMismatchVanillaToEditor = 9,
}

impl PacketWrite for CPlayStatus {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as i32).write_be(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bedrock::packet_encoder::serialize_packet;

    #[test]
    fn writes_vanilla_version_rejection_codes() {
        let mut outdated_client = Vec::new();
        CPlayStatus::OutdatedClient
            .write(&mut outdated_client)
            .unwrap();
        assert_eq!(outdated_client, [0, 0, 0, 1]);

        let mut outdated_server = Vec::new();
        CPlayStatus::OutdatedServer
            .write(&mut outdated_server)
            .unwrap();
        assert_eq!(outdated_server, [0, 0, 0, 2]);

        assert_eq!(
            serialize_packet(&CPlayStatus::OutdatedServer)
                .unwrap()
                .as_ref(),
            [0xfe, 5, 2, 0, 0, 0, 2]
        );
    }
}
