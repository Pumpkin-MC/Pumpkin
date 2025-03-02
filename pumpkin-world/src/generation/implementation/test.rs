use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use crate::{
    WORLD_LOWEST_Y, WORLD_MAX_Y,
    chunk::{ChunkData, Subchunks},
    coordinates::ChunkRelativeBlockCoordinates,
    generation::{
        GlobalRandomConfig, Seed, WorldGenerator, generator::GeneratorInit,
        noise_router::proto_noise_router::GlobalProtoNoiseRouter, proto_chunk::ProtoChunk,
    },
    noise_router::NOISE_ROUTER_ASTS,
};

pub struct TestGenerator {
    random_config: GlobalRandomConfig,
    base_router: GlobalProtoNoiseRouter,
}

impl GeneratorInit for TestGenerator {
    fn new(seed: Seed) -> Self {
        let random_config = GlobalRandomConfig::new(seed.0);
        let base_router =
            GlobalProtoNoiseRouter::generate(&NOISE_ROUTER_ASTS.overworld, &random_config);
        Self {
            random_config,
            base_router,
        }
    }
}

impl WorldGenerator for TestGenerator {
    fn generate_chunk(&self, at: Vector2<i32>) -> ChunkData {
        let mut subchunks = Subchunks::Single(0);
        let mut proto_chunk = ProtoChunk::new(at, &self.base_router, &self.random_config);
        proto_chunk.populate_noise();

        for x in 0..16u8 {
            for z in 0..16u8 {
                // TODO: This can be chunk specific
                for y in (WORLD_LOWEST_Y..WORLD_MAX_Y).rev() {
                    let coordinates = ChunkRelativeBlockCoordinates {
                        x: x.into(),
                        y: y.into(),
                        z: z.into(),
                    };

                    let block =
                        proto_chunk.get_block_state(&Vector3::new(x.into(), y.into(), z.into()));

                    //println!("{:?}: {:?}", coordinates, block);
                    subchunks.set_block(coordinates, block.state_id);
                }
            }
        }

        ChunkData {
            subchunks,
            heightmap: Default::default(),
            position: at,
        }
    }
}
