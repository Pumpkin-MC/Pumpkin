use pumpkin_data::{Block, BlockState};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use super::weeping_vines::WeepingVinesFeature;
use crate::{
    generation::{block_predicate::BlockPredicate, proto_chunk::GenerationCache},
    world::WorldPortalExt,
};

pub struct HugeFungusFeature {
    pub valid_base_block: &'static BlockState,
    pub stem_state: &'static BlockState,
    pub hat_state: &'static BlockState,
    pub decor_state: &'static BlockState,
    pub replaceable_blocks: BlockPredicate,
    pub planted: bool,
}

impl HugeFungusFeature {
    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let below = pos.down();
        let below_block = GenerationCache::get_block_state(chunk, &below.0).to_block_id();
        let valid_base_id = Block::from_state_id(self.valid_base_block.id).id;
        if below_block != valid_base_id {
            return false;
        }

        let mut total_height = random.next_inbetween_i32(4, 13);
        if random.next_bounded_i32(12) == 0 {
            total_height *= 2;
        }

        // HugeFungusFeature.place uses the generator depth, not the dimension's build height.
        if !self.planted && pos.0.y + total_height + 1 >= i32::from(height) {
            return false;
        }

        let is_huge = !self.planted && random.next_f32() < 0.06;
        chunk.set_block_state(&pos.0, Block::AIR.default_state);
        self.place_stem(chunk, block_registry, random, pos, total_height, is_huge);
        self.place_hat(chunk, block_registry, random, pos, total_height, is_huge);
        true
    }

    fn is_replaceable<T: GenerationCache>(
        &self,
        chunk: &T,
        block_registry: &dyn WorldPortalExt,
        pos: BlockPos,
        check_non_replaceable_plants: bool,
    ) -> bool {
        if GenerationCache::get_block_state(chunk, &pos.0)
            .to_state()
            .replaceable()
        {
            true
        } else if check_non_replaceable_plants {
            self.replaceable_blocks.test(block_registry, chunk, &pos)
        } else {
            false
        }
    }

    fn place_stem<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
        total_height: i32,
        is_huge: bool,
    ) {
        let stem_radius = i32::from(is_huge);

        for dx in -stem_radius..=stem_radius {
            for dz in -stem_radius..=stem_radius {
                let corner_of_huge_stem =
                    is_huge && dx.abs() == stem_radius && dz.abs() == stem_radius;

                for dy in 0..total_height {
                    let block_pos = pos.add(dx, dy, dz);
                    if self.is_replaceable(chunk, block_registry, block_pos, true) {
                        if self.planted {
                            chunk.set_block_state(&block_pos.0, self.stem_state);
                        } else if corner_of_huge_stem {
                            if random.next_f32() < 0.1 {
                                chunk.set_block_state(&block_pos.0, self.stem_state);
                            }
                        } else {
                            chunk.set_block_state(&block_pos.0, self.stem_state);
                        }
                    }
                }
            }
        }
    }

    fn place_hat<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
        total_height: i32,
        is_huge: bool,
    ) {
        let place_vines = Block::from_state_id(self.hat_state.id).id == Block::NETHER_WART_BLOCK.id;
        let hat_height = (random.next_bounded_i32(1 + total_height / 3) + 5).min(total_height);
        let hat_start_y = total_height - hat_height;

        for dy in hat_start_y..=total_height {
            let rnd = random.next_bounded_i32(3);
            let mut radius = if hat_height > 8 && dy < hat_start_y + 4 {
                3
            } else if dy < total_height - rnd {
                2
            } else {
                1
            };
            if is_huge {
                radius += 1;
            }

            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let is_edge_x = dx == -radius || dx == radius;
                    let is_edge_z = dz == -radius || dz == radius;
                    let inside = !is_edge_x && !is_edge_z && dy != total_height;
                    let corner = is_edge_x && is_edge_z;
                    let is_hat_bottom = dy < hat_start_y + 3;
                    let block_pos = pos.add(dx, dy, dz);

                    if self.is_replaceable(chunk, block_registry, block_pos, false) {
                        if is_hat_bottom {
                            if !inside {
                                self.place_hat_drop_block(chunk, random, block_pos, place_vines);
                            }
                        } else if inside {
                            self.place_hat_block(
                                chunk,
                                random,
                                block_pos,
                                0.1,
                                0.2,
                                if place_vines { 0.1 } else { 0.0 },
                            );
                        } else if corner {
                            self.place_hat_block(
                                chunk,
                                random,
                                block_pos,
                                0.01,
                                0.7,
                                if place_vines { 0.083 } else { 0.0 },
                            );
                        } else {
                            self.place_hat_block(
                                chunk,
                                random,
                                block_pos,
                                5.0e-4,
                                0.98,
                                if place_vines { 0.07 } else { 0.0 },
                            );
                        }
                    }
                }
            }
        }
    }

    fn place_hat_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        block_pos: BlockPos,
        decor_prob: f32,
        hat_prob: f32,
        vines_prob: f32,
    ) {
        if random.next_f32() < decor_prob {
            chunk.set_block_state(&block_pos.0, self.decor_state);
        } else if random.next_f32() < hat_prob {
            chunk.set_block_state(&block_pos.0, self.hat_state);
            if random.next_f32() < vines_prob {
                Self::try_place_weeping_vines(chunk, random, block_pos);
            }
        }
    }

    fn place_hat_drop_block<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        block_pos: BlockPos,
        place_vines: bool,
    ) {
        let below = block_pos.down();
        let below_id = GenerationCache::get_block_state(chunk, &below.0).to_block_id();
        let hat_id = Block::from_state_id(self.hat_state.id).id;

        if below_id == hat_id {
            chunk.set_block_state(&block_pos.0, self.hat_state);
        } else if random.next_f32() < 0.15 {
            chunk.set_block_state(&block_pos.0, self.hat_state);
            if place_vines && random.next_bounded_i32(11) == 0 {
                Self::try_place_weeping_vines(chunk, random, block_pos);
            }
        }
    }

    fn try_place_weeping_vines<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        hat_block_pos: BlockPos,
    ) {
        let place_pos = hat_block_pos.down();
        if chunk.is_air(&place_pos.0) {
            let mut goal_vine_height = random.next_inbetween_i32(1, 5);
            if random.next_bounded_i32(7) == 0 {
                goal_vine_height *= 2;
            }

            WeepingVinesFeature::place_weeping_vines_column(
                chunk,
                random,
                place_pos,
                goal_vine_height,
                23,
                25,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ProtoChunk,
        chunk_system::{Cache, Chunk},
        generation::{
            feature::configured_features::{CONFIGURED_FEATURES, ConfiguredFeature},
            get_world_gen,
            proto_chunk_test::stage_parity::TestBlockRegistry,
        },
    };
    use pumpkin_data::{
        configured_feature::ConfiguredFeature as FeatureKey, dimension::Dimension,
        placed_feature::PlacedFeature,
    };
    use pumpkin_util::{random::legacy_rand::LegacyRand, world_seed::Seed};

    fn cache(pos: BlockPos) -> Cache {
        let generator = get_world_gen(
            Seed(1790201723474643596),
            Dimension::THE_NETHER,
            false,
            Vec::new(),
            String::new(),
        );
        let mut cache = Cache::new((pos.0.x >> 4) - 1, (pos.0.z >> 4) - 1, 3);
        for x in cache.x..cache.x + cache.size {
            for z in cache.z..cache.z + cache.size {
                cache
                    .chunks
                    .push(Chunk::Proto(Box::new(ProtoChunk::new(x, z, &generator))));
            }
        }
        cache
    }

    #[expect(clippy::panic, reason = "The test requires a configured huge fungus")]
    fn fungus(key: FeatureKey) -> &'static HugeFungusFeature {
        match &CONFIGURED_FEATURES[&key] {
            ConfiguredFeature::HugeFungus(feature) => feature,
            _ => panic!("Expected a huge fungus feature"),
        }
    }

    fn generate(feature: &HugeFungusFeature, cache: &mut Cache, pos: BlockPos) -> (bool, i32) {
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(0));
        let height = cache.get_center_chunk().generation_height();
        let placed = feature.generate(
            cache,
            &TestBlockRegistry,
            0,
            height,
            PlacedFeature::CrimsonFungi,
            &mut random,
            pos,
        );
        (placed, random.next_i32())
    }

    #[test]
    fn natural_fungus_respects_generation_depth_and_rejection_random_order() {
        for key in [FeatureKey::CrimsonFungus, FeatureKey::WarpedFungus] {
            let feature = fungus(key);
            // Java LegacyRandomSource seed 0 selects height 4 without doubling.
            for (y, expected) in [(122, true), (123, false)] {
                let pos = BlockPos::new(1187, y, -1160);
                let mut cache = cache(pos);
                cache.set_block_state(&pos.down().0, feature.valid_base_block);
                cache.set_block_state(&pos.0, Block::CRIMSON_FUNGUS.default_state);
                let (placed, next_random) = generate(feature, &mut cache, pos);
                assert_eq!(placed, expected);
                if !expected {
                    assert_eq!(
                        GenerationCache::get_block_state(&cache, &pos.0),
                        Block::CRIMSON_FUNGUS.default_state.id
                    );
                    assert!(cache.is_air(&pos.up().0));
                    let mut random = LegacyRand::from_seed(0);
                    random.next_bounded_i32(10);
                    random.next_bounded_i32(12);
                    assert_eq!(next_random, random.next_i32());
                }
            }
        }
    }

    #[test]
    fn planted_fungus_can_grow_above_generation_depth() {
        for key in [
            FeatureKey::CrimsonFungusPlanted,
            FeatureKey::WarpedFungusPlanted,
        ] {
            let feature = fungus(key);
            let pos = BlockPos::new(1322, 126, -2725);
            let mut cache = cache(pos);
            cache.set_block_state(&pos.down().0, feature.valid_base_block);
            assert!(generate(feature, &mut cache, pos).0);
            assert_eq!(
                GenerationCache::get_block_state(&cache, &pos.add(0, 3, 0).0),
                feature.stem_state.id
            );
        }
    }

    #[test]
    fn replaceable_vegetation_matches_air_across_chunk_boundaries() {
        // Minecraft 26.3 server output: x/z/y traversal, FNV-1a of block names,
        // followed by the next LegacyRandomSource(0) integer after placement.
        for (key, expected_hash, expected_next) in [
            (FeatureKey::CrimsonFungus, 8902037722595437051, 1781447028),
            (FeatureKey::WarpedFungus, 13021075935757590473, 768747578),
            (
                FeatureKey::CrimsonFungusPlanted,
                13687396282542717472,
                1781447028,
            ),
            (
                FeatureKey::WarpedFungusPlanted,
                18350003501919296469,
                -840115328,
            ),
        ] {
            let feature = fungus(key);
            let pos = BlockPos::new(-1, 60, -1);
            let mut air = cache(pos);
            let mut vegetation = cache(pos);
            for dx in -4..=4 {
                for dz in -4..=4 {
                    for dy in 0..=8 {
                        let target = pos.add(dx, dy, dz);
                        vegetation.set_block_state(&target.0, Block::SHORT_GRASS.default_state);
                    }
                    // Vanilla skips solid blocks individually, rather than rejecting the fungus.
                    let target = pos.add(dx, 2, dz);
                    air.set_block_state(&target.0, Block::NETHERRACK.default_state);
                    vegetation.set_block_state(&target.0, Block::NETHERRACK.default_state);
                }
            }
            for cache in [&mut air, &mut vegetation] {
                cache.set_block_state(&pos.down().0, feature.valid_base_block);
            }
            let air_result = generate(feature, &mut air, pos);
            assert_eq!(air_result, (true, expected_next));
            assert_eq!(generate(feature, &mut vegetation, pos), air_result);
            assert_eq!(
                GenerationCache::get_block_state(&air, &pos.add(0, 3, 0).0),
                feature.stem_state.id
            );
            let mut hash = 0xcbf2_9ce4_8422_2325u64;
            for dx in -4..=4 {
                for dz in -4..=4 {
                    for dy in 0..=8 {
                        let target = pos.add(dx, dy, dz);
                        let actual = GenerationCache::get_block_state(&vegetation, &target.0);
                        let actual = if actual == Block::SHORT_GRASS.default_state.id {
                            Block::AIR.default_state.id
                        } else {
                            actual
                        };
                        assert_eq!(actual, GenerationCache::get_block_state(&air, &target.0));
                        for byte in actual.to_block().name.bytes() {
                            hash ^= u64::from(byte);
                            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
                        }
                        if dy == 2 {
                            assert_eq!(actual, Block::NETHERRACK.default_state.id);
                        }
                    }
                }
            }
            assert_eq!(hash, expected_hash);
        }
    }
}
