use std::io::{Error, Write};

use crate::serial::PacketWrite;

pub struct AsciiString(pub String);

impl PacketWrite for AsciiString {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&(self.0.len() as u16).to_be_bytes())?;
        writer.write_all(self.0.as_bytes())
    }
}
