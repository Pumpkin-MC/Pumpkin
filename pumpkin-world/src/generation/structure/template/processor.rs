use crate::ProtoChunk;
use pumpkin_data::BlockState;
use pumpkin_util::math::vector3::Vector3;

pub trait StructureProcessor: Send + Sync {
    fn process(
        &self,
        chunk: &mut ProtoChunk,
        pos: Vector3<i32>,
        state: &'static BlockState,
    ) -> &'static BlockState;
}

pub struct GravityProcessor {
    pub heightmap: pumpkin_util::HeightMap,
    pub offset: i32,
}

impl StructureProcessor for GravityProcessor {
    fn process(
        &self,
        chunk: &mut ProtoChunk,
        pos: Vector3<i32>,
        state: &'static BlockState,
    ) -> &'static BlockState {
        let top_y = chunk.get_top_y(&self.heightmap, pos.x, pos.z);
        let _target_y = top_y + self.offset;

        // For now, just a simple gravity: if we are placing above the ground, shift it down?
        // Actually, GravityProcessor in vanilla is used to place blocks relative to the ground.
        // But our jigsaw already adjusted the piece origin.

        let block = pumpkin_data::Block::from_state_id(state.id);
        if block.name == "cobblestone" || block.name == "stone_bricks" {
            // Fill down to ground
            for y in (top_y..pos.y).rev() {
                chunk.set_block_state(pos.x, y, pos.z, state);
            }
        }

        state
    }
}
