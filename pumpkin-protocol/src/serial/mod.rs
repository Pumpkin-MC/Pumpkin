pub mod deserializer;
pub mod serializer;
pub use pumpkin_macros::{PacketRead, PacketWrite};
use std::io::{Error, Read, Write};

pub trait PacketWrite {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
    fn write_be<W: Write>(&self, _writer: &mut W) -> Result<(), Error> {
        todo!()
    }
}

pub trait PacketRead: Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error>;
    fn read_be<R: Read>(_reader: &mut R) -> Result<Self, Error> {
        todo!()
    }
}
