use std::io::{Error, Write};

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

#[derive(Default, Clone)]
pub struct NetworkItemDescriptor {
    // I hate mojang
    // https://mojang.github.io/bedrock-protocol-docs/html/NetworkItemInstanceDescriptor.html
    pub id: VarInt,
    pub stack_size: u16,
    pub aux_value: VarUInt,
    pub block_runtime_id: VarInt,
    pub user_data_buffer: ItemInstanceUserData,
}

impl PacketWrite for NetworkItemDescriptor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;
        if self.id.0 != 0 {
            self.stack_size.write(writer)?;
            self.aux_value.write(writer)?;
            self.block_runtime_id.write(writer)?;
            self.user_data_buffer.write(writer)?;
        }
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct ItemInstanceUserData {
    // https://mojang.github.io/bedrock-protocol-docs/html/ItemInstanceUserData.html
    //compound
    place_on_block_size: VarUInt,
    destroy_blocks_size: VarUInt,
}

impl PacketWrite for ItemInstanceUserData {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let mut buf = Vec::new();
        (-1i16).write(&mut buf)?;
        (1i8).write(&mut buf)?;

        (10i8).write(&mut buf)?;
        VarUInt(0).write(&mut buf)?;
        (0i8).write(&mut buf)?;

        self.place_on_block_size.write(&mut buf)?;
        self.destroy_blocks_size.write(&mut buf)?;

        VarUInt(buf.len() as u32).write(writer)?;
        writer.write_all(&buf)
    }
}

#[derive(Default, Clone)]
pub struct NetworkItemStackDescriptor {
    // I hate mojang
    // https://mojang.github.io/bedrock-protocol-docs/html/NetworkItemStackDescriptor.html
    pub id: VarInt,
    pub stack_size: u16,
    pub aux_value: VarUInt,
    pub net_id: Option<VarInt>,
    pub block_runtime_id: VarInt,
    pub user_data_buffer: ItemInstanceUserData,
}

impl PacketWrite for NetworkItemStackDescriptor {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.id.write(writer)?;
        if self.id.0 != 0 {
            self.stack_size.write(writer)?;
            self.aux_value.write(writer)?;
            self.net_id.write(writer)?;
            self.block_runtime_id.write(writer)?;
            self.user_data_buffer.write(writer)?;
        }
        Ok(())
    }
}
