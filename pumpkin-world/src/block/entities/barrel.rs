use pumpkin_util::math::position::BlockPos;

use super::BlockEntity;

pub struct BarrelBlockEntity {
    pub position: BlockPos,
    //pub items: [Item; 27],
}

impl BlockEntity for BarrelBlockEntity {
    fn identifier(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(_nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self { position }
    }

    fn write_nbt(&self, _nbt: &mut pumpkin_nbt::compound::NbtCompound) {}
}

impl BarrelBlockEntity {
    pub const ID: &'static str = "minecraft:barrel";
    pub fn new(position: BlockPos) -> Self {
        Self { position }
    }
}
