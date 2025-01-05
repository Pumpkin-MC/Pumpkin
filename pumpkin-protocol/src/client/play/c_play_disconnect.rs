use pumpkin_core::text::Text;

use pumpkin_macros::client_packet;
use serde::Serialize;

#[derive(Serialize)]
#[client_packet("play:disconnect")]
pub struct CPlayDisconnect {
    reason: Text,
}

impl CPlayDisconnect {
    pub fn new(reason: impl Into<Text>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}
