use std::io::{Error, Write};

use uuid::Uuid;

use crate::serial::PacketWrite;

impl PacketWrite for Uuid {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(self.as_bytes())
    }
}
