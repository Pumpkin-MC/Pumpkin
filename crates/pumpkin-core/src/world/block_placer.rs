use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::HeightMap;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use pumpkin_world::generation::structure::template::BlockPlacer;
use pumpkin_world::level::Level;

use crate::world::World;

impl World {
    pub fn clear_synced_block_events_in_box(&self, min: &BlockPos, max: &BlockPos) {
        let mut events = self
            .synced_block_event_queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        events.retain(|event| {
            let pos = event.pos;
            pos.0.x < min.0.x
                || pos.0.x >= max.0.x
                || pos.0.y < min.0.y
                || pos.0.y >= max.0.y
                || pos.0.z < min.0.z
                || pos.0.z >= max.0.z
        });
    }
}

pub struct WorldBlockPlacer<'a> {
    world: &'a World,
    pub block_entity_nbts: Vec<NbtCompound>,
    pub changed_positions: Vec<(BlockPos, BlockStateId)>,
}

impl<'a> WorldBlockPlacer<'a> {
    #[must_use]
    pub const fn new(world: &'a World) -> Self {
        Self {
            world,
            block_entity_nbts: Vec::new(),
            changed_positions: Vec::new(),
        }
    }

    #[allow(clippy::unused_async)]
    pub fn finalize(&self) {
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
        Level::set_block_state(&self.world.level, &block_pos, state.id);
        self.changed_positions.push((block_pos, state.id));
    }

    fn add_block_entity(&mut self, nbt: NbtCompound) {
        self.block_entity_nbts.push(nbt);
    }

    fn get_height(&self, heightmap: HeightMap, x: i32, z: i32) -> Option<i32> {
        // Like `GravityProcessor` on a `ServerLevel`, the worldgen heightmaps read their live
        // counterparts. Loaded chunks keep no ocean floor heightmap.
        let heightmap = match heightmap {
            HeightMap::WorldSurfaceWg | HeightMap::WorldSurface => ChunkHeightmapType::WorldSurface,
            HeightMap::MotionBlocking => ChunkHeightmapType::MotionBlocking,
            HeightMap::MotionBlockingNoLeaves => ChunkHeightmapType::MotionBlockingNoLeaves,
            HeightMap::OceanFloorWg | HeightMap::OceanFloor => return None,
        };
        Some(self.world.get_heightmap_height(heightmap, x, z) + 1)
    }
}
