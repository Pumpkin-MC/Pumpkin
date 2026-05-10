use std::io::{Error, Write};

use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[derive(Clone, Copy)]
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
