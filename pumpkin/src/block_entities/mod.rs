use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

pub trait BlockEntity {
    fn write_nbt(&self, nbt: &mut NbtCompound);
    fn read_nbt(&mut self, nbt: &NbtCompound);
    fn identifier(&self) -> &'static str;
    fn get_position(&self) -> BlockPos;
    fn write_identifier_and_position(&self, nbt: &mut NbtCompound) {
        nbt.put_string("id", self.identifier().to_string());
        let position = self.get_position();
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);
    }
}
