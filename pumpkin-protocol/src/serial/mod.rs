pub mod deserializer;
pub mod serializer;
pub use pumpkin_macros::{PacketRead, PacketWrite};
use std::io::{Error, Read, Write};

pub trait PacketWrite {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
    fn write_be<W: Write>(&self, _writer: &mut W) -> Result<(), Error> {
        Ok(())
    }
}

pub trait PacketRead: Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error>;
}

pub enum WError {
    Interupted,
    Errr,
}
