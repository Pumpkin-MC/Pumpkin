use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    future::Future,
    panic::AssertUnwindSafe,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard, Weak},
    task::{Context, Poll},
    time::Instant,
};

use futures::{
    FutureExt,
    channel::oneshot,
    task::{ArcWake, AtomicWaker, waker_ref},
};

use crate::{
    ExecutionDomain, SchedulerConfig, SchedulerError, SchedulerService, SchedulerSnapshot,
    SchedulerState, SchedulerTaskId, TaskContext, TaskExecutor, TaskHandle, TaskRequest,
};

type Completion = Box<dyn FnOnce() + Send>;
type Work = Pin<Box<dyn Future<Output = Completion> + Send>>;

struct Entry {
    work: Option<Work>,
    signal: Arc<TaskSignal>,
    queued: bool,
}

struct TaskQueue {
    tasks: HashMap<SchedulerTaskId, Entry>,
    ready: VecDeque<SchedulerTaskId>,
    running: Option<SchedulerTaskId>,
    next_id: u64,
    state: SchedulerState,
    slow_turns: u64,
}

struct Shared {
    queue: Mutex<TaskQueue>,
    wake: AtomicWaker,
    owner: Arc<()>,
    config: SchedulerConfig,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, TaskQueue> {
        self.queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

struct TaskSignal {
    shared: Weak<Shared>,
    id: SchedulerTaskId,
}

impl ArcWake for TaskSignal {
    fn wake_by_ref(signal: &Arc<Self>) {
        let Some(shared) = signal.shared.upgrade() else {
            return;
        };
        let queued = {
            let mut queue = shared.lock();
            if let Some(entry) = queue.tasks.get_mut(&signal.id)
                && !entry.queued
            {
                entry.queued = true;
                queue.ready.push_back(signal.id);
                true
            } else {
                false
            }
        };
        if queued {
            shared.wake.wake();
        }
    }
}

/// Global domain scheduler with bounded admission, driven by one injected executor task
///
/// Admission and wakeups append to a FIFO queue. Each turn polls one future
/// once; a pending task releases the domain until it is woken. Duplicate wakes
/// coalesce, including wakes during a poll. Nested calls use the same queue and
/// capacity as roots, so an awaiting parent does not prevent its child running
///
/// Returning `Pending` permits interleaving, including unrelated chains. Callers must
/// revalidate state after suspension and must not hold domain locks or borrows
/// across it. CPU-heavy work belongs on the application's CPU executor. Turn
/// budgets yield between polls and diagnose slow polls, but cannot preempt one
///
/// Capacity includes ready, running, and pending work. A full queue rejects
/// immediately, including child requests; callers must handle that error.
/// The injected executor owns the driver's lifetime. Dropping it fails pending
/// handles and closes admission; graceful draining is not provided here
#[derive(Clone)]
pub struct GlobalScheduler {
    shared: Arc<Shared>,
}

impl GlobalScheduler {
    pub fn start(
        config: SchedulerConfig,
        executor: &dyn TaskExecutor,
    ) -> Result<Self, SchedulerError> {
        let shared = Arc::new(Shared {
            queue: Mutex::new(TaskQueue {
                tasks: HashMap::new(),
                ready: VecDeque::new(),
                running: None,
                next_id: 1,
                state: SchedulerState::Accepting,
                slow_turns: 0,
            }),
            wake: AtomicWaker::new(),
            owner: Arc::new(()),
            config,
        });
        executor.spawn(Box::pin(Driver {
            shared: Arc::clone(&shared),
        }))?;
        if shared.lock().state == SchedulerState::Stopped {
            return Err(SchedulerError::Stopped);
        }
        Ok(Self { shared })
    }

    pub fn submit<T: Send + 'static>(
        &self,
        request: TaskRequest<T>,
    ) -> Result<TaskHandle<T>, SchedulerError> {
        let TaskRequest {
            domain,
            parent,
            work,
        } = request;
        if domain != ExecutionDomain::Global {
            return Err(SchedulerError::UnsupportedDomain { domain });
        }
        let (result, receiver) = oneshot::channel();
        let context = {
            let mut queue = self.shared.lock();
            if queue.state != SchedulerState::Accepting {
                return Err(SchedulerError::Stopped); // Check if scheduler is accepting work
            }
            if let Some(parent) = &parent {
                if !Arc::ptr_eq(&parent.owner, &self.shared.owner) {
                    // Check if the task runs in the same domain as the parent
                    return Err(SchedulerError::ForeignContext);
                }
                if !queue.tasks.contains_key(&parent.id) {
                    // Ensure active parent, or it cannot schedule
                    return Err(SchedulerError::InactiveParent { task: parent.id });
                }
            }
            if queue.tasks.len() == self.shared.config.maximum_tasks().get() {
                return Err(SchedulerError::QueueFull { domain });
            }
            let id = SchedulerTaskId(queue.next_id);
            queue.next_id = queue
                .next_id
                .checked_add(1)
                .ok_or(SchedulerError::TaskIdsExhausted)?;
            let context = TaskContext {
                id,
                chain: parent.as_ref().map_or(id, TaskContext::chain),
                parent: parent.as_ref().map(TaskContext::id),
                owner: Arc::clone(&self.shared.owner),
            };
            let task_context = context.clone();
            let future = Box::pin(async move {
                // Include the factory in the unwind boundary, not just its future
                let output = AssertUnwindSafe(async move { work(task_context).await })
                    .catch_unwind()
                    .await
                    .unwrap_or_else(|payload| {
                        Err(SchedulerError::TaskPanicked {
                            task: id,
                            message: panic_message(payload.as_ref()),
                        })
                    });
                Box::new(move || {
                    let _ = result.send(output);
                }) as Completion
            });
            queue.tasks.insert(
                id,
                Entry {
                    work: Some(future),
                    signal: Arc::new(TaskSignal {
                        shared: Arc::downgrade(&self.shared),
                        id,
                    }),
                    queued: true,
                },
            );
            queue.ready.push_back(id);
            context
        };
        self.shared.wake.wake();
        Ok(TaskHandle {
            context,
            result: receiver,
        })
    }
}

impl SchedulerService for GlobalScheduler {
    fn config(&self) -> SchedulerConfig {
        self.shared.config
    }

    fn state(&self) -> SchedulerState {
        self.shared.lock().state
    }

    fn snapshot(&self) -> SchedulerSnapshot {
        let queue = self.shared.lock();
        let running = usize::from(queue.running.is_some());
        let ready = queue
            .ready
            .iter()
            .filter(|id| Some(**id) != queue.running)
            .count();
        SchedulerSnapshot {
            ready,
            running,
            pending: queue.tasks.len() - ready - running,
            slow_turns: queue.slow_turns,
        }
    }

    fn submit(&self, request: TaskRequest) -> Result<TaskHandle, SchedulerError> {
        self.submit(request)
    }
}

struct Driver {
    shared: Arc<Shared>,
}

impl Future for Driver {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.shared.wake.register(cx.waker());
        for _ in 0..self.shared.config.turns_per_poll().get() {
            let next = {
                let mut queue = self.shared.lock();
                queue.ready.pop_front().and_then(|id| {
                    let entry = queue.tasks.get_mut(&id)?;
                    entry.queued = false;
                    let work = entry.work.take()?;
                    let signal = Arc::clone(&entry.signal);
                    queue.running = Some(id);
                    Some((id, work, signal))
                })
            };
            let Some((id, mut work, signal)) = next else {
                return Poll::Pending;
            };
            let wake = waker_ref(&signal);
            let mut task_cx = Context::from_waker(&wake);
            let started = Instant::now();
            let result = work.as_mut().poll(&mut task_cx);
            let elapsed = started.elapsed();
            let slow = elapsed >= self.shared.config.slow_turn_threshold();
            {
                let mut queue = self.shared.lock();
                queue.running = None;
                if slow {
                    queue.slow_turns = queue.slow_turns.saturating_add(1);
                }
                if result.is_ready() {
                    if queue.tasks.remove(&id).is_some_and(|entry| entry.queued) {
                        // A future may wake itself and then complete on the same poll
                        queue.ready.retain(|queued| *queued != id);
                    }
                } else if let Some(entry) = queue.tasks.get_mut(&id) {
                    entry.work = Some(work);
                }
            }
            if slow {
                tracing::warn!(task_id = id.get(), ?elapsed, "global domain task was slow:");
            }
            if let Poll::Ready(complete) = result {
                // Release admission before a waiter observes completion
                complete();
            }
        }
        if !self.shared.lock().ready.is_empty() {
            cx.waker().wake_by_ref();
        }
        Poll::Pending
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        let tasks = {
            let mut queue = self.shared.lock();
            queue.state = SchedulerState::Stopped;
            queue.ready.clear();
            queue.running = None;
            std::mem::take(&mut queue.tasks)
        };
        // Future destructors and result wakeups must run without the queue lock
        drop(tasks);
    }
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .unwrap_or_else(|| {
            payload.downcast_ref::<&str>().map_or_else(
                || "non-string panic payload".to_owned(),
                |message| (*message).to_owned(),
            )
        })
}
