use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

pub trait Coordinate {
    fn distance(&self, other: &dyn Coordinate) -> f32;
    fn distance_xz(&self, other: &dyn Coordinate) -> f32;
    fn distance_sqr(&self, other: &dyn Coordinate) -> f32;
    fn distance_manhattan(&self, other: &dyn Coordinate) -> f32;

    fn as_blockpos(&self) -> BlockPos;
    fn as_node(&self) -> Node;
    fn as_vector3(&self) -> Vector3<i32>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub pos: BlockPos,
    pub heap_idx: i32,
    // Cost from start to this node
    pub g: f32,
    // Heuristic cost from this node to target
    pub h: f32,
    // g + h
    pub f: f32,
    pub came_from: Option<Box<Self>>,
    pub closed: bool,
    pub walked_dist: f32,
    pub cost_malus: f32,
    pub path_type: PathType,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            pos: BlockPos::new(0, 0, 0),
            heap_idx: -1,
            g: 0.0,
            h: 0.0,
            f: 0.0,
            came_from: None,
            closed: false,
            walked_dist: 0.0,
            cost_malus: 0.0,
            path_type: PathType::Blocked,
        }
    }
}

impl Node {
    #[must_use]
    pub fn new(pos: BlockPos) -> Self {
        Self {
            pos,
            ..Default::default()
        }
    }

    #[must_use]
    pub fn clone_and_move(&self, pos: BlockPos) -> Self {
        Self {
            pos,
            heap_idx: self.heap_idx,
            g: self.g,
            h: self.h,
            f: self.f,
            came_from: self.came_from.clone(),
            closed: self.closed,
            walked_dist: self.walked_dist,
            cost_malus: self.cost_malus,
            path_type: self.path_type,
        }
    }

    #[must_use]
    pub const fn create_hash(pos: BlockPos) -> i32 {
        pos.0.y & 0xFF
            | (pos.0.x & 32767) << 8
            | (pos.0.z & 32767) << 24
            | if pos.0.x < 0 { i32::MIN } else { 0 }
            | if pos.0.z < 0 { 32768 } else { 0 }
    }
}

impl Coordinate for Node {
    fn distance(&self, other: &dyn Coordinate) -> f32 {
        (self.pos.0.squared_distance_to_vec(&other.as_vector3()) as f32).sqrt()
    }

    fn distance_xz(&self, other: &dyn Coordinate) -> f32 {
        (self.pos.0.squared_distance_to_vec_xz(other.as_vector3()) as f32).sqrt()
    }

    fn distance_sqr(&self, other: &dyn Coordinate) -> f32 {
        self.pos.0.squared_distance_to_vec(&other.as_vector3()) as f32
    }

    fn distance_manhattan(&self, other: &dyn Coordinate) -> f32 {
        let v = other.as_vector3();
        let x = (self.pos.0.x - v.x).abs();
        let y = (self.pos.0.y - v.y).abs();
        let z = (self.pos.0.z - v.z).abs();
        (x + y + z) as f32
    }

    fn as_blockpos(&self) -> BlockPos {
        self.pos
    }

    fn as_node(&self) -> Node {
        self.clone()
    }

    fn as_vector3(&self) -> Vector3<i32> {
        self.pos.0
    }
}

#[derive(Default, Debug)]
pub struct Target {
    pub node: Node,
    pub best_heuristic: f32,
    pub best_node: Option<Box<Node>>,
    pub reached: bool,
}

impl Target {
    #[must_use]
    pub fn new(node: Node) -> Self {
        Self {
            node,
            best_heuristic: f32::MAX,
            ..Default::default()
        }
    }

    pub fn update_best(&mut self, heuristic: f32, node: &Node) {
        if heuristic < self.best_heuristic {
            self.best_heuristic = heuristic;
            self.best_node = Some(Box::new(node.clone()));
        }
    }
}

impl Coordinate for Target {
    fn distance(&self, other: &dyn Coordinate) -> f32 {
        (self.node.pos.0.squared_distance_to_vec(&other.as_vector3()) as f32).sqrt()
    }

    fn distance_xz(&self, other: &dyn Coordinate) -> f32 {
        (self
            .node
            .pos
            .0
            .squared_distance_to_vec_xz(other.as_vector3()) as f32)
            .sqrt()
    }

    fn distance_sqr(&self, other: &dyn Coordinate) -> f32 {
        self.node.pos.0.squared_distance_to_vec(&other.as_vector3()) as f32
    }

    fn distance_manhattan(&self, other: &dyn Coordinate) -> f32 {
        let v = other.as_vector3();
        let x = (self.node.pos.0.x - v.x).abs();
        let y = (self.node.pos.0.y - v.y).abs();
        let z = (self.node.pos.0.z - v.z).abs();
        (x + y + z) as f32
    }

    fn as_blockpos(&self) -> BlockPos {
        self.node.pos
    }

    fn as_node(&self) -> Node {
        self.node.clone()
    }

    fn as_vector3(&self) -> Vector3<i32> {
        self.node.pos.0
    }
}

impl Coordinate for BlockPos {
    fn distance(&self, other: &dyn Coordinate) -> f32 {
        (self.0.squared_distance_to_vec(&other.as_vector3()) as f32).sqrt()
    }

    fn distance_xz(&self, other: &dyn Coordinate) -> f32 {
        (self.0.squared_distance_to_vec_xz(other.as_vector3()) as f32).sqrt()
    }

    fn distance_sqr(&self, other: &dyn Coordinate) -> f32 {
        self.0.squared_distance_to_vec(&other.as_vector3()) as f32
    }

    fn distance_manhattan(&self, other: &dyn Coordinate) -> f32 {
        let v = other.as_vector3();
        let x = (self.0.x - v.x).abs();
        let y = (self.0.y - v.y).abs();
        let z = (self.0.z - v.z).abs();
        (x + y + z) as f32
    }

    fn as_blockpos(&self) -> BlockPos {
        *self
    }

    fn as_node(&self) -> Node {
        Node::new(*self)
    }

    fn as_vector3(&self) -> Vector3<i32> {
        self.0
    }
}

impl Coordinate for Vector3<i32> {
    fn distance(&self, other: &dyn Coordinate) -> f32 {
        (self.squared_distance_to_vec(&other.as_vector3()) as f32).sqrt()
    }

    fn distance_xz(&self, other: &dyn Coordinate) -> f32 {
        (self.squared_distance_to_vec_xz(other.as_vector3()) as f32).sqrt()
    }

    fn distance_sqr(&self, other: &dyn Coordinate) -> f32 {
        self.squared_distance_to_vec(&other.as_vector3()) as f32
    }

    fn distance_manhattan(&self, other: &dyn Coordinate) -> f32 {
        let v = other.as_vector3();
        let x = (self.x - v.x).abs();
        let y = (self.y - v.y).abs();
        let z = (self.z - v.z).abs();
        (x + y + z) as f32
    }

    fn as_blockpos(&self) -> BlockPos {
        BlockPos(*self)
    }

    fn as_node(&self) -> Node {
        Node::new(BlockPos(*self))
    }

    fn as_vector3(&self) -> Vector3<i32> {
        *self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PathType {
    Blocked,
    Open,
    Walkable,
    WalkableDoor,
    Trapdoor,
    PowderSnow,
    DangerPowderSnow,
    Fence,
    Lava,
    Water,
    WaterBorder,
    Rail,
    UnpassableRail,
    DangerFire,
    DamageFire,
    DangerOther,
    DamageOther,
    DoorOpen,
    DoorWoodClosed,
    DoorIronClosed,
    Breach,
    Leaves,
    StickyHoney,
    Cocoa,
    DamageCautious,
    DangerTrapdoor,
}

impl PathType {
    #[must_use]
    pub const fn get_malus(self) -> f32 {
        match self {
            Self::Blocked
            | Self::PowderSnow
            | Self::Fence
            | Self::Lava
            | Self::UnpassableRail
            | Self::DamageOther
            | Self::DoorWoodClosed
            | Self::DoorIronClosed
            | Self::Leaves => -1.0,
            Self::Open
            | Self::Walkable
            | Self::WalkableDoor
            | Self::Trapdoor
            | Self::DangerPowderSnow
            | Self::Rail
            | Self::DoorOpen
            | Self::Cocoa
            | Self::DamageCautious
            | Self::DangerTrapdoor => 0.0,
            Self::Breach => 4.0,
            Self::Water
            | Self::WaterBorder
            | Self::DangerFire
            | Self::DangerOther
            | Self::StickyHoney => 8.0,
            Self::DamageFire => 16.0,
        }
    }

    #[must_use]
    pub fn is_passable(self) -> bool {
        self.get_malus() >= 0.0
    }

    #[must_use]
    pub fn is_blocked(self) -> bool {
        self.get_malus() < 0.0
    }

    #[must_use]
    pub const fn is_water(self) -> bool {
        matches!(self, Self::Water | Self::WaterBorder)
    }

    #[must_use]
    pub const fn is_dangerous(self) -> bool {
        matches!(
            self,
            Self::Lava
                | Self::DangerFire
                | Self::DamageFire
                | Self::DangerOther
                | Self::DamageOther
                | Self::DangerPowderSnow
                | Self::DangerTrapdoor
        )
    }

    #[must_use]
    pub const fn is_door(self) -> bool {
        matches!(
            self,
            Self::WalkableDoor | Self::DoorOpen | Self::DoorWoodClosed | Self::DoorIronClosed
        )
    }

    #[must_use]
    pub const fn has_partial_collision(self) -> bool {
        matches!(
            self,
            Self::Fence
                | Self::WalkableDoor
                | Self::DoorOpen
                | Self::DoorWoodClosed
                | Self::DoorIronClosed
        )
    }
}
