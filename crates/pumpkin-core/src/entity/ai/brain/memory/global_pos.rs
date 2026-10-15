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
