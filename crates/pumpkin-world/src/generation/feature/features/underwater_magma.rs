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
    /// Y of the floor of the water column the origin sits in, or `None`.
    ///
    /// Vanilla `UnderwaterMagmaFeature.getFloorY` scans the column around the origin with water
    /// as the "inside" predicate and non-water as the "edge" one, and takes the floor of the
    /// result. That scan bails out immediately when the origin itself is not water; the downward
    /// walk then steps one block at a time while the block stays inside and the search range is
    /// not exhausted, and returns the reached Y only when the block there matches the edge
    /// predicate.
    fn get_floor_y<T: GenerationCache>(&self, chunk: &T, origin: BlockPos) -> Option<i32> {
        let x = origin.0.x;
        let z = origin.0.z;
        let is_water = |chunk: &T, y: i32| {
            GenerationCache::get_block_state(chunk, &BlockPos::new(x, y, z).0).to_block_id()
                == Block::WATER
        };

        // `Column.scan` returns empty unless the origin is inside the column.
        if !is_water(chunk, origin.0.y) {
            return None;
        }

        let mut y = origin.0.y;
        let mut i = 1;
        while i < self.floor_search_range && is_water(chunk, y) {
            y -= 1;
            i += 1;
        }

        // The edge predicate is `!state.is(WATER)`.
        if is_water(chunk, y) { None } else { Some(y) }
    }

    /// Vanilla `UnderwaterMagmaFeature.isValidPlacement`.
    ///
    /// `!isWaterOrAir(state(pos)) && !isVisibleFromOutside(below, UP)` and then no horizontal
    /// neighbour may be visible from outside. `isVisibleFromOutside` asks for
    /// `getFaceOcclusionShape(dir)`, which `BlockBehaviour.BlockStateBase` fills with
    /// `FULL_BLOCK_OCCLUSION_SHAPES` exactly when the state is solid-render (`canOcclude()` and
    /// a full-block occlusion shape) and with slices otherwise, so the test collapses to
    /// `!state.isSolidRender()`.
    fn is_valid_placement<T: GenerationCache>(chunk: &T, target: &BlockPos) -> bool {
        let target_state = GenerationCache::get_block_state(chunk, &target.0);
        if target_state.to_block_id() == Block::WATER || target_state.to_state().is_air() {
            return false;
        }

        let below = target.offset(BlockDirection::Down.to_offset());
        if !GenerationCache::get_block_state(chunk, &below.0).is_solid_render() {
            return false;
        }

        for dir in &BlockDirection::horizontal() {
            let neighbour = target.offset(dir.to_offset());
            if !GenerationCache::get_block_state(chunk, &neighbour.0).is_solid_render() {
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
        _feature_name: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // Locate floor; abort if none
        let Some(floor_y) = self.get_floor_y(chunk, pos) else {
            return false;
        };

        let floor_pos = BlockPos::new(pos.0.x, floor_y, pos.0.z);

        // Sample a cube around the floor, placing magma with probability and validity checks.
        // `BlockPos.betweenClosedStream` unpacks a running index, so X varies fastest, then Y,
        // then Z, and the draw happens per visited position — the loop nesting decides which
        // position gets which float.
        let mut placed = 0i32;
        let r = self.placement_radius;

        for dz in -r..=r {
            for dy in -r..=r {
                for dx in -r..=r {
                    if random.next_f32() >= self.placement_probability {
                        continue;
                    }

                    let target_pos =
                        BlockPos::new(floor_pos.0.x + dx, floor_pos.0.y + dy, floor_pos.0.z + dz);

                    if !Self::is_valid_placement(chunk, &target_pos) {
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

#[cfg(test)]
mod tests {
    use pumpkin_data::{Block, BlockState};

    /// `UnderwaterMagmaFeature.place` walks its cube with X varying fastest, then Y, then Z, and
    /// spends one `nextFloat` per visited position — so the loop nesting has to match.
    #[test]
    fn cube_walk_matches_between_closed() {
        let r = 1i32;
        let width = 2 * r + 1;
        let height = width;

        let mut ours = Vec::new();
        for dz in -r..=r {
            for dy in -r..=r {
                for dx in -r..=r {
                    ours.push((dx, dy, dz));
                }
            }
        }

        let mut vanilla = Vec::new();
        for index in 0..(width * height * width) {
            let x = index % width;
            let y = index / width % height;
            let z = index / width / height;
            vanilla.push((-r + x, -r + y, -r + z));
        }

        assert_eq!(ours, vanilla);
        assert_eq!(ours[0], (-1, -1, -1));
        assert_eq!(ours[1], (0, -1, -1));
        assert_eq!(ours[3], (-1, 0, -1));
        assert_eq!(ours[9], (-1, -1, 0));
    }

    /// `isVisibleFromOutside` reduces to `!state.isSolidRender()`, so `cave_air` is on the
    /// visible side of it, not just `air`.
    #[test]
    fn cave_air_is_visible_from_outside() {
        assert!(!BlockState::from_id(Block::CAVE_AIR.default_state.id).is_solid_render());
        assert!(!BlockState::from_id(Block::AIR.default_state.id).is_solid_render());
        assert!(!BlockState::from_id(Block::WATER.default_state.id).is_solid_render());
        assert!(BlockState::from_id(Block::STONE.default_state.id).is_solid_render());
        assert!(BlockState::from_id(Block::DEEPSLATE.default_state.id).is_solid_render());
        assert!(BlockState::from_id(Block::MAGMA_BLOCK.default_state.id).is_solid_render());
    }
}
