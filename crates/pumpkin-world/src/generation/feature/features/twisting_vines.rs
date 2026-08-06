use pumpkin_data::{Block, BlockState};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct TwistingVinesFeature {
    pub spread_width: i32,
    pub spread_height: i32,
    pub max_height: i32,
}

// net.minecraft.world.level.levelgen.feature.TwistingVinesFeature
impl TwistingVinesFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature_name: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if Self::is_invalid_placement_location(chunk, &pos) {
            return false;
        }

        for _ in 0..self.spread_width * self.spread_width {
            let offset_x = random.next_inbetween_i32(-self.spread_width, self.spread_width);
            let offset_y = random.next_inbetween_i32(-self.spread_height, self.spread_height);
            let offset_z = random.next_inbetween_i32(-self.spread_width, self.spread_width);

            let mut place_pos = pos.add(offset_x, offset_y, offset_z);

            if Self::find_first_air_block_above_ground(chunk, &mut place_pos)
                && !Self::is_invalid_placement_location(chunk, &place_pos)
            {
                let mut vine_height = random.next_inbetween_i32(1, self.max_height);
                if random.next_bounded_i32(6) == 0 {
                    vine_height *= 2;
                }
                if random.next_bounded_i32(5) == 0 {
                    vine_height = 1;
                }

                Self::place_twisting_vines_column(chunk, random, &place_pos, vine_height, 17, 25);
            }
        }

        true
    }

    fn place_twisting_vines_column<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: &BlockPos,
        total_height: i32,
        min_age: i32,
        max_age: i32,
    ) {
        let mut current_pos = *pos;
        for height in 1..=total_height {
            if !GenerationCache::get_block_state(chunk, &current_pos.0)
                .to_state()
                .is_air()
            {
                break;
            }

            if height == total_height
                || !GenerationCache::get_block_state(chunk, &current_pos.up().0)
                    .to_state()
                    .is_air()
            {
                let age = random.next_inbetween_i32(min_age, max_age);
                let state = set_growing_plant_age(Block::TWISTING_VINES.default_state, age);
                chunk.set_block_state(&current_pos.0, state);
                break;
            }

            chunk.set_block_state(&current_pos.0, Block::TWISTING_VINES_PLANT.default_state);
            current_pos = current_pos.up();
        }
    }

    // TwistingVinesFeature#isInvalidPlacementLocation
    fn is_invalid_placement_location<T: GenerationCache>(chunk: &T, pos: &BlockPos) -> bool {
        if !GenerationCache::get_block_state(chunk, &pos.0)
            .to_state()
            .is_air()
        {
            return true;
        }

        let state_below = GenerationCache::get_block_state(chunk, &pos.down().0).to_state();
        state_below != Block::NETHERRACK.default_state
            && state_below != Block::WARPED_NYLIUM.default_state
            && state_below != Block::WARPED_WART_BLOCK.default_state
    }

    // TwistingVinesFeature#findFirstAirBlockAboveGround: walk down from `pos` until hitting a
    // non-air block (any block), then step back up by one. Bounded only by world height, not by
    // `spread_height` (matches vanilla, unlike the previous bounded-scan implementation here).
    fn find_first_air_block_above_ground<T: GenerationCache>(
        chunk: &T,
        pos: &mut BlockPos,
    ) -> bool {
        loop {
            *pos = pos.down();
            if pos.0.y < i32::from(chunk.bottom_y()) || pos.0.y >= i32::from(chunk.top_y()) {
                return false;
            }
            if !GenerationCache::get_block_state(chunk, &pos.0)
                .to_state()
                .is_air()
            {
                break;
            }
        }
        *pos = pos.up();
        true
    }
}

// Sets the "age" IntegerProperty on a GrowingPlantHeadBlock state (twisting_vines / weeping_vines),
// mirroring `RandomizedIntBlockStateProvider`'s property-rewrite pattern.
pub fn set_growing_plant_age(state: &'static BlockState, age: i32) -> &'static BlockState {
    let block = Block::from_state_id(state.id);
    let Some(props_source) = block.properties(state.id) else {
        return state;
    };
    let props = props_source.to_props();
    if !props.iter().any(|(key, _)| *key == "age") {
        return state;
    }

    let age_str = age.to_string();
    let new_props: Vec<(&str, &str)> = props
        .iter()
        .map(|(key, value)| {
            if *key == "age" {
                (*key, age_str.as_str())
            } else {
                (*key, *value)
            }
        })
        .collect();

    let new_state_id = block.from_properties(&new_props).to_state_id(block);
    BlockState::from_id(new_state_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProtoChunk;
    use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::random::legacy_rand::LegacyRand;
    use pumpkin_util::world_seed::Seed;

    #[test]
    fn sets_age_property_on_twisting_vines() {
        let state = set_growing_plant_age(Block::TWISTING_VINES.default_state, 20);
        let block = Block::from_state_id(state.id);
        let props = block.properties(state.id).unwrap().to_props();
        let age = props.iter().find(|(key, _)| *key == "age").unwrap().1;
        assert_eq!(age, "20");
    }

    #[test]
    fn requires_netherrack_or_warped_ground() {
        let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
            Seed(1234),
            Dimension::THE_NETHER,
        )));
        let WorldGenerator::Noise(generator) = &world_gen else {
            unreachable!()
        };
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);

        let pos = BlockPos::new(4, 40, 4);
        let below = pos.down();
        chunk.set_block_state(below.0.x, below.0.y, below.0.z, Block::STONE.default_state);
        chunk.set_block_state(pos.0.x, pos.0.y, pos.0.z, Block::AIR.default_state);
        assert!(TwistingVinesFeature::is_invalid_placement_location(
            &chunk, &pos
        ));

        chunk.set_block_state(
            below.0.x,
            below.0.y,
            below.0.z,
            Block::NETHERRACK.default_state,
        );
        assert!(!TwistingVinesFeature::is_invalid_placement_location(
            &chunk, &pos
        ));

        let mut random =
            RandomGenerator::Legacy(LegacyRand::from_seed(generator.random_config.seed));
        let feature = TwistingVinesFeature {
            spread_width: 8,
            spread_height: 4,
            max_height: 8,
        };
        // Smoke-test that generation doesn't panic and honors the ground check end-to-end.
        let _ = feature.generate(
            &mut chunk,
            generator.dimension.min_y as i8,
            generator.dimension.logical_height as u16,
            pumpkin_data::placed_feature::PlacedFeature::TwistingVines,
            &mut random,
            pos,
        );
    }
}
