use async_trait::async_trait;
use pumpkin_data::BlockStateId;
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::world::BlockFlags;

use crate::error::GameTestResult;
use crate::model::TestRotation;

#[async_trait]
pub trait GameTestWorld: Send + Sync {
    async fn block_state_id(&self, position: &BlockPos) -> BlockStateId;

    async fn set_block_state(
        &self,
        position: &BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> GameTestResult<()>;

    async fn rotate_block_state(
        &self,
        block_state_id: BlockStateId,
        rotation: TestRotation,
    ) -> GameTestResult<BlockStateId>;

    async fn set_block_entity_nbt(
        &self,
        position: &BlockPos,
        nbt: &NbtCompound,
    ) -> GameTestResult<()>;

    /// Removes all non-player entities intersecting the half-open world-space box
    /// `[min, max)`. Vanilla does this before every `GameTest` structure placement,
    /// and again around a successful structure, so reruns never inherit entities
    /// spawned by the previous attempt.
    async fn clear_non_player_entities(&self, min: &BlockPos, max: &BlockPos)
    -> GameTestResult<()>;

    /// Removes scheduled block ticks inside `[min, max)` after a structure has been
    /// replaced, matching vanilla `GameTest` placement cleanup.
    async fn clear_scheduled_block_ticks(
        &self,
        min: &BlockPos,
        max: &BlockPos,
    ) -> GameTestResult<()>;

    /// Removes queued block events inside `[min, max)` after structure replacement.
    async fn clear_block_events(&self, min: &BlockPos, max: &BlockPos) -> GameTestResult<()>;

    async fn set_test_instance_running(&self, position: &BlockPos) -> GameTestResult<()>;

    async fn set_test_instance_success(&self, position: &BlockPos) -> GameTestResult<()>;

    async fn set_test_instance_failure(
        &self,
        position: &BlockPos,
        message: &str,
        marker: Option<(BlockPos, String)>,
    ) -> GameTestResult<()>;

    async fn trigger_test_block(&self, position: &BlockPos) -> GameTestResult<()>;

    async fn reset_test_block(&self, position: &BlockPos) -> GameTestResult<()>;

    async fn test_block_triggered(&self, position: &BlockPos) -> GameTestResult<bool>;

    async fn test_block_message(&self, position: &BlockPos) -> GameTestResult<String>;

    async fn surface_height(&self, x: i32, z: i32) -> i32;
}
