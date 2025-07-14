use std::io::{Error, Write};

use pumpkin_macros::packet;
use pumpkin_world::chunk::ChunkData;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

#[packet(58)]
pub struct CLevelChunk<'a> {
    // https://mojang.github.io/bedrock-protocol-docs/html/LevelChunkPacket.html
    pub dimension: VarInt,
    pub sub_chunks_count: VarUInt,
    pub cache_enabled: bool,

    // https://gist.github.com/Tomcc/a96af509e275b1af483b25c543cfbf37
    pub chunk: &'a ChunkData,
}

impl<'a> PacketWrite for CLevelChunk<'a> {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.chunk.position.x).write(writer)?;
        VarInt(self.chunk.position.y).write(writer)?;
        self.sub_chunks_count.write(writer)?;
        self.dimension.write(writer)?;
        self.cache_enabled.write(writer)?;

        writer.write_all(&[9, 0])
    }
}
