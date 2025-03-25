use pumpkin_inventory::ChestContainer;
use pumpkin_util::{block_entity::BlockEntity, math::position::BlockPos};

pub struct Chest {
    pub position: BlockPos,
    pub items: ChestContainer,
}

impl BlockEntity for Chest {
    fn identifier(&self) -> &'static str {
        "chest"
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(_nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        Self {
            position,
            items: ChestContainer::new(),
        }
    }

    fn write_nbt(&self, _nbt: &mut pumpkin_nbt::compound::NbtCompound) {}
}
