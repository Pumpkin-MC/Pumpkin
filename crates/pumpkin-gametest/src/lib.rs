pub mod block_based;
pub mod error;
pub mod helper;
pub mod manager;
pub mod model;
pub mod runner;
pub mod structure;
pub mod world;

pub use block_based::BlockBasedTest;
pub use error::{GameTestError, GameTestResult};
pub use helper::{
    GameTestFunction, GameTestHelper, GameTestSequence, get_test_function,
    register_flaky_function_test, register_function_test, register_test_function,
    registered_test_definitions,
};
pub use manager::{
    GameTestBatchReport, GameTestManager, GameTestReporter, GameTestRetryOptions, GameTestRunner,
};
pub use model::{GameTestDefinition, GameTestRotation, TestType};
pub use runner::{GameTestSession, GameTestState, TestRunner};
pub use structure::{
    GameTestPosition, GameTestStructureBlock, GameTestStructureTemplate, TestBlockMode,
    TestStructureInstance, clear_structure_area, encase_structure, place_structure,
    place_structure_with_controller_rotation, remove_barriers,
};
pub use world::GameTestWorld;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use pumpkin_data::{Block, BlockState, BlockStateId};
    use pumpkin_nbt::{NbtCompound, tag::NbtTag};
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::text::TextComponent;
    use pumpkin_world::world::BlockFlags;

    use super::*;

    type InstanceState = Option<Result<(), (String, Option<(BlockPos, String)>)>>;

    #[derive(Default)]
    struct InMemoryTestWorld {
        blocks: Mutex<HashMap<BlockPos, BlockStateId>>,
        test_blocks_triggered: Mutex<HashMap<BlockPos, bool>>,
        test_block_messages: Mutex<HashMap<BlockPos, String>>,
        instance_state: Mutex<InstanceState>,
    }

    #[async_trait]
    impl GameTestWorld for InMemoryTestWorld {
        async fn block_state_id(&self, position: &BlockPos) -> BlockStateId {
            self.blocks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(position)
                .copied()
                .unwrap_or(Block::AIR.default_state.id)
        }

        async fn set_block_state(
            &self,
            position: &BlockPos,
            block_state_id: BlockStateId,
            _flags: BlockFlags,
        ) -> GameTestResult<()> {
            self.blocks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(*position, block_state_id);
            Ok(())
        }

        async fn rotate_block_state(
            &self,
            block_state_id: BlockStateId,
            rotation: GameTestRotation,
        ) -> GameTestResult<BlockStateId> {
            let (block, _) = BlockState::from_id_with_block(block_state_id);
            Ok(block
                .rotate(block_state_id, rotation.as_block_rotation())
                .id)
        }

        async fn set_block_entity_nbt(
            &self,
            position: &BlockPos,
            nbt: &NbtCompound,
        ) -> GameTestResult<()> {
            if let Some(message) = nbt.get_string("message") {
                self.test_block_messages
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert(*position, message.to_owned());
            }
            Ok(())
        }

        async fn clear_non_player_entities(
            &self,
            _min: &BlockPos,
            _max: &BlockPos,
        ) -> GameTestResult<()> {
            Ok(())
        }

        async fn clear_scheduled_block_ticks(
            &self,
            _min: &BlockPos,
            _max: &BlockPos,
        ) -> GameTestResult<()> {
            Ok(())
        }

        async fn clear_block_events(&self, _min: &BlockPos, _max: &BlockPos) -> GameTestResult<()> {
            Ok(())
        }

        async fn set_test_instance_running(&self, _position: &BlockPos) -> GameTestResult<()> {
            *self
                .instance_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            Ok(())
        }

        async fn set_test_instance_success(&self, _position: &BlockPos) -> GameTestResult<()> {
            *self
                .instance_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(Ok(()));
            Ok(())
        }

        async fn set_test_instance_failure(
            &self,
            _position: &BlockPos,
            message: &str,
            marker: Option<(BlockPos, String)>,
        ) -> GameTestResult<()> {
            *self
                .instance_state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                Some(Err((message.to_owned(), marker)));
            Ok(())
        }

        async fn trigger_test_block(&self, position: &BlockPos) -> GameTestResult<()> {
            self.test_blocks_triggered
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(*position, true);
            Ok(())
        }

        async fn reset_test_block(&self, position: &BlockPos) -> GameTestResult<()> {
            self.test_blocks_triggered
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(*position, false);
            Ok(())
        }

        async fn test_block_triggered(&self, position: &BlockPos) -> GameTestResult<bool> {
            Ok(self
                .test_blocks_triggered
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(position)
                .copied()
                .unwrap_or(false))
        }

        async fn test_block_message(&self, position: &BlockPos) -> GameTestResult<String> {
            Ok(self
                .test_block_messages
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(position)
                .cloned()
                .unwrap_or_default())
        }

        async fn surface_height(&self, _x: i32, _z: i32) -> i32 {
            64
        }
    }

    struct SilentReporter;

    impl GameTestReporter for SilentReporter {
        fn send_message(&self, _message: TextComponent) {}
    }

    #[test]
    fn parses_modern_and_legacy_structure_palette_nbt_and_empty_ids() {
        let mut modern_entry = NbtCompound::new();
        modern_entry.put_string("id", "minecraft:test_block".to_string());
        let mut props = NbtCompound::new();
        props.put_string("mode", "accept".to_string());
        modern_entry.put_compound("properties", props);

        let mut block = NbtCompound::new();
        block.put(
            "pos",
            NbtTag::List(vec![NbtTag::Int(0), NbtTag::Int(0), NbtTag::Int(0)]),
        );
        block.put_int("state", 0);

        let mut root = NbtCompound::new();
        root.put(
            "size",
            NbtTag::List(vec![NbtTag::Int(1), NbtTag::Int(1), NbtTag::Int(1)]),
        );
        root.put(
            "palette",
            NbtTag::List(vec![NbtTag::Compound(modern_entry)]),
        );
        root.put("blocks", NbtTag::List(vec![NbtTag::Compound(block)]));

        #[expect(clippy::expect_used, reason = "unit test assertion")]
        let template =
            GameTestStructureTemplate::from_nbt(&root).expect("26.3 palette should parse");
        assert_eq!(template.size(), [1, 1, 1]);
        assert_eq!(template.blocks().len(), 1);
        assert_eq!(template.blocks()[0].test_mode, Some(TestBlockMode::Accept));

        let empty_default = GameTestStructureTemplate::parse_empty_id("minecraft:empty");
        assert!(empty_default.is_some());
        if let Some(empty) = empty_default {
            assert_eq!(empty.size(), [1, 1, 1]);
        }

        let empty_sized = GameTestStructureTemplate::parse_empty_id("pumpkin:empty_5x3x7");
        assert!(empty_sized.is_some());
        if let Some(empty) = empty_sized {
            assert_eq!(empty.size(), [5, 3, 7]);
        }
    }

    #[tokio::test]
    async fn executes_function_test_sequence_and_completes_on_schedule() {
        let world: Arc<dyn GameTestWorld> = Arc::new(InMemoryTestWorld::default());
        #[expect(clippy::expect_used, reason = "unit test assertion")]
        let template =
            Arc::new(GameTestStructureTemplate::empty([3, 3, 3]).expect("3x3x3 empty template"));
        let definition = GameTestDefinition {
            instance_type: TestType::Function,
            environment: serde_json::Value::String("minecraft:default".to_string()),
            function: Some("test:seq".to_string()),
            structure: "minecraft:empty_3x3x3".to_string(),
            max_ticks: 10,
            setup_ticks: 0,
            required: true,
            rotation: GameTestRotation::None,
            manual_only: false,
            max_attempts: 1,
            required_successes: 1,
            sky_access: false,
            padding: 0,
        };

        let func: Arc<dyn GameTestFunction> = Arc::new(|helper: GameTestHelper| async move {
            let pos = BlockPos::new(1, 1, 1);
            helper
                .start_sequence()
                .then_execute(move |helper| async move {
                    helper.set_block_default(&pos, &Block::REDSTONE_BLOCK).await
                })
                .then_execute_after(2, move |helper| async move {
                    helper
                        .assert_block_present(&pos, &Block::REDSTONE_BLOCK)
                        .await
                })
                .then_succeed();
            Ok(())
        });

        let test = BlockBasedTest::new("test:seq", definition).with_function(func);
        let mut session = GameTestSession::new(test, world, template, 0, 0);

        for _ in 0..8 {
            session.tick().await;
            if matches!(session.state, GameTestState::Passed { .. }) {
                break;
            }
        }

        assert!(
            matches!(session.state, GameTestState::Passed { .. }),
            "expected Passed state, got {:?}",
            session.state
        );
    }

    #[tokio::test]
    async fn retries_flaky_test_until_required_successes_reached() {
        let attempts = Arc::new(Mutex::new(0u32));
        let world: Arc<dyn GameTestWorld> = Arc::new(InMemoryTestWorld::default());
        #[expect(clippy::expect_used, reason = "unit test assertion")]
        let template =
            Arc::new(GameTestStructureTemplate::empty([1, 1, 1]).expect("1x1x1 empty template"));
        let definition = GameTestDefinition {
            instance_type: TestType::Function,
            environment: serde_json::Value::String("minecraft:default".to_string()),
            function: Some("test:flaky".to_string()),
            structure: "minecraft:empty".to_string(),
            max_ticks: 5,
            setup_ticks: 0,
            required: true,
            rotation: GameTestRotation::None,
            manual_only: false,
            max_attempts: 3,
            required_successes: 2,
            sky_access: false,
            padding: 0,
        };

        let attempts_clone = attempts.clone();
        let func: Arc<dyn GameTestFunction> = Arc::new(move |helper: GameTestHelper| {
            let attempts = attempts_clone.clone();
            async move {
                let current = {
                    let mut guard = attempts
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    *guard += 1;
                    *guard
                };
                if current == 1 {
                    Err(helper.fail("First attempt fails"))
                } else {
                    helper.succeed();
                    Ok(())
                }
            }
        });

        let test = BlockBasedTest::new("test:flaky", definition).with_function(func);
        let session = GameTestSession::new(test, world, template, 0, 0);
        let reporter: Arc<dyn GameTestReporter> = Arc::new(SilentReporter);
        let report = Arc::new(GameTestBatchReport::new(reporter.clone(), 1));
        let mut runner = GameTestRunner::new();
        runner.enqueue(GameTestManager::new(
            session,
            GameTestRetryOptions::new(1, true),
            report.clone(),
            reporter,
        ));

        for _ in 0..20 {
            runner.tick().await;
            if runner.is_empty() {
                break;
            }
        }

        assert!(report.is_done());
        assert!(report.all_required_passed());
        assert_eq!(report.failed_required(), 0);
    }
}
