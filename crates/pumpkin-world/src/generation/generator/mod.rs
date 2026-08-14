use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;

pub mod chunk_generator;
pub mod flat;
pub mod noise;
pub mod structure_finder;
pub mod vanilla;
mod vanilla_impl;

pub use chunk_generator::{ChunkGenerator, ChunkGeneratorDecode, ChunkGeneratorType};
pub use noise::NoiseGeneratorConfig;
pub use vanilla::VanillaGenerator;

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
}

#[derive(Clone, Debug)]
pub struct FlatLayer {
    pub block: String,
    pub height: i32,
}

pub enum WorldGenerator {
    Noise(Box<VanillaGenerator>),
    Flat(Box<flat::FlatGenerator>),
}

impl WorldGenerator {
    #[must_use]
    pub fn as_generator(&self) -> &dyn ChunkGenerator {
        match self {
            Self::Noise(generator) => generator.as_ref(),
            Self::Flat(generator) => generator.as_ref(),
        }
    }

    #[must_use]
    pub fn dimension(&self) -> &Dimension {
        match self {
            Self::Noise(noise_gen) => &noise_gen.dimension,
            Self::Flat(flat_gen) => &flat_gen.dimension,
        }
    }

    #[must_use]
    pub fn seed(&self) -> u64 {
        match self {
            Self::Noise(noise_gen) => noise_gen.random_config.seed,
            Self::Flat(flat_gen) => flat_gen.seed,
        }
    }

    #[must_use]
    pub fn global_structure_cache(
        &self,
    ) -> Option<&crate::generation::structure::placement::GlobalStructureCache> {
        match self {
            Self::Noise(noise_gen) => Some(&noise_gen.global_structure_cache),
            Self::Flat(_) => None,
        }
    }
}
