use pumpkin_macros::packet;

use std::io::{Error, Write};

use crate::serial::PacketWrite;

/// Instructs the client to disconnect and reconnect to another server address and port.
#[packet(85)]
pub struct CTransfer {
    /// Destination server hostname or IP address.
    pub server_address: String,
    /// Destination server UDP port.
    pub server_port: u16,
    /// Whether the client should reload its local world context upon transferring.
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
    /// Constructs a transfer packet with target server connection parameters.
    #[must_use]
    pub const fn new(server_address: String, server_port: u16, reload_world: bool) -> Self {
        Self {
            server_address,
            server_port,
            reload_world,
        }
    }
}
