use crate::{generation::proto_chunk::GenerationCache, world::WorldPortalExt};
use pumpkin_data::{BlockDirection, tag};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use super::CoralFeature;

pub struct CoralClawFeature;

impl CoralClawFeature {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // First lets get a random coral
        let block = CoralFeature::get_random_tag_entry(tag::Block::MINECRAFT_CORAL_BLOCKS, random);
        if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, pos) {
            return false;
        }
        let direction = BlockDirection::random_horizontal(random);
        let i = random.next_bounded_i32(2) + 2;
        // TODO: vanilla iterates the first `i` of Util.toShuffledList([direction,
        // direction.getClockWise(), direction.getCounterClockWise()], random) — the
        // shuffle consumes RNG draws and the opposite of `direction` is never visited.
        let directions = BlockDirection::horizontal_worldgen()
            .into_iter()
            .take(i as usize);
        'block0: for direction2 in directions {
            let mut pos = pos;
            let j = random.next_bounded_i32(2) + 1;
            pos = pos.offset(direction2.to_offset());

            let branch_direction;

            let k = if direction2 == direction {
                branch_direction = direction;
                random.next_bounded_i32(3) + 2
            } else {
                pos = pos.up();
                //let _directions = [direction2, BlockDirection::Up];
                branch_direction = direction2; // TODO: make this random
                random.next_bounded_i32(3) + 5
            };

            for _ in 0..j {
                if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, pos) {
                    break;
                }
                pos = pos.offset(branch_direction.to_offset());
            }

            pos = pos.offset(branch_direction.to_offset());
            pos = pos.up();

            for _l in 0..k {
                pos = pos.offset(direction.opposite().to_offset());
                if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, pos) {
                    continue 'block0;
                }
                if random.next_f32() < 0.25 {
                    pos = pos.up();
                }
            }
        }
        true
    }
}
