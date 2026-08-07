use pumpkin_data::{Block, BlockState};
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use super::weeping_vines::WeepingVinesFeature;
use crate::generation::proto_chunk::GenerationCache;

/// `HugeFungusConfiguration.replaceable_blocks`, read directly from the 26.2
/// datapack (`data/minecraft/worldgen/configured_feature/crimson_fungus.json`
/// and `warped_fungus.json`, identical `matching_blocks` list in both).
const REPLACEABLE_BLOCKS: &[pumpkin_data::BlockId] = &[
    Block::OAK_SAPLING.id,
    Block::SPRUCE_SAPLING.id,
    Block::BIRCH_SAPLING.id,
    Block::JUNGLE_SAPLING.id,
    Block::ACACIA_SAPLING.id,
    Block::CHERRY_SAPLING.id,
    Block::DARK_OAK_SAPLING.id,
    Block::PALE_OAK_SAPLING.id,
    Block::MANGROVE_PROPAGULE.id,
    Block::DANDELION.id,
    Block::TORCHFLOWER.id,
    Block::POPPY.id,
    Block::BLUE_ORCHID.id,
    Block::ALLIUM.id,
    Block::AZURE_BLUET.id,
    Block::RED_TULIP.id,
    Block::ORANGE_TULIP.id,
    Block::WHITE_TULIP.id,
    Block::PINK_TULIP.id,
    Block::OXEYE_DAISY.id,
    Block::CORNFLOWER.id,
    Block::WITHER_ROSE.id,
    Block::LILY_OF_THE_VALLEY.id,
    Block::BROWN_MUSHROOM.id,
    Block::RED_MUSHROOM.id,
    Block::WHEAT.id,
    Block::SUGAR_CANE.id,
    Block::ATTACHED_PUMPKIN_STEM.id,
    Block::ATTACHED_MELON_STEM.id,
    Block::PUMPKIN_STEM.id,
    Block::MELON_STEM.id,
    Block::LILY_PAD.id,
    Block::NETHER_WART.id,
    Block::COCOA.id,
    Block::CARROTS.id,
    Block::POTATOES.id,
    Block::CHORUS_PLANT.id,
    Block::CHORUS_FLOWER.id,
    Block::TORCHFLOWER_CROP.id,
    Block::PITCHER_CROP.id,
    Block::BEETROOTS.id,
    Block::SWEET_BERRY_BUSH.id,
    Block::WARPED_FUNGUS.id,
    Block::CRIMSON_FUNGUS.id,
    Block::WEEPING_VINES.id,
    Block::WEEPING_VINES_PLANT.id,
    Block::TWISTING_VINES.id,
    Block::TWISTING_VINES_PLANT.id,
    Block::CAVE_VINES.id,
    Block::CAVE_VINES_PLANT.id,
    Block::SPORE_BLOSSOM.id,
    Block::AZALEA.id,
    Block::FLOWERING_AZALEA.id,
    Block::MOSS_CARPET.id,
    Block::PINK_PETALS.id,
    Block::WILDFLOWERS.id,
    Block::BIG_DRIPLEAF.id,
    Block::BIG_DRIPLEAF_STEM.id,
    Block::SMALL_DRIPLEAF.id,
];

/// The `HugeFungusConfiguration` values the configured feature table drops on
/// the floor, read from the 26.2 `crimson_fungus.json` / `warped_fungus.json`
/// configured features above.
struct FungusConfig {
    valid_base: pumpkin_data::BlockId,
    stem: &'static BlockState,
    hat: &'static BlockState,
    decor: &'static BlockState,
    /// Always false for the worldgen configured features; the `_planted`
    /// variants are only reachable through bone meal, which does not route
    /// through a `PlacedFeature` in this codebase.
    planted: bool,
}

impl FungusConfig {
    fn crimson() -> Self {
        Self {
            valid_base: Block::CRIMSON_NYLIUM.id,
            stem: Block::CRIMSON_STEM.default_state,
            hat: Block::NETHER_WART_BLOCK.default_state,
            decor: Block::SHROOMLIGHT.default_state,
            planted: false,
        }
    }

    fn warped() -> Self {
        Self {
            valid_base: Block::WARPED_NYLIUM.id,
            stem: Block::WARPED_STEM.default_state,
            hat: Block::WARPED_WART_BLOCK.default_state,
            decor: Block::SHROOMLIGHT.default_state,
            planted: false,
        }
    }
}

pub struct HugeFungusFeature;

impl HugeFungusFeature {
    fn is_replaceable<T: GenerationCache>(
        chunk: &T,
        pos: &BlockPos,
        use_config_predicate: bool,
    ) -> bool {
        let state_id = GenerationCache::get_block_state(chunk, &pos.0);
        if state_id.to_state().replaceable() {
            return true;
        }
        use_config_predicate && REPLACEABLE_BLOCKS.contains(&state_id.to_block_id())
    }

    /// `HugeFungusFeature.placeStem`
    fn place_stem<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        config: &FungusConfig,
        origin: BlockPos,
        stem_height: i32,
        huge: bool,
    ) {
        let radius = i32::from(huge);

        for j in -radius..=radius {
            for k in -radius..=radius {
                let corner = huge && j.abs() == radius && k.abs() == radius;

                for l in 0..stem_height {
                    let pos = BlockPos::new(origin.0.x + j, origin.0.y + l, origin.0.z + k);
                    if !Self::is_replaceable(chunk, &pos, true) {
                        continue;
                    }
                    if config.planted {
                        chunk.set_block_state(&pos.0, config.stem);
                    } else if corner {
                        if random.next_f32() < 0.1 {
                            chunk.set_block_state(&pos.0, config.stem);
                        }
                    } else {
                        chunk.set_block_state(&pos.0, config.stem);
                    }
                }
            }
        }
    }

    /// `HugeFungusFeature.placeHat`
    fn place_hat<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        config: &FungusConfig,
        origin: BlockPos,
        stem_height: i32,
        huge: bool,
    ) {
        let nether_wart_hat = config.hat.id.to_block_id() == Block::NETHER_WART_BLOCK.id;
        let hat_height = (random.next_bounded_i32(1 + stem_height / 3) + 5).min(stem_height);
        let hat_bottom = stem_height - hat_height;

        for k in hat_bottom..=stem_height {
            // Vanilla draws this taper roll unconditionally, before the `hat_height > 8`
            // override can discard it; keep it out front so the RNG stream matches.
            let taper = random.next_bounded_i32(3);
            let mut radius = if hat_height > 8 && k < hat_bottom + 4 {
                3
            } else if k < stem_height - taper {
                2
            } else {
                1
            };
            if huge {
                radius += 1;
            }

            for i1 in -radius..=radius {
                for j1 in -radius..=radius {
                    let on_x_edge = i1 == -radius || i1 == radius;
                    let on_z_edge = j1 == -radius || j1 == radius;
                    let interior = !on_x_edge && !on_z_edge && k != stem_height;
                    let corner = on_x_edge && on_z_edge;
                    let skirt = k < hat_bottom + 3;
                    let pos = BlockPos::new(origin.0.x + i1, origin.0.y + k, origin.0.z + j1);
                    if !Self::is_replaceable(chunk, &pos, false) {
                        continue;
                    }

                    if skirt {
                        if !interior {
                            Self::place_hat_drop_block(
                                chunk,
                                random,
                                pos,
                                config.hat,
                                nether_wart_hat,
                            );
                        }
                    } else if interior {
                        Self::place_hat_block(
                            chunk,
                            random,
                            config,
                            pos,
                            0.1,
                            0.2,
                            if nether_wart_hat { 0.1 } else { 0.0 },
                        );
                    } else if corner {
                        Self::place_hat_block(
                            chunk,
                            random,
                            config,
                            pos,
                            0.01,
                            0.7,
                            if nether_wart_hat { 0.083 } else { 0.0 },
                        );
                    } else {
                        Self::place_hat_block(
                            chunk,
                            random,
                            config,
                            pos,
                            5.0e-4,
                            0.98,
                            if nether_wart_hat { 0.07 } else { 0.0 },
                        );
                    }
                }
            }
        }
    }

    /// `HugeFungusFeature.placeHatBlock`
    fn place_hat_block<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        config: &FungusConfig,
        pos: BlockPos,
        decor_chance: f32,
        hat_chance: f32,
        vine_chance: f32,
    ) {
        if random.next_f32() < decor_chance {
            chunk.set_block_state(&pos.0, config.decor);
        } else if random.next_f32() < hat_chance {
            chunk.set_block_state(&pos.0, config.hat);
            if random.next_f32() < vine_chance {
                Self::try_place_weeping_vines(chunk, random, pos);
            }
        }
    }

    /// `HugeFungusFeature.placeHatDropBlock`
    fn place_hat_drop_block<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
        hat: &'static BlockState,
        nether_wart_hat: bool,
    ) {
        let below = GenerationCache::get_block_state(chunk, &pos.down().0).to_block_id();
        if below == hat.id.to_block_id() {
            chunk.set_block_state(&pos.0, hat);
        } else if random.next_f32() < 0.15 {
            chunk.set_block_state(&pos.0, hat);
            if nether_wart_hat && random.next_bounded_i32(11) == 0 {
                Self::try_place_weeping_vines(chunk, random, pos);
            }
        }
    }

    /// `HugeFungusFeature.tryPlaceWeepingVines`
    fn try_place_weeping_vines<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) {
        let below = pos.down();
        if !chunk.is_air(&below.0) {
            return;
        }
        let mut vine_height = random.next_inbetween_i32(1, 5);
        if random.next_bounded_i32(7) == 0 {
            vine_height *= 2;
        }
        WeepingVinesFeature::place_weeping_vines_column(chunk, random, below, vine_height, 23, 25);
    }

    /// `HugeFungusFeature.place`
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        height: u16,
        feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // The generated configured-feature table constructs this feature with
        // no configuration at all, so the crimson/warped variant is recovered
        // from the placed feature that referenced it.
        let config = match feature {
            pumpkin_data::placed_feature::PlacedFeature::WarpedFungi => FungusConfig::warped(),
            _ => FungusConfig::crimson(),
        };

        let below = GenerationCache::get_block_state(chunk, &pos.down().0).to_block_id();
        if below != config.valid_base {
            return false;
        }

        let mut stem_height = random.next_inbetween_i32(4, 13);
        if random.next_bounded_i32(12) == 0 {
            stem_height *= 2;
        }

        if !config.planted && pos.0.y + stem_height + 1 >= i32::from(height) {
            return false;
        }

        let huge = !config.planted && random.next_f32() < 0.06;
        chunk.set_block_state(&pos.0, Block::AIR.default_state);
        Self::place_stem(chunk, random, &config, pos, stem_height, huge);
        Self::place_hat(chunk, random, &config, pos, stem_height, huge);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProtoChunk;
    use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
    use pumpkin_data::dimension::Dimension;
    use pumpkin_data::placed_feature::PlacedFeature;
    use pumpkin_util::random::legacy_rand::LegacyRand;
    use pumpkin_util::world_seed::Seed;

    fn run(
        feature: PlacedFeature,
        base: &'static BlockState,
    ) -> (bool, Vec<pumpkin_data::BlockId>) {
        let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
            Seed(7),
            Dimension::THE_NETHER,
        )));
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(7));

        let pos = BlockPos::new(8, 40, 8);
        chunk.set_block_state(8, 39, 8, base);

        let placed = HugeFungusFeature.generate(&mut chunk, 0, 128, feature, &mut random, pos);

        let mut ids = Vec::new();
        for x in 0..16 {
            for z in 0..16 {
                for y in 40..80 {
                    let id = chunk
                        .get_block_state(&pumpkin_util::math::vector3::Vector3::new(x, y, z))
                        .to_block_id();
                    if id != Block::AIR.id {
                        ids.push(id);
                    }
                }
            }
        }
        (placed, ids)
    }

    /// `HugeFungusFeature.place` bails unless the block below the origin is the
    /// configured `validBaseState` (Mojang-named 1.21.4 decompile). Without this
    /// gate every `count_on_every_layer(8)` candidate places.
    #[test]
    fn rejects_origin_that_is_not_on_the_configured_nylium() {
        let (placed, ids) = run(PlacedFeature::CrimsonFungi, Block::NETHERRACK.default_state);
        assert!(!placed, "a fungus must not generate off nylium");
        assert!(ids.is_empty(), "a rejected fungus must place no blocks");

        let (placed, _) = run(
            PlacedFeature::CrimsonFungi,
            Block::WARPED_NYLIUM.default_state,
        );
        assert!(
            !placed,
            "the crimson fungus must not accept warped nylium as its base"
        );
    }

    /// The generated configured-feature table drops the `HugeFungusConfiguration`,
    /// so the crimson/warped block set is recovered from the placed feature. Guard
    /// both directions: the variant must follow the feature, not the RNG.
    #[test]
    fn block_set_follows_the_placed_feature_variant() {
        let (placed, ids) = run(
            PlacedFeature::CrimsonFungi,
            Block::CRIMSON_NYLIUM.default_state,
        );
        assert!(placed);
        assert!(ids.contains(&Block::CRIMSON_STEM.id));
        assert!(ids.contains(&Block::NETHER_WART_BLOCK.id));
        assert!(!ids.contains(&Block::WARPED_STEM.id));
        assert!(!ids.contains(&Block::WARPED_WART_BLOCK.id));

        let (placed, ids) = run(
            PlacedFeature::WarpedFungi,
            Block::WARPED_NYLIUM.default_state,
        );
        assert!(placed);
        assert!(ids.contains(&Block::WARPED_STEM.id));
        assert!(ids.contains(&Block::WARPED_WART_BLOCK.id));
        assert!(!ids.contains(&Block::CRIMSON_STEM.id));
        assert!(!ids.contains(&Block::NETHER_WART_BLOCK.id));
    }
}
