use pumpkin_util::math::vector2::Vector2;

use crate::chunk::ChunkData;
use crate::generation::Seed;

pub trait GeneratorInit {
    fn new(seed: Seed) -> Self;
}

pub trait WorldGenerator: Sync + Send {
    fn generate_chunk(&self, at: &Vector2<i32>) -> ChunkData;
}
