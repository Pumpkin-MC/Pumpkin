use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, LazyLock};
use std::time::Instant;

use async_trait::async_trait;
use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_gametest::{
    BlockBasedTest, GameTestError, GameTestResult, GameTestWorld, StructureTemplate, TestRotation,
    TestRun, TestState,
};
use pumpkin_nbt::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::text::{TextComponent, color::NamedColor};
use pumpkin_world::{chunk::ChunkHeightmapType, world::BlockFlags};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{
    block::entities::{
        BlockEntity, block_entity_from_nbt, test_block::TestBlockBlockEntity,
        test_instance_block::TestInstanceBlockBlockEntity,
    },
    command::CommandSender,
    server::Server,
    world::World,
};

static GAME_TEST_QUEUE: LazyLock<Mutex<Vec<GameTestRequest>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));
static STOP_GAME_TESTS: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Copy, Debug)]
pub struct GameTestRetryOptions {
    number_of_tries: i32,
    halt_on_failure: bool,
}

impl GameTestRetryOptions {
    #[must_use]
    pub const fn new(number_of_tries: i32, halt_on_failure: bool) -> Self {
        Self {
            number_of_tries,
            halt_on_failure,
        }
    }

    #[must_use]
    const fn has_retries(self) -> bool {
        self.number_of_tries != 1
    }

    #[must_use]
    const fn unlimited_tries(self) -> bool {
        self.number_of_tries < 1
    }

    #[must_use]
    fn has_tries_left(self, attempts: u32, successes: u32) -> bool {
        // Exact RetryOptions::hasTriesLeft semantics from vanilla.
        let has_failures = attempts != successes;
        let has_more_attempts = self.unlimited_tries()
            || attempts < u32::try_from(self.number_of_tries).unwrap_or(u32::MAX);
        has_more_attempts && (!has_failures || !self.halt_on_failure)
    }
}

pub struct GameTestBatchReport {
    sender: CommandSender,
    remaining_tests: AtomicUsize,
    total_runs: AtomicUsize,
    failed_required: AtomicUsize,
    failed_optional: AtomicUsize,
}

impl GameTestBatchReport {
    #[must_use]
    pub const fn new(sender: CommandSender, test_count: usize) -> Self {
        Self {
            sender,
            remaining_tests: AtomicUsize::new(test_count),
            total_runs: AtomicUsize::new(0),
            failed_required: AtomicUsize::new(0),
            failed_optional: AtomicUsize::new(0),
        }
    }

    fn fail_to_start(&self, error: &GameTestError) {
        self.sender
            .send_message(TextComponent::text(error.to_string()).color_named(NamedColor::Red));
        self.finish_test(true, 1, 0);
    }

    fn finish_test(&self, required: bool, attempts: u32, successes: u32) {
        let attempts = usize::try_from(attempts).unwrap_or(usize::MAX);
        let successes = usize::try_from(successes).unwrap_or(usize::MAX);
        self.total_runs.fetch_add(attempts, Ordering::AcqRel);
        let failures = attempts.saturating_sub(successes);
        if failures != 0 {
            if required {
                self.failed_required.fetch_add(failures, Ordering::AcqRel);
            } else {
                self.failed_optional.fetch_add(failures, Ordering::AcqRel);
            }
        }

        if self.remaining_tests.fetch_sub(1, Ordering::AcqRel) != 1 {
            return;
        }

        let total = self.total_runs.load(Ordering::Acquire);
        let failed_required = self.failed_required.load(Ordering::Acquire);
        let failed_optional = self.failed_optional.load(Ordering::Acquire);

        self.sender.send_message(
            TextComponent::translate_cross(
                "commands.test.summary",
                "commands.test.summary",
                [TextComponent::text(total.to_string())],
            )
            .color_named(NamedColor::White),
        );

        if failed_required != 0 {
            self.sender.send_message(
                TextComponent::translate_cross(
                    "commands.test.summary.failed",
                    "commands.test.summary.failed",
                    [TextComponent::text(failed_required.to_string())],
                )
                .color_named(NamedColor::Red),
            );
        } else {
            self.sender.send_message(
                TextComponent::translate_cross(
                    "commands.test.summary.all_required_passed",
                    "commands.test.summary.all_required_passed",
                    [],
                )
                .color_named(NamedColor::Green),
            );
        }

        if failed_optional != 0 {
            self.sender.send_message(TextComponent::translate_cross(
                "commands.test.summary.optional_failed",
                "commands.test.summary.optional_failed",
                [TextComponent::text(failed_optional.to_string())],
            ));
        }
    }
}

/// A request to start a `GameTest`.
pub struct GameTestRequest {
    test_id: String,
    world: Arc<World>,
    test_x: i32,
    test_z: i32,
    rotation_steps: i32,
    retry_options: GameTestRetryOptions,
    report: Arc<GameTestBatchReport>,
}

impl GameTestRequest {
    #[must_use]
    pub fn new(
        test_id: impl Into<String>,
        world: Arc<World>,
        test_x: i32,
        test_z: i32,
        rotation_steps: i32,
        retry_options: GameTestRetryOptions,
        report: Arc<GameTestBatchReport>,
    ) -> Self {
        Self {
            test_id: test_id.into(),
            world,
            test_x,
            test_z,
            rotation_steps,
            retry_options,
            report,
        }
    }
}

pub async fn enqueue_game_test(request: GameTestRequest) {
    GAME_TEST_QUEUE.lock().await.push(request);
}

pub async fn stop_game_tests() {
    // Keep the queue mutex held while publishing the stop request. drain_game_test_queue
    // takes the same mutex before consuming STOP_GAME_TESTS, so a stop+new-run command
    // cannot race between the runner-clear and queue-drain phases.
    let mut queue = GAME_TEST_QUEUE.lock().await;
    queue.clear();
    STOP_GAME_TESTS.store(true, Ordering::Release);
}

pub(super) struct ServerGameTestRunner {
    active: Vec<ManagedGameTest>,
}

impl ServerGameTestRunner {
    #[must_use]
    pub const fn new() -> Self {
        Self { active: Vec::new() }
    }

    fn enqueue(&mut self, run: ManagedGameTest) {
        self.active.push(run);
    }

    fn apply_stop_request(&mut self) {
        if STOP_GAME_TESTS.swap(false, Ordering::AcqRel) {
            self.active.clear();
        }
    }

    pub async fn tick(&mut self) {
        // Vanilla's GameTestTicker iterates a copy-on-write collection. A retry is
        // scheduled as a copyReset GameTestInfo and is not turned back into a queued
        // state from inside the completion callback. Mirror that two-phase lifecycle:
        // finish every current execution first, then install scheduled reruns.
        for managed in &mut self.active {
            if managed.done || managed.rerun_scheduled {
                continue;
            }

            managed.run.tick().await;
            if managed.run.state.is_finished() {
                managed.handle_completion();
            }
        }

        for managed in &mut self.active {
            managed.install_scheduled_rerun();
        }

        self.active.retain(|managed| !managed.done);
    }
}

struct ManagedGameTest {
    run: TestRun,
    world: Arc<World>,
    retry_options: GameTestRetryOptions,
    report: Arc<GameTestBatchReport>,
    attempts: u32,
    successes: u32,
    started_at: Instant,
    rerun_scheduled: bool,
    done: bool,
}

impl ManagedGameTest {
    #[expect(clippy::too_many_lines)]
    fn handle_completion(&mut self) {
        let (passed, tick, error) = match &self.run.state {
            TestState::Passed { tick } => (true, *tick, None),
            TestState::Failed { tick, error } => (false, *tick, Some(error)),
            _ => return,
        };

        self.attempts = self.attempts.saturating_add(1);
        if passed {
            self.successes = self.successes.saturating_add(1);
        }
        let elapsed_ms = self.started_at.elapsed().as_millis();
        let is_flaky = self.run.test.max_attempts() > 1;

        // This intentionally follows ReportGameListener's ordering. Command retry
        // options take precedence for a passing execution. Flaky failure handling,
        // however, uses max_attempts/required_successes exactly as vanilla does.
        let should_rerun = if passed {
            if self.retry_options.has_retries() {
                broadcast_world(
                    &self.world,
                    &TextComponent::text(self.retry_status(true, elapsed_ms))
                        .color_named(NamedColor::Green),
                );
                self.retry_options
                    .has_tries_left(self.attempts, self.successes)
            } else if !is_flaky {
                broadcast_world(
                    &self.world,
                    &TextComponent::text(format!(
                        "{} passed! ({}ms / {}gameticks)",
                        self.run.test.id(),
                        elapsed_ms,
                        tick
                    ))
                    .color_named(NamedColor::Green),
                );
                false
            } else if self.successes >= self.run.test.required_successes() {
                broadcast_world(
                    &self.world,
                    &TextComponent::text(format!(
                        "{} passed {} times of {} attempts.",
                        self.run.test.id(),
                        self.successes,
                        self.attempts
                    ))
                    .color_named(NamedColor::Green),
                );
                false
            } else {
                broadcast_world(
                    &self.world,
                    &TextComponent::text(format!(
                        "Flaky test {} succeeded, attempt: {} successes: {}",
                        self.run.test.id(),
                        self.attempts,
                        self.successes
                    ))
                    .color_named(NamedColor::Green),
                );
                true
            }
        } else if !is_flaky {
            let error_message = error.map(ToString::to_string);
            self.report_failure(error_message.as_deref());
            if self.retry_options.has_retries() {
                broadcast_world(
                    &self.world,
                    &TextComponent::text(self.retry_status(false, elapsed_ms))
                        .color_named(NamedColor::Red),
                );
                self.retry_options
                    .has_tries_left(self.attempts, self.successes)
            } else {
                false
            }
        } else {
            let max_attempts = self.run.test.max_attempts();
            let required_successes = self.run.test.required_successes();
            let successes_detail = if required_successes > 1 {
                format!(
                    ", successes: {} ({} required)",
                    self.successes, required_successes
                )
            } else {
                String::new()
            };
            let text = format!(
                "Flaky test {} failed, attempt: {}/{}{successes_detail}",
                self.run.test.id(),
                self.attempts,
                max_attempts
            );
            broadcast_world(
                &self.world,
                &TextComponent::text(text).color_named(NamedColor::Yellow),
            );

            if max_attempts
                .saturating_sub(self.attempts)
                .saturating_add(self.successes)
                >= required_successes
            {
                true
            } else {
                let last_error =
                    error.map_or_else(|| "unknown error".to_string(), ToString::to_string);
                let exhausted = GameTestError::ExhaustedAttempts {
                    attempts: self.attempts,
                    successes: self.successes,
                    required_successes,
                    last_error,
                };
                let exhausted_message = exhausted.to_string();
                self.report_failure(Some(&exhausted_message));
                false
            }
        };

        if should_rerun {
            self.rerun_scheduled = true;
            return;
        }

        self.report
            .finish_test(self.run.test.is_required(), self.attempts, self.successes);
        self.done = true;
    }

    fn install_scheduled_rerun(&mut self) {
        if !self.rerun_scheduled || self.done {
            return;
        }

        self.run = self.run.copy_reset();
        self.started_at = Instant::now();
        self.rerun_scheduled = false;
    }

    fn report_failure(&self, error_message: Option<&str>) {
        let optional = if self.run.test.is_required() {
            ""
        } else {
            "(optional) "
        };
        let text = format!(
            "{}{} failed! {}",
            optional,
            self.run.test.id(),
            error_message.unwrap_or("unknown error")
        );
        let color = if self.run.test.is_required() {
            NamedColor::Red
        } else {
            NamedColor::Yellow
        };
        broadcast_world(&self.world, &TextComponent::text(text).color_named(color));
    }

    fn retry_status(&self, passed: bool, elapsed_ms: u128) -> String {
        let failures = self.attempts.saturating_sub(self.successes);
        let tries_left = if self.retry_options.unlimited_tries() {
            String::new()
        } else {
            let left = u32::try_from(self.retry_options.number_of_tries)
                .unwrap_or_default()
                .saturating_sub(self.attempts);
            format!(", Left: {left:4}")
        };
        let report = format!(
            "[Run: {:4}, Ok: {:4}, Fail: {:4}{tries_left}]",
            self.attempts, self.successes, failures
        );
        let name = format!(
            "{} {}! {}ms",
            self.run.test.id(),
            if passed { "passed" } else { "failed" },
            elapsed_ms
        );
        format!("{report:<53}{name}")
    }
}

pub(super) async fn drain_game_test_queue(server: &Arc<Server>, runner: &mut ServerGameTestRunner) {
    // Hold the same queue mutex used by stop_game_tests while consuming the stop
    // flag and draining requests. This closes the async race where a new /test run
    // could otherwise be drained before the old runner was cleared.
    let queued = {
        let mut queue = GAME_TEST_QUEUE.lock().await;
        runner.apply_stop_request();
        std::mem::take(&mut *queue)
    };

    for request in queued {
        let test_id = request.test_id.clone();
        let report = request.report.clone();
        match prepare_test_run(server, request).await {
            Ok(run) => {
                info!(target: "pumpkin::gametest", test = %test_id, "Starting queued GameTest");
                runner.enqueue(run);
            }
            Err(error) => {
                warn!(
                    target: "pumpkin::gametest",
                    test = %test_id,
                    error = %error,
                    "Unable to start queued GameTest"
                );
                report.fail_to_start(&error);
            }
        }
    }
}

async fn prepare_test_run(
    server: &Arc<Server>,
    request: GameTestRequest,
) -> GameTestResult<ManagedGameTest> {
    let test_instance = server
        .datapack_manager
        .get_test_instance(&request.test_id)
        .ok_or_else(|| {
            GameTestError::World(format!("Unknown test instance '{}'", request.test_id))
        })?;

    let structure = server
        .datapack_manager
        .load_structure(&test_instance.structure)
        .await
        .map_err(GameTestError::World)?;
    let template = StructureTemplate::from_nbt(&structure)?;
    let test = BlockBasedTest::new(request.test_id, test_instance);
    let adapter_world: Arc<dyn GameTestWorld> = Arc::new(ServerGameTestWorld {
        world: request.world.clone(),
    });
    let extra_rotation = TestRotation::from_steps(request.rotation_steps);

    Ok(ManagedGameTest {
        run: TestRun::new_with_extra_rotation(
            test,
            adapter_world,
            Arc::new(template),
            request.test_x,
            request.test_z,
            extra_rotation,
        ),
        world: request.world,
        retry_options: request.retry_options,
        report: request.report,
        attempts: 0,
        successes: 0,
        started_at: Instant::now(),
        rerun_scheduled: false,
        done: false,
    })
}

fn broadcast_world(world: &World, message: &TextComponent) {
    let players = world.players.load_full();
    for player in players.iter() {
        player.send_system_message(message);
    }
}

struct ServerGameTestWorld {
    world: Arc<World>,
}

impl ServerGameTestWorld {
    fn test_block_entity(&self, position: &BlockPos) -> GameTestResult<Arc<TestBlockBlockEntity>> {
        let entity = self.world.get_block_entity(position).ok_or_else(|| {
            GameTestError::World(format!("Missing test block entity at {position}"))
        })?;

        Arc::downcast::<TestBlockBlockEntity>(entity).map_err(|_| {
            GameTestError::World(format!("Block entity at {position} is not a test block"))
        })
    }

    fn test_instance_block_entity(
        &self,
        position: &BlockPos,
    ) -> GameTestResult<Arc<TestInstanceBlockBlockEntity>> {
        let entity = self.world.get_block_entity(position).ok_or_else(|| {
            GameTestError::World(format!("Missing test instance block entity at {position}"))
        })?;

        Arc::downcast::<TestInstanceBlockBlockEntity>(entity).map_err(|_| {
            GameTestError::World(format!(
                "Block entity at {position} is not a test instance block"
            ))
        })
    }

    fn sync_block_entity<T: BlockEntity + 'static>(&self, entity: Arc<T>) {
        let entity: Arc<dyn BlockEntity> = entity;
        self.world.update_block_entity(&entity);
    }
}

#[async_trait]
impl GameTestWorld for ServerGameTestWorld {
    async fn block_state_id(&self, position: &BlockPos) -> BlockStateId {
        self.world.get_block_state_id_async(position).await
    }

    async fn set_block_state(
        &self,
        position: &BlockPos,
        block_state_id: BlockStateId,
        flags: BlockFlags,
    ) -> GameTestResult<()> {
        self.world.set_block_state(position, block_state_id, flags);
        Ok(())
    }

    async fn rotate_block_state(
        &self,
        block_state_id: BlockStateId,
        rotation: TestRotation,
    ) -> GameTestResult<BlockStateId> {
        let (block, _) = BlockState::from_id_with_block(block_state_id);
        Ok(self
            .world
            .block_registry
            .rotate(block, block_state_id, rotation.as_block_rotation())
            .id)
    }

    async fn set_block_entity_nbt(
        &self,
        position: &BlockPos,
        nbt: &NbtCompound,
    ) -> GameTestResult<()> {
        let mut nbt = nbt.clone();
        nbt.put_int("x", position.0.x);
        nbt.put_int("y", position.0.y);
        nbt.put_int("z", position.0.z);

        let entity = block_entity_from_nbt(&nbt).ok_or_else(|| {
            let id = nbt.get_string("id").unwrap_or("<missing id>");
            GameTestError::World(format!(
                "Unable to create block entity '{id}' at {position}"
            ))
        })?;

        self.world.remove_block_entity(position);
        self.world.add_block_entity(entity);
        Ok(())
    }

    async fn clear_non_player_entities(
        &self,
        min: &BlockPos,
        max: &BlockPos,
    ) -> GameTestResult<()> {
        let min_x = f64::from(min.0.x);
        let min_y = f64::from(min.0.y);
        let min_z = f64::from(min.0.z);
        let max_x = f64::from(max.0.x);
        let max_y = f64::from(max.0.y);
        let max_z = f64::from(max.0.z);

        // World::entities intentionally excludes players, matching vanilla's
        // `removeEntities` filter while avoiding any player removal path entirely.
        let entities = self.world.entities.load_full();
        let to_remove: Vec<_> = entities
            .iter()
            .filter(|entity| {
                let bounds = entity.get_entity().bounding_box.load();
                bounds.max.x > min_x
                    && bounds.min.x < max_x
                    && bounds.max.y > min_y
                    && bounds.min.y < max_y
                    && bounds.max.z > min_z
                    && bounds.min.z < max_z
            })
            .cloned()
            .collect();
        drop(entities);

        for entity in to_remove {
            self.world.remove_entity(entity.as_ref());
        }
        Ok(())
    }

    async fn clear_scheduled_block_ticks(
        &self,
        min: &BlockPos,
        max: &BlockPos,
    ) -> GameTestResult<()> {
        if max.0.x <= min.0.x || max.0.y <= min.0.y || max.0.z <= min.0.z {
            return Ok(());
        }

        let min_chunk_x = min.0.x >> 4;
        let max_chunk_x = (max.0.x - 1) >> 4;
        let min_chunk_z = min.0.z >> 4;
        let max_chunk_z = (max.0.z - 1) >> 4;

        for chunk_x in min_chunk_x..=max_chunk_x {
            for chunk_z in min_chunk_z..=max_chunk_z {
                let chunk_pos = pumpkin_util::math::vector2::Vector2::new(chunk_x, chunk_z);
                if let Some(chunk) = self.world.level.loaded_chunks.get(&chunk_pos) {
                    chunk.block_ticks.clear_area(min, max);
                    if !chunk.block_ticks.has_ticks() && !chunk.fluid_ticks.has_ticks() {
                        self.world
                            .level
                            .chunks_with_scheduled_ticks
                            .remove(&chunk_pos);
                    }
                }
            }
        }
        Ok(())
    }

    async fn clear_block_events(&self, min: &BlockPos, max: &BlockPos) -> GameTestResult<()> {
        self.world.clear_synced_block_events_in_box(min, max);
        Ok(())
    }

    async fn set_test_instance_running(&self, position: &BlockPos) -> GameTestResult<()> {
        let entity = self.test_instance_block_entity(position)?;
        entity.clear_error_markers();
        entity.set_running();
        self.sync_block_entity(entity);
        Ok(())
    }

    async fn set_test_instance_success(&self, position: &BlockPos) -> GameTestResult<()> {
        let entity = self.test_instance_block_entity(position)?;
        entity.clear_error_markers();
        entity.set_success();
        self.sync_block_entity(entity);
        Ok(())
    }

    async fn set_test_instance_failure(
        &self,
        position: &BlockPos,
        message: &str,
        marker: Option<(BlockPos, String)>,
    ) -> GameTestResult<()> {
        let entity = self.test_instance_block_entity(position)?;
        entity.clear_error_markers();
        if let Some((marker_position, marker_text)) = marker {
            entity.mark_error(marker_position, marker_text);
        }
        entity.set_error_message(message.to_string());
        self.sync_block_entity(entity);
        Ok(())
    }

    async fn trigger_test_block(&self, position: &BlockPos) -> GameTestResult<()> {
        self.test_block_entity(position)?.trigger(&self.world);
        Ok(())
    }

    async fn reset_test_block(&self, position: &BlockPos) -> GameTestResult<()> {
        self.test_block_entity(position)?.reset(&self.world);
        Ok(())
    }

    async fn test_block_triggered(&self, position: &BlockPos) -> GameTestResult<bool> {
        Ok(self.test_block_entity(position)?.has_triggered())
    }

    async fn test_block_message(&self, position: &BlockPos) -> GameTestResult<String> {
        Ok(self.test_block_entity(position)?.message())
    }

    async fn surface_height(&self, x: i32, z: i32) -> i32 {
        self.world
            .get_heightmap_height_async(ChunkHeightmapType::WorldSurface, x, z)
            .await
    }
}
