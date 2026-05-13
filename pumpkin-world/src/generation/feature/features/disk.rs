use pumpkin_util::math::{int_provider::IntProvider, position::BlockPos};
use pumpkin_util::random::RandomGenerator;

use crate::generation::{
    block_predicate::BlockPredicate, block_state_provider::BlockStateProvider,
    proto_chunk::GenerationCache,
};
use crate::world::WorldPortalExt;

pub struct DiskFeature {
    pub state_provider: BlockStateProvider,
    pub target: BlockPredicate,
    pub radius: IntProvider,
    pub half_height: i32,
}

impl DiskFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut placed_any = false;
        let origin_y = pos.0.y;
        let top = origin_y + self.half_height;
        let bottom = origin_y - self.half_height - 1;
        let r = self.radius.get(random);

        for column_pos in BlockPos::iterate(pos.add(-r, 0, -r), pos.add(r, 0, r)) {
            let xd = column_pos.0.x - pos.0.x;
            let zd = column_pos.0.z - pos.0.z;
            if xd * xd + zd * zd <= r * r {
                placed_any |= self.place_column(
                    chunk,
                    block_registry,
                    random,
                    top,
                    bottom,
                    column_pos.0.x,
                    column_pos.0.z,
                );
            }
        }

        placed_any
    }

    #[expect(clippy::too_many_arguments)]
    fn place_column<T: GenerationCache>(
        &self,
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        random: &mut RandomGenerator,
        top: i32,
        bottom: i32,
        column_x: i32,
        column_z: i32,
    ) -> bool {
        let mut placed_any = false;

        let mut y = top;
        while y > bottom {
            let cur_pos = BlockPos::new(column_x, y, column_z);
            if self.target.test(block_registry, chunk, &cur_pos) {
                let state =
                    self.state_provider
                        .get_with_context(block_registry, chunk, random, cur_pos);
                chunk.set_block_state(&cur_pos.0, state);
                placed_any = true;
            }
            y -= 1;
        }

        placed_any
    }
}
