use std::io::Write;

use crate::codec::bit_set::BitSet;
use crate::codec::var_int::VarInt;
use crate::ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError};
use crate::{ClientPacket, ServerPacket};
use pumpkin_data::packet::clientbound::play::LIGHT_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::format::LightContainer;

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

    pub fn from_chunk(
        chunk: &ChunkData,
        version: JavaMinecraftVersion,
    ) -> Result<Self, WritingError> {
        let light_data = LightData::from_chunk(chunk, version)?;
        Ok(Self {
            chunk_x: VarInt(chunk.x),
            chunk_z: VarInt(chunk.z),
            light_data,
        })
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

    #[expect(clippy::too_many_lines)]
    pub fn from_chunk(
        chunk: &ChunkData,
        version: JavaMinecraftVersion,
    ) -> Result<Self, WritingError> {
        let light_engine = chunk
            .light_engine
            .lock()
            .map_err(|_| WritingError::Message("light_engine lock poisoned".into()))?;

        if version < JavaMinecraftVersion::V_1_18 {
            let base_section = (0 - chunk.section.min_y).max(0) as usize / 16;
            let mut sky_light_mask = 0u64;
            let mut block_light_mask = 0u64;
            let mut sky_light_empty_mask = 0u64;
            let mut block_light_empty_mask = 0u64;
            let mut sky_light_arrays = Vec::new();
            let mut block_light_arrays = Vec::new();

            // Bit 0: Y = -1 (below world section 0)
            if base_section > 0 && base_section - 1 < light_engine.sky_light.len() {
                match &light_engine.sky_light[base_section - 1] {
                    LightContainer::Full(data) => {
                        sky_light_mask |= 1 << 0;
                        sky_light_arrays.push(data.to_vec());
                    }
                    LightContainer::Empty(val) if *val > 0 => {
                        sky_light_mask |= 1 << 0;
                        sky_light_arrays.push(vec![*val << 4 | *val; 2048]);
                    }
                    LightContainer::Empty(_) => {
                        sky_light_empty_mask |= 1 << 0;
                    }
                }
            } else {
                sky_light_empty_mask |= 1 << 0;
            }

            if base_section > 0 && base_section - 1 < light_engine.block_light.len() {
                match &light_engine.block_light[base_section - 1] {
                    LightContainer::Full(data) => {
                        block_light_mask |= 1 << 0;
                        block_light_arrays.push(data.to_vec());
                    }
                    LightContainer::Empty(val) if *val > 0 => {
                        block_light_mask |= 1 << 0;
                        block_light_arrays.push(vec![*val << 4 | *val; 2048]);
                    }
                    LightContainer::Empty(_) => {
                        block_light_empty_mask |= 1 << 0;
                    }
                }
            } else {
                block_light_empty_mask |= 1 << 0;
            }

            // Bits 1..=16: world sections (Y = 0..15)
            for i in 0..16 {
                let bit_index = i + 1;
                let sec_idx = base_section + i;

                if sec_idx < light_engine.sky_light.len() {
                    match &light_engine.sky_light[sec_idx] {
                        LightContainer::Full(data) => {
                            sky_light_mask |= 1 << bit_index;
                            sky_light_arrays.push(data.to_vec());
                        }
                        LightContainer::Empty(val) if *val > 0 => {
                            sky_light_mask |= 1 << bit_index;
                            sky_light_arrays.push(vec![*val << 4 | *val; 2048]);
                        }
                        LightContainer::Empty(_) => {
                            sky_light_empty_mask |= 1 << bit_index;
                        }
                    }
                } else {
                    sky_light_empty_mask |= 1 << bit_index;
                }

                if sec_idx < light_engine.block_light.len() {
                    match &light_engine.block_light[sec_idx] {
                        LightContainer::Full(data) => {
                            block_light_mask |= 1 << bit_index;
                            block_light_arrays.push(data.to_vec());
                        }
                        LightContainer::Empty(val) if *val > 0 => {
                            block_light_mask |= 1 << bit_index;
                            block_light_arrays.push(vec![*val << 4 | *val; 2048]);
                        }
                        LightContainer::Empty(_) => {
                            block_light_empty_mask |= 1 << bit_index;
                        }
                    }
                } else {
                    block_light_empty_mask |= 1 << bit_index;
                }
            }

            // Bit 17: Y = 16 (above world section 15)
            let top_sec = base_section + 16;
            if top_sec < light_engine.sky_light.len() {
                match &light_engine.sky_light[top_sec] {
                    LightContainer::Full(data) => {
                        sky_light_mask |= 1 << 17;
                        sky_light_arrays.push(data.to_vec());
                    }
                    LightContainer::Empty(val) if *val > 0 => {
                        sky_light_mask |= 1 << 17;
                        sky_light_arrays.push(vec![*val << 4 | *val; 2048]);
                    }
                    LightContainer::Empty(_) => {
                        sky_light_empty_mask |= 1 << 17;
                    }
                }
            } else {
                sky_light_empty_mask |= 1 << 17;
            }

            if top_sec < light_engine.block_light.len() {
                match &light_engine.block_light[top_sec] {
                    LightContainer::Full(data) => {
                        block_light_mask |= 1 << 17;
                        block_light_arrays.push(data.to_vec());
                    }
                    LightContainer::Empty(val) if *val > 0 => {
                        block_light_mask |= 1 << 17;
                        block_light_arrays.push(vec![*val << 4 | *val; 2048]);
                    }
                    LightContainer::Empty(_) => {
                        block_light_empty_mask |= 1 << 17;
                    }
                }
            } else {
                block_light_empty_mask |= 1 << 17;
            }

            Ok(Self {
                trust_edges: true,
                sky_light_mask: BitSet::from_u64(sky_light_mask),
                block_light_mask: BitSet::from_u64(block_light_mask),
                empty_sky_light_mask: BitSet::from_u64(sky_light_empty_mask),
                empty_block_light_mask: BitSet::from_u64(block_light_empty_mask),
                sky_light_arrays,
                block_light_arrays,
            })
        } else {
            let num_sections = light_engine.sky_light.len();
            let mut sky_light_empty_mask = 0u64;
            let mut block_light_empty_mask = 0u64;
            let mut sky_light_mask = 0u64;
            let mut block_light_mask = 0u64;

            let mut sky_light_arrays = Vec::new();
            let mut block_light_arrays = Vec::new();

            sky_light_empty_mask |= 1 << 0;
            block_light_empty_mask |= 1 << 0;

            for section_index in 0..num_sections {
                let bit_index = section_index + 1;

                if let LightContainer::Full(data) = &light_engine.sky_light[section_index] {
                    sky_light_mask |= 1 << bit_index;
                    sky_light_arrays.push(data.to_vec());
                } else {
                    sky_light_empty_mask |= 1 << bit_index;
                }

                if let LightContainer::Full(data) = &light_engine.block_light[section_index] {
                    block_light_mask |= 1 << bit_index;
                    block_light_arrays.push(data.to_vec());
                } else {
                    block_light_empty_mask |= 1 << bit_index;
                }
            }

            sky_light_empty_mask |= 1 << (num_sections + 1);
            block_light_empty_mask |= 1 << (num_sections + 1);

            Ok(Self {
                trust_edges: true,
                sky_light_mask: BitSet::from_u64(sky_light_mask),
                block_light_mask: BitSet::from_u64(block_light_mask),
                empty_sky_light_mask: BitSet::from_u64(sky_light_empty_mask),
                empty_block_light_mask: BitSet::from_u64(block_light_empty_mask),
                sky_light_arrays,
                block_light_arrays,
            })
        }
    }

    pub fn write(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        // Trust edges (1.16 - 1.19.4; added in 1.16, removed in 1.20)
        if *version >= JavaMinecraftVersion::V_1_16 && *version <= JavaMinecraftVersion::V_1_19_4 {
            write.write_bool(self.trust_edges)?;
        }

        // Chunk bitmasks
        if *version >= JavaMinecraftVersion::V_1_17 {
            write.write_bitset(&self.sky_light_mask)?;
            write.write_bitset(&self.block_light_mask)?;
            write.write_bitset(&self.empty_sky_light_mask)?;
            write.write_bitset(&self.empty_block_light_mask)?;
        } else {
            write.write_var_int(&VarInt(self.sky_light_mask.as_u64() as i32))?;
            write.write_var_int(&VarInt(self.block_light_mask.as_u64() as i32))?;
            write.write_var_int(&VarInt(self.empty_sky_light_mask.as_u64() as i32))?;
            write.write_var_int(&VarInt(self.empty_block_light_mask.as_u64() as i32))?;
        }

        // Sky light arrays
        if *version >= JavaMinecraftVersion::V_1_17 {
            write.write_var_int(&VarInt(self.sky_light_arrays.len() as i32))?;
            for array in &self.sky_light_arrays {
                write.write_var_int(&VarInt(array.len() as i32))?;
                write.write_slice(array)?;
            }
        } else {
            let mut array_idx = 0;
            for i in 0..18 {
                if self.sky_light_mask.get_bit(i)
                    && let Some(array) = self.sky_light_arrays.get(array_idx)
                {
                    write.write_var_int(&VarInt(array.len() as i32))?;
                    write.write_slice(array)?;
                    array_idx += 1;
                }
            }
        }

        // Block light arrays
        if *version >= JavaMinecraftVersion::V_1_17 {
            write.write_var_int(&VarInt(self.block_light_arrays.len() as i32))?;
            for array in &self.block_light_arrays {
                write.write_var_int(&VarInt(array.len() as i32))?;
                write.write_slice(array)?;
            }
        } else {
            let mut array_idx = 0;
            for i in 0..18 {
                if self.block_light_mask.get_bit(i)
                    && let Some(array) = self.block_light_arrays.get(array_idx)
                {
                    write.write_var_int(&VarInt(array.len() as i32))?;
                    write.write_slice(array)?;
                    array_idx += 1;
                }
            }
        }

        Ok(())
    }

    pub fn read(bytebuf: &mut &[u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let trust_edges = if *version >= JavaMinecraftVersion::V_1_16
            && *version <= JavaMinecraftVersion::V_1_19_4
        {
            bytebuf.get_bool()?
        } else {
            false
        };

        let (sky_light_mask, block_light_mask, empty_sky_light_mask, empty_block_light_mask) =
            if *version >= JavaMinecraftVersion::V_1_17 {
                (
                    BitSet::decode(bytebuf)?,
                    BitSet::decode(bytebuf)?,
                    BitSet::decode(bytebuf)?,
                    BitSet::decode(bytebuf)?,
                )
            } else {
                (
                    BitSet::from_u64(bytebuf.get_var_int()?.0 as u64),
                    BitSet::from_u64(bytebuf.get_var_int()?.0 as u64),
                    BitSet::from_u64(bytebuf.get_var_int()?.0 as u64),
                    BitSet::from_u64(bytebuf.get_var_int()?.0 as u64),
                )
            };

        let sky_light_arrays = if *version >= JavaMinecraftVersion::V_1_17 {
            let count = bytebuf.get_var_int()?.0 as usize;
            let mut arrays = Vec::with_capacity(count);
            for _ in 0..count {
                let len = bytebuf.get_var_int()?.0 as usize;
                let mut buf = vec![0u8; len];
                bytebuf.read_bytes_to_buf(&mut buf)?;
                arrays.push(buf);
            }
            arrays
        } else {
            let mut arrays = Vec::new();
            for i in 0..18 {
                if sky_light_mask.get_bit(i) {
                    let len = bytebuf.get_var_int()?.0 as usize;
                    let mut buf = vec![0u8; len];
                    bytebuf.read_bytes_to_buf(&mut buf)?;
                    arrays.push(buf);
                }
            }
            arrays
        };

        let block_light_arrays = if *version >= JavaMinecraftVersion::V_1_17 {
            let count = bytebuf.get_var_int()?.0 as usize;
            let mut arrays = Vec::with_capacity(count);
            for _ in 0..count {
                let len = bytebuf.get_var_int()?.0 as usize;
                let mut buf = vec![0u8; len];
                bytebuf.read_bytes_to_buf(&mut buf)?;
                arrays.push(buf);
            }
            arrays
        } else {
            let mut arrays = Vec::new();
            for i in 0..18 {
                if block_light_mask.get_bit(i) {
                    let len = bytebuf.get_var_int()?.0 as usize;
                    let mut buf = vec![0u8; len];
                    bytebuf.read_bytes_to_buf(&mut buf)?;
                    arrays.push(buf);
                }
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
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.chunk_x)?;
        write.write_var_int(&self.chunk_z)?;
        self.light_data.write(&mut write, version)
    }
}

impl<'a> ServerPacket<'a> for CLightUpdate {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let chunk_x = bytebuf.get_var_int()?;
        let chunk_z = bytebuf.get_var_int()?;
        let light_data = LightData::read(bytebuf, version)?;
        Ok(Self {
            chunk_x,
            chunk_z,
            light_data,
        })
    }
}
