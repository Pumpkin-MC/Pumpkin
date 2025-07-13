use std::sync::Arc;
use crossbeam::atomic::AtomicCell;
use tokio::sync::RwLock;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use crate::entity::ai::path::path_node::PathNodeType;
use crate::entity::mob::Mob;
use crate::world::World;

pub struct PathContext {
    pub world: Arc<RwLock<Arc<World>>>,
    pub entity_pos: AtomicCell<BlockPos>,
    pub last_node_pos: AtomicCell<BlockPos>,
}

impl PathContext {
    #[must_use]
    pub fn new(world: Arc<RwLock<Arc<World>>>, mob: &dyn Mob) -> Self {
        let entity = mob.get_entity();
        Self {
            world,
            entity_pos: AtomicCell::new(entity.block_pos.load()),
            last_node_pos: AtomicCell::new(BlockPos::ZERO),
        }
    }

    pub async fn get_node_type(&mut self, location: Vector3<i32>) -> PathNodeType {
        let block_pos = BlockPos(location);
        self.last_node_pos.store(block_pos);
        let world = self.world.read().await;
        let mut node_type_cache = world.path_node_type_cache.lock().await;
        node_type_cache.add(world.clone(), block_pos)
    }
}