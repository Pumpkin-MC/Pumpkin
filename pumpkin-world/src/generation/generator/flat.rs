use super::FlatLayer;
use crate::chunk_system::StagedChunkEnum;
use crate::generation::Seed;
use crate::generation::positions::chunk_pos::{start_block_x, start_block_z};
use crate::generation::proto_chunk::ProtoChunk;
use pumpkin_data::Block;
use pumpkin_data::dimension::Dimension;

pub struct FlatGenerator {
    pub seed: u64,
    pub dimension: Dimension,
    pub layers: Vec<FlatLayer>,
    pub biome: String,
}

impl FlatGenerator {
    #[must_use]
    pub const fn new(
        seed: Seed,
        dimension: Dimension,
        layers: Vec<FlatLayer>,
        biome: String,
    ) -> Self {
        Self {
            seed: seed.0,
            dimension,
            layers,
            biome,
        }
    }

    pub fn step_to_biomes(&self, chunk: &mut ProtoChunk) {
        let clean_biome = self.biome.strip_prefix("minecraft:").unwrap_or(&self.biome);
        let biome_id = pumpkin_data::chunk::Biome::from_name(clean_biome)
            .map_or(pumpkin_data::chunk::Biome::PLAINS.id, |b| b.id);
        chunk.flat_biome_map.fill(biome_id);
        chunk.stage = StagedChunkEnum::Biomes;
    }

    pub fn step_to_noise(&self, chunk: &mut ProtoChunk) {
        let start_x = start_block_x(chunk.x);
        let start_z = start_block_z(chunk.z);
        for x in 0..16 {
            for z in 0..16 {
                let mut current_y = chunk.bottom_y() as i32;
                for layer in &self.layers {
                    let block = Block::from_name(&layer.block);
                    let state = block.map_or(Block::AIR.default_state, |b| b.default_state);
                    for _ in 0..layer.height {
                        if current_y < chunk.bottom_y() as i32 + chunk.height() as i32 {
                            chunk.set_block_state(start_x + x, current_y, start_z + z, state);
                            current_y += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        chunk.stage = StagedChunkEnum::Noise;
    }

    pub const fn step_to_surface(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Surface;
    }

    pub const fn step_to_carvers(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Carvers;
    }
}
