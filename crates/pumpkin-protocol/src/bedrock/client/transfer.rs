use pumpkin_macros::packet;

use std::io::{Error, Write};

use crate::serial::PacketWrite;

#[packet(85)]
pub struct CTransfer {
    pub server_address: String,
    pub server_port: u16,
    pub reload_world: bool,
}

impl PacketWrite for CTransfer {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.server_address.write(writer)?;
        self.server_port.write(writer)?;
        self.reload_world.write(writer)?;
        // Optional GatheringsConfigurationJoinInfo.
        false.write(writer)
    }
}

impl CTransfer {
    #[must_use]
    pub const fn new(server_address: String, server_port: u16, reload_world: bool) -> Self {
        Self {
            server_address,
            server_port,
            reload_world,
        }
    }
}
