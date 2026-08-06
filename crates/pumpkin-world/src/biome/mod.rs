use sha2::{Digest, Sha256};
use std::cell::RefCell;

use pumpkin_data::chunk::{Biome, BiomeTree, NETHER_BIOME_SOURCE, OVERWORLD_BIOME_SOURCE};

use crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;
pub mod end;
pub mod multi_noise;
pub mod position_finder;

thread_local! {
    /// A shortcut; check if last used biome is what we should use
    static LAST_RESULT_NODE: RefCell<Option<&'static BiomeTree>> = const {RefCell::new(None) };
}

pub trait BiomeSupplier {
    fn biome(&self, x: i32, y: i32, z: i32, noise: &mut MultiNoiseSampler<'_>) -> &'static Biome;
}

pub struct MultiNoiseBiomeSupplier {
    source: &'static BiomeTree,
}

impl MultiNoiseBiomeSupplier {
    pub const OVERWORLD: Self = Self::new(&OVERWORLD_BIOME_SOURCE);
    pub const NETHER: Self = Self::new(&NETHER_BIOME_SOURCE);

    const fn new(source: &'static BiomeTree) -> Self {
        Self { source }
    }
}

impl BiomeSupplier for MultiNoiseBiomeSupplier {
    fn biome(&self, x: i32, y: i32, z: i32, noise: &mut MultiNoiseSampler<'_>) -> &'static Biome {
        let point = noise.sample(x, y, z);
        let point_list = point.convert_to_list();
        LAST_RESULT_NODE.with_borrow_mut(|last_result| self.source.get(&point_list, last_result))
    }
}

#[must_use]
pub fn hash_seed(seed: u64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(seed.to_le_bytes());
    let result = hasher.finalize();
    i64::from_le_bytes(result[..8].try_into().unwrap())
}

#[cfg(test)]
mod test {
    use pumpkin_data::{chunk::Biome, dimension::Dimension};
    use pumpkin_util::read_data_from_file;
    use serde::Deserialize;

    use crate::{
        ProtoChunk,
        chunk::palette::BIOME_NETWORK_MAX_BITS,
        generation::noise::router::multi_noise_sampler::{
            MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
        },
    };

    use super::{BiomeSupplier, MultiNoiseBiomeSupplier, hash_seed};

    #[test]
    fn biome_desert() {
        use crate::generation::generator::{GeneratorInit, VanillaGenerator};
        use pumpkin_util::world_seed::Seed;
        let seed = 13579;
        let generator = VanillaGenerator::new(Seed(seed as u64), Dimension::OVERWORLD);
        let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(1, 1, 1);
        let mut sampler =
            MultiNoiseSampler::generate(&generator.base_router.multi_noise, &multi_noise_config);
        let biome = MultiNoiseBiomeSupplier::OVERWORLD.biome(-24, 1, 8, &mut sampler);
        assert_eq!(biome, &Biome::DESERT);
    }

    #[test]
    fn wide_area_surface() {
        use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
        use crate::generation::noise::router::multi_noise_sampler::{
            MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
        };
        use crate::generation::{biome_coords, positions::chunk_pos};
        use pumpkin_util::world_seed::Seed;
        #[derive(Deserialize)]
        struct BiomeData {
            x: i32,
            z: i32,
            data: Vec<(i32, i32, i32, u8)>,
        }

        let expected_data: Vec<BiomeData> =
            read_data_from_file!("../../../../assets/tests/biome_no_blend_no_beard_0.json");

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

            // Create MultiNoiseSampler for populate_biomes

            let start_x = chunk_pos::start_block_x(chunk_x);
            let start_z = chunk_pos::start_block_z(chunk_z);

            let horizontal_biome_end = biome_coords::from_block(16);
            let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(
                biome_coords::from_block(start_x),
                biome_coords::from_block(start_z),
                horizontal_biome_end as usize,
            );
            let mut multi_noise_sampler = MultiNoiseSampler::generate(
                &generator.base_router.multi_noise,
                &multi_noise_config,
            );

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
}

/// Regression coverage for the overworld multi-noise biome search against a
/// vanilla-derived fixture of 724271 biome cells (biome coords -50..=50 on x/z,
/// -20..=50 on y, seed 0), sampled with one `MultiNoiseSampler` window per chunk
/// and in the same iteration order `ProtoChunk::populate_biomes` uses.
///
/// Every sample matches vanilla exactly except for 943 that are all attributable
/// to `assets/multi_noise_biome_tree.json` having been extracted from an older
/// game version than `assets/biome.json`:
///
/// * 872 cells whose vanilla biome is `sulfur_caves`, which has a registry entry
///   (`Biome::SULFUR_CAVES`, id 53) but no leaf in the extracted tree, so the
///   search can never return it.
/// * 71 cells in the single biome column (x=-49, z=-24) where the noise point is
///   exactly equidistant from a `forest` leaf and a `birch_forest` leaf at every
///   y, so the winner is decided purely by leaf order.
///
/// See `CENSUS_FIXLIST.md`.
#[cfg(test)]
mod multi_noise_wide_area_test {
    use pumpkin_data::{chunk::Biome, dimension::Dimension};
    use pumpkin_util::read_data_from_file;
    use std::collections::{BTreeMap, HashMap};

    use crate::generation::noise::router::multi_noise_sampler::{
        MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
    };

    use super::{BiomeSupplier, MultiNoiseBiomeSupplier};

    const SULFUR_CAVES_ID: u8 = 53;
    /// The one biome column where the search hits an exact distance tie.
    const KNOWN_TIE_COLUMN: (i32, i32) = (-49, -24);

    type ChunkExpectations = HashMap<(i32, i32, i32), u8>;

    #[test]
    fn multi_noise_biome_source_wide_area() {
        use crate::generation::generator::{GeneratorInit, VanillaGenerator};
        use pumpkin_util::world_seed::Seed;

        let expected_data: Vec<(i32, i32, i32, u8)> =
            read_data_from_file!("../../../../assets/tests/multi_noise_biome_source_test.json");

        let generator = VanillaGenerator::new(Seed(0), Dimension::OVERWORLD);

        let mut by_chunk: BTreeMap<(i32, i32), ChunkExpectations> = BTreeMap::new();
        for entry in expected_data {
            by_chunk
                .entry((entry.0.div_euclid(4), entry.2.div_euclid(4)))
                .or_default()
                .insert((entry.0, entry.1, entry.2), entry.3);
        }

        let mut total = 0usize;
        let mut missing_sulfur_caves = 0usize;
        let mut tie_column_diffs = 0usize;
        let mut unexplained: Vec<String> = Vec::new();

        for ((chunk_x, chunk_z), entries) in &by_chunk {
            let options = MultiNoiseSamplerBuilderOptions::new(chunk_x * 4, chunk_z * 4, 4);
            let mut sampler =
                MultiNoiseSampler::generate(&generator.base_router.multi_noise, &options);

            // Match `populate_biomes`: section-major, then x, y, z. The search keeps a
            // thread-local "last result node" shortcut, so iteration order is load bearing.
            for section in -6i32..=13 {
                for x in 0..4 {
                    for y in 0..4 {
                        for z in 0..4 {
                            let biome_x = chunk_x * 4 + x;
                            let biome_y = section * 4 + y;
                            let biome_z = chunk_z * 4 + z;
                            let Some(&expected) = entries.get(&(biome_x, biome_y, biome_z)) else {
                                continue;
                            };
                            total += 1;
                            let actual = MultiNoiseBiomeSupplier::OVERWORLD.biome(
                                biome_x,
                                biome_y,
                                biome_z,
                                &mut sampler,
                            );
                            if actual.id == expected {
                                continue;
                            }
                            if expected == SULFUR_CAVES_ID {
                                missing_sulfur_caves += 1;
                            } else if (biome_x, biome_z) == KNOWN_TIE_COLUMN {
                                tie_column_diffs += 1;
                            } else if unexplained.len() < 20 {
                                unexplained.push(format!(
                                    "at biome({biome_x},{biome_y},{biome_z}) expected {:?} got {:?}",
                                    Biome::from_id(expected).map(|b| b.registry_id),
                                    actual.registry_id
                                ));
                            }
                        }
                    }
                }
            }
        }

        assert_eq!(total, 724271, "fixture coverage changed");
        assert!(
            unexplained.is_empty(),
            "unexplained biome mismatches vs vanilla: {unexplained:#?}"
        );
        assert_eq!(
            missing_sulfur_caves, 872,
            "sulfur_caves coverage changed; if this dropped to 0 the biome tree asset was \
             re-extracted and this test should assert an exact match instead"
        );
        assert_eq!(tie_column_diffs, 71, "tie-break column coverage changed");
    }
}
