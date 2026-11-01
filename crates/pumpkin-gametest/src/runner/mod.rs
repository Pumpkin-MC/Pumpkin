mod state;

pub use state::GameTestState;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use pumpkin_util::math::position::BlockPos;

use crate::block_based::BlockBasedTest;
use crate::error::{GameTestError, GameTestResult};
use crate::helper::{
    GameTestControllerState, GameTestHelper, SequenceEvent, SequenceEventKind, get_test_function,
};
use crate::model::{GameTestRotation, TestType};
use crate::structure::{
    GameTestPosition, GameTestStructureTemplate, TestBlockMode, TestStructureInstance,
    clear_success_entities, encase_structure, place_structure_with_controller_rotation,
    remove_barriers,
};
use crate::world::GameTestWorld;

enum RunningEvaluation {
    Continue,
    Passed,
    Failed(GameTestError),
}

pub struct GameTestSession {
    pub test: BlockBasedTest,
    pub state: GameTestState,
    pub placement: Option<TestStructureInstance>,
    world: Arc<dyn GameTestWorld>,
    template: Arc<GameTestStructureTemplate>,
    extra_rotation: GameTestRotation,
    effective_rotation: GameTestRotation,
    test_x: i32,
    test_y: Option<i32>,
    test_z: i32,
    chunks_loaded: bool,
    started_at: Option<Instant>,
    controller: Arc<Mutex<GameTestControllerState>>,
}

impl GameTestSession {
    #[must_use]
    pub fn new(
        test: BlockBasedTest,
        world: Arc<dyn GameTestWorld>,
        template: Arc<GameTestStructureTemplate>,
        test_x: i32,
        test_z: i32,
    ) -> Self {
        Self::new_with_extra_rotation(
            test,
            world,
            template,
            test_x,
            test_z,
            GameTestRotation::None,
        )
    }

    #[must_use]
    pub fn new_with_extra_rotation(
        test: BlockBasedTest,
        world: Arc<dyn GameTestWorld>,
        template: Arc<GameTestStructureTemplate>,
        test_x: i32,
        test_z: i32,
        extra_rotation: GameTestRotation,
    ) -> Self {
        let effective_rotation = test.rotation().then(extra_rotation);
        Self {
            test,
            state: GameTestState::Queued,
            placement: None,
            world,
            template,
            extra_rotation,
            effective_rotation,
            test_x,
            test_y: None,
            test_z,
            chunks_loaded: false,
            started_at: None,
            controller: Arc::new(Mutex::new(GameTestControllerState::default())),
        }
    }

    #[must_use]
    pub const fn with_y(mut self, test_y: Option<i32>) -> Self {
        self.test_y = test_y;
        self
    }

    /// Creates the equivalent of vanilla `GameTestInfo::copyReset()`.
    ///
    /// A rerun is a new execution object, not a finished run mutated back to Queued.
    /// The controller coordinates and resolved Y are retained so the replacement is
    /// prepared in place, while all per-attempt state and placement handles are fresh.
    #[must_use]
    pub fn copy_reset(&self) -> Self {
        Self {
            test: self.test.clone(),
            state: GameTestState::Queued,
            placement: None,
            world: self.world.clone(),
            template: self.template.clone(),
            extra_rotation: self.extra_rotation,
            effective_rotation: self.effective_rotation,
            test_x: self.test_x,
            test_y: self.test_y,
            test_z: self.test_z,
            chunks_loaded: false,
            started_at: None,
            controller: Arc::new(Mutex::new(GameTestControllerState::default())),
        }
    }

    #[must_use]
    pub(crate) fn run_time_ms(&self) -> u128 {
        self.started_at
            .as_ref()
            .map_or(0, |started_at| started_at.elapsed().as_millis())
    }

    pub async fn tick(&mut self) {
        if self.state.is_finished() {
            return;
        }

        // Move the current state out so state transitions can freely borrow `self`
        // across async calls without holding a borrow into `self.state`.
        let state = std::mem::replace(&mut self.state, GameTestState::Queued);
        match state {
            GameTestState::Queued => self.tick_queued().await,
            GameTestState::SettingUp { elapsed_ticks } => self.tick_setup(elapsed_ticks).await,
            GameTestState::Running { elapsed_ticks } => self.tick_running(elapsed_ticks).await,
            finished @ (GameTestState::Passed { .. } | GameTestState::Failed { .. }) => {
                self.state = finished;
            }
        }
    }

    async fn tick_queued(&mut self) {
        let placement = place_structure_with_controller_rotation(
            self.world.as_ref(),
            &self.template,
            self.test.id(),
            self.effective_rotation,
            self.extra_rotation,
            GameTestPosition::new(self.test_x, self.test_y, self.test_z),
            self.test.definition().padding,
        )
        .await;

        match placement {
            Ok(placement) => {
                if let Err(error) = encase_structure(
                    self.world.as_ref(),
                    &placement,
                    self.test.definition().sky_access,
                )
                .await
                {
                    self.finish_failure(0, error, None).await;
                    return;
                }

                self.test_y = Some(placement.test_instance_pos().0.y);
                self.placement = Some(placement);
                // Vanilla's StructureSpawner calls startExecution(1), so even a test
                // with zero setup ticks waits until the next server tick to start.
                self.state = GameTestState::SettingUp { elapsed_ticks: 0 };
            }
            Err(error) => self.finish_failure(0, error, None).await,
        }
    }

    async fn tick_setup(&mut self, elapsed_ticks: u32) {
        // GameTestInfo::tick does not advance tickCount until every chunk intersecting
        // the placed structure is actually loaded and ticking. This check is one-shot
        // per attempt, exactly like vanilla's chunksLoaded flag.
        if !self.chunks_loaded {
            let Some(placement) = &self.placement else {
                self.finish_failure(
                    0,
                    GameTestError::World(
                        "GameTest is ticking without a placed structure".to_string(),
                    ),
                    None,
                )
                .await;
                return;
            };
            let origin = placement.origin();
            let size = placement.size();
            let max = BlockPos::new(
                origin.0.x + size[0],
                origin.0.y + size[1],
                origin.0.z + size[2],
            );
            if !self.world.test_area_loaded_and_ticking(origin, &max).await {
                self.state = GameTestState::SettingUp { elapsed_ticks };
                return;
            }
            self.chunks_loaded = true;
        }

        let elapsed_ticks = elapsed_ticks.saturating_add(1);
        if elapsed_ticks <= self.test.setup_ticks() {
            self.state = GameTestState::SettingUp { elapsed_ticks };
            return;
        }

        match self.begin_running(0).await {
            Ok(()) if self.test.max_ticks() > 0 => self.evaluate_test_tick(0).await,
            Ok(()) => self.state = GameTestState::Running { elapsed_ticks: 0 },
            Err(error) => {
                let marker = assertion_marker(&error);
                self.finish_failure(0, error, marker).await;
            }
        }
    }

    async fn tick_running(&mut self, elapsed_ticks: u32) {
        let tick = elapsed_ticks.saturating_add(1);
        if tick > self.test.max_ticks() {
            if self.test.instance_type() == TestType::Function {
                match self.evaluate_function_tick(tick, true).await {
                    Ok(RunningEvaluation::Passed) => {
                        self.handle_attempt_pass(tick).await;
                        return;
                    }
                    Ok(RunningEvaluation::Failed(error)) | Err(error) => {
                        let marker = assertion_marker(&error);
                        self.finish_failure(tick, error, marker).await;
                        return;
                    }
                    Ok(RunningEvaluation::Continue) => {}
                }
            }

            self.finish_failure(
                tick,
                GameTestError::Timeout {
                    max_ticks: self.test.max_ticks(),
                },
                None,
            )
            .await;
            return;
        }

        // BlockBasedTestInstance installs onEachTick for the half-open range
        // [0, timeoutTicks), so timeoutTicks itself has no ACCEPT/FAIL/LOG check.
        if self.test.instance_type() == TestType::BlockBased && tick == self.test.max_ticks() {
            self.state = GameTestState::Running {
                elapsed_ticks: tick,
            };
            return;
        }

        self.evaluate_test_tick(tick).await;
    }

    async fn evaluate_test_tick(&mut self, tick: u32) {
        let evaluation = match self.test.instance_type() {
            TestType::BlockBased => self.evaluate_running(tick).await,
            TestType::Function => self.evaluate_function_tick(tick, false).await,
        };

        match evaluation {
            Ok(RunningEvaluation::Passed) => self.handle_attempt_pass(tick).await,
            Ok(RunningEvaluation::Failed(error)) | Err(error) => {
                let marker = assertion_marker(&error);
                self.finish_failure(tick, error, marker).await;
            }
            Ok(RunningEvaluation::Continue) => {
                self.state = GameTestState::Running {
                    elapsed_ticks: tick,
                };
            }
        }
    }

    async fn begin_running(&mut self, tick: u32) -> GameTestResult<()> {
        // Vanilla starts GameTestInfo's stopwatch immediately before invoking the
        // test body, so structure placement and setup ticks are not part of run time.
        self.started_at.get_or_insert_with(Instant::now);
        *self
            .controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            GameTestControllerState::default();

        if let Some(placement) = &self.placement {
            // GameTestInfo.startTest marks the controller RUNNING immediately before
            // invoking GameTestInstance.run.
            self.world
                .set_test_instance_running(placement.test_instance_pos())
                .await?;
        }

        match self.test.instance_type() {
            TestType::BlockBased => {
                let start_blocks = self.test_block_positions(TestBlockMode::Start);
                if start_blocks.is_empty() {
                    return Err(GameTestError::Assertion {
                        tick,
                        position: None,
                        message: "missing START test block".to_string(),
                    });
                }
                if start_blocks.len() != 1 {
                    return Err(GameTestError::Assertion {
                        tick,
                        position: None,
                        message: format!(
                            "expected exactly one START test block, found {}",
                            start_blocks.len()
                        ),
                    });
                }

                self.world.trigger_test_block(&start_blocks[0]).await
            }
            TestType::Function => {
                let function = self
                    .test
                    .function()
                    .cloned()
                    .or_else(|| {
                        self.test
                            .definition()
                            .function
                            .as_deref()
                            .and_then(get_test_function)
                    })
                    .or_else(|| get_test_function(self.test.id()))
                    .ok_or_else(|| {
                        let name = self
                            .test
                            .definition()
                            .function
                            .as_deref()
                            .unwrap_or_else(|| self.test.id());
                        GameTestError::World(format!(
                            "Trying to access missing test function: {name}"
                        ))
                    })?;

                let helper = self.make_helper(tick)?;
                function.run(helper).await
            }
        }
    }

    fn make_helper(&self, tick: u32) -> GameTestResult<GameTestHelper> {
        let placement = self.placement.ok_or_else(|| {
            GameTestError::World("GameTest has no placed structure instance".to_string())
        })?;
        Ok(GameTestHelper::with_controller(
            self.world.clone(),
            placement,
            tick,
            self.test.max_ticks(),
            self.controller.clone(),
        ))
    }

    async fn evaluate_function_tick(
        &self,
        tick: u32,
        fail_if_not_complete: bool,
    ) -> GameTestResult<RunningEvaluation> {
        let helper = self.make_helper(tick)?;

        if let Some(eval) = self.check_controller_outcome() {
            return Ok(eval);
        }

        let due_callbacks = {
            let mut state = self
                .controller
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut due = Vec::new();
            let mut remaining = Vec::with_capacity(state.scheduled.len());
            for entry in state.scheduled.drain(..) {
                if entry.tick <= tick {
                    due.push(entry.callback);
                } else {
                    remaining.push(entry);
                }
            }
            state.scheduled = remaining;
            due
        };

        for mut callback in due_callbacks {
            if let Err(error) = callback(helper.with_tick(tick)).await {
                self.controller
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .fail(error);
            }
            if let Some(eval @ RunningEvaluation::Failed(_)) = self.check_controller_outcome() {
                return Ok(eval);
            }
        }

        let sequence_count = self
            .controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .sequences
            .len();

        for seq_idx in 0..sequence_count {
            if let Some(eval) = self
                .tick_sequence(seq_idx, tick, &helper, fail_if_not_complete)
                .await
            {
                return Ok(eval);
            }
        }

        if let Some(eval) = self.check_controller_outcome() {
            return Ok(eval);
        }

        Ok(RunningEvaluation::Continue)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "sequence event dispatch covers all SequenceEventKind variants"
    )]
    async fn tick_sequence(
        &self,
        seq_idx: usize,
        tick: u32,
        helper: &GameTestHelper,
        fail_if_not_complete: bool,
    ) -> Option<RunningEvaluation> {
        loop {
            let (last_tick, event) = {
                let mut state = self
                    .controller
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let seq = state.sequences.get_mut(seq_idx)?;
                let event = seq.events.pop_front()?;
                (seq.last_tick, event)
            };

            let minimum_delay = event.minimum_delay;
            let expected_delay = event.expected_delay;

            match event.kind {
                SequenceEventKind::WaitUntil(mut callback) => {
                    match callback(helper.with_tick(tick)).await {
                        Ok(()) => {
                            if let Some(failure) = self.advance_sequence_tick(
                                seq_idx,
                                last_tick,
                                tick,
                                minimum_delay,
                                expected_delay,
                            ) {
                                return Some(RunningEvaluation::Failed(failure));
                            }
                        }
                        Err(error) => {
                            self.push_front_sequence_event(
                                seq_idx,
                                SequenceEvent {
                                    minimum_delay,
                                    expected_delay,
                                    kind: SequenceEventKind::WaitUntil(callback),
                                },
                            );
                            if fail_if_not_complete {
                                return Some(RunningEvaluation::Failed(error));
                            }
                            return None;
                        }
                    }
                }
                SequenceEventKind::Execute(mut callback) => {
                    if let Err(error) = callback(helper.with_tick(tick)).await {
                        return Some(RunningEvaluation::Failed(error));
                    }
                    if let Some(failure) = self.advance_sequence_tick(
                        seq_idx,
                        last_tick,
                        tick,
                        minimum_delay,
                        expected_delay,
                    ) {
                        return Some(RunningEvaluation::Failed(failure));
                    }
                }
                SequenceEventKind::ExecuteAfter {
                    delta,
                    mut callback,
                } => {
                    if tick < last_tick.saturating_add(delta) {
                        self.push_front_sequence_event(
                            seq_idx,
                            SequenceEvent {
                                minimum_delay,
                                expected_delay,
                                kind: SequenceEventKind::ExecuteAfter { delta, callback },
                            },
                        );
                        if fail_if_not_complete {
                            return Some(RunningEvaluation::Failed(GameTestError::Assertion {
                                tick,
                                position: None,
                                message: "sequence not completed".to_string(),
                            }));
                        }
                        return None;
                    }
                    if let Err(error) = callback(helper.with_tick(tick)).await {
                        return Some(RunningEvaluation::Failed(error));
                    }
                    if let Some(failure) = self.advance_sequence_tick(
                        seq_idx,
                        last_tick,
                        tick,
                        minimum_delay,
                        expected_delay,
                    ) {
                        return Some(RunningEvaluation::Failed(failure));
                    }
                }
                SequenceEventKind::ExecuteFor {
                    delta,
                    mut callback,
                } => {
                    if tick < last_tick.saturating_add(delta) {
                        if let Err(error) = callback(helper.with_tick(tick)).await {
                            return Some(RunningEvaluation::Failed(error));
                        }
                        self.push_front_sequence_event(
                            seq_idx,
                            SequenceEvent {
                                minimum_delay,
                                expected_delay,
                                kind: SequenceEventKind::ExecuteFor { delta, callback },
                            },
                        );
                        if fail_if_not_complete {
                            return Some(RunningEvaluation::Failed(GameTestError::Assertion {
                                tick,
                                position: None,
                                message: "sequence not completed".to_string(),
                            }));
                        }
                        return None;
                    }
                    if let Some(failure) = self.advance_sequence_tick(
                        seq_idx,
                        last_tick,
                        tick,
                        minimum_delay,
                        expected_delay,
                    ) {
                        return Some(RunningEvaluation::Failed(failure));
                    }
                }
                SequenceEventKind::Succeed => {
                    if let Some(failure) = self.advance_sequence_tick(
                        seq_idx,
                        last_tick,
                        tick,
                        minimum_delay,
                        expected_delay,
                    ) {
                        return Some(RunningEvaluation::Failed(failure));
                    }
                    self.controller
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .succeed();
                }
                SequenceEventKind::Fail(message) => {
                    return Some(RunningEvaluation::Failed(GameTestError::Assertion {
                        tick,
                        position: None,
                        message,
                    }));
                }
            }
        }
    }

    fn advance_sequence_tick(
        &self,
        seq_idx: usize,
        prev_tick: u32,
        tick: u32,
        minimum_delay: Option<u32>,
        expected_delay: Option<u32>,
    ) -> Option<GameTestError> {
        let mut state = self
            .controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(seq) = state.sequences.get_mut(seq_idx) {
            seq.last_tick = tick;
        }
        drop(state);

        let delay = tick.saturating_sub(prev_tick);
        if let Some(min_delay) = minimum_delay
            && delay < min_delay
        {
            return Some(GameTestError::Assertion {
                tick,
                position: None,
                message: format!(
                    "sequence step finished too soon at tick {tick}, minimum tick was {}",
                    prev_tick.saturating_add(min_delay)
                ),
            });
        }
        if let Some(exp_delay) = expected_delay
            && delay != exp_delay
        {
            return Some(GameTestError::Assertion {
                tick,
                position: None,
                message: format!(
                    "sequence step finished at tick {tick}, expected tick {}",
                    prev_tick.saturating_add(exp_delay)
                ),
            });
        }
        None
    }

    fn push_front_sequence_event(&self, seq_idx: usize, event: SequenceEvent) {
        let mut state = self
            .controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(seq) = state.sequences.get_mut(seq_idx) {
            seq.events.push_front(event);
        }
    }

    fn check_controller_outcome(&self) -> Option<RunningEvaluation> {
        let mut state = self
            .controller
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(error) = state.error.take() {
            return Some(RunningEvaluation::Failed(error));
        }
        if state.succeeded {
            return Some(RunningEvaluation::Passed);
        }
        None
    }

    async fn evaluate_running(&self, tick: u32) -> GameTestResult<RunningEvaluation> {
        let accept_blocks = self.test_block_positions(TestBlockMode::Accept);
        if accept_blocks.is_empty() {
            return Ok(RunningEvaluation::Failed(GameTestError::Assertion {
                tick,
                position: None,
                message: "missing ACCEPT test block".to_string(),
            }));
        }

        // Vanilla checks ACCEPT before FAIL; ACCEPT wins if both trigger this tick.
        for position in &accept_blocks {
            if self.world.test_block_triggered(position).await? {
                return Ok(RunningEvaluation::Passed);
            }
        }

        for position in self.test_block_positions(TestBlockMode::Fail) {
            if self.world.test_block_triggered(&position).await? {
                let message = self.world.test_block_message(&position).await?;
                return Ok(RunningEvaluation::Failed(GameTestError::Assertion {
                    tick,
                    position: Some(position),
                    message,
                }));
            }
        }

        for position in self.test_block_positions(TestBlockMode::Log) {
            if self.world.test_block_triggered(&position).await? {
                self.world.trigger_test_block(&position).await?;
                self.world.reset_test_block(&position).await?;
            }
        }

        Ok(RunningEvaluation::Continue)
    }

    async fn handle_attempt_pass(&mut self, tick: u32) {
        if let Some(placement) = &self.placement {
            // GameTestInfo::succeed removes non-player entities before the listeners
            // report success or schedule a copyReset rerun.
            if let Err(error) = clear_success_entities(self.world.as_ref(), placement).await {
                self.state = GameTestState::Failed { tick, error };
                return;
            }

            if let Err(error) = self
                .world
                .set_test_instance_success(placement.test_instance_pos())
                .await
            {
                self.state = GameTestState::Failed { tick, error };
                return;
            }

            // GameTestRunner's batch listener removes the test-instance barrier shell
            // on every passed execution, including executions that will be rerun.
            if let Err(error) = remove_barriers(
                self.world.as_ref(),
                placement,
                self.test.definition().sky_access,
            )
            .await
            {
                self.state = GameTestState::Failed { tick, error };
                return;
            }
        }
        self.state = GameTestState::Passed { tick };
    }

    async fn finish_failure(
        &mut self,
        tick: u32,
        error: GameTestError,
        marker: Option<(BlockPos, String)>,
    ) {
        if let Some(placement) = &self.placement {
            let message = error.to_string();
            if let Err(controller_error) = self
                .world
                .set_test_instance_failure(placement.test_instance_pos(), &message, marker)
                .await
            {
                self.state = GameTestState::Failed {
                    tick,
                    error: controller_error,
                };
                return;
            }
        }
        self.state = GameTestState::Failed { tick, error };
    }

    fn test_block_positions(&self, mode: TestBlockMode) -> Vec<BlockPos> {
        let Some(placement) = &self.placement else {
            return Vec::new();
        };

        self.template
            .blocks()
            .iter()
            .filter(|block| block.test_mode == Some(mode))
            .map(|block| {
                placement.transform(&BlockPos::new(
                    block.position[0],
                    block.position[1],
                    block.position[2],
                ))
            })
            .collect()
    }
}

fn assertion_marker(error: &GameTestError) -> Option<(BlockPos, String)> {
    match error {
        GameTestError::Assertion {
            position: Some(position),
            message,
            ..
        } => Some((*position, message.clone())),
        _ => None,
    }
}

#[derive(Default)]
pub struct TestRunner {
    active: Vec<GameTestSession>,
}

impl TestRunner {
    #[must_use]
    pub const fn new() -> Self {
        Self { active: Vec::new() }
    }

    pub fn enqueue(&mut self, run: GameTestSession) {
        self.active.push(run);
    }

    pub async fn tick(&mut self) {
        for run in &mut self.active {
            run.tick().await;
        }
    }

    #[must_use]
    pub fn active(&self) -> &[GameTestSession] {
        &self.active
    }

    pub fn active_mut(&mut self) -> &mut [GameTestSession] {
        &mut self.active
    }
}
