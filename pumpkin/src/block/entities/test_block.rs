use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_data::block_properties::{BlockProperties, TestBlockLikeProperties, TestBlockMode};
use pumpkin_data::{Block, BlockId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;
use tokio::sync::Mutex;

use crate::world::World;

use super::BlockEntity;

/// `net.minecraft.world.level.block.entity.TestBlockEntity`.
///
/// `mode` mirrors `TestBlock.MODE` on the block state; `message` and `powered` persist,
/// while `triggered` is transient (vanilla `saveAdditional` writes only the first three).
pub struct TestBlockBlockEntity {
    pub position: BlockPos,
    pub mode: Mutex<TestBlockMode>,
    pub message: Mutex<String>,
    pub powered: AtomicBool,
    pub triggered: AtomicBool,
}

fn mode_from_value(value: &str) -> Option<TestBlockMode> {
    match value {
        "start" => Some(TestBlockMode::Start),
        "log" => Some(TestBlockMode::Log),
        "fail" => Some(TestBlockMode::Fail),
        "accept" => Some(TestBlockMode::Accept),
        _ => None,
    }
}

const fn mode_to_value(mode: TestBlockMode) -> &'static str {
    match mode {
        TestBlockMode::Start => "start",
        TestBlockMode::Log => "log",
        TestBlockMode::Fail => "fail",
        TestBlockMode::Accept => "accept",
    }
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
        // loadAdditional: an absent or unrecognised mode falls back to FAIL, not to the
        // block state's mode.
        let mode = nbt
            .get_string("mode")
            .and_then(mode_from_value)
            .unwrap_or(TestBlockMode::Fail);
        Self {
            position,
            mode: Mutex::new(mode),
            message: Mutex::new(nbt.get_string("message").unwrap_or_default().to_string()),
            powered: AtomicBool::new(nbt.get_bool("powered").unwrap_or(false)),
            triggered: AtomicBool::new(false),
        }
    }

    fn write_nbt<'a>(
        &'a self,
        nbt: &'a mut NbtCompound,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            nbt.put_string("mode", mode_to_value(*self.mode.lock().await).to_string());
            nbt.put_string("message", self.message.lock().await.clone());
            nbt.put_bool("powered", self.powered.load(Ordering::Relaxed));
        })
    }

    fn chunk_data_nbt(&self) -> Option<NbtCompound> {
        // getUpdateTag == saveCustomOnly: the client receives the full custom payload.
        let mut nbt = NbtCompound::new();
        nbt.put_string(
            "mode",
            mode_to_value(*self.mode.try_lock().ok()?).to_string(),
        );
        nbt.put_string("message", self.message.try_lock().ok()?.clone());
        nbt.put_bool("powered", self.powered.load(Ordering::Relaxed));
        Some(nbt)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl TestBlockBlockEntity {
    pub const ID: &'static str = "minecraft:test_block";

    /// The vanilla constructor seeds `mode` from `blockState.getValue(TestBlock.MODE)`;
    /// the placement path here only has the position, so the block's default state mode
    /// is used, matching a freshly placed block with no `block_state` component.
    #[must_use]
    pub fn new(position: BlockPos) -> Self {
        Self::with_mode(
            position,
            TestBlockLikeProperties::default(&Block::TEST_BLOCK).mode,
        )
    }

    #[must_use]
    pub fn with_mode(position: BlockPos, mode: TestBlockMode) -> Self {
        Self {
            position,
            mode: Mutex::new(mode),
            message: Mutex::new(String::new()),
            powered: AtomicBool::new(false),
            triggered: AtomicBool::new(false),
        }
    }

    pub async fn get_mode(&self) -> TestBlockMode {
        *self.mode.lock().await
    }

    pub fn is_powered(&self) -> bool {
        self.powered.load(Ordering::Relaxed)
    }

    pub fn set_powered(&self, powered: bool) {
        self.powered.store(powered, Ordering::Relaxed);
    }

    pub fn has_triggered(&self) -> bool {
        self.triggered.load(Ordering::Relaxed)
    }

    pub async fn get_message(&self) -> String {
        self.message.lock().await.clone()
    }

    pub async fn set_message(&self, message: String) {
        *self.message.lock().await = message;
    }

    /// `setMode` + `updateBlockState`: writes the mode back onto the block state with
    /// vanilla flag 2 (`BLOCK_UPDATE`: notify clients, do not notify neighbours).
    pub async fn set_mode(&self, world: &Arc<World>, mode: TestBlockMode) {
        *self.mode.lock().await = mode;
        let (block, state) = world.get_block_and_state(&self.position);
        if block.id != BlockId::TEST_BLOCK {
            return;
        }
        let mut props = TestBlockLikeProperties::from_state_id(state.id, block);
        props.mode = mode;
        world
            .set_block_state(
                &self.position,
                props.to_state_id(block),
                BlockFlags::NOTIFY_LISTENERS,
            )
            .await;
    }

    /// `reset`: clears the transient trigger flag and, in START mode, drops the emitted
    /// redstone signal back to zero.
    pub async fn reset(&self, world: &Arc<World>) {
        self.triggered.store(false, Ordering::Relaxed);
        if self.get_mode().await == TestBlockMode::Start {
            self.set_powered(false);
            world.update_neighbors(&self.position, None).await;
        }
    }

    /// `trigger`: START powers the block and schedules its reset; LOG logs; every
    /// non-START mode records that it fired.
    pub async fn trigger(&self, world: &Arc<World>) {
        let mode = self.get_mode().await;
        if mode == TestBlockMode::Start {
            self.set_powered(true);
            world.update_neighbors(&self.position, None).await;
            // getBlockTicks().willTickThisTick(pos, block): reset on this same tick.
            world.schedule_block_tick(&Block::TEST_BLOCK, self.position, 0, TickPriority::Normal);
            self.log().await;
        } else {
            if mode == TestBlockMode::Log {
                self.log().await;
            }
            self.triggered.store(true, Ordering::Relaxed);
        }
    }

    /// `log`: blank messages are not logged.
    pub async fn log(&self) {
        let message = self.get_message().await;
        if message.trim().is_empty() {
            return;
        }
        tracing::info!(
            "Test {} (at {:?}): {}",
            mode_to_value(self.get_mode().await),
            self.position.0,
            message
        );
    }
}
