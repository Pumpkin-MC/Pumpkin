use pumpkin_data::packet::clientbound::CONFIG_CLEAR_DIALOG;
use pumpkin_macros::java_packet;
use serde::Serialize;

#[derive(Serialize)]
#[java_packet(CONFIG_CLEAR_DIALOG)]
pub struct CConfigClearDialog;

impl CConfigClearDialog {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for CConfigClearDialog {
    fn default() -> Self {
        Self::new()
    }
}
