use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{Block, BlockDirection, BlockState};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

pub struct UnderwaterMagmaFeature {
    /// vertical search limit from origin for a water column floor
    pub floor_search_range: i32,
    /// horizontal radius from the floor spot to sample
    pub placement_radius: i32,
    /// chance each inspected block turns into magma
    pub placement_probability: f32,
}

impl UnderwaterMagmaFeature {
    /// Find the Y coordinate of a solid floor beneath a water column at the origin's XZ.
    fn get_floor_y<T: GenerationCache>(&self, chunk: &T, origin: BlockPos) -> Option<i32> {
        let x = origin.0.x;
        let z = origin.0.z;

        // Drop down until water found
        for dy in 0..=self.floor_search_range {
            let check_y = origin.0.y - dy;
            let pos = BlockPos::new(x, check_y, z);
            let state_id = GenerationCache::get_block_state(chunk, &pos.0).to_block_id();

            if state_id == Block::WATER {
                // Sink to first non-water block below the column
                let mut floor_y = check_y - 1;
                loop {
                    let floor_pos = BlockPos::new(x, floor_y, z);
                    let floor_id =
                        GenerationCache::get_block_state(chunk, &floor_pos.0).to_block_id();
                    if floor_id != Block::WATER {
                        return Some(floor_y);
                    }
                    if (check_y - floor_y) > self.floor_search_range {
                        break;
                    }
                    floor_y -= 1;
                }
            }
        }
        None
    }

    /// Check if a block can host magma
    fn is_valid_placement<T: GenerationCache>(&self, chunk: &T, target: &BlockPos) -> bool {
        // Reject water/air or unsupported blocks
        let target_id = GenerationCache::get_block_state(chunk, &target.0).to_block_id();
        if target_id == Block::WATER || target_id == Block::AIR {
            return false;
        }

        // Below must be solid
        let below = target.offset(BlockDirection::Down.to_offset());
        let below_id = GenerationCache::get_block_state(chunk, &below.0).to_block_id();
        if below_id == Block::WATER || below_id == Block::AIR {
            return false;
        }

        // No open horizontal faces
        for dir in BlockDirection::horizontal().iter() {
            let neighbour = target.offset(dir.to_offset());
            let n_id = GenerationCache::get_block_state(chunk, &neighbour.0).to_block_id();
            if n_id == Block::WATER || n_id == Block::AIR {
                return false;
            }
        }

        true
    }

    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature_name: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // Locate floor; abort if none
        let floor_y = match self.get_floor_y(chunk, pos) {
            Some(y) => y,
            None => return false,
        };

        let floor_pos = BlockPos::new(pos.0.x, floor_y, pos.0.z);

        // Sample a cube around the floor, placing magma with probability and validity checks
        let mut placed = 0i32;
        let r = self.placement_radius;

        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    if random.next_f32() >= self.placement_probability {
                        continue;
                    }

                    let target_pos =
                        BlockPos::new(floor_pos.0.x + dx, floor_pos.0.y + dy, floor_pos.0.z + dz);

                    if !self.is_valid_placement(chunk, &target_pos) {
                        continue;
                    }

                    let magma_state = BlockState::from_id(Block::MAGMA_BLOCK.default_state.id);
                    chunk.set_block_state(&target_pos.0, magma_state);
                    placed += 1;
                }
            }
        }

        placed > 0
    }
}
