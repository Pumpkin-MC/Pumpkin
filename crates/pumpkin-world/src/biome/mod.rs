use sha2::{Digest, Sha256};
use std::cell::RefCell;

use pumpkin_data::chunk::{Biome, BiomeTree, NETHER_BIOME_SOURCE, OVERWORLD_BIOME_SOURCE};

use crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;
pub mod end;
pub mod multi_noise;
pub mod position_finder;

pub use position_finder::{
    Climate, ClimateSampler, DistanceMetric, FittestPositionFinder, FittestPositionFinderResult,
    ParameterList, RTree, RTreeLeaf, RTreeNode, RTreeSubTree, SpawnFinder, SpawnFinderResult,
};
pub use pumpkin_data::chunk::{
    Parameter, ParameterPoint, ParameterRange, TargetPoint, quantize_coord, unquantize_coord,
};

thread_local! {
    /// A shortcut; check if last used biome is what we should use.
    ///
    /// Keyed by the tree the leaf came from. `BiomeTree::get` seeds its search distance from
    /// this leaf and only accepts strictly closer nodes, so a leaf cached from one dimension's
    /// tree would otherwise beat every node of another's and be returned unchanged. Vanilla
    /// stores `lastResult` on the `RTree` itself rather than sharing one across all of them.
    static LAST_RESULT_NODE: RefCell<Option<(&'static BiomeTree, &'static BiomeTree)>> =
        const { RefCell::new(None) };
}

pub trait BiomeSupplier {
    fn biome(&self, x: i32, y: i32, z: i32, noise: &mut MultiNoiseSampler<'_>) -> &'static Biome;
}

#[derive(Clone, Copy)]
pub struct MultiNoiseBiomeSupplier {
    source: &'static BiomeTree,
}

impl MultiNoiseBiomeSupplier {
    pub const OVERWORLD: Self = Self::new(&OVERWORLD_BIOME_SOURCE);
    pub const NETHER: Self = Self::new(&NETHER_BIOME_SOURCE);

    #[must_use]
    pub fn from_preset(preset: &str) -> Option<Self> {
        let preset = preset.strip_prefix("minecraft:").unwrap_or(preset);
        match preset {
            "overworld" | "large_biomes" | "amplified" => Some(Self::OVERWORLD),
            "nether" => Some(Self::NETHER),
            _ => None,
        }
    }

    const fn new(source: &'static BiomeTree) -> Self {
        Self { source }
    }
}

impl BiomeSupplier for MultiNoiseBiomeSupplier {
    fn biome(&self, x: i32, y: i32, z: i32, noise: &mut MultiNoiseSampler<'_>) -> &'static Biome {
        let point = noise.sample(x, y, z);
        let point_list = point.convert_to_list();
        LAST_RESULT_NODE.with_borrow_mut(|last_result| {
            let mut node = last_result
                .and_then(|(tree, node)| std::ptr::eq(tree, self.source).then_some(node));
            let biome = self.source.get(&point_list, &mut node);
            *last_result = node.map(|node| (self.source, node));
            biome
        })
    }
}

#[derive(Clone, Copy)]
pub struct FixedBiomeSupplier {
    pub biome: &'static Biome,
}

impl FixedBiomeSupplier {
    #[must_use]
    pub const fn new(biome: &'static Biome) -> Self {
        Self { biome }
    }
}

impl BiomeSupplier for FixedBiomeSupplier {
    fn biome(
        &self,
        _x: i32,
        _y: i32,
        _z: i32,
        _noise: &mut MultiNoiseSampler<'_>,
    ) -> &'static Biome {
        self.biome
    }
}

#[derive(Clone, Copy)]
pub enum ActiveBiomeSupplier {
    MultiNoise(MultiNoiseBiomeSupplier),
    End(end::TheEndBiomeSupplier),
    Fixed(FixedBiomeSupplier),
}

impl BiomeSupplier for ActiveBiomeSupplier {
    fn biome(&self, x: i32, y: i32, z: i32, noise: &mut MultiNoiseSampler<'_>) -> &'static Biome {
        match self {
            Self::MultiNoise(supplier) => supplier.biome(x, y, z, noise),
            Self::End(supplier) => supplier.biome(x, y, z, noise),
            Self::Fixed(supplier) => supplier.biome(x, y, z, noise),
        }
    }
}

impl ActiveBiomeSupplier {
    #[must_use]
    pub fn from_biome_source(
        source: Option<&crate::world_info::BiomeSource>,
        dimension: &pumpkin_data::dimension::Dimension,
    ) -> Self {
        match source {
            Some(crate::world_info::BiomeSource::Fixed { biome, .. }) => {
                let clean = biome.strip_prefix("minecraft:").unwrap_or(biome);
                let resolved = pumpkin_data::biome::Biome::from_name(clean)
                    .unwrap_or(&pumpkin_data::biome::Biome::PLAINS);
                Self::Fixed(FixedBiomeSupplier::new(resolved))
            }
            Some(crate::world_info::BiomeSource::WithPreset { preset, .. }) => {
                let clean = preset.strip_prefix("minecraft:").unwrap_or(preset);
                if clean == "nether" {
                    Self::MultiNoise(MultiNoiseBiomeSupplier::NETHER)
                } else {
                    Self::MultiNoise(MultiNoiseBiomeSupplier::OVERWORLD)
                }
            }
            Some(crate::world_info::BiomeSource::Simple { biome_type }) => {
                let clean = biome_type.strip_prefix("minecraft:").unwrap_or(biome_type);
                if clean == "the_end" {
                    Self::End(end::TheEndBiomeSupplier)
                } else if clean == "the_nether" {
                    Self::MultiNoise(MultiNoiseBiomeSupplier::NETHER)
                } else {
                    Self::from_dimension(dimension)
                }
            }
            None => Self::from_dimension(dimension),
        }
    }

    #[must_use]
    pub fn from_dimension(dimension: &pumpkin_data::dimension::Dimension) -> Self {
        if dimension == &pumpkin_data::dimension::Dimension::THE_END {
            Self::End(end::TheEndBiomeSupplier)
        } else if dimension == &pumpkin_data::dimension::Dimension::THE_NETHER {
            Self::MultiNoise(MultiNoiseBiomeSupplier::NETHER)
        } else {
            Self::MultiNoise(MultiNoiseBiomeSupplier::OVERWORLD)
        }
    }
}

#[must_use]
pub fn hash_seed(seed: u64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(seed.to_le_bytes());
    let result = hasher.finalize();
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&result[..8]);
    i64::from_le_bytes(bytes)
}

#[cfg(test)]
mod test {
    use pumpkin_data::{chunk::Biome, dimension::Dimension};
    use pumpkin_util::read_data_from_file;
    use serde::Deserialize;

    use crate::{
        ProtoChunk, chunk::palette::BIOME_NETWORK_MAX_BITS,
        generation::noise::router::multi_noise_sampler::MultiNoiseSampler,
    };

    use super::{BiomeSupplier, MultiNoiseBiomeSupplier, hash_seed};

    #[test]
    fn biome_desert() {
        use crate::generation::generator::{GeneratorInit, VanillaGenerator};
        use pumpkin_util::world_seed::Seed;
        let seed = 13579;
        let generator = VanillaGenerator::new(Seed(seed as u64), Dimension::OVERWORLD);
        let mut sampler = MultiNoiseSampler::generate(&generator.base_router.multi_noise);
        let biome = MultiNoiseBiomeSupplier::OVERWORLD.biome(-24, 1, 8, &mut sampler);
        assert_eq!(biome, &Biome::DESERT);
    }

    #[test]
    fn wide_area_surface() {
        use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
        use crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;
        use pumpkin_util::world_seed::Seed;
        #[derive(Deserialize)]
        struct BiomeData {
            x: i32,
            z: i32,
            data: Vec<(i32, i32, i32, u8)>,
        }

        let expected_data: Vec<BiomeData> =
            read_data_from_file!("../../assets/tests/biome_no_blend_no_beard_0.json");

        let seed = 0;
        let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
            Seed(seed as u64),
            Dimension::OVERWORLD,
        )));
        let WorldGenerator::Noise(generator) = &world_gen else {
            unreachable!()
        };

        for data in expected_data {
            let chunk_x = data.x;
            let chunk_z = data.z;

            let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);

            let mut multi_noise_sampler =
                MultiNoiseSampler::generate(&generator.base_router.multi_noise);

            chunk.populate_biomes(generator, &mut multi_noise_sampler);

            for (biome_x, biome_y, biome_z, biome_id) in data.data {
                let calculated_biome = chunk.get_biome(biome_x, biome_y, biome_z);

                assert_eq!(
                    biome_id,
                    calculated_biome.id,
                    "Expected {:?} was {:?} at {},{},{} ({},{})",
                    Biome::from_id(biome_id),
                    calculated_biome,
                    biome_x,
                    biome_y,
                    biome_z,
                    data.x,
                    data.z
                );
            }
        }
    }

    #[test]
    fn hash_seed_test() {
        let hashed_seed = hash_seed(0);
        assert_eq!(8794265229978523055, hashed_seed);

        let hashed_seed = hash_seed((-777i64) as u64);
        assert_eq!(-1087248400229165450, hashed_seed);
    }

    #[test]
    fn proper_network_bits_per_entry() {
        let id_to_test = 1 << BIOME_NETWORK_MAX_BITS;
        assert!(
            Biome::from_id(id_to_test).is_none(),
            "We need to update our constants!"
        );
    }

    /// A leaf cached from one dimension's biome tree must never seed another dimension's
    /// search, or the Nether fills with overworld biomes (and their mobs and features).
    #[test]
    fn nether_chunks_only_contain_nether_biomes() {
        use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
        use crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;
        use pumpkin_util::world_seed::Seed;

        let fill = |dimension: Dimension, chunk_x: i32, chunk_z: i32| -> Vec<u8> {
            let world_gen =
                WorldGenerator::Noise(Box::new(VanillaGenerator::new(Seed(0), dimension)));
            let WorldGenerator::Noise(generator) = &world_gen else {
                unreachable!()
            };
            let mut chunk = ProtoChunk::new(chunk_x, chunk_z, &world_gen);
            let mut sampler = MultiNoiseSampler::generate(&generator.base_router.multi_noise);
            chunk.populate_biomes(generator, &mut sampler);
            let mut biomes = Vec::new();
            for y in 0..8 {
                for x in 0..4 {
                    for z in 0..4 {
                        biomes.push(chunk.get_biome(x, y, z).id);
                    }
                }
            }
            biomes
        };

        // Warm the thread-local leaf cache with overworld leaves, then generate a Nether
        // chunk on the same thread.
        let _ = fill(Dimension::OVERWORLD, 0, 0);
        let nether = fill(Dimension::THE_NETHER, 0, 0);

        let nether_ids: Vec<u8> = [
            "nether_wastes",
            "soul_sand_valley",
            "crimson_forest",
            "warped_forest",
            "basalt_deltas",
        ]
        .iter()
        .map(|name| Biome::from_name(name).unwrap().id)
        .collect();

        let leaked: Vec<&str> = nether
            .iter()
            .filter(|id| !nether_ids.contains(id))
            .filter_map(|id| Biome::from_id(*id).map(|biome| biome.registry_id))
            .collect();
        assert!(
            leaked.is_empty(),
            "non-Nether biomes generated in a Nether chunk: {leaked:?}"
        );
    }
}
