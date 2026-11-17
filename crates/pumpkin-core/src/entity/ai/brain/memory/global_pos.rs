use std::fmt;
use std::hash::{Hash, Hasher};

use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::position::BlockPos;

#[derive(Clone, Copy)]
pub struct GlobalPos {
    pub dimension: &'static Dimension,
    pub pos: BlockPos,
}

impl GlobalPos {
    #[must_use]
    pub const fn new(dimension: &'static Dimension, pos: BlockPos) -> Self {
        Self { dimension, pos }
    }

    #[must_use]
    pub fn is_close_enough(&self, dimension: &Dimension, pos: BlockPos, max_distance: u32) -> bool {
        self.dimension.id == dimension.id
            && self
                .pos
                .0
                .x
                .abs_diff(pos.0.x)
                .max(self.pos.0.y.abs_diff(pos.0.y))
                .max(self.pos.0.z.abs_diff(pos.0.z))
                <= max_distance
    }
}

impl PartialEq for GlobalPos {
    fn eq(&self, other: &Self) -> bool {
        self.dimension.id == other.dimension.id && self.pos == other.pos
    }
}

impl Eq for GlobalPos {}

impl Hash for GlobalPos {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dimension.id.hash(state);
        self.pos.hash(state);
    }
}

impl fmt::Debug for GlobalPos {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?}", self.dimension.minecraft_name, self.pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_enough_uses_inclusive_chessboard_distance_and_dimension() {
        let pos = GlobalPos::new(&Dimension::OVERWORLD, BlockPos::new(0, 64, 0));
        assert!(pos.is_close_enough(
            &Dimension::OVERWORLD,
            BlockPos::new(1024, 1088, -1024),
            1024
        ));
        assert!(!pos.is_close_enough(&Dimension::OVERWORLD, BlockPos::new(1025, 64, 0), 1024));
        assert!(!pos.is_close_enough(&Dimension::THE_NETHER, BlockPos::new(0, 64, 0), 1024));
    }
}
