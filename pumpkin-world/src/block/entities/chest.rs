use async_trait::async_trait;
use pumpkin_util::math::position::BlockPos;

use super::BlockEntity;

pub struct ChestBlockEntity {
    pub position: BlockPos,
    //pub items: [Item; 27],
}

#[async_trait]
impl BlockEntity for ChestBlockEntity {
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

    async fn write_nbt(&self, _nbt: &mut pumpkin_nbt::compound::NbtCompound) {}

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ChestBlockEntity {
    pub const ID: &'static str = "minecraft:chest";
    pub fn new(position: BlockPos) -> Self {
        Self { position }
    }
}
