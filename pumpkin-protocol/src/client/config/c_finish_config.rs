use pumpkin_data::packet::clientbound::CONFIG_FINISH_CONFIGURATION;
use pumpkin_macros::client_packet;

#[derive(serde::Serialize)]
#[client_packet(CONFIG_FINISH_CONFIGURATION)]
pub struct CFinishConfig {}

impl Default for CFinishConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CFinishConfig {
    pub fn new() -> Self {
        Self {}
    }
}
