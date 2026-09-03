use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use super::BlockEntity;

pub struct TestBlockBlockEntity {
    pub position: BlockPos,
    pub mode: Mutex<String>,
    pub message: Mutex<String>,
    pub powered: AtomicBool,
}

impl BlockEntity for TestBlockBlockEntity {
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
        let mode = nbt.get_string("mode").unwrap_or("FAIL").to_string();
        let message = nbt.get_string("message").unwrap_or("").to_string();
        let powered = nbt.get_bool("powered").unwrap_or(false);
        Self {
            position,
            mode: Mutex::new(mode),
            message: Mutex::new(message),
            powered: AtomicBool::new(powered),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        if let Ok(mode) = self.mode.lock() {
            nbt.put_string("mode", mode.clone());
        }
        if let Ok(message) = self.message.lock() {
            nbt.put_string("message", message.clone());
        }
        nbt.put_bool("powered", self.powered.load(Ordering::Relaxed));
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        if let Ok(mode) = self.mode.try_lock() {
            nbt.put_string("mode", mode.clone());
        }
        if let Ok(message) = self.message.try_lock() {
            nbt.put_string("message", message.clone());
        }
        nbt.put_bool("powered", self.powered.load(Ordering::Relaxed));
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TestBlockBlockEntity {
    pub const ID: &'static str = "minecraft:test_block";
    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            mode: Mutex::new("FAIL".to_string()),
            message: Mutex::new(String::new()),
            powered: AtomicBool::new(false),
        }
    }
}
