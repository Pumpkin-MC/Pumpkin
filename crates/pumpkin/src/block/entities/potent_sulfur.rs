use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::atomic::{AtomicI32, Ordering};

use super::BlockEntity;

pub struct PotentSulfurBlockEntity {
    pub position: BlockPos,
    pub waiting_countdown: AtomicI32,
}

impl BlockEntity for PotentSulfurBlockEntity {
    fn resource_location(&self) -> &'static str {
        Self::ID
    }

    fn get_position(&self) -> BlockPos {
        self.position
    }

    fn from_nbt(nbt: &pumpkin_nbt::compound::NbtCompound, position: BlockPos) -> Self
    where
        Self: Sized,
    {
        let countdown = nbt.get_int("countdown").unwrap_or(-1);
        Self {
            position,
            waiting_countdown: AtomicI32::new(countdown),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        let countdown = self.waiting_countdown.load(Ordering::Relaxed);
        if countdown != -1 {
            nbt.put_int("countdown", countdown);
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl PotentSulfurBlockEntity {
    pub const ID: &'static str = "minecraft:potent_sulfur";
    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            waiting_countdown: AtomicI32::new(-1),
        }
    }
}
