use crate::{generation::proto_chunk::GenerationCache, world::WorldPortalExt};
use pumpkin_data::{BlockDirection, HorizontalFacingExt, tag};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use super::CoralFeature;

/// `Util.shuffle`: Fisher-Yates from the end, `random.nextInt(i)` per swap. Must match this
/// exact call order (not e.g. shuffling from the front) to stay seed-reproducible with vanilla.
fn shuffle<T>(slice: &mut [T], random: &mut RandomGenerator) {
    let mut i = slice.len();
    while i > 1 {
        let swap_to = random.next_bounded_i32(i as i32) as usize;
        slice.swap(i - 1, swap_to);
        i -= 1;
    }
}

pub struct CoralClawFeature;

impl CoralClawFeature {
    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // First lets get a random coral
        let block = CoralFeature::get_random_tag_entry(tag::Block::MINECRAFT_CORAL_BLOCKS, random);
        if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, pos) {
            return false;
        }
        // CoralClawFeature.placeFeature: claw_direction = Direction.Plane.HORIZONTAL
        // .getRandomDirection(random) -- a uniform pick over all 4 horizontal directions.
        let claw_direction = BlockDirection::random_horizontal(random).to_block_direction();
        let branch_count = random.next_bounded_i32(2) + 2;

        // Util.toShuffledList(Stream.of(clawDirection, clockWise, counterClockWise), random),
        // then take the first `branch_count` entries -- NOT an arbitrary subset of all 4
        // horizontal directions in enum order.
        let mut possible_directions = [
            claw_direction,
            claw_direction.rotate_clockwise(),
            claw_direction.rotate_counter_clockwise(),
        ];
        shuffle(&mut possible_directions, random);

        'branches: for &branch_direction in &possible_directions[..branch_count as usize] {
            let mut pos = pos;
            let sideway_length = random.next_bounded_i32(2) + 1;
            pos = pos.offset(branch_direction.to_offset());

            let segment_direction;
            let inway_length = if branch_direction == claw_direction {
                segment_direction = claw_direction;
                random.next_bounded_i32(3) + 2
            } else {
                pos = pos.up();
                // Util.getRandom([branchDirection, Direction.UP], random): a uniform pick
                // between continuing sideways or heading straight up.
                segment_direction = if random.next_bounded_i32(2) == 0 {
                    branch_direction
                } else {
                    BlockDirection::Up
                };
                random.next_bounded_i32(3) + 3
            };

            for _ in 0..sideway_length {
                if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, pos) {
                    break;
                }
                pos = pos.offset(segment_direction.to_offset());
            }

            pos = pos.offset(segment_direction.opposite().to_offset());
            pos = pos.up();

            for _ in 0..inway_length {
                pos = pos.offset(claw_direction.to_offset());
                if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, pos) {
                    continue 'branches;
                }
                if random.next_f32() < 0.25 {
                    pos = pos.up();
                }
            }
        }
        true
    }
}
