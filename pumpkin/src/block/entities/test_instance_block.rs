use std::pin::Pin;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use super::BlockEntity;

pub struct TestInstanceBlockBlockEntity {
    pub position: BlockPos,
}

impl BlockEntity for TestInstanceBlockBlockEntity {
    fn resource_location(&self) -> &'static str {
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

    fn write_nbt<'a>(
        &'a self,
        _nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TestInstanceBlockBlockEntity {
    pub const ID: &'static str = "minecraft:test_instance_block";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self { position }
    }
}
