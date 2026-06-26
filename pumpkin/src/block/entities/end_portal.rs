use std::pin::Pin;

use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use super::BlockEntity;

pub struct EndPortalBlockEntity {
    pub position: BlockPos,
}

impl EndPortalBlockEntity {
    pub const ID: &'static str = "minecraft:end_portal";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self { position }
    }

    #[must_use]
    pub fn create_nbt(position: BlockPos) -> NbtCompound {
        let mut nbt = NbtCompound::new();
        nbt.put_string("id", Self::ID.to_string());
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);
        nbt
    }
}

impl BlockEntity for EndPortalBlockEntity {
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
