pub mod superflat;

use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use crate::{
    chunk::{ChunkBlocks, ChunkData},
    coordinates::ChunkRelativeBlockCoordinates,
    generation::{
        GlobalRandomConfig, Seed, WorldGenerator, generator::GeneratorInit,
        noise_router::proto_noise_router::GlobalProtoNoiseRouter, proto_chunk::ProtoChunk,
    },
    noise_router::NOISE_ROUTER_ASTS,
};

use super::settings::{GENERATION_SETTINGS, GeneratorSetting};

pub struct VanillaGenerator {
    random_config: GlobalRandomConfig,
    base_router: GlobalProtoNoiseRouter,
}

impl GeneratorInit for VanillaGenerator {
    fn new(seed: Seed) -> Self {
        let random_config = GlobalRandomConfig::new(seed.0, false);
        // TODO: The generation settings contains (part of?) the noise routers too; do we keep the separate or
        // use only the generation settings?
        let base_router =
            GlobalProtoNoiseRouter::generate(&NOISE_ROUTER_ASTS.overworld, &random_config);
        Self {
            random_config,
            base_router,
        }
    }
}

impl WorldGenerator for VanillaGenerator {
    fn generate_chunk(&self, at: Vector2<i32>) -> ChunkData {
        let mut blocks = ChunkBlocks::Homogeneous(0);
        // TODO: This is bad, but it works
        let generation_settings = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();
        let mut proto_chunk = ProtoChunk::new(
            at,
            &self.base_router,
            &self.random_config,
            generation_settings,
        );
        proto_chunk.populate_biomes();
        proto_chunk.populate_noise();
        proto_chunk.build_surface();

        for x in 0..16u8 {
            for z in 0..16u8 {
                // TODO: This can be chunk specific
                for y in 0..generation_settings.noise.height {
                    let y = generation_settings.noise.min_y as i32 + y as i32;
                    let coordinates = ChunkRelativeBlockCoordinates {
                        x: x.into(),
                        y: y.into(),
                        z: z.into(),
                    };

                    let block = proto_chunk.get_block_state(&Vector3::new(x.into(), y, z.into()));
                    // TODO: check air
                    if block.block_id == 0 {
                        continue;
                    }

                    blocks.set_block(coordinates, block.state_id);
                }
            }
        }

        ChunkData {
            blocks,
            heightmap: Default::default(),
            position: at,
            dirty: true,
        }
    }
}
