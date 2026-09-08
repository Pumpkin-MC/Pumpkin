use pumpkin_data::{
    Block, BlockDirection, BlockState,
    block_properties::{DoubleBlockHalf, TallSeagrassLikeProperties},
    tag,
};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct SeagrassFeature {
    pub probability: f32,
}

impl SeagrassFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut placed_any = false;
        let x = random.next_bounded_i32(8) - random.next_bounded_i32(8);
        let z = random.next_bounded_i32(8) - random.next_bounded_i32(8);
        let y = chunk.ocean_floor_height_exclusive(pos.0.x + x, pos.0.z + z);
        let grass_pos = BlockPos::new(pos.0.x + x, y, pos.0.z + z);

        // Vanilla draws the tall/short coin as soon as the target is water, *before* the
        // survivability check: `nextDouble() < probability` picks tall over short seagrass, and
        // only then is the chosen state tested for survival. The tall variant additionally needs
        // water in the block above before either half is written, but the feature reports
        // success for any surviving spot even when that second water check fails.
        //
        // Skipping the `nextDouble` on the ocean floor blocks that cannot hold seagrass left
        // every later draw of the feature one behind.
        let target_is_water =
            GenerationCache::get_block_state(chunk, &grass_pos.0).to_block_id() == Block::WATER;
        let can_survive = target_is_water && Self::can_survive(chunk, grass_pos);
        if let Some(is_tall) =
            Self::choose_variant(random, self.probability, target_is_water, can_survive)
        {
            if is_tall {
                let above = grass_pos.up();
                if GenerationCache::get_block_state(chunk, &above.0).to_block_id() == Block::WATER {
                    let mut upper_props =
                        TallSeagrassLikeProperties::default(&Block::TALL_SEAGRASS);
                    upper_props.half = DoubleBlockHalf::Upper;
                    let upper_state =
                        BlockState::from_id(upper_props.to_state_id(&Block::TALL_SEAGRASS));

                    chunk.set_block_state(&grass_pos.0, Block::TALL_SEAGRASS.default_state);
                    chunk.set_block_state(&above.0, upper_state);
                }
            } else {
                chunk.set_block_state(&grass_pos.0, Block::SEAGRASS.default_state);
            }
            placed_any = true;
        }

        placed_any
    }

    /// The RNG-consuming half of vanilla's `SeagrassFeature.place`, split out so its draw
    /// order can be pinned by a test.
    ///
    /// `nextDouble` is spent whenever the target block is water, *before* `canSurvive` is
    /// consulted, and not at all when the target is not water. Returns `None` when nothing is
    /// placed, `Some(is_tall)` otherwise.
    fn choose_variant(
        random: &mut RandomGenerator,
        probability: f32,
        target_is_water: bool,
        can_survive: bool,
    ) -> Option<bool> {
        if !target_is_water {
            return None;
        }
        let is_tall = random.next_f64() < f64::from(probability);
        can_survive.then_some(is_tall)
    }

    fn can_survive<T: GenerationCache>(chunk: &T, pos: BlockPos) -> bool {
        let below = pos.down();
        let below_state = GenerationCache::get_block_state(chunk, &below.0);
        let below_id = below_state.to_block_id();

        if below_id.has_tag(tag::Block::MINECRAFT_CANNOT_SUPPORT_SEAGRASS) {
            return false;
        }

        below_state.to_state().is_side_solid(BlockDirection::Up)
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_util::random::{RandomGenerator, RandomImpl, xoroshiro128::Xoroshiro};

    use super::SeagrassFeature;

    /// Vanilla spends the `nextDouble` on every water target, even one the seagrass cannot
    /// survive on, and spends none when the target is not water: the coin sits inside the
    /// "target is water" branch and ahead of the survivability test.
    #[test]
    fn the_tall_coin_is_drawn_before_the_survivability_check() {
        // Not water: no draw at all.
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(13579));
        let mut reference = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(13579));
        assert_eq!(
            SeagrassFeature::choose_variant(&mut random, 0.3, false, false),
            None
        );
        assert_eq!(random.next_i64(), reference.next_i64());

        // Water but nothing to stand on: the coin is still drawn, only the placement is lost.
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(13579));
        let mut reference = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(13579));
        reference.next_f64();
        assert_eq!(
            SeagrassFeature::choose_variant(&mut random, 0.3, true, false),
            None
        );
        assert_eq!(random.next_i64(), reference.next_i64());

        // Water with support: the same single draw, and the coin decides tall vs short.
        let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(13579));
        let mut reference = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(13579));
        let expected = reference.next_f64() < 0.3;
        assert_eq!(
            SeagrassFeature::choose_variant(&mut random, 0.3, true, true),
            Some(expected)
        );
        assert_eq!(random.next_i64(), reference.next_i64());
    }
}
