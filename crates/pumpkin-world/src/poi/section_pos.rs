use pumpkin_util::math::position::BlockPos;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SectionPos {
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn of(pos: &BlockPos) -> Self {
        Self {
            x: pos.0.x >> 4,
            y: pos.0.y >> 4,
            z: pos.0.z >> 4,
        }
    }

    #[must_use]
    pub const fn manhattan_distance(self, other: Self) -> i32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        let dx = if dx < 0 { -dx } else { dx };
        let dy = if dy < 0 { -dy } else { dy };
        let dz = if dz < 0 { -dz } else { dz };
        dx + dy + dz
    }
}
