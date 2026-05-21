use pumpkin_macros::packet;
use std::io::{Error, Write};

use crate::serial::PacketWrite;

#[derive(PacketWrite)]
#[packet(85)]
pub struct CTransfer {
    pub address: String,
    pub port: u16,
    pub reload_world: bool,
}

impl CTransfer {
    #[must_use]
    pub const fn new(address: String, port: u16, reload_world: bool) -> Self {
        Self {
            address,
            port,
            reload_world,
        }
    }
}