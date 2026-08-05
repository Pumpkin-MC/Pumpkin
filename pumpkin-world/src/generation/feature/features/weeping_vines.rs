use pumpkin_data::Block;
use pumpkin_data::BlockDirection;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use super::twisting_vines::set_growing_plant_age;
use crate::generation::proto_chunk::GenerationCache;

pub struct WeepingVinesFeature;

// net.minecraft.world.level.levelgen.feature.WeepingVinesFeature
impl WeepingVinesFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) -> bool {
        if !chunk.is_air(&origin.0) {
            return false;
        }

        let state_above = GenerationCache::get_block_state(chunk, &origin.up().0).to_state();
        if state_above != Block::NETHERRACK.default_state
            && state_above != Block::NETHER_WART_BLOCK.default_state
        {
            return false;
        }

        Self::place_roof_nether_wart(chunk, random, origin);
        Self::place_roof_weeping_vines(chunk, random, origin);
        true
    }

    // WeepingVinesFeature#placeRoofNetherWart
    fn place_roof_nether_wart<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) {
        chunk.set_block_state(&origin.0, Block::NETHER_WART_BLOCK.default_state);

        for _ in 0..200 {
            let place_pos = origin.add(
                random.next_bounded_i32(6) - random.next_bounded_i32(6),
                random.next_bounded_i32(2) - random.next_bounded_i32(5),
                random.next_bounded_i32(6) - random.next_bounded_i32(6),
            );

            if !chunk.is_air(&place_pos.0) {
                continue;
            }

            let mut neighbours = 0;
            for direction in BlockDirection::all() {
                let offset = direction.to_offset();
                let neighbour_pos = place_pos.add(offset.x, offset.y, offset.z);
                let neighbour_state =
                    GenerationCache::get_block_state(chunk, &neighbour_pos.0).to_state();
                if neighbour_state == Block::NETHERRACK.default_state
                    || neighbour_state == Block::NETHER_WART_BLOCK.default_state
                {
                    neighbours += 1;
                }
                if neighbours > 1 {
                    break;
                }
            }

            if neighbours == 1 {
                chunk.set_block_state(&place_pos.0, Block::NETHER_WART_BLOCK.default_state);
            }
        }
    }

    // WeepingVinesFeature#placeRoofWeepingVines
    fn place_roof_weeping_vines<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        origin: BlockPos,
    ) {
        for _ in 0..100 {
            let place_pos = origin.add(
                random.next_bounded_i32(8) - random.next_bounded_i32(8),
                random.next_bounded_i32(2) - random.next_bounded_i32(7),
                random.next_bounded_i32(8) - random.next_bounded_i32(8),
            );

            if !chunk.is_air(&place_pos.0) {
                continue;
            }

            let state_above = GenerationCache::get_block_state(chunk, &place_pos.up().0).to_state();
            if state_above != Block::NETHERRACK.default_state
                && state_above != Block::NETHER_WART_BLOCK.default_state
            {
                continue;
            }

            let mut vine_height = random.next_inbetween_i32(1, 8);
            if random.next_bounded_i32(6) == 0 {
                vine_height *= 2;
            }
            if random.next_bounded_i32(5) == 0 {
                vine_height = 1;
            }

            Self::place_weeping_vines_column(chunk, random, place_pos, vine_height, 17, 25);
        }
    }

    // WeepingVinesFeature#placeWeepingVinesColumn (grows downward from the ceiling)
    pub(crate) fn place_weeping_vines_column<T: GenerationCache>(
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
        total_height: i32,
        min_age: i32,
        max_age: i32,
    ) {
        let mut current_pos = pos;
        for height in 0..=total_height {
            if !chunk.is_air(&current_pos.0) {
                break;
            }

            if height == total_height || !chunk.is_air(&current_pos.down().0) {
                let age = random.next_inbetween_i32(min_age, max_age);
                let state = set_growing_plant_age(Block::WEEPING_VINES.default_state, age);
                chunk.set_block_state(&current_pos.0, state);
                break;
            }

            chunk.set_block_state(&current_pos.0, Block::WEEPING_VINES_PLANT.default_state);
            current_pos = current_pos.down();
        }
    }
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
    fn rejects_non_air_origin() {
        let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
            Seed(7),
            Dimension::THE_NETHER,
        )));
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(7));

        let pos = BlockPos::new(4, 60, 4);
        chunk.set_block_state(4, 60, 4, Block::STONE.default_state);

        let feature = WeepingVinesFeature;
        assert!(!feature.generate(
            &mut chunk,
            0,
            256,
            pumpkin_data::placed_feature::PlacedFeature::WeepingVines,
            &mut random,
            pos,
        ));
    }

    #[test]
    fn places_nether_wart_roof_below_netherrack_ceiling() {
        let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(
            Seed(7),
            Dimension::THE_NETHER,
        )));
        let mut chunk = ProtoChunk::new(0, 0, &world_gen);
        let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(7));

        let pos = BlockPos::new(4, 60, 4);
        chunk.set_block_state(4, 60, 4, Block::AIR.default_state);
        chunk.set_block_state(4, 61, 4, Block::NETHERRACK.default_state);

        let feature = WeepingVinesFeature;
        assert!(feature.generate(
            &mut chunk,
            0,
            256,
            pumpkin_data::placed_feature::PlacedFeature::WeepingVines,
            &mut random,
            pos,
        ));

        assert_eq!(
            GenerationCache::get_block_state(&chunk, &pos.0),
            Block::NETHER_WART_BLOCK.default_state.id
        );
    }
}
