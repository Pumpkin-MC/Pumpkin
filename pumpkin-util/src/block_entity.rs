use crate::math::position::BlockPos;
use pumpkin_nbt::compound::NbtCompound;

pub trait BlockEntity: Send + Sync {
    fn write_nbt(&self, nbt: &mut NbtCompound);
    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized;
    fn identifier(&self) -> &'static str;
    fn get_position(&self) -> BlockPos;
    fn write_internal(&self, nbt: &mut NbtCompound) {
        nbt.put_string("id", self.identifier().to_string());
        let position = self.get_position();
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);
        self.write_nbt(nbt);
    }
}

pub fn block_entity_from_generic<T: BlockEntity>(nbt: &NbtCompound) -> T {
    let x = nbt.get_int("x").unwrap();
    let y = nbt.get_int("y").unwrap();
    let z = nbt.get_int("z").unwrap();
    T::from_nbt(nbt, BlockPos::new(x, y, z))
}
