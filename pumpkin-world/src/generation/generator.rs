use noise::Perlin;
use pumpkin_data::chunk::Biome;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;

use crate::block::block_state::BlockState;
use crate::chunk::{ChunkData, Subchunks};
use crate::coordinates::{BlockCoordinates, ChunkRelativeBlockCoordinates, XZBlockCoordinates};
use crate::generation::Seed;

pub trait GeneratorInit {
    fn new(seed: Seed) -> Self;
}

pub trait WorldGenerator: Sync + Send {
    fn generate_chunk(&self, at: Vector2<i32>) -> ChunkData;
}

pub(crate) trait BiomeGenerator: Sync + Send {
    fn generate_biome(&self, at: XZBlockCoordinates) -> Biome;
}

pub(crate) trait TerrainGenerator: Sync + Send {
    fn prepare_chunk(&self, at: &Vector2<i32>);

    fn clean_chunk(&self, at: &Vector2<i32>);

    /// Is static
    fn generate_block(
        &self,
        chunk_pos: &Vector2<i32>,
        at: Vector3<i32>,
        biome: Biome,
    ) -> BlockState;
}

pub(crate) trait PerlinTerrainGenerator: Sync + Send {
    fn height_variation(&self) -> f64 {
        4.0
    }

    fn prepare_chunk(&self, at: &Vector2<i32>, perlin: &Perlin);

    /// Depends on the perlin noise height
    fn generate_block(
        &self,
        coordinates: ChunkRelativeBlockCoordinates,
        at: BlockCoordinates,
        subchunks: &mut Subchunks,
        chunk_height: i16,
        biome: Biome,
    );
}
