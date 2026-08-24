use std::io::{Error, ErrorKind, Read, Write};

use crate::serial::{PacketRead, PacketWrite};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RespawnState {
    SearchingForSpawn = 0,
    ReadyToSpawn = 1,
    ClientReadyToSpawn = 2,
}

impl PacketRead for RespawnState {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        match u8::read(reader)? {
            0 => Ok(Self::SearchingForSpawn),
            1 => Ok(Self::ReadyToSpawn),
            2 => Ok(Self::ClientReadyToSpawn),
            state => Err(Error::new(
                ErrorKind::InvalidData,
                format!("invalid Bedrock respawn state {state}"),
            )),
        }
    }
}

impl PacketWrite for RespawnState {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        (*self as u8).write(writer)
    }
}
