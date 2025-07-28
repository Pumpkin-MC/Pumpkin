pub mod macros;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

#[async_trait::async_trait]
pub trait TaskHandler: Send + Sync + 'static {
    async fn run(&self);
    async fn cancel(&self);
}

pub enum ScheduledTaskType {
    Later {
        run_at_tick: u64,
    },
    Timer {
        interval_ticks: u64,
        next_run_tick: u64,
    },
}

pub struct ScheduledTask {
    pub task_type: ScheduledTaskType,
    pub handler: Arc<dyn TaskHandler>,
    pub cancel_flag: Arc<AtomicBool>,
}

pub struct TaskScheduler {
    tasks: Mutex<Vec<ScheduledTask>>,
    tick_count: std::sync::atomic::AtomicU64,
}

impl TaskScheduler {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tasks: Mutex::new(Vec::new()),
            tick_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    pub async fn schedule_once(
        &self,
        delay_ticks: u64,
        handler: Arc<dyn TaskHandler>,
    ) -> Arc<AtomicBool> {
        let current_tick = self.tick_count.load(Ordering::Relaxed);
        let cancel_flag = Arc::new(AtomicBool::new(false));
        self.tasks.lock().await.push(ScheduledTask {
            task_type: ScheduledTaskType::Later {
                run_at_tick: current_tick + delay_ticks,
            },
            handler,
            cancel_flag: cancel_flag.clone(),
        });
        cancel_flag
    }

    pub async fn schedule_repeating(
        &self,
        interval_ticks: u64,
        handler: Arc<dyn TaskHandler>,
    ) -> Arc<AtomicBool> {
        let current_tick = self.tick_count.load(Ordering::Relaxed);
        let cancel_flag = Arc::new(AtomicBool::new(false));
        self.tasks.lock().await.push(ScheduledTask {
            task_type: ScheduledTaskType::Timer {
                interval_ticks,
                next_run_tick: current_tick + interval_ticks,
            },
            handler,
            cancel_flag: cancel_flag.clone(),
        });
        cancel_flag
    }

    pub async fn tick(&self) {
        let current_tick = self.tick_count.fetch_add(1, Ordering::Relaxed) + 1;

        let mut tasks = self.tasks.lock().await;

        tasks.retain_mut(|task| {
            if task.cancel_flag.load(Ordering::Relaxed) {
                let handler = task.handler.clone();
                tokio::spawn(async move {
                    handler.cancel().await;
                });
                return false;
            }

            match &mut task.task_type {
                ScheduledTaskType::Later { run_at_tick } => {
                    if *run_at_tick <= current_tick {
                        let handler = task.handler.clone();
                        tokio::spawn(async move {
                            handler.run().await;
                        });
                        false
                    } else {
                        true
                    }
                }
                ScheduledTaskType::Timer {
                    interval_ticks,
                    next_run_tick,
                } => {
                    if *next_run_tick <= current_tick {
                        *next_run_tick = current_tick + *interval_ticks;
                        let handler = task.handler.clone();
                        tokio::spawn(async move {
                            handler.run().await;
                        });
                    }
                    true
                }
            }
        });
    }
}

#[derive(Clone)]
pub struct ScheduledHandle {
    handler: Arc<dyn TaskHandler>,
    cancel_flag: Arc<AtomicBool>,
}

impl ScheduledHandle {
    pub async fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
        self.handler.cancel().await;
    }
}

#[derive(Clone)]
pub struct RepeatingHandle {
    pub(crate) cancel_flag: Arc<AtomicBool>,
}

impl RepeatingHandle {
    pub fn new(cancel_flag: Arc<AtomicBool>) -> Self {
        Self { cancel_flag }
    }

    pub fn cancel(&self) {
        self.cancel_flag.store(true, Ordering::Relaxed);
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}
