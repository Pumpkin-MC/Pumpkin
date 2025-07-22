use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

#[packet(121)]
pub struct CNetworkChunkPublisherUpdate {
    // https://mojang.github.io/bedrock-protocol-docs/html/NetworkChunkPublisherUpdatePacket.html
    pub pos_for_view: BlockPos,
    // Is in blocks, not chunks!
    pub new_radius: VarUInt,
    // TODO
    pub server_build_chunk_list: Vec<Vector2<i32>>,
}

impl CNetworkChunkPublisherUpdate {
    pub fn new(pos_for_view: BlockPos, new_radius: u32) -> Self {
        let mut chunk_list = Vec::new();

        for x in -1..1 {
            for z in -1..1 {
                chunk_list.push(Vector2::new(x, z));
            }
        }
        Self {
            pos_for_view,
            new_radius: VarUInt(new_radius << 4),
            server_build_chunk_list: chunk_list,
        }
    }
}

impl PacketWrite for CNetworkChunkPublisherUpdate {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.pos_for_view.write(writer)?;
        self.new_radius.write(writer)?;

        (self.server_build_chunk_list.len() as u32).write(writer)?;
        for chunk in &self.server_build_chunk_list {
            VarInt(chunk.x).write(writer)?;
            VarInt(chunk.y).write(writer)?;
        }
        Ok(())
    }
}
