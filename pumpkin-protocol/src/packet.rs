use std::io::{Error, Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    BClientPacket, BServerPacket, ClientPacket, ReadingError, ServerPacket, WritingError,
    codec::var_int::VarIntType,
    ser::{deserializer, serializer},
    serial::{PacketRead, PacketWrite},
};

pub trait Packet {
    const PACKET_ID: VarIntType;
}

impl<P: Packet + Serialize> ClientPacket for P {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut serializer = serializer::Serializer::new(write);
        self.serialize(&mut serializer)
    }
}

impl<P: Packet + DeserializeOwned> ServerPacket for P {
    fn read(read: impl Read) -> Result<P, ReadingError> {
        let mut deserializer = deserializer::Deserializer::new(read);
        P::deserialize(&mut deserializer)
    }
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
