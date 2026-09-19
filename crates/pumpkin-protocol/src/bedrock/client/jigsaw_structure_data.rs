use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_nbt::compound::NbtCompound;

use crate::serial::PacketWrite;

#[packet(313)]
pub struct CJigsawStructureData;

impl PacketWrite for CJigsawStructureData {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut data = NbtCompound::new();
        for name in ["processors", "template_pools", "jigsaws", "structure_sets"] {
            data.put_list(name, Vec::new());
        }
        data.write(writer)
    }
}
