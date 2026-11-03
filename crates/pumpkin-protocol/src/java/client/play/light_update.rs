use std::io::Write;

use crate::codec::bit_set::BitSet;
use crate::codec::var_int::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use crate::{ClientPacket, ServerPacket};
use pumpkin_data::packet::clientbound::play::LIGHT_UPDATE;
use pumpkin_macros::java_packet;

/// Sent by the server to update light levels (block light and sky light) for a chunk.
///
/// This packet updates lighting data for a specific chunk without sending the full chunk data.
/// It was introduced in Minecraft 1.14 (protocol version 477).
#[derive(Debug, PartialEq, Eq, Clone)]
#[java_packet(LIGHT_UPDATE)]
pub struct CLightUpdate {
    pub chunk_x: VarInt,
    pub chunk_z: VarInt,
    pub light_data: LightData,
}

pub type CUpdateLight = CLightUpdate;

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LightData {
    pub trust_edges: bool,
    pub sky_light_mask: BitSet,
    pub block_light_mask: BitSet,
    pub empty_sky_light_mask: BitSet,
    pub empty_block_light_mask: BitSet,
    pub sky_light_arrays: Vec<Vec<u8>>,
    pub block_light_arrays: Vec<Vec<u8>>,
}

impl CLightUpdate {
    #[must_use]
    pub const fn new(chunk_x: VarInt, chunk_z: VarInt, light_data: LightData) -> Self {
        Self {
            chunk_x,
            chunk_z,
            light_data,
        }
    }
}

impl LightData {
    #[must_use]
    pub const fn new(
        trust_edges: bool,
        sky_light_mask: BitSet,
        block_light_mask: BitSet,
        empty_sky_light_mask: BitSet,
        empty_block_light_mask: BitSet,
        sky_light_arrays: Vec<Vec<u8>>,
        block_light_arrays: Vec<Vec<u8>>,
    ) -> Self {
        Self {
            trust_edges,
            sky_light_mask,
            block_light_mask,
            empty_sky_light_mask,
            empty_block_light_mask,
            sky_light_arrays,
            block_light_arrays,
        }
    }

    pub fn write(&self, mut write: impl Write) -> Result<(), WritingError> {
        // Trust edges (1.16 - 1.19.4; added in 1.16, removed in 1.20)

        // Chunk bitmasks
        self.sky_light_mask.encode(&mut write)?;
        self.block_light_mask.encode(&mut write)?;
        self.empty_sky_light_mask.encode(&mut write)?;
        self.empty_block_light_mask.encode(&mut write)?;

        // Sky light arrays
        write.write_var_int(&VarInt(self.sky_light_arrays.len() as i32))?;
        for array in &self.sky_light_arrays {
            write.write_var_int(&VarInt(array.len() as i32))?;
            write.write_slice(array)?;
        }

        // Block light arrays
        write.write_var_int(&VarInt(self.block_light_arrays.len() as i32))?;
        for array in &self.block_light_arrays {
            write.write_var_int(&VarInt(array.len() as i32))?;
            write.write_slice(array)?;
        }

        Ok(())
    }

    pub fn read(bytebuf: &mut &[u8]) -> Result<Self, ReadingError> {
        let trust_edges = false;

        let (sky_light_mask, block_light_mask, empty_sky_light_mask, empty_block_light_mask) = (
            BitSet::decode(bytebuf)?,
            BitSet::decode(bytebuf)?,
            BitSet::decode(bytebuf)?,
            BitSet::decode(bytebuf)?,
        );

        let sky_light_arrays = {
            let count = bytebuf.get_var_int()?.0 as usize;
            let mut arrays = Vec::with_capacity(count);
            for _ in 0..count {
                let len = bytebuf.get_var_int()?.0 as usize;
                let mut buf = vec![0u8; len];
                bytebuf.read_bytes_to_buf(&mut buf)?;
                arrays.push(buf);
            }
            arrays
        };

        let block_light_arrays = {
            let count = bytebuf.get_var_int()?.0 as usize;
            let mut arrays = Vec::with_capacity(count);
            for _ in 0..count {
                let len = bytebuf.get_var_int()?.0 as usize;
                let mut buf = vec![0u8; len];
                bytebuf.read_bytes_to_buf(&mut buf)?;
                arrays.push(buf);
            }
            arrays
        };

        Ok(Self {
            trust_edges,
            sky_light_mask,
            block_light_mask,
            empty_sky_light_mask,
            empty_block_light_mask,
            sky_light_arrays,
            block_light_arrays,
        })
    }
}

impl ClientPacket for CLightUpdate {
    fn write_packet_data(&self, mut write: impl Write) -> Result<(), WritingError> {
        write.write_var_int(&self.chunk_x)?;
        write.write_var_int(&self.chunk_z)?;
        self.light_data.write(&mut write)
    }
}

impl<'a> ServerPacket<'a> for CLightUpdate {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let chunk_x = bytebuf.get_var_int()?;
        let chunk_z = bytebuf.get_var_int()?;
        let light_data = LightData::read(bytebuf)?;
        Ok(Self {
            chunk_x,
            chunk_z,
            light_data,
        })
    }
}
