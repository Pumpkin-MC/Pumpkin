use pumpkin_util::math::position::BlockPos;

use super::BlockEntity;

pub struct ComparatorBlockEntity {
    pub position: BlockPos,
    pub output_signal: i32,
}

impl ComparatorBlockEntity {
    pub const ID: &'static str = "minecraft:comparator";
}

const OUTPUT_SIGNAL: &str = "OutputSignal";

impl BlockEntity for ComparatorBlockEntity {
    fn identifier(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let output_signal = nbt.get_int(OUTPUT_SIGNAL).unwrap_or(0);
        Self {
            position,
            output_signal,
        }
    }

    fn write_nbt(&self, nbt: &mut pumpkin_nbt::compound::NbtCompound) {
        nbt.put_int(OUTPUT_SIGNAL, self.output_signal);
    }
}
