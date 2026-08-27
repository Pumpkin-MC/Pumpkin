use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};
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
}

pub struct TestBlockBlockEntity {
    pub position: BlockPos,
    mode: RwLock<TestBlockMode>,
    message: RwLock<String>,
    powered: AtomicBool,
    triggered: AtomicBool,
    dirty: AtomicBool,
}

impl TestBlockBlockEntity {
    pub const ID: &'static str = "minecraft:test_block";

    #[must_use]
    pub const fn new(position: BlockPos) -> Self {
        Self {
            position,
            mode: RwLock::new(TestBlockMode::Start),
            message: RwLock::new(String::new()),
            powered: AtomicBool::new(false),
            triggered: AtomicBool::new(false),
            dirty: AtomicBool::new(false),
        }
    }

    pub fn mode(&self) -> TestBlockMode {
        *self
            .mode
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub fn message(&self) -> String {
        self.message
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[must_use]
    pub fn is_powered(&self) -> bool {
        self.powered.load(Ordering::Acquire)
    }

    pub fn set_powered(&self, powered: bool) {
        self.powered.store(powered, Ordering::Release);
    }

    #[must_use]
    pub fn has_triggered(&self) -> bool {
        self.triggered.load(Ordering::Acquire)
    }

    pub fn trigger(&self, world: &Arc<World>) {
        let mode = self.mode();
        if mode == TestBlockMode::Start {
            self.set_powered(true);
            world.update_neighbors(&self.position, None);
            self.log();
            return;
        }

        if mode == TestBlockMode::Log {
            self.log();
        }

        self.triggered.store(true, Ordering::Release);
    }

    pub fn reset(&self, world: &Arc<World>) {
        self.triggered.store(false, Ordering::Release);
        if self.mode() == TestBlockMode::Start {
            self.set_powered(false);
            world.update_neighbors(&self.position, None);
        }
    }

    fn log(&self) {
        let message = self.message();
        if !message.trim().is_empty() {
            let mode = self.mode();
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
        let mode = nbt
            .get_string("mode")
            .and_then(TestBlockMode::from_serialized_name)
            .unwrap_or(TestBlockMode::Fail);

        Self {
            position,
            mode: RwLock::new(mode),
            message: RwLock::new(nbt.get_string("message").unwrap_or("").to_string()),
            powered: AtomicBool::new(nbt.get_bool("powered").unwrap_or(false)),
            triggered: AtomicBool::new(false),
            dirty: AtomicBool::new(false),
        }
    }

    fn write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_string("mode", self.mode().serialized_name().to_string());
        nbt.put_string("message", self.message());
        nbt.put_bool("powered", self.is_powered());
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        let mut nbt = NbtCompound::new();
        self.write_nbt(&mut nbt);
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
