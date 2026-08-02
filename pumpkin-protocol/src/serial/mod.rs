pub mod deserializer;
pub mod serializer;
pub use pumpkin_macros::{PacketRead, PacketWrite};
use std::io::{Error, Read, Write};

pub trait PacketWrite {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error>;
    fn write_be<W: Write>(&self, _writer: &mut W) -> Result<(), Error> {
        Err(Error::other("Not implemented"))
    }
}

pub trait PacketRead: Sized {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error>;
    fn read_be<R: Read>(_reader: &mut R) -> Result<Self, Error> {
        Err(Error::other("Not implemented"))
    }
}

pub trait PacketReadSlice<'a>: Sized {
    fn read_slice(buf: &mut &'a [u8]) -> Result<Self, Error>;
}

pub(crate) fn read_str_slice<'a>(buf: &mut &'a [u8]) -> Result<&'a str, Error> {
    use crate::codec::var_uint::VarUInt;
    use std::io::ErrorKind;

    const MAX_STRING_LENGTH: usize = 32767;

    let len = VarUInt::read(buf)?.0 as usize;
    if len > MAX_STRING_LENGTH {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("String length {len} exceeds maximum of {MAX_STRING_LENGTH}"),
        ));
    }
    if len > buf.len() {
        return Err(Error::new(
            ErrorKind::UnexpectedEof,
            "Not enough bytes for string",
        ));
    }
    let (str_bytes, rest) = buf.split_at(len);
    *buf = rest;
    std::str::from_utf8(str_bytes)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid UTF-8 sequence"))
}
