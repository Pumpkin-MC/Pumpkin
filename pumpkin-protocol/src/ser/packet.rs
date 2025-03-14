use serde::{Serialize, de::DeserializeOwned};

use crate::{ClientPacket, ServerPacket, codec::var_int::VarIntType};

use super::{
    NetworkRead, NetworkWrite, ReadingError, WritingError, deserializer,
    serializer::{self},
};

pub trait Packet {
    const PACKET_ID: VarIntType;
}

impl<P> ClientPacket for P
where
    P: Packet + Serialize,
{
    fn write(&self, write: impl NetworkWrite) -> Result<(), WritingError> {
        let mut serializer = serializer::Serializer::new(write);
        self.serialize(&mut serializer)
    }
}

impl<P> ServerPacket for P
where
    P: Packet + DeserializeOwned,
{
    fn read(read: impl NetworkRead) -> Result<P, ReadingError> {
        let mut deserializer = deserializer::Deserializer::new(read);
        P::deserialize(&mut deserializer)
    }
}
