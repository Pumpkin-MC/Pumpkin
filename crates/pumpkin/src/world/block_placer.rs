use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::ChunkHeightmapType;
use pumpkin_world::generation::structure::template::{BlockPlacer, processor::HeightmapType};
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
    fn column_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32 {
        let cached = match heightmap {
            HeightmapType::WorldSurfaceWg | HeightmapType::WorldSurface => {
                Some(ChunkHeightmapType::WorldSurface)
            }
            HeightmapType::MotionBlocking => Some(ChunkHeightmapType::MotionBlocking),
            HeightmapType::MotionBlockingNoLeaves => {
                Some(ChunkHeightmapType::MotionBlockingNoLeaves)
            }
            HeightmapType::OceanFloorWg | HeightmapType::OceanFloor => None,
        };
        self.world
            .level
            .read_chunk_sync(&Vector2::new(x >> 4, z >> 4), |chunk| {
                if let Some(cached) = cached {
                    return chunk
                        .heightmap
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .get(cached, x, z, self.world.min_y)
                        + 1;
                }
                (self.world.min_y..self.world.min_y + self.world.dimension.height)
                    .rev()
                    .find(|&y| {
                        chunk
                            .section
                            .get_block_absolute_y((x & 15) as usize, y, (z & 15) as usize)
                            .is_some_and(|id| {
                                pumpkin_data::block_properties::blocks_movement(
                                    BlockState::from_id(id),
                                    id.to_block_id(),
                                )
                            })
                    })
                    .map_or(self.world.min_y, |y| y + 1)
            })
            .unwrap_or(self.world.min_y)
    }

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
}
