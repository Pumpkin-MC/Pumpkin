use pumpkin_macros::packet;

use crate::serial::PacketWrite;

#[derive(PacketWrite)]
#[packet(2)]
pub struct CPlayStatus {
    #[serial(big_endian)]
    status: i32,
}

impl CPlayStatus {
    pub fn new(status: PlayStatus) -> Self {
        Self {
            status: status as i32,
        }
    }
}

#[repr(i32)]
pub enum PlayStatus {
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
