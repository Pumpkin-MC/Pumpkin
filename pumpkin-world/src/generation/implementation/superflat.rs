use pumpkin_data::chunk::Biome;
use pumpkin_util::math::vector2::Vector2;

use crate::{
    biome::SuperflatBiomeGenerator,
    block::block_state::BlockState,
    coordinates::XZBlockCoordinates,
    generation::{
        generator::{GeneratorInit, TerrainGenerator},
        generic_generator::GenericGenerator,
        Seed,
    },
};

#[expect(dead_code)]
pub type SuperflatGenerator = GenericGenerator<SuperflatBiomeGenerator, SuperflatTerrainGenerator>;

pub(crate) struct SuperflatTerrainGenerator {}

impl GeneratorInit for SuperflatTerrainGenerator {
    fn new(_: Seed) -> Self {
        Self {}
    }
}

impl TerrainGenerator for SuperflatTerrainGenerator {
    fn prepare_chunk(&self, _at: &Vector2<i32>) {}
    fn clean_chunk(&self, _at: &Vector2<i32>) {}

    // TODO allow specifying which blocks should be at which height in the config.
    fn generate_block(
        &self,
        _chunk_pos: &Vector2<i32>,
        _at: pumpkin_util::math::vector3::Vector3<i32>,
        _biome: Biome,
    ) -> BlockState {
        todo!()
    }

    /*
    fn generate_block(&self, at: BlockCoordinates, _: Biome) -> BlockState {
        match *at.y {
            -64 => block_state!("bedrock"),
            -63..=-62 => block_state!("dirt"),
            -61 => block_state!("grass_block"),
            _ => BlockState::AIR,
        }
    }
    */
}
