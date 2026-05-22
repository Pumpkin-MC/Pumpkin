use pumpkin_data::packet::clientbound::PLAY_CLEAR_DIALOG;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(PLAY_CLEAR_DIALOG)]
pub struct CPlayClearDialog;

impl CPlayClearDialog {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for CPlayClearDialog {
    fn default() -> Self {
        Self::new()
    }
}
