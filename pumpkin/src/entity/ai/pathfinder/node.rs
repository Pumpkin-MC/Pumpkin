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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    pub pos: BlockPos,
    pub heap_idx: i32,
    // Cost from start to this node
    pub g: f32,
    // Heuristic cost from this node to target
    pub h: f32,
    // g + h
    pub f: f32,
    /// Position of the predecessor node for path reconstruction.
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
    pub const fn clone_and_move(&self, pos: BlockPos) -> Self {
        Self {
            pos,
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
        *self
    }

    fn as_vector3(&self) -> Vector3<i32> {
        self.pos.0
    }
}

#[derive(Default, Debug)]
pub struct Target {
    pub node: Node,
    pub best_heuristic: f32,
    pub best_node: Option<Node>,
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
            self.best_node = Some(*node);
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
}

pub const PATH_TYPE_COUNT: usize = 26;

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

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_PATH_TYPES: [PathType; PATH_TYPE_COUNT] = [
        PathType::Blocked,
        PathType::Open,
        PathType::Walkable,
        PathType::WalkableDoor,
        PathType::Trapdoor,
        PathType::PowderSnow,
        PathType::DangerPowderSnow,
        PathType::Fence,
        PathType::Lava,
        PathType::Water,
        PathType::WaterBorder,
        PathType::Rail,
        PathType::UnpassableRail,
        PathType::DangerFire,
        PathType::DamageFire,
        PathType::DangerOther,
        PathType::DamageOther,
        PathType::DoorOpen,
        PathType::DoorWoodClosed,
        PathType::DoorIronClosed,
        PathType::Breach,
        PathType::Leaves,
        PathType::StickyHoney,
        PathType::Cocoa,
        PathType::DamageCautious,
        PathType::DangerTrapdoor,
    ];

    /// `is_passable`/`is_blocked` must partition every variant by malus sign.
    #[test]
    fn passable_and_blocked_partition_all_variants() {
        for ty in ALL_PATH_TYPES {
            assert_eq!(ty.is_passable(), ty.get_malus() >= 0.0, "{ty:?}");
            assert_eq!(ty.is_blocked(), !ty.is_passable(), "{ty:?}");
        }
    }

    #[test]
    fn known_malus_values_are_stable() {
        assert_eq!(PathType::Blocked.get_malus(), -1.0);
        assert_eq!(PathType::Open.get_malus(), 0.0);
        assert_eq!(PathType::Walkable.get_malus(), 0.0);
        assert_eq!(PathType::Breach.get_malus(), 4.0);
        assert_eq!(PathType::Water.get_malus(), 8.0);
        assert_eq!(PathType::DamageFire.get_malus(), 16.0);
    }

    #[test]
    fn door_classification_matches_variants() {
        for ty in ALL_PATH_TYPES {
            let expected = matches!(
                ty,
                PathType::WalkableDoor
                    | PathType::DoorOpen
                    | PathType::DoorWoodClosed
                    | PathType::DoorIronClosed
            );
            assert_eq!(ty.is_door(), expected, "{ty:?}");
        }
    }

    #[test]
    fn dangerous_and_water_classification() {
        assert!(PathType::Lava.is_dangerous());
        assert!(PathType::DamageFire.is_dangerous());
        assert!(!PathType::Walkable.is_dangerous());

        assert!(PathType::Water.is_water());
        assert!(PathType::WaterBorder.is_water());
        assert!(!PathType::Lava.is_water());
    }

    #[test]
    fn partial_collision_variants() {
        assert!(PathType::Fence.has_partial_collision());
        assert!(PathType::DoorIronClosed.has_partial_collision());
        assert!(!PathType::Walkable.has_partial_collision());
    }

    #[test]
    fn coordinate_distance_metrics() {
        // dx=3, dy=4, dz=0 -> sqr=25, dist=5, xz=3, manhattan=7
        let a = BlockPos::new(1, 2, 3);
        let b = BlockPos::new(4, 6, 3);

        assert_eq!(a.distance_sqr(&b), 25.0);
        assert_eq!(a.distance(&b), 5.0);
        assert_eq!(a.distance_xz(&b), 3.0);
        assert_eq!(a.distance_manhattan(&b), 7.0);
    }

    /// The `Coordinate` impls for `Node`, `BlockPos` and `Vector3<i32>` must agree.
    #[test]
    fn coordinate_impls_are_consistent() {
        let target = BlockPos::new(10, -3, 7);
        let block = BlockPos::new(2, 4, 1);
        let vec = block.0;
        let node = Node::new(block);

        assert_eq!(block.distance(&target), vec.distance(&target));
        assert_eq!(block.distance(&target), node.distance(&target));
        assert_eq!(
            block.distance_manhattan(&target),
            node.distance_manhattan(&target)
        );
        assert_eq!(block.distance_xz(&target), vec.distance_xz(&target));

        assert_eq!(node.as_vector3(), vec);
        assert_eq!(node.as_blockpos(), block);
    }
}
