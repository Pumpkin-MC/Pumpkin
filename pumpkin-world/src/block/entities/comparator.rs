use std::sync::atomic::{AtomicU8, Ordering};

use async_trait::async_trait;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;

use super::BlockEntity;

pub struct ComparatorBlockEntity {
    pub position: BlockPos,
    pub output_signal: AtomicU8,
}

impl ComparatorBlockEntity {
    pub const ID: &'static str = "minecraft:comparator";
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            output_signal: AtomicU8::new(0),
        }
    }
}

const OUTPUT_SIGNAL: &str = "OutputSignal";

#[async_trait]
impl BlockEntity for ComparatorBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let output_signal = nbt.get_int(OUTPUT_SIGNAL).unwrap_or(0) as u8;
        Self {
            position,
            output_signal: AtomicU8::new(output_signal),
        }
    }

    async fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int(
            OUTPUT_SIGNAL,
            self.output_signal.load(Ordering::Relaxed) as i32,
        );
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        nbt.put_int(
            OUTPUT_SIGNAL,
            self.output_signal.load(Ordering::Relaxed) as i32,
        );
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
