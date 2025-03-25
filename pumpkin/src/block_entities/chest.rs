use pumpkin_util::{block_entity::BlockEntity, math::position::BlockPos};

pub struct Chest {
    pub position: BlockPos,
    //pub items: [Item; 27],
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
        println!("Chest created from NBT at {:?}", position);
        Self { position }
    }

    fn write_nbt(&self, _nbt: &mut pumpkin_nbt::compound::NbtCompound) {}
}

impl Chest {
    pub fn new(position: BlockPos) -> Self {
        println!("Chest created at {:?}", position);
        Self { position }
    }
}
