use std::collections::HashMap;

use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::entity::ai::pathfinder::{
    node::{Node, PathType, Target},
    pathfinding_context::PathfindingContext,
};

pub trait NodeEvaluator {
    fn prepare(&mut self, context: PathfindingContext, mob_data: MobData);
    fn done(&mut self);
    fn get_start(&mut self) -> impl std::future::Future<Output = Option<Node>> + Send;
    fn get_target(&mut self, pos: BlockPos) -> Target;
    fn get_neighbors(
        &mut self,
        current: &Node,
    ) -> impl std::future::Future<Output = Vec<Node>> + Send;
    fn get_path_type_of_mob(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
        mob_data: &MobData,
    ) -> impl std::future::Future<Output = PathType> + Send;
    fn get_path_type(
        &mut self,
        context: &mut PathfindingContext,
        pos: Vector3<i32>,
    ) -> impl std::future::Future<Output = PathType> + Send;
    fn set_can_pass_doors(&mut self, can_pass: bool);
    fn set_can_open_doors(&mut self, can_open: bool);
    fn set_can_float(&mut self, can_float: bool);
    fn set_can_walk_over_fences(&mut self, can_walk: bool);
    fn can_pass_doors(&self) -> bool;
    fn can_open_doors(&self) -> bool;
    fn can_float(&self) -> bool;
    fn can_walk_over_fences(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct MobData {
    pub position: Vector3<f64>,
    pub width: f32,
    pub height: f32,
    pub max_step_height: f32,
    pub max_fall_distance: f32,
    pub can_swim: bool,
    pub can_walk_on_water: bool,
    pub avoids_fire: bool,
    pub avoids_water: bool,
    pub on_ground: bool,
    pub path_type_malus: HashMap<PathType, f32>,
}

impl MobData {
    #[must_use]
    pub fn new_zombie(position: Vector3<f64>, on_ground: bool) -> Self {
        let mut path_type_malus = HashMap::new();

        path_type_malus.insert(PathType::DangerFire, 16.0);
        path_type_malus.insert(PathType::DamageFire, -1.0);
        path_type_malus.insert(PathType::Water, 8.0);
        path_type_malus.insert(PathType::Lava, -1.0);
        path_type_malus.insert(PathType::DangerOther, 8.0);

        Self {
            position,
            width: 0.6,
            height: 1.95,
            max_step_height: 1.0,
            max_fall_distance: 3.0,
            can_swim: false,
            can_walk_on_water: false,
            avoids_fire: true,
            avoids_water: false,
            on_ground,
            path_type_malus,
        }
    }

    #[must_use]
    pub fn new(position: Vector3<f64>, width: f32, height: f32, max_step_height: f32) -> Self {
        Self {
            position,
            width,
            height,
            max_step_height,
            max_fall_distance: 3.0,
            can_swim: false,
            can_walk_on_water: false,
            avoids_fire: true,
            avoids_water: false,
            on_ground: true,
            path_type_malus: HashMap::new(),
        }
    }

    #[must_use]
    pub fn get_pathfinding_malus(&self, path_type: PathType) -> f32 {
        self.path_type_malus
            .get(&path_type)
            .copied()
            .unwrap_or_else(|| path_type.get_malus())
    }

    pub fn set_pathfinding_malus(&mut self, path_type: PathType, malus: f32) {
        self.path_type_malus.insert(path_type, malus);
    }

    #[must_use]
    pub const fn block_position(&self) -> (i32, i32, i32) {
        (
            self.position.x.floor() as i32,
            self.position.y.floor() as i32,
            self.position.z.floor() as i32,
        )
    }

    #[must_use]
    pub fn get_bb_width(&self) -> i32 {
        (self.width + 1.0).floor() as i32
    }

    #[must_use]
    pub fn get_bb_height(&self) -> i32 {
        (self.height + 1.0).floor() as i32
    }
}

pub struct BaseNodeEvaluator {
    pub context: Option<PathfindingContext>,
    pub mob_data: Option<MobData>,
    pub nodes: HashMap<i32, Node>,
    pub entity_width: i32,
    pub entity_height: i32,
    pub entity_depth: i32, // Same as width?
    pub can_pass_doors: bool,
    pub can_open_doors: bool,
    pub can_float: bool,
    pub can_walk_over_fences: bool,
}

impl Default for BaseNodeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseNodeEvaluator {
    #[must_use]
    pub fn new() -> Self {
        Self {
            context: None,
            mob_data: None,
            nodes: HashMap::new(),
            entity_width: 1,
            entity_height: 2,
            entity_depth: 1,
            can_pass_doors: true,
            can_open_doors: false,
            can_float: false,
            can_walk_over_fences: false,
        }
    }

    pub fn get_node(&mut self, pos: BlockPos) -> Node {
        let hash = Node::create_hash(pos);

        if let Some(node) = self.nodes.get(&hash) {
            node.clone()
        } else {
            let node = Node::new(pos);
            self.nodes.insert(hash, node.clone());
            node
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
        self.context = None;
        self.mob_data = None;
    }

    #[must_use]
    pub fn is_position_in_bounds(&self, x: i32, y: i32, z: i32) -> bool {
        self.mob_data.as_ref().is_none_or(|mob_data| {
            let mob_pos = mob_data.block_position();
            let dx = (x - mob_pos.0).abs();
            let dy = (y - mob_pos.1).abs();
            let dz = (z - mob_pos.2).abs();

            dx <= self.entity_width / 2 && dy <= self.entity_height && dz <= self.entity_depth / 2
        })
    }
}
