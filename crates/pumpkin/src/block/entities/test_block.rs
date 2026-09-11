use pumpkin_data::{Block, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::world::World;

use super::BlockEntity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestBlockMode {
    Start,
    Log,
    Fail,
    Accept,
}

impl TestBlockMode {
    #[must_use]
    pub const fn from_serialized_name(name: &str) -> Option<Self> {
        match name.as_bytes() {
            b"start" => Some(Self::Start),
            b"log" => Some(Self::Log),
            b"fail" => Some(Self::Fail),
            b"accept" => Some(Self::Accept),
            _ => None,
        }
    }

    #[must_use]
    pub const fn serialized_name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Log => "log",
            Self::Fail => "fail",
            Self::Accept => "accept",
        }
    }

    /// Vanilla stores the test-block mode in the block state and mirrors it in the
    /// block entity. Treat the block state as authoritative so redstone behavior
    /// cannot diverge from the mode used to discover START/ACCEPT/FAIL/LOG blocks.
    #[must_use]
    pub fn from_block_state(state_id: BlockStateId) -> Option<Self> {
        Block::TEST_BLOCK
            .properties(state_id)?
            .to_props()
            .into_iter()
            .find_map(|(name, value)| {
                if name == "mode" {
                    Self::from_serialized_name(value)
                } else {
                    None
                }
            })
    }
}

pub struct TestBlockBlockEntity {
    pub position: BlockPos,
    pub mode: Mutex<String>,
    pub message: Mutex<String>,
    pub powered: AtomicBool,
    triggered: AtomicBool,
    pub dirty: AtomicBool,
}

impl BlockEntity for TestBlockBlockEntity {
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
        let mode = nbt.get_string("mode").unwrap_or("FAIL").to_string();
        let message = nbt.get_string("message").unwrap_or("").to_string();
        let powered = nbt.get_bool("powered").unwrap_or(false);

        Self {
            position,
            mode: Mutex::new(mode),
            message: Mutex::new(message),
            powered: AtomicBool::new(powered),
            triggered: AtomicBool::new(false),
            dirty: AtomicBool::new(false),
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

    fn is_dirty(&self) -> bool {
        self.dirty.load(Ordering::Relaxed)
    }

    fn clear_dirty(&self) {
        self.dirty.store(false, Ordering::Relaxed);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TestBlockBlockEntity {
    pub const ID: &'static str = "minecraft:test_block";

    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self::new_with_mode(position, TestBlockMode::Start)
    }

    #[must_use]
    pub fn new_with_mode(position: BlockPos, mode: TestBlockMode) -> Self {
        Self {
            position,
            mode: Mutex::new(mode.serialized_name().to_string()),
            message: Mutex::new(String::new()),
            powered: AtomicBool::new(false),
            triggered: AtomicBool::new(false),
            dirty: AtomicBool::new(false),
        }
    }

    #[must_use]
    pub fn mode(&self) -> TestBlockMode {
        let mode = self
            .mode
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        TestBlockMode::from_serialized_name(&mode.to_ascii_lowercase())
            .unwrap_or(TestBlockMode::Fail)
    }

    fn world_mode(&self, world: &World) -> TestBlockMode {
        TestBlockMode::from_block_state(world.get_block_state_id(&self.position))
            .unwrap_or_else(|| self.mode())
    }

    pub fn message(&self) -> String {
        self.message
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[must_use]
    pub fn is_powered(&self) -> bool {
        self.powered.load(Ordering::Acquire)
    }

    pub fn set_powered(&self, powered: bool) {
        self.powered.store(powered, Ordering::Release);
        self.dirty.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn has_triggered(&self) -> bool {
        self.triggered.load(Ordering::Acquire)
    }

    pub fn trigger(&self, world: &Arc<World>) {
        let mode = self.world_mode(world);
        if mode == TestBlockMode::Start {
            self.set_powered(true);
            world.update_neighbors(&self.position, None);

            // START is a one-tick redstone pulse. TestBlock::on_scheduled_tick
            // calls reset(), which drops the signal and notifies neighbors again.
            let block = world.get_block(&self.position);
            world.schedule_block_tick(block, self.position, 1, TickPriority::Normal);

            self.log(mode);
            return;
        }

        if mode == TestBlockMode::Log {
            self.log(mode);
        }

        self.triggered.store(true, Ordering::Release);
        self.dirty.store(true, Ordering::Release);
    }

    pub fn reset(&self, world: &Arc<World>) {
        self.triggered.store(false, Ordering::Release);
        self.dirty.store(true, Ordering::Release);
        if self.world_mode(world) == TestBlockMode::Start {
            self.set_powered(false);
            world.update_neighbors(&self.position, None);
        }
    }

    fn log(&self, mode: TestBlockMode) {
        let message = self.message();
        if !message.trim().is_empty() {
            info!(
                target: "pumpkin::gametest",
                mode = mode.serialized_name(),
                position = %self.position,
                message = %message,
                "Test block"
            );
        }
    }
}
