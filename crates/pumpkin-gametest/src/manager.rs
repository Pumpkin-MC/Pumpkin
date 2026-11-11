use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use pumpkin_data::translation::java;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::{GameTestError, GameTestSession, GameTestState};

pub const VERIFY_TEST_BATCH_SIZE: usize = 100;
const MAX_TESTS_PER_BATCH: usize = 50;

/// Receives fully constructed `GameTest` report messages.
///
/// The `GameTest` runtime owns when and what to report. Integrations only decide
/// where those messages are delivered.
pub trait GameTestReporter: Send + Sync {
    fn send_message(&self, message: TextComponent);

    /// Receives one final result for each test in the batch.
    fn on_outcome(&self, _outcome: &GameTestOutcome) {}
}

/// Stores plain-text `GameTest` messages for later inspection.
#[derive(Default)]
pub struct RecordingGameTestReporter {
    messages: Mutex<Vec<String>>,
}

impl RecordingGameTestReporter {
    #[must_use]
    pub fn messages(&self) -> Vec<String> {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl GameTestReporter for RecordingGameTestReporter {
    fn send_message(&self, message: TextComponent) {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(message.get_text());
    }
}

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
    pub const fn no_retries() -> Self {
        Self::new(1, true)
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

/// Final status of one `GameTest` in a batch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameTestOutcomeStatus {
    Passed,
    Failed,
    Cancelled,
}

/// Final result of one `GameTest`, with counts for executions that actually started.
#[derive(Clone, Debug)]
pub struct GameTestOutcome {
    pub id: String,
    pub required: bool,
    pub status: GameTestOutcomeStatus,
    pub attempts: u32,
    pub successes: u32,
    pub error: Option<GameTestError>,
}

/// Shared accounting and reporting for a group of `GameTests`.
pub struct GameTestBatchReport {
    reporter: Arc<dyn GameTestReporter>,
    outcomes: Mutex<Vec<GameTestOutcome>>,
    remaining_tests: AtomicUsize,
    total_runs: AtomicUsize,
    failed_required: AtomicUsize,
    failed_optional: AtomicUsize,
}

impl GameTestBatchReport {
    #[must_use]
    pub fn new(reporter: Arc<dyn GameTestReporter>, test_count: usize) -> Self {
        Self {
            reporter,
            outcomes: Mutex::new(Vec::with_capacity(test_count)),
            remaining_tests: AtomicUsize::new(test_count),
            total_runs: AtomicUsize::new(0),
            failed_required: AtomicUsize::new(0),
            failed_optional: AtomicUsize::new(0),
        }
    }

    #[must_use]
    pub const fn reporter(&self) -> &Arc<dyn GameTestReporter> {
        &self.reporter
    }

    pub fn fail_to_start(&self, id: &str, error: &GameTestError, required: bool) {
        self.reporter
            .send_message(TextComponent::text(error.to_string()).color_named(NamedColor::Red));
        self.finish_test(
            GameTestOutcome {
                id: id.to_string(),
                required,
                status: GameTestOutcomeStatus::Failed,
                attempts: 0,
                successes: 0,
                error: Some(error.clone()),
            },
            1,
            0,
        );
    }

    /// Records a queued test that was cancelled before it started.
    pub fn cancel_before_start(&self, id: &str, required: bool) {
        let error = GameTestError::World("GameTest cancelled".to_string());
        self.reporter
            .send_message(TextComponent::text(error.to_string()).color_named(NamedColor::Red));
        self.finish_test(
            GameTestOutcome {
                id: id.to_string(),
                required,
                status: GameTestOutcomeStatus::Cancelled,
                attempts: 0,
                successes: 0,
                error: Some(error),
            },
            1,
            0,
        );
    }

    #[must_use]
    pub fn remaining_tests(&self) -> usize {
        self.remaining_tests.load(Ordering::Acquire)
    }

    #[must_use]
    pub fn total_runs(&self) -> usize {
        self.total_runs.load(Ordering::Acquire)
    }

    #[must_use]
    pub fn failed_required(&self) -> usize {
        self.failed_required.load(Ordering::Acquire)
    }

    #[must_use]
    pub fn failed_optional(&self) -> usize {
        self.failed_optional.load(Ordering::Acquire)
    }

    #[must_use]
    pub fn is_done(&self) -> bool {
        self.remaining_tests() == 0
    }

    /// Returns a snapshot of the tests that have finished so far.
    #[must_use]
    pub fn outcomes(&self) -> Vec<GameTestOutcome> {
        self.outcomes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[must_use]
    pub fn all_required_passed(&self) -> bool {
        self.is_done() && self.failed_required() == 0
    }

    fn finish_test(&self, outcome: GameTestOutcome, counted_attempts: u32, counted_successes: u32) {
        let attempts = usize::try_from(counted_attempts).unwrap_or(usize::MAX);
        let successes = usize::try_from(counted_successes).unwrap_or(usize::MAX);
        let required = outcome.required;
        self.reporter.on_outcome(&outcome);
        self.outcomes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(outcome);
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

        self.reporter.send_message(
            pumpkin_macros::translate_cross!(
                java::COMMANDS_TEST_SUMMARY,
                java::COMMANDS_TEST_SUMMARY,
                TextComponent::text(total.to_string()),
            )
            .color_named(NamedColor::White),
        );

        if failed_required != 0 {
            self.reporter.send_message(
                pumpkin_macros::translate_cross!(
                    java::COMMANDS_TEST_SUMMARY_FAILED,
                    java::COMMANDS_TEST_SUMMARY_FAILED,
                    TextComponent::text(failed_required.to_string()),
                )
                .color_named(NamedColor::Red),
            );
        } else {
            self.reporter.send_message(
                pumpkin_macros::translate_cross!(
                    java::COMMANDS_TEST_SUMMARY_ALL_REQUIRED_PASSED,
                    java::COMMANDS_TEST_SUMMARY_ALL_REQUIRED_PASSED,
                )
                .color_named(NamedColor::Green),
            );
        }

        if failed_optional != 0 {
            self.reporter.send_message(pumpkin_macros::translate_cross!(
                java::COMMANDS_TEST_SUMMARY_OPTIONAL_FAILED,
                java::COMMANDS_TEST_SUMMARY_OPTIONAL_FAILED,
                TextComponent::text(failed_optional.to_string()),
            ));
        }
    }
}

/// Runs a game test and manages retries and reporting.
pub struct GameTestManager {
    run: GameTestSession,
    retry_options: GameTestRetryOptions,
    report: Arc<GameTestBatchReport>,
    sink: Arc<dyn GameTestReporter>,
    attempts: u32,
    successes: u32,
    last_error: Option<GameTestError>,
    rerun_scheduled: bool,
    rerun_order: Option<usize>,
    done: bool,
}

impl GameTestManager {
    #[must_use]
    pub fn new(
        run: GameTestSession,
        retry_options: GameTestRetryOptions,
        report: Arc<GameTestBatchReport>,
        sink: Arc<dyn GameTestReporter>,
    ) -> Self {
        Self {
            run,
            retry_options,
            report,
            sink,
            attempts: 0,
            successes: 0,
            last_error: None,
            rerun_scheduled: false,
            rerun_order: None,
            done: false,
        }
    }

    #[expect(clippy::too_many_lines)]
    fn handle_completion(&mut self) {
        let (passed, tick, error) = match &self.run.state {
            GameTestState::Passed { tick } => (true, *tick, None),
            GameTestState::Failed { tick, error } => (false, *tick, Some(error)),
            _ => return,
        };
        if let Some(error) = error {
            self.last_error = Some(error.clone());
        }

        self.attempts = self.attempts.saturating_add(1);
        if passed {
            self.successes = self.successes.saturating_add(1);
        }
        let elapsed_ms = self.run.run_time_ms();
        let is_flaky = self.run.test.max_attempts() > 1;

        // This intentionally follows ReportGameListener's ordering. Command retry
        // options take precedence for a passing execution. Flaky failure handling,
        // however, uses max_attempts/required_successes exactly as vanilla does.
        let should_rerun = if passed {
            if self.retry_options.has_retries() {
                self.sink.send_message(
                    TextComponent::text(self.retry_status(true, elapsed_ms))
                        .color_named(NamedColor::Green),
                );
                self.retry_options
                    .has_tries_left(self.attempts, self.successes)
            } else if !is_flaky {
                self.sink.send_message(
                    TextComponent::text(format!(
                        "{} passed! ({}ms / {}gameticks)",
                        self.run.test.id(),
                        elapsed_ms,
                        tick
                    ))
                    .color_named(NamedColor::Green),
                );
                false
            } else if self.successes >= self.run.test.required_successes() {
                self.sink.send_message(
                    TextComponent::text(format!(
                        "{} passed {} times of {} attempts.",
                        self.run.test.id(),
                        self.successes,
                        self.attempts
                    ))
                    .color_named(NamedColor::Green),
                );
                false
            } else {
                self.sink.send_message(
                    TextComponent::text(format!(
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
                self.sink.send_message(
                    TextComponent::text(self.retry_status(false, elapsed_ms))
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
            self.sink
                .send_message(TextComponent::text(text).color_named(NamedColor::Yellow));

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
                self.last_error = Some(exhausted);
                self.report_failure(Some(&exhausted_message));
                false
            }
        };

        if should_rerun {
            self.rerun_scheduled = true;
            return;
        }

        let effective_successes = if is_flaky
            && !self.retry_options.has_retries()
            && self.successes >= self.run.test.required_successes()
        {
            self.attempts
        } else {
            self.successes
        };

        let status = if effective_successes == self.attempts {
            GameTestOutcomeStatus::Passed
        } else {
            GameTestOutcomeStatus::Failed
        };
        self.report.finish_test(
            GameTestOutcome {
                id: self.run.test.id().to_string(),
                required: self.run.test.is_required(),
                status,
                attempts: self.attempts,
                successes: self.successes,
                error: if status == GameTestOutcomeStatus::Failed {
                    self.last_error.clone()
                } else {
                    None
                },
            },
            self.attempts,
            effective_successes,
        );
        self.done = true;
    }

    fn install_scheduled_rerun(&mut self) {
        if !self.rerun_scheduled || self.done {
            return;
        }

        self.run = self.run.copy_reset();
        self.rerun_scheduled = false;
        self.rerun_order = None;
    }

    fn cancel(&mut self) {
        if self.done {
            return;
        }
        self.sink.send_message(
            TextComponent::text(format!("{} cancelled", self.run.test.id()))
                .color_named(NamedColor::Yellow),
        );
        self.report.finish_test(
            GameTestOutcome {
                id: self.run.test.id().to_string(),
                required: self.run.test.is_required(),
                status: GameTestOutcomeStatus::Cancelled,
                attempts: self.attempts,
                successes: self.successes,
                error: Some(GameTestError::World("GameTest cancelled".to_string())),
            },
            self.attempts.saturating_add(1),
            self.successes,
        );
        self.done = true;
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
        self.sink
            .send_message(TextComponent::text(text).color_named(color));
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

/// Ticks ready `GameTest` runs and owns their retry lifecycle.
#[derive(Default)]
pub struct GameTestRunner {
    active: Vec<GameTestManager>,
    pending: VecDeque<GameTestManager>,
    environment: Option<(
        Arc<dyn crate::world::GameTestWorld>,
        Box<dyn std::any::Any + Send>,
    )>,
    completed: Vec<GameTestSession>,
    collect_completed: bool,
    next_rerun_order: usize,
}

impl GameTestRunner {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active: Vec::new(),
            pending: VecDeque::new(),
            environment: None,
            completed: Vec::new(),
            collect_completed: false,
            next_rerun_order: 0,
        }
    }

    pub fn enqueue(&mut self, run: GameTestManager) {
        self.pending.push_back(run);
    }

    pub fn clear(&mut self) {
        if let Some((world, saved)) = self.environment.take() {
            world.teardown_test_environment(saved);
        }
        self.active.clear();
        self.pending.clear();
        self.completed.clear();
        self.collect_completed = false;
        self.next_rerun_order = 0;
    }

    pub async fn cancel_and_clear(&mut self) -> Vec<GameTestError> {
        let mut errors = Vec::new();
        for managed in &mut self.active {
            if let Err(error) = managed.run.clear_placement().await {
                errors.push(error);
            }
            managed.cancel();
        }
        for completed in &self.completed {
            if let Err(error) = completed.clear_placement().await {
                errors.push(error);
            }
        }
        for managed in &mut self.pending {
            managed.cancel();
        }
        self.clear();
        errors
    }

    pub const fn collect_completed(&mut self) {
        self.collect_completed = true;
    }

    pub fn take_completed(&mut self) -> Vec<GameTestSession> {
        std::mem::take(&mut self.completed)
    }

    pub fn stop_collecting_completed(&mut self) {
        self.completed.clear();
        self.collect_completed = false;
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.active.is_empty() && self.pending.is_empty()
    }

    #[must_use]
    pub fn active_len(&self) -> usize {
        self.active.len() + self.pending.len()
    }

    pub async fn tick(&mut self) {
        if self.active.is_empty() {
            if let Some((world, saved)) = self.environment.take() {
                world.teardown_test_environment(saved);
            }
            if let Some(first) = self.pending.front() {
                let environment = first.run.environment().clone();
                let world_key = first.run.environment_world_key();
                let world = first.run.environment_world();
                match world.activate_test_environment(&environment) {
                    Ok(saved) => {
                        self.environment = Some((world, saved));
                        while self.active.len() < MAX_TESTS_PER_BATCH
                            && self.pending.front().is_some_and(|next| {
                                next.run.environment() == &environment
                                    && next.run.environment_world_key() == world_key
                            })
                        {
                            if let Some(next) = self.pending.pop_front() {
                                self.active.push(next);
                            }
                        }
                    }
                    Err(error) => {
                        if let Some(mut failed) = self.pending.pop_front() {
                            failed.report.fail_to_start(
                                failed.run.test.id(),
                                &error,
                                failed.run.test.is_required(),
                            );
                            failed.done = true;
                        }
                    }
                }
            }
        }
        // Vanilla queues copyReset reruns and does not start them until the current
        // set of batches has completed. Treat all currently active executions as one
        // wave, then start copies in the order their original runs finished.
        for managed in &mut self.active {
            if managed.done || managed.rerun_scheduled {
                continue;
            }

            managed.run.tick().await;
            if managed.run.state.is_finished() {
                if let Err(error) = managed.run.disconnect_simulated_players().await {
                    let tick = match &managed.run.state {
                        GameTestState::Passed { tick } | GameTestState::Failed { tick, .. } => {
                            *tick
                        }
                        _ => 0,
                    };
                    managed.run.state = GameTestState::Failed { tick, error };
                }
                managed.handle_completion();
                if managed.rerun_scheduled {
                    managed.rerun_order = Some(self.next_rerun_order);
                    self.next_rerun_order += 1;
                }
            }
        }

        if self.collect_completed {
            let mut active = Vec::with_capacity(self.active.len());
            for managed in self.active.drain(..) {
                if managed.done {
                    self.completed.push(managed.run);
                } else {
                    active.push(managed);
                }
            }
            self.active = active;
        } else {
            self.active.retain(|managed| !managed.done);
        }
        if self.active.iter().all(|managed| managed.rerun_scheduled) {
            self.active.sort_by_key(|managed| managed.rerun_order);
            for managed in &mut self.active {
                managed.install_scheduled_rerun();
            }
            self.next_rerun_order = 0;
        }
        if self.active.is_empty()
            && let Some((world, saved)) = self.environment.take()
        {
            world.teardown_test_environment(saved);
        }
    }
}
