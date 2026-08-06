use pumpkin_data::Block;
use pumpkin_data::block_properties::is_air;
use pumpkin_data::tag;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use crate::generation::proto_chunk::GenerationCache;

pub struct HugeRedMushroomFeature;

impl HugeRedMushroomFeature {
    /// Vanilla `AbstractHugeMushroomFeature.isValidPosition`: the ground below
    /// the origin must match `huge_red_mushroom_can_place_on`, and every block
    /// the trunk/cap will occupy must currently be air or leaves, or vanilla
    /// aborts placement entirely rather than overwriting it (e.g. water).
    #[allow(clippy::unused_self)]
    fn is_valid_position<T: GenerationCache>(&self, chunk: &T, pos: BlockPos, height: i32) -> bool {
        let below = GenerationCache::get_block_state(chunk, &pos.down().0).to_block_id();
        if !below.has_tag(tag::Block::MINECRAFT_HUGE_RED_MUSHROOM_CAN_PLACE_ON) {
            return false;
        }

        let check = |dx: i32, dy: i32, dz: i32| -> bool {
            let check_pos = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
            let state_id = GenerationCache::get_block_state(chunk, &check_pos.0);
            is_air(state_id) || state_id.to_block_id().has_tag(tag::Block::MINECRAFT_LEAVES)
        };

        // Trunk column: dy in 0..height, radius 0.
        for dy in 0..height {
            if !check(0, dy, 0) {
                return false;
            }
        }

        // Cap: dy in height..=height + 2, radius 2 / 1 / 1.
        for dy in 0..=2 {
            let radius = if dy == 0 { 2 } else { 1 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if !check(dx, height + dy, dz) {
                        return false;
                    }
                }
            }
        }
        true
    }

    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let height = random.next_bounded_i32(3) + 5;

        if !self.is_valid_position(chunk, pos, height) {
            return false;
        }

        for i in 0..height {
            let stem_pos = BlockPos::new(pos.0.x, pos.0.y + i, pos.0.z);
            chunk.set_block_state(&stem_pos.0, Block::MUSHROOM_STEM.default_state);
        }

        let cap_y = pos.0.y + height;
        for dy in 0..=2 {
            let radius = if dy == 0 { 2 } else { 1 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let cap_pos = BlockPos::new(pos.0.x + dx, cap_y + dy, pos.0.z + dz);
                    chunk.set_block_state(&cap_pos.0, Block::RED_MUSHROOM_BLOCK.default_state);
                }
            }
        }
        true
    }
}
