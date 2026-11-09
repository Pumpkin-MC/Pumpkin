pub mod block_based;
pub mod error;
pub mod helper;
pub mod manager;
pub mod model;
pub mod runner;
pub mod selector;
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
    GameTestBatchReport, GameTestManager, GameTestOutcome, GameTestOutcomeStatus, GameTestReporter,
    GameTestRetryOptions, GameTestRunner, VERIFY_TEST_BATCH_SIZE,
};
pub use model::{GameTestDefinition, GameTestRotation, TestType};
pub use runner::{GameTestSession, GameTestState, TestRunner};
pub use selector::resource_selector_matches;
pub use structure::{
    GameTestPosition, GameTestStructureBlock, GameTestStructureEntity, GameTestStructureTemplate,
    TestBlockMode,
    TestStructureInstance, clear_placed_test, clear_structure_area, encase_structure,
    place_existing_structure, place_structure, place_structure_with_controller_rotation,
    remove_barriers,
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
    use pumpkin_world::{CURRENT_MC_DATA_VERSION, world::BlockFlags};

    use super::*;

    type InstanceState = Option<Result<(), (String, Option<(BlockPos, String)>)>>;

    #[derive(Default)]
    struct InMemoryTestWorld {
        blocks: Mutex<HashMap<BlockPos, BlockStateId>>,
        test_blocks_triggered: Mutex<HashMap<BlockPos, bool>>,
        test_block_messages: Mutex<HashMap<BlockPos, String>>,
        instance_state: Mutex<InstanceState>,
        spawned_entities: Mutex<Vec<NbtCompound>>,
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

        async fn spawn_structure_entity(&self, nbt: &NbtCompound) -> GameTestResult<()> {
            self.spawned_entities
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(nbt.clone());
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
    fn cancelled_optional_request_does_not_fail_required_summary() {
        let report = GameTestBatchReport::new(Arc::new(SilentReporter), 1);
        report.cancel_before_start("test", false);
        assert!(report.is_done());
        assert_eq!(report.failed_required(), 0);
        assert_eq!(report.failed_optional(), 1);
    }

    #[test]
    fn parses_modern_structure_palette_nbt_and_empty_ids() {
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
        root.put_int("DataVersion", CURRENT_MC_DATA_VERSION);
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
        assert_eq!(report.total_runs(), 3);
        let outcomes = report.outcomes();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].id, "test:flaky");
        assert_eq!(outcomes[0].status, GameTestOutcomeStatus::Passed);
        assert_eq!(outcomes[0].attempts, 3);
        assert_eq!(outcomes[0].successes, 2);
        assert!(outcomes[0].error.is_none());
    }

    #[tokio::test]
    async fn stopping_test_clears_placement_and_records_one_cancellation() -> GameTestResult<()> {
        let world = Arc::new(InMemoryTestWorld::default());
        let template = Arc::new(GameTestStructureTemplate::new(
            [1, 1, 1],
            vec![GameTestStructureBlock {
                position: [0, 0, 0],
                state: Block::STONE.default_state.id,
                nbt: None,
                test_mode: None,
            }],
        )?);
        let definition = GameTestDefinition {
            instance_type: TestType::BlockBased,
            environment: serde_json::Value::String("minecraft:default".to_string()),
            function: None,
            structure: "test:stop".to_string(),
            max_ticks: 20,
            setup_ticks: 0,
            required: true,
            rotation: GameTestRotation::None,
            manual_only: false,
            max_attempts: 1,
            required_successes: 1,
            sky_access: false,
            padding: 0,
        };
        let test = BlockBasedTest::new("test:stop", definition);
        let session = GameTestSession::new(test, world.clone(), template, 0, 0);
        let reporter: Arc<dyn GameTestReporter> = Arc::new(SilentReporter);
        let report = Arc::new(GameTestBatchReport::new(reporter.clone(), 1));
        let mut runner = GameTestRunner::new();
        runner.enqueue(GameTestManager::new(
            session,
            GameTestRetryOptions::no_retries(),
            report.clone(),
            reporter,
        ));

        runner.tick().await;
        assert_eq!(runner.active_len(), 1);
        assert!(!report.is_done());
        assert_eq!(
            world.block_state_id(&BlockPos::new(0, 66, 1)).await,
            Block::STONE.default_state.id
        );
        assert!(
            world
                .blocks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .values()
                .any(|state| *state == Block::BARRIER.default_state.id)
        );

        assert!(runner.cancel_and_clear().await.is_empty());
        assert!(runner.cancel_and_clear().await.is_empty());
        assert!(runner.is_empty());
        assert!(report.is_done());
        assert_eq!(report.remaining_tests(), 0);
        assert_eq!(report.total_runs(), 1);
        let outcomes = report.outcomes();
        assert_eq!(outcomes.len(), 1);
        assert_eq!(outcomes[0].id, "test:stop");
        assert_eq!(outcomes[0].status, GameTestOutcomeStatus::Cancelled);
        assert_eq!(outcomes[0].attempts, 0);
        assert!(
            world
                .blocks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .values()
                .all(|state| *state == Block::AIR.default_state.id)
        );
        Ok(())
    }

    #[tokio::test]
    async fn places_rotated_structure_entity_with_transformed_nbt() -> GameTestResult<()> {
        let mut entity_nbt = NbtCompound::new();
        entity_nbt.put_string("id", "minecraft:armor_stand".to_string());
        entity_nbt.put_string("UUID", "old-uuid".to_string());
        entity_nbt.put("block_pos", NbtTag::IntArray(vec![0, 1, 1]));
        entity_nbt.put(
            "Rotation",
            NbtTag::List(vec![NbtTag::Float(45.0), NbtTag::Float(10.0)]),
        );

        let mut entity = NbtCompound::new();
        entity.put(
            "pos",
            NbtTag::List(vec![
                NbtTag::Double(0.25),
                NbtTag::Double(1.0),
                NbtTag::Double(1.5),
            ]),
        );
        entity.put(
            "blockPos",
            NbtTag::List(vec![NbtTag::Int(0), NbtTag::Int(1), NbtTag::Int(1)]),
        );
        entity.put_compound("nbt", entity_nbt);

        let mut root = NbtCompound::new();
        root.put_int("DataVersion", CURRENT_MC_DATA_VERSION);
        root.put(
            "size",
            NbtTag::List(vec![NbtTag::Int(3), NbtTag::Int(2), NbtTag::Int(4)]),
        );
        root.put("palette", NbtTag::List(Vec::new()));
        root.put("blocks", NbtTag::List(Vec::new()));
        root.put("entities", NbtTag::List(vec![NbtTag::Compound(entity)]));

        let template = GameTestStructureTemplate::from_nbt(&root)?;
        let world = InMemoryTestWorld::default();
        place_structure_with_controller_rotation(
            &world,
            &template,
            "test:entity",
            GameTestRotation::Clockwise90,
            GameTestRotation::None,
            GameTestPosition::new(10, Some(70), 20),
            0,
        )
        .await?;
        let entities = world
            .spawned_entities
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(entities.len(), 1);
        let placed = &entities[0];
        assert_eq!(placed.get_string("id"), Some("minecraft:armor_stand"));
        assert!(placed.get("UUID").is_none());
        assert_eq!(
            placed.get("block_pos"),
            Some(&NbtTag::IntArray(vec![12, 72, 21]))
        );
        assert_eq!(
            placed.get_list("Pos"),
            Some(
                &[
                    NbtTag::Double(12.5),
                    NbtTag::Double(72.0),
                    NbtTag::Double(21.25)
                ][..]
            )
        );
        assert_eq!(
            placed.get_list("Rotation"),
            Some(&[NbtTag::Float(135.0), NbtTag::Float(10.0)][..])
        );
        Ok(())
    }

    #[tokio::test]
    async fn reruns_follow_completion_order_after_the_current_wave() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let world: Arc<dyn GameTestWorld> = Arc::new(InMemoryTestWorld::default());
        #[expect(clippy::expect_used, reason = "unit test fixture")]
        let template =
            Arc::new(GameTestStructureTemplate::empty([1, 1, 1]).expect("1x1x1 empty template"));
        let reporter: Arc<dyn GameTestReporter> = Arc::new(SilentReporter);
        let report = Arc::new(GameTestBatchReport::new(reporter.clone(), 2));
        let mut runner = GameTestRunner::new();

        for (index, (name, setup_ticks)) in [("slow", 2), ("fast", 0)].into_iter().enumerate() {
            let definition = GameTestDefinition {
                instance_type: TestType::Function,
                environment: serde_json::Value::String("minecraft:default".to_string()),
                function: Some(format!("test:{name}")),
                structure: "minecraft:empty".to_string(),
                max_ticks: 10,
                setup_ticks,
                required: true,
                rotation: GameTestRotation::None,
                manual_only: false,
                max_attempts: 1,
                required_successes: 1,
                sky_access: false,
                padding: 0,
            };
            let calls = calls.clone();
            let function: Arc<dyn GameTestFunction> = Arc::new(move |helper: GameTestHelper| {
                let calls = calls.clone();
                async move {
                    calls
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .push(name);
                    helper.succeed();
                    Ok(())
                }
            });
            let test =
                BlockBasedTest::new(format!("test:{name}"), definition).with_function(function);
            let session =
                GameTestSession::new(test, world.clone(), template.clone(), index as i32 * 8, 0);
            runner.enqueue(GameTestManager::new(
                session,
                GameTestRetryOptions::new(2, false),
                report.clone(),
                reporter.clone(),
            ));
        }

        for _ in 0..30 {
            runner.tick().await;
            if runner.is_empty() {
                break;
            }
        }
        assert!(report.is_done());
        let calls = calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(calls.as_slice(), ["fast", "slow", "fast", "slow"]);
    }
}
