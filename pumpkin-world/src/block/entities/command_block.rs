use std::sync::atomic::AtomicBool;

use async_trait::async_trait;
use pumpkin_util::math::position::BlockPos;

use super::BlockEntity;

pub struct CommandBlockEntity {
    pub position: BlockPos,
    pub powered: AtomicBool,
    _condition_met: AtomicBool,
    _auto: AtomicBool,
    pub dirty: AtomicBool,
}

impl CommandBlockEntity {
    pub const ID: &'static str = "minecraft:command_block";
    pub fn new(position: BlockPos) -> Self {
        Self {
            position,
            powered: AtomicBool::new(false),
            _condition_met: AtomicBool::new(false),
            _auto: AtomicBool::new(false),
            dirty: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl BlockEntity for CommandBlockEntity {
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
        Self::new(position)
    }

    async fn write_nbt(&self, _nbt: &mut pumpkin_nbt::compound::NbtCompound) {}

    fn is_dirty(&self) -> bool {
        self.dirty.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
