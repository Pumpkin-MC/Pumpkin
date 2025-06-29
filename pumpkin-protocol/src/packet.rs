use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};

use crate::{
    ClientPacket, ReadingError, ServerPacket, WritingError,
    codec::var_int::VarIntType,
    ser::{deserializer, serializer},
};

pub trait Packet {
    const PACKET_ID: VarIntType;
}

impl<P> ClientPacket for P
where
    P: Packet + Serialize,
{
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut serializer = serializer::Serializer::new(write);
        self.serialize(&mut serializer)
    }
}

impl<P> ServerPacket for P
where
    P: Packet + DeserializeOwned,
{
    fn read(read: impl Read) -> Result<P, ReadingError> {
        let mut deserializer = deserializer::Deserializer::new(read);
        P::deserialize(&mut deserializer)
    }
}
