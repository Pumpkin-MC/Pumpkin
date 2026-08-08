use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::generation::structure::template::BlockPlacer;
use pumpkin_world::level::Level;
use std::collections::HashMap;

use crate::world::World;

pub struct WorldBlockPlacer<'a> {
    world: &'a World,
    pub block_entity_nbts: Vec<NbtCompound>,
    pub changed_positions: Vec<(BlockPos, BlockStateId)>,
    lighting_changes: Vec<(BlockPos, BlockStateId, BlockStateId)>,
}

impl<'a> WorldBlockPlacer<'a> {
    #[must_use]
    pub const fn new(world: &'a World) -> Self {
        Self {
            world,
            block_entity_nbts: Vec::new(),
            changed_positions: Vec::new(),
            lighting_changes: Vec::new(),
        }
    }

    #[allow(clippy::unused_async)]
    pub fn finalize(&self) {
        // BlockPlacer writes the live chunk directly so a structure can be applied
        // as one batch. Vanilla still runs the light engine after those block-state
        // changes; update each final position once, in a stable order, before the
        // caller flushes the queued block and light packets.
        let mut changes = HashMap::new();
        for &(position, old_state, new_state) in &self.lighting_changes {
            changes
                .entry(position)
                .and_modify(|change: &mut (BlockStateId, BlockStateId)| change.1 = new_state)
                .or_insert((old_state, new_state));
        }
        let mut lighting_positions: Vec<_> = changes
            .into_iter()
            .map(|(position, (old_state, new_state))| (position, old_state, new_state))
            .collect();
        lighting_positions
            .sort_by_key(|(position, _, _)| (position.0.x, position.0.y, position.0.z));
        self.world
            .level
            .light_engine
            .update_lighting_batch_with_states(&self.world.level, &lighting_positions);

        for nbt in &self.block_entity_nbts {
            if let Some(block_entity) = crate::block::entities::block_entity_from_nbt(nbt) {
                self.world.add_block_entity(block_entity);
            }
        }
    }
}

impl BlockPlacer for WorldBlockPlacer<'_> {
    fn get_block_state(&self, pos: &Vector3<i32>) -> BlockStateId {
        self.world
            .get_block_state_id(&BlockPos::new(pos.x, pos.y, pos.z))
    }

    fn set_block_state(&mut self, pos: &Vector3<i32>, state: &BlockState) {
        let block_pos = BlockPos::new(pos.x, pos.y, pos.z);
        let old_state = Level::set_block_state(&self.world.level, &block_pos, state.id);
        self.changed_positions.push((block_pos, state.id));
        if old_state != state.id {
            self.lighting_changes.push((block_pos, old_state, state.id));
        }
    }

    fn add_block_entity(&mut self, nbt: NbtCompound) {
        self.block_entity_nbts.push(nbt);
    }
}
