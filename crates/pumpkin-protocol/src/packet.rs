use std::io::{Error, Read, Write};

use crate::{
    BClientPacket, BServerPacket,
    codec::var_int::VarIntType,
    serial::{PacketRead, PacketWrite},
};

pub trait Packet {
    const PACKET_ID: VarIntType;
}

/// A Java packet of the current version.
pub trait JavaPacket {
    const PACKET_ID: i32;
}

impl<P: Packet + PacketWrite> BClientPacket for P {
    fn write_packet(&self, mut writer: impl Write) -> Result<(), Error> {
        self.write(&mut writer)
    }
}

impl<P: Packet + PacketRead> BServerPacket for P {
    fn read(mut read: impl Read) -> Result<Self, Error> {
        P::read(&mut read)
    }
}
