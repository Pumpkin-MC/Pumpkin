use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[packet(0x02)]
pub struct CPlayStatus {
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
