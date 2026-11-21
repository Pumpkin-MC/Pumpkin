use async_trait::async_trait;
use pumpkin_data::BlockStateId;
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};
use pumpkin_world::world::BlockFlags;

use crate::error::{GameTestError, GameTestResult};
use crate::model::GameTestRotation;

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
        rotation: GameTestRotation,
    ) -> GameTestResult<BlockStateId>;

    async fn set_block_entity_nbt(
        &self,
        position: &BlockPos,
        nbt: &NbtCompound,
    ) -> GameTestResult<()>;

    async fn block_entity_nbt(&self, _position: &BlockPos) -> GameTestResult<Option<NbtCompound>> {
        Ok(None)
    }

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

    /// Returns whether every chunk intersecting the half-open structure box
    /// `[min, max)` is loaded and currently ticking.
    ///
    /// Vanilla `GameTestInfo` waits on this condition once after structure placement
    /// before advancing the setup/test clock. Non-server adapters can use the default
    /// because they do not have a separate chunk-ticking lifecycle.
    async fn test_area_loaded_and_ticking(&self, _min: &BlockPos, _max: &BlockPos) -> bool {
        true
    }

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

    async fn spawn_entity(
        &self,
        _entity_name: &str,
        _position: Vector3<f64>,
    ) -> GameTestResult<()> {
        Err(GameTestError::World(
            "entity spawning is not supported by this GameTestWorld".to_string(),
        ))
    }

    async fn spawn_item(
        &self,
        _item_name: &str,
        _count: u8,
        _position: Vector3<f64>,
    ) -> GameTestResult<()> {
        Err(GameTestError::World(
            "item spawning is not supported by this GameTestWorld".to_string(),
        ))
    }

    async fn entity_count_in_box(
        &self,
        _entity_name: &str,
        _min: Vector3<f64>,
        _max: Vector3<f64>,
    ) -> GameTestResult<usize> {
        Ok(0)
    }

    async fn item_count_in_box(
        &self,
        _item_name: &str,
        _min: Vector3<f64>,
        _max: Vector3<f64>,
    ) -> GameTestResult<u32> {
        Ok(0)
    }

    async fn container_item_count(
        &self,
        _position: &BlockPos,
        _item_name: &str,
    ) -> GameTestResult<u32> {
        Ok(0)
    }

    async fn is_container_empty(&self, _position: &BlockPos) -> GameTestResult<bool> {
        Ok(true)
    }

    async fn set_container_item(
        &self,
        _position: &BlockPos,
        _slot: usize,
        _item_name: &str,
        _count: u8,
    ) -> GameTestResult<()> {
        Err(GameTestError::World(
            "container mutation is not supported by this GameTestWorld".to_string(),
        ))
    }

    async fn press_button(&self, _position: &BlockPos) -> GameTestResult<()> {
        Err(GameTestError::World(
            "press_button is not supported by this GameTestWorld".to_string(),
        ))
    }

    async fn pull_lever(&self, _position: &BlockPos) -> GameTestResult<()> {
        Err(GameTestError::World(
            "pull_lever is not supported by this GameTestWorld".to_string(),
        ))
    }

    async fn use_block(&self, _position: &BlockPos) -> GameTestResult<()> {
        Err(GameTestError::World(
            "use_block is not supported by this GameTestWorld".to_string(),
        ))
    }

    async fn random_tick(&self, _position: &BlockPos) -> GameTestResult<()> {
        Err(GameTestError::World(
            "random_tick is not supported by this GameTestWorld".to_string(),
        ))
    }

    async fn execute_command(
        &self,
        _position: Vector3<f64>,
        _command: &str,
    ) -> GameTestResult<u32> {
        Err(GameTestError::World(
            "execute_command is not supported by this GameTestWorld".to_string(),
        ))
    }
}
