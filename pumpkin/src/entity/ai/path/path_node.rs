use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

#[derive(Debug, Clone)]
pub struct PathNode {
    pub location: Vector3<f64>,
    pub heap_index: i32,
    pub penalized_path_length: f32,
    pub distance_to_nearest_target: f32,
    pub heap_weight: f32,
    pub previous: Option<Box<PathNode>>,
    pub visited: bool,
    pub path_length: f32,
    pub penality: f32,
    pub node_type: PathNodeType,
}

impl Default for PathNode {
    fn default() -> Self {
        Self {
            location: Vector3::new(0.0, 0.0, 0.0),
            heap_index: -1,
            penalized_path_length: 0.0,
            distance_to_nearest_target: 0.0,
            heap_weight: 0.0,
            previous: None,
            visited: false,
            path_length: 0.0,
            penality: 0.0,
            node_type: PathNodeType::Blocked,
        }
    }
}

impl PathNode {
    #[must_use]
    pub fn new(location: Vector3<f64>) -> Self {
        Self {
            location,
            ..Default::default()
        }
    }

    pub fn copy_with_new_position(&self, location: Vector3<f64>) -> Self {
        Self {
            location,
            ..self.clone()
        }
    }

    /// How expensive is it to go to a location?
    ///
    /// Returns an `f64`; higher means more expensive.
    #[must_use]
    pub fn get_expense(&self, end: Vector3<f64>) -> f64 {
        self.location.squared_distance_to_vec(end).sqrt()
    }

    pub fn get_horizontal_distance(&self, node: &PathNode) -> f64 {
        let x = self.location.x - node.location.x;
        let z = self.location.z - node.location.z;
        (x * x + z * z).sqrt()
    }

    pub fn get_squared_distance(&self, node: &PathNode) -> f64 {
        let x = self.location.x - node.location.x;
        let y = self.location.y - node.location.y;
        let z = self.location.z - node.location.z;
        x * x + y * y + z * z
    }

    pub fn get_manhattan_distance(&self, node: &PathNode) -> f64 {
        self.location.manhattan_distance(node.location)
    }

    pub fn get_manhattan_distance_to_pos(&self, pos: BlockPos) -> f64 {
        self.location.manhattan_distance(pos.to_f64())
    }

    pub fn is_in_heap(&self) -> bool {
        self.heap_index >= 0
    }

    pub fn hash(x: i32, y: i32, z: i32) -> i32 {
        y & 0xFF | (x & 32767) << 8 | (z & 32767) << 24 |
            if x < 0 { i32::MIN } else { 0 } | if z < 0 { 32768 } else { 0 }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PathNodeType {
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

impl PathNodeType {
    pub fn default_penality(&self) -> f32 {
        match self {
            PathNodeType::Blocked |
            PathNodeType::PowderSnow |
            PathNodeType::Fence |
            PathNodeType::Lava |
            PathNodeType::UnpassableRail |
            PathNodeType::DamageOther |
            PathNodeType::DoorWoodClosed |
            PathNodeType::DoorIronClosed |
            PathNodeType::Leaves => -1.0,
            PathNodeType::Open |
            PathNodeType::Walkable |
            PathNodeType::WalkableDoor |
            PathNodeType::Trapdoor |
            PathNodeType::DangerPowderSnow |
            PathNodeType::Rail |
            PathNodeType::DoorOpen |
            PathNodeType::Cocoa |
            PathNodeType::DamageCautious |
            PathNodeType::DangerTrapdoor => 0.0,
            PathNodeType::Breach => 4.0,
            PathNodeType::Water |
            PathNodeType::WaterBorder |
            PathNodeType::DangerFire |
            PathNodeType::DangerOther |
            PathNodeType::StickyHoney => 8.0,
            PathNodeType::DamageFire => 16.0,
        }
    }
}