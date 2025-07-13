use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use crate::entity::ai::path::path_context::PathContext;
use crate::entity::ai::path::path_node::{PathNode, PathNodeType};
use crate::entity::ai::path::target_path_node::TargetPathNode;
use crate::entity::mob::Mob;
use crate::world::World;

pub struct PathNodeMaker {
    pub context: Option<PathContext>,
    pub path_node_cache: HashMap<i32, PathNode>,
    pub entity_block_size: Vector3<i32>,
    pub can_enter_open_doors: bool,
    pub can_swim: bool,
    pub can_walk_over_fences: bool,
}

pub trait PathNodeMakerTrait {
    fn init(&mut self, world: Arc<RwLock<Arc<World>>>, mob: &dyn Mob) {
        let maker = self.get_path_node_maker_mut();
        maker.context = Some(PathContext::new(world, mob));
        maker.path_node_cache.clear();
        let entity = mob.get_entity();
        let x = (entity.width() + 1.0).floor() as i32;
        let y = (entity.height() + 1.0).floor() as i32;
        let z = (entity.width() + 1.0).floor() as i32;
        maker.entity_block_size = Vector3::new(x, y, z);
    }

    fn clear(&mut self, mob: &dyn Mob) {
        let maker = self.get_path_node_maker_mut();
        maker.context = None
    }

    fn get_path_node(&mut self, x: f64, y: f64, z: f64) -> &PathNode {
        let maker = self.get_path_node_maker_mut();
        let key = PathNode::hash(x as i32, y as i32, z as i32);
        maker.path_node_cache
            .entry(key)
            .or_insert_with_key(|_| PathNode::new(Vector3::new(x, y, z)))
    }

    fn get_default_node_type_with_mob(&self, mob: &dyn Mob, pos: BlockPos) -> PathNodeType {
        self.get_default_node_type(&PathContext::new(mob.get_entity().world.clone(), mob), pos.0.to_f64())
    }

    fn create_node(&mut self, location: Vector3<f64>) -> TargetPathNode {
        TargetPathNode::from_path_node(self.get_path_node(location.x, location.y, location.z))
    }

    fn get_start(&self, mob: &dyn Mob) -> PathNode;

    fn get_node(&self, x: f64, y: f64, z: f64) -> TargetPathNode;

    fn get_successors(&self, successors: &[PathNode], node: &PathNode) -> i32;

    fn get_node_type(&self, context: &PathContext, location: Vector3<f64>, mob: &dyn Mob) -> PathNodeType;

    fn get_default_node_type(&self, context: &PathContext, location: Vector3<f64>) -> PathNodeType;

    fn get_path_node_maker(&self) -> &PathNodeMaker;
    fn get_path_node_maker_mut(&mut self) -> &mut PathNodeMaker;
}