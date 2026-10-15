use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

pub use crate::block::PathComputationType;

pub trait Coordinate {
    fn distance(&self, other: &dyn Coordinate) -> f32;
    fn distance_xz(&self, other: &dyn Coordinate) -> f32;
    fn distance_sqr(&self, other: &dyn Coordinate) -> f32;
    fn distance_manhattan(&self, other: &dyn Coordinate) -> f32;

    fn as_blockpos(&self) -> BlockPos;
    fn as_node(&self) -> Node;
    fn as_vector3(&self) -> Vector3<i32>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub pos: BlockPos,
    pub hash: i32,
    pub heap_idx: i32,
    pub g: f32,
    pub h: f32,
    pub f: f32,
    pub came_from: Option<Vector3<i32>>,
    pub closed: bool,
    pub walked_dist: f32,
    pub cost_malus: f32,
    pub path_type: PathType,
}

impl Default for Node {
    fn default() -> Self {
        Self {
            pos: BlockPos::new(0, 0, 0),
            hash: Self::create_hash(0, 0, 0),
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
            hash: Self::create_hash(pos.0.x, pos.0.y, pos.0.z),
            ..Default::default()
        }
    }

    #[must_use]
    pub const fn clone_and_move(&self, pos: BlockPos) -> Self {
        Self {
            pos,
            hash: Self::create_hash(pos.0.x, pos.0.y, pos.0.z),
            heap_idx: self.heap_idx,
            g: self.g,
            h: self.h,
            f: self.f,
            came_from: self.came_from,
            closed: self.closed,
            walked_dist: self.walked_dist,
            cost_malus: self.cost_malus,
            path_type: self.path_type,
        }
    }

    #[must_use]
    pub const fn create_hash(x: i32, y: i32, z: i32) -> i32 {
        let x_neg = if x < 0 { i32::MIN } else { 0 };
        let z_neg = if z < 0 { 32768 } else { 0 };
        (y & 0xFF) | ((x & 32767) << 8) | ((z & 32767) << 24) | x_neg | z_neg
    }

    #[must_use]
    pub const fn in_open_set(&self) -> bool {
        self.heap_idx >= 0
    }

    #[must_use]
    pub fn distance(&self, other: &dyn Coordinate) -> f32 {
        (self.pos.0.squared_distance_to_vec(&other.as_vector3()) as f32).sqrt()
    }

    #[must_use]
    pub fn distance_xz(&self, other: &dyn Coordinate) -> f32 {
        (self.pos.0.squared_distance_to_vec_xz(other.as_vector3()) as f32).sqrt()
    }

    #[must_use]
    pub fn distance_sqr(&self, other: &dyn Coordinate) -> f32 {
        self.pos.0.squared_distance_to_vec(&other.as_vector3()) as f32
    }

    #[must_use]
    pub fn distance_manhattan(&self, other: &dyn Coordinate) -> f32 {
        let v = other.as_vector3();
        let x = (self.pos.0.x - v.x).abs();
        let y = (self.pos.0.y - v.y).abs();
        let z = (self.pos.0.z - v.z).abs();
        (x + y + z) as f32
    }

    #[must_use]
    pub fn distance_to_node(&self, other: &Self) -> f32 {
        let xd = (other.pos.0.x - self.pos.0.x) as f32;
        let yd = (other.pos.0.y - self.pos.0.y) as f32;
        let zd = (other.pos.0.z - self.pos.0.z) as f32;
        (xd * xd + yd * yd + zd * zd).sqrt()
    }

    #[must_use]
    pub fn distance_to_xz_node(&self, other: &Self) -> f32 {
        let xd = (other.pos.0.x - self.pos.0.x) as f32;
        let zd = (other.pos.0.z - self.pos.0.z) as f32;
        xd.hypot(zd)
    }

    #[must_use]
    pub fn distance_to_block_pos(&self, pos: BlockPos) -> f32 {
        let xd = (pos.0.x - self.pos.0.x) as f32;
        let yd = (pos.0.y - self.pos.0.y) as f32;
        let zd = (pos.0.z - self.pos.0.z) as f32;
        (xd * xd + yd * yd + zd * zd).sqrt()
    }

    #[must_use]
    pub fn distance_to_sqr_node(&self, other: &Self) -> f32 {
        let xd = (other.pos.0.x - self.pos.0.x) as f32;
        let yd = (other.pos.0.y - self.pos.0.y) as f32;
        let zd = (other.pos.0.z - self.pos.0.z) as f32;
        xd * xd + yd * yd + zd * zd
    }

    #[must_use]
    pub fn distance_to_sqr_block_pos(&self, pos: BlockPos) -> f32 {
        let xd = (pos.0.x - self.pos.0.x) as f32;
        let yd = (pos.0.y - self.pos.0.y) as f32;
        let zd = (pos.0.z - self.pos.0.z) as f32;
        xd * xd + yd * yd + zd * zd
    }

    #[must_use]
    pub const fn distance_manhattan_node(&self, other: &Self) -> f32 {
        let xd = (other.pos.0.x - self.pos.0.x).abs();
        let yd = (other.pos.0.y - self.pos.0.y).abs();
        let zd = (other.pos.0.z - self.pos.0.z).abs();
        (xd + yd + zd) as f32
    }

    #[must_use]
    pub const fn distance_manhattan_block_pos(&self, pos: BlockPos) -> f32 {
        let xd = (pos.0.x - self.pos.0.x).abs();
        let yd = (pos.0.y - self.pos.0.y).abs();
        let zd = (pos.0.z - self.pos.0.z).abs();
        (xd + yd + zd) as f32
    }

    #[must_use]
    pub const fn as_vec3(&self) -> Vector3<f64> {
        Vector3::new(
            self.pos.0.x as f64,
            self.pos.0.y as f64,
            self.pos.0.z as f64,
        )
    }
}

impl Coordinate for Node {
    fn distance(&self, other: &dyn Coordinate) -> f32 {
        self.distance(other)
    }

    fn distance_xz(&self, other: &dyn Coordinate) -> f32 {
        self.distance_xz(other)
    }

    fn distance_sqr(&self, other: &dyn Coordinate) -> f32 {
        self.distance_sqr(other)
    }

    fn distance_manhattan(&self, other: &dyn Coordinate) -> f32 {
        self.distance_manhattan(other)
    }

    fn as_blockpos(&self) -> BlockPos {
        self.pos
    }

    fn as_node(&self) -> Node {
        *self
    }

    fn as_vector3(&self) -> Vector3<i32> {
        self.pos.0
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Target {
    pub node: Node,
    pub best_heuristic: f32,
    pub best_node: Option<Node>,
    pub reached: bool,
}

impl Target {
    #[must_use]
    pub const fn new(node: Node) -> Self {
        Self {
            node,
            best_heuristic: f32::MAX,
            best_node: None,
            reached: false,
        }
    }

    #[must_use]
    pub fn from_pos(pos: BlockPos) -> Self {
        Self::new(Node::new(pos))
    }

    pub fn update_best(&mut self, heuristic: f32, node: &Node) {
        if heuristic < self.best_heuristic {
            self.best_heuristic = heuristic;
            self.best_node = Some(*node);
        }
    }

    #[must_use]
    pub const fn get_best_node(&self) -> Option<Node> {
        self.best_node
    }

    pub const fn set_reached(&mut self) {
        self.reached = true;
    }

    #[must_use]
    pub const fn is_reached(&self) -> bool {
        self.reached
    }

    #[must_use]
    pub fn distance(&self, other: &dyn Coordinate) -> f32 {
        self.node.distance(other)
    }

    #[must_use]
    pub fn distance_xz(&self, other: &dyn Coordinate) -> f32 {
        self.node.distance_xz(other)
    }

    #[must_use]
    pub fn distance_sqr(&self, other: &dyn Coordinate) -> f32 {
        self.node.distance_sqr(other)
    }

    #[must_use]
    pub fn distance_manhattan(&self, other: &dyn Coordinate) -> f32 {
        self.node.distance_manhattan(other)
    }
}

impl Coordinate for Target {
    fn distance(&self, other: &dyn Coordinate) -> f32 {
        self.node.distance(other)
    }

    fn distance_xz(&self, other: &dyn Coordinate) -> f32 {
        self.node.distance_xz(other)
    }

    fn distance_sqr(&self, other: &dyn Coordinate) -> f32 {
        self.node.distance_sqr(other)
    }

    fn distance_manhattan(&self, other: &dyn Coordinate) -> f32 {
        self.node.distance_manhattan(other)
    }

    fn as_blockpos(&self) -> BlockPos {
        self.node.pos
    }

    fn as_node(&self) -> Node {
        self.node
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
#[repr(u8)]
pub enum PathType {
    Blocked = 0,
    Open = 1,
    Walkable = 2,
    WalkableDoor = 3,
    Trapdoor = 4,
    PowderSnow = 5,
    DangerPowderSnow = 6,
    Fence = 7,
    Lava = 8,
    Water = 9,
    WaterBorder = 10,
    Rail = 11,
    UnpassableRail = 12,
    DangerFire = 13,
    DamageFire = 14,
    DangerOther = 15,
    DamageOther = 16,
    DoorOpen = 17,
    DoorWoodClosed = 18,
    DoorIronClosed = 19,
    Breach = 20,
    Leaves = 21,
    StickyHoney = 22,
    Cocoa = 23,
    DamageCautious = 24,
    DangerTrapdoor = 25,
    BigMobsCloseToDanger = 26,
}

pub const PATH_TYPE_COUNT: usize = 27;

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
            Self::Breach | Self::BigMobsCloseToDanger => 4.0,
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
