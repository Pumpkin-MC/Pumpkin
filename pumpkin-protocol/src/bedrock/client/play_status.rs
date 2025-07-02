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
            status: status.to_index(),
        }
    }
}

pub enum PlayStatus {
    LoginSuccess,
    OutdatedClient,
    OutdatedServer,
    PlayerSpawn,
    InvalidTenant,
    EditionMismatchEduToVanilla,
    EditionMismatchVanillaToEdu,
    ServerFullSubClient,
    EditorMismatchEditorToVanilla,
    EditorMismatchVanillaToEditor,
}

impl PlayStatus {
    pub fn to_index(&self) -> i32 {
        match self {
            PlayStatus::LoginSuccess => 0,
            PlayStatus::OutdatedClient => 1,
            PlayStatus::OutdatedServer => 2,
            PlayStatus::PlayerSpawn => 3,
            PlayStatus::InvalidTenant => 4,
            PlayStatus::EditionMismatchEduToVanilla => 5,
            PlayStatus::EditionMismatchVanillaToEdu => 6,
            PlayStatus::ServerFullSubClient => 7,
            PlayStatus::EditorMismatchEditorToVanilla => 8,
            PlayStatus::EditorMismatchVanillaToEditor => 9,
        }
    }
}
