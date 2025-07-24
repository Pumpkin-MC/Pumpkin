use std::io::{Error, Read, Write};

use crate::serial::{PacketRead, PacketWrite};

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct u24(pub u32);

impl PacketWrite for u24 {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let data = self.0.to_le_bytes();
        writer.write_all(&data[0..3])
    }
}

impl PacketRead for u24 {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let mut buf = [0; 3];
        reader.read_exact(&mut buf)?;
        Ok(u24(u32::from_le_bytes([buf[0], buf[1], buf[2], 0])))
    }
}
