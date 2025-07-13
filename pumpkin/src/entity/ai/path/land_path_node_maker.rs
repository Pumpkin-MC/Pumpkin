use std::collections::HashMap;
use std::ops::Mul;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use pumpkin_util::math::boundingbox::BoundingBox;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use crate::entity::ai::path::path_context::PathContext;
use crate::entity::ai::path::path_node::{PathNode, PathNodeType};
use crate::entity::ai::path::path_node_maker::{PathNodeMaker, PathNodeMakerTrait};
use crate::entity::ai::path::target_path_node::TargetPathNode;
use crate::entity::mob::Mob;
use crate::world::World;

const Y_OFFSET: f64 = 0.5;
const MIN_STEP_HEIGHT: f64 = 1.125;

pub struct LandPathNodeMaker {
    pub path_node_maker: PathNodeMaker,
    pub node_types: HashMap<i64, PathNodeType>,
    pub collided_boxes: Mutex<HashMap<BoundingBox, bool>>,
    pub successors: [PathNode; 4],
}

impl LandPathNodeMaker {

    pub fn get_common_node_type(world: Arc<World>, pos: BlockPos) -> PathNodeType {

    }

    fn is_valid_adjacent_successor(node: Option<&PathNode>, successor: &PathNode) -> bool {
        if let Some(node) = node {
            !node.visited && (node.penality >= 0.0 || successor.penality < 0.0)
        } else {
            false
        }
    }

    fn is_valid_diagonal_successor(mob: &dyn Mob, x_node: &PathNode, z_node: Option<&PathNode>, x_diag_node: Option<&PathNode>) -> bool {
        let Some(z_node) = z_node else {
            return false;
        };
        let Some(x_diag_node) = x_diag_node else {
            return false;
        };
        if x_diag_node.location.y > x_node.location.y || z_node.location.y > x_node.location.y {
            return false;
        }
        if z_node.node_type != PathNodeType::WalkableDoor && x_diag_node.node_type != PathNodeType::WalkableDoor {
            let bl = x_diag_node.node_type == PathNodeType::Fence && z_node.node_type == PathNodeType::Fence && mob.get_entity().width() < 0.5;
            return (x_diag_node.location.y < x_node.location.y || x_diag_node.penality >= 0.0 || bl) && (z_node.location.y < x_node.location.y || z_node.penality >= 0.0 || bl);
        }

        false
    }

    fn is_valid_diagonal_successor_path(node: Option<&PathNode>) -> bool {
        if let Some(node) = node {
            !node.visited && if node.node_type == PathNodeType::WalkableDoor {
                false
            } else {
                node.penality >= 0.0
            }
        } else {
            false
        }
    }

    fn is_blocked(node_type: PathNodeType) -> bool {
        match node_type {
            PathNodeType::Fence |
            PathNodeType::DoorWoodClosed |
            PathNodeType::DoorIronClosed => true,
            _ => false
        }
    }

    async fn is_blocked_path(&self, mob: &dyn Mob, node: &PathNode) -> bool {
        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let mut bounding_box = entity.bounding_box.load();
        let mut vec3 = Vector3::new(
            node.location.x - pos.x + bounding_box.get_length_x() / 2.0,
            node.location.y - pos.y + bounding_box.get_length_y() / 2.0,
            node.location.z - pos.z + bounding_box.get_length_z() / 2.0,
        );
        let i = (vec3.length() / bounding_box.get_average_side_length()).ceil() as i32;
        vec3 = vec3.mul(1.0 / i as f64);

        for j in 1..=i {
            bounding_box = bounding_box.offset_vec(&vec3);
            if self.check_box_collision(mob, bounding_box).await {
                return false;
            }
        }

        true
    }

    async fn check_box_collision(&self, mob: &dyn Mob, bounding_box: BoundingBox) -> bool {
        let Some(context) =  &self.path_node_maker.context else {
            return false;
        };
        let world = context.world.read().await;
        let mut collided_boxes = self.collided_boxes.lock().await;
        let result = collided_boxes
            .entry(bounding_box)
            .or_insert_with_key(|_| !world.is_space_empty(mob, bounding_box));
        result.clone()
    }
}

impl PathNodeMakerTrait for LandPathNodeMaker {
    fn init(&mut self, world: Arc<RwLock<Arc<World>>>, mob: &dyn Mob) {
        PathNodeMakerTrait::init(self, world, mob);
        mob.on_start_pathfinding();
    }

    fn clear(&mut self, mob: &dyn Mob) {
        mob.on_finish_pathfinding();
        self.node_types.clear();
        self.collided_boxes.clear();
        PathNodeMakerTrait::clear(self, mob);
    }

    fn get_start(&self, mob: &dyn Mob) -> PathNode {
        let block_pos = BlockPos::ZERO;
        let entity = mob.get_entity();
        let y = entity.block_pos.load().0.y;
        let block_state = self.path_node_maker.context.unwrap()
    }

    fn get_node(&mut self, x: f64, y: f64, z: f64) -> TargetPathNode {
        self.create_node(Vector3::new(x, y, z))
    }

    fn get_successors(&self, successors: &[PathNode], node: &PathNode) -> i32 {
        todo!()
    }

    fn get_node_type(&self, context: &PathContext, location: Vector3<f64>, mob: &dyn Mob) -> PathNodeType {
        todo!()
    }

    fn get_default_node_type(&self, context: &PathContext, location: Vector3<f64>) -> PathNodeType {
        todo!()
    }

    fn get_path_node_maker(&self) -> &PathNodeMaker {
        &self.path_node_maker
    }

    fn get_path_node_maker_mut(&mut self) -> &mut PathNodeMaker {
        &mut self.path_node_maker
    }
}