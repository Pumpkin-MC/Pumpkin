use std::{
    collections::{HashMap, HashSet, VecDeque},
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, MutexGuard, Weak},
    task::{Context, Poll},
    time::Instant,
};

use futures::{
    channel::oneshot,
    task::{ArcWake, AtomicWaker, waker_ref},
};

use crate::{
    CancellationHandle, ExecutionDomain, SchedulerConfig, SchedulerError, SchedulerService,
    SchedulerSnapshot, SchedulerState, SchedulerTaskId, TaskContext, TaskExecutor, TaskHandle,
    TaskRequest,
    completion::{self, Outcome, ScheduledTask},
};

type Work = Box<dyn ScheduledTask>;

struct Entry {
    work: Option<Work>,
    signal: Arc<TaskSignal>,
    queued: bool,
    cancellation: CancellationHandle,
    // Reparent surviving children to the nearest active ancestor for cancellation
    // This avoids retaining completed tasks
    // TaskContext retains the original parent ID
    parent: Option<SchedulerTaskId>,
    children: HashSet<SchedulerTaskId>,
}

struct TaskQueue {
    tasks: HashMap<SchedulerTaskId, Entry>,
    ready: VecDeque<SchedulerTaskId>,
    running: Option<SchedulerTaskId>,
    next_id: u64,
    state: SchedulerState,
    slow_turns: u64,
    completed: u64,
    cancelled: u64,
    failed: u64,
}

impl TaskQueue {
    fn enqueue(&mut self, id: SchedulerTaskId) -> bool {
        if let Some(entry) = self.tasks.get_mut(&id)
            && !entry.queued
        {
            entry.queued = true;
            self.ready.push_back(id);
            true
        } else {
            false
        }
    }

    fn remove(&mut self, id: SchedulerTaskId) -> Option<Entry> {
        let entry = self.tasks.remove(&id)?;
        if entry.queued {
            self.ready.retain(|queued| *queued != id);
        }
        if self.running == Some(id) {
            self.running = None;
        }
        if let Some(parent) = entry.parent.and_then(|id| self.tasks.get_mut(&id)) {
            parent.children.remove(&id);
            parent.children.extend(entry.children.iter().copied());
        }
        for child in &entry.children {
            if let Some(child) = self.tasks.get_mut(child) {
                child.parent = entry.parent;
            }
        }
        Some(entry)
    }

    const fn record(&mut self, outcome: &Outcome) {
        let counter = match outcome {
            Outcome::Completed => &mut self.completed,
            Outcome::Cancelled => &mut self.cancelled,
            Outcome::Failed => &mut self.failed,
        };
        *counter = counter.saturating_add(1);
    }
}

pub struct Shared {
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

    pub(crate) fn cancel(&self, id: SchedulerTaskId) -> bool {
        {
            let mut queue = self.lock();
            if queue.state != SchedulerState::Accepting {
                return false;
            }
            let Some(entry) = queue.tasks.get(&id) else {
                return false;
            };
            if entry.cancellation.is_requested() {
                return false;
            }

            // Reparenting preserves cancellation reachability through completed ancestors
            let mut descendants = vec![id];
            while let Some(candidate) = descendants.pop() {
                if let Some(entry) = queue.tasks.get(&candidate) {
                    entry.cancellation.request();
                    descendants.extend(entry.children.iter().copied());
                }
                queue.enqueue(candidate);
            }
        }
        self.wake.wake();
        true
    }

    /// Releases admission and resolves the result through a shared completion path
    fn finish(&self, id: SchedulerTaskId, work: Option<Work>, error: Option<SchedulerError>) {
        let (entry, error) = {
            let mut queue = self.lock();
            let Some(entry) = queue.remove(id) else {
                return;
            };
            let error = if entry.cancellation.is_requested() {
                Some(SchedulerError::Cancelled { task: id })
            } else {
                error
            };
            (entry, error)
        };
        if let Some(work) = work.or(entry.work) {
            // Destructors and result wakeups can re-enter the scheduler
            // Release the queue lock before running either operation
            let completion = work.finish(error);
            self.lock().record(&completion.outcome);
            completion.deliver();
        } else {
            // Driver unwinding may have already dropped the active task
            // Channel closure reports the stopped driver to the result receiver
            self.lock().record(&Outcome::Failed);
        }
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
        let queued = shared.lock().enqueue(signal.id);
        if queued {
            shared.wake.wake();
        }
    }
}

/// Schedules the Global domain through one driver with bounded admission
///
/// Admission and wakeups append to a FIFO ready queue
/// Each turn polls one future once
/// Pending tasks release their turn until woken, allowing children to run while
/// their parents await results
/// Duplicate wakeups are coalesced
///
/// Polls are serialized and call chains may interleave during suspension
/// Callers must release domain locks and borrows before suspension and
/// revalidate state when they resume
/// CPU-heavy work belongs on the application's CPU executor
/// Turn budgets yield between polls and a running poll continues until it returns
///
/// Admission limits include ready, running, and pending tasks
/// Child requests share the root admission limit
/// Cancelled tasks retain admission until processed by the driver
///
/// The injected executor owns the driver's lifetime
/// Dropping the driver closes admission and fails outstanding work
/// Graceful draining requires a separate lifecycle operation
/// Explicit cancellation affects the selected task and its active descendants
/// Existing side effects remain after cancellation or driver loss
#[derive(Clone)]
pub struct GlobalScheduler {
    shared: Arc<Shared>,
}

impl GlobalScheduler {
    /// Starts a single driver on the application's executor
    ///
    /// # Errors
    ///
    /// Returns the executor's error if it rejects the driver, or
    /// [`SchedulerError::Stopped`] if it immediately drops the driver
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
                completed: 0,
                cancelled: 0,
                failed: 0,
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

    /// Admits work without running its factory or waiting for a free slot
    ///
    /// # Errors
    ///
    /// Rejects unsupported domains, stopped schedulers, full admission, and
    /// exhausted task IDs
    /// A child also requires an active, uncancelled parent from this scheduler
    /// Execution failures are reported through the result handle
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
                return Err(SchedulerError::Stopped);
            }
            if let Some(parent) = &parent {
                if !Arc::ptr_eq(&parent.owner, &self.shared.owner) {
                    return Err(SchedulerError::ForeignContext);
                }
                let Some(entry) = queue.tasks.get(&parent.id) else {
                    return Err(SchedulerError::InactiveParent { task: parent.id });
                };
                if entry.cancellation.is_requested() {
                    return Err(SchedulerError::Cancelled { task: parent.id });
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
                cancellation: CancellationHandle::new(&self.shared, id),
            };
            let work = completion::task(context.clone(), work, result);
            queue.tasks.insert(
                id,
                Entry {
                    work: Some(work),
                    signal: Arc::new(TaskSignal {
                        shared: Arc::downgrade(&self.shared),
                        id,
                    }),
                    queued: true,
                    cancellation: context.cancellation_handle(),
                    parent: context.parent(),
                    children: HashSet::new(),
                },
            );
            if let Some(parent) = context.parent().and_then(|id| queue.tasks.get_mut(&id)) {
                parent.children.insert(id);
            }
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
            completed: queue.completed,
            cancelled: queue.cancelled,
            failed: queue.failed,
        }
    }

    fn submit(&self, request: TaskRequest) -> Result<TaskHandle, SchedulerError> {
        self.submit(request)
    }
}

struct Driver {
    shared: Arc<Shared>,
}

impl Driver {
    fn run_turn(
        &self,
        id: SchedulerTaskId,
        mut work: Work,
        signal: &Arc<TaskSignal>,
        cancellation: &CancellationHandle,
    ) {
        if cancellation.is_requested() {
            self.shared
                .finish(id, Some(work), Some(SchedulerError::Cancelled { task: id }));
            return;
        }
        let wake = waker_ref(signal);
        let mut task_cx = Context::from_waker(&wake);
        let started = Instant::now();
        let result = work.poll(&mut task_cx);
        let elapsed = started.elapsed();
        if elapsed >= self.shared.config.slow_turn_threshold() {
            let mut queue = self.shared.lock();
            queue.slow_turns = queue.slow_turns.saturating_add(1);
            drop(queue);
            tracing::warn!(task_id = id.get(), ?elapsed, "global domain task was slow:");
        }
        if result.is_ready() {
            self.shared.finish(id, Some(work), None);
        } else {
            let mut queue = self.shared.lock();
            queue.running = None;
            if let Some(entry) = queue.tasks.get_mut(&id) {
                entry.work = Some(work);
            }
        }
    }
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
                    let cancellation = entry.cancellation.clone();
                    queue.running = Some(id);
                    Some((id, work, signal, cancellation))
                })
            };
            let Some((id, work, signal, cancellation)) = next else {
                return Poll::Pending;
            };
            self.run_turn(id, work, &signal, &cancellation);
        }
        if !self.shared.lock().ready.is_empty() {
            cx.waker().wake_by_ref();
        }
        Poll::Pending
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        let ids: Vec<_> = {
            let mut queue = self.shared.lock();
            queue.state = SchedulerState::Stopped;
            queue.tasks.keys().copied().collect()
        };
        for id in ids {
            self.shared.finish(id, None, Some(SchedulerError::Stopped));
        }
    }
}
