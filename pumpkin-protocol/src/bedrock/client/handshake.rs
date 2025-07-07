use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[packet(0x03)]
pub struct CHandshake {
    jwt_data: String,
}

impl CHandshake {
    pub fn new(jwt_data: String) -> Self {
        Self { jwt_data }
    }
}
