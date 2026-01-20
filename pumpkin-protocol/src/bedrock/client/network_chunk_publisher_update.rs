use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};

use crate::{
    codec::{bedrock_block_pos::NetworkPos, var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

#[packet(121)]
pub struct CNetworkChunkPublisherUpdate {
    // https://mojang.github.io/bedrock-protocol-docs/html/NetworkChunkPublisherUpdatePacket.html
    pub pos_for_view: NetworkPos,
    // Is in blocks, not chunks!
    pub new_radius: VarUInt,
    // TODO
    pub server_build_chunk_list: Vec<Vector2<i32>>,
}

impl CNetworkChunkPublisherUpdate {
    #[must_use] 
    pub const fn new(pos_for_view: BlockPos, new_radius: u32) -> Self {
        Self {
            pos_for_view: NetworkPos(pos_for_view),
            new_radius: VarUInt(new_radius),
            server_build_chunk_list: Vec::new(),
        }
    }
}

impl PacketWrite for CNetworkChunkPublisherUpdate {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.pos_for_view.write_signed(writer)?;
        self.new_radius.write(writer)?;

        (self.server_build_chunk_list.len() as u32).write(writer)?;
        for chunk in &self.server_build_chunk_list {
            VarInt(chunk.x).write(writer)?;
            VarInt(chunk.y).write(writer)?;
        }
        Ok(())
    }
}
