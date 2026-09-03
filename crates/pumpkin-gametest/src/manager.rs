use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use pumpkin_data::translation::java;
use pumpkin_util::text::{TextComponent, color::NamedColor};

use crate::{GameTestError, GameTestSession, GameTestState};

/// Receives fully constructed `GameTest` report messages.
///
/// The `GameTest` runtime owns when and what to report. Integrations only decide
/// where those messages are delivered.
pub trait GameTestReporter: Send + Sync {
    fn send_message(&self, message: TextComponent);
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

/// Shared accounting and reporting for a group of `GameTests`.
pub struct GameTestBatchReport {
    reporter: Arc<dyn GameTestReporter>,
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
            remaining_tests: AtomicUsize::new(test_count),
            total_runs: AtomicUsize::new(0),
            failed_required: AtomicUsize::new(0),
            failed_optional: AtomicUsize::new(0),
        }
    }

    pub fn fail_to_start(&self, error: &GameTestError) {
        self.reporter
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
    rerun_scheduled: bool,
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
            rerun_scheduled: false,
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
}

impl GameTestRunner {
    #[must_use]
    pub const fn new() -> Self {
        Self { active: Vec::new() }
    }

    pub fn enqueue(&mut self, run: GameTestManager) {
        self.active.push(run);
    }

    pub fn clear(&mut self) {
        self.active.clear();
    }

    pub async fn tick(&mut self) {
        // Vanilla queues copyReset reruns and does not start them until the current
        // set of batches has completed. Treat all currently active executions as one
        // wave: finish the wave first, then install the scheduled copies so they begin
        // on the following server tick.
        for managed in &mut self.active {
            if managed.done || managed.rerun_scheduled {
                continue;
            }

            managed.run.tick().await;
            if managed.run.state.is_finished() {
                managed.handle_completion();
            }
        }

        self.active.retain(|managed| !managed.done);
        if self.active.iter().all(|managed| managed.rerun_scheduled) {
            for managed in &mut self.active {
                managed.install_scheduled_rerun();
            }
        }
    }
}
