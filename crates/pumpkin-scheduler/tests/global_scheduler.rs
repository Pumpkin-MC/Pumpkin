use std::{
    future::poll_fn,
    num::NonZeroUsize,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
    time::Duration,
};

use futures::{
    FutureExt,
    task::{ArcWake, noop_waker_ref, waker},
};
use pumpkin_scheduler::{
    ExecutionDomain, ExecutorFuture, GlobalScheduler, SchedulerConfig, SchedulerError,
    SchedulerService, SchedulerState, TaskExecutor, TaskRequest,
};
use tokio::sync::{mpsc, oneshot};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn config(capacity: usize, turns: usize) -> SchedulerConfig {
    SchedulerConfig::new(
        NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::MIN),
        NonZeroUsize::new(turns).unwrap_or(NonZeroUsize::MIN),
        Duration::from_millis(10),
    )
}

struct TokioExecutor;

impl TaskExecutor for TokioExecutor {
    fn spawn(&self, future: ExecutorFuture) -> Result<(), SchedulerError> {
        tokio::spawn(future);
        Ok(())
    }
}

#[tokio::test(flavor = "current_thread")]
async fn pending_roots_and_nested_chains_make_progress_on_one_worker() -> TestResult {
    tokio::time::timeout(Duration::from_secs(5), async {
        let scheduler = GlobalScheduler::start(config(16, 4), &TokioExecutor)?;
        let (started, mut arrivals) = mpsc::unbounded_channel();
        let mut handles = Vec::new();
        let mut releases = Vec::new();
        for number in 0..8 {
            let (release, released) = oneshot::channel();
            releases.push(release);
            let started = started.clone();
            handles.push(scheduler.submit(TaskRequest::new(
                ExecutionDomain::Global,
                move |context| async move {
                    let id = context.id();
                    let _ = started.send(number);
                    released.await.map_err(|_| SchedulerError::Stopped)?;
                    assert_eq!(context.id(), id);
                    assert_eq!(context.chain(), id);
                    Ok(number)
                },
            ))?);
        }
        for number in 0..8 {
            assert_eq!(arrivals.recv().await, Some(number));
        }
        assert_eq!(scheduler.snapshot().pending(), 8);

        let nested_scheduler = scheduler.clone();
        let nested = scheduler.submit(TaskRequest::new(
            ExecutionDomain::Global,
            move |parent| async move {
                let grandchild_scheduler = nested_scheduler.clone();
                let parent_id = parent.id();
                let chain = parent.chain();
                let child = nested_scheduler.submit(TaskRequest::child(
                    &parent,
                    move |child| async move {
                        assert_eq!(child.parent(), Some(parent_id));
                        assert_eq!(child.chain(), chain);
                        let child_id = child.id();
                        let grandchild = grandchild_scheduler.submit(TaskRequest::child(
                            &child,
                            move |grandchild| async move {
                                assert_eq!(grandchild.parent(), Some(child_id));
                                assert_eq!(grandchild.chain(), chain);
                                Ok(String::from("owned result"))
                            },
                        ))?;
                        grandchild.await
                    },
                ))?;
                let result = child.await?;
                assert_eq!(parent.id(), parent_id);
                Ok(result)
            },
        ))?;
        let finished_parent = nested.context().clone();
        assert_eq!(nested.await?, "owned result");
        assert_eq!(scheduler.snapshot().pending(), 8);
        assert!(matches!(
            scheduler.submit(TaskRequest::child(&finished_parent, |_| async { Ok(()) })),
            Err(SchedulerError::InactiveParent { .. })
        ));

        for release in releases {
            let _ = release.send(());
        }
        for (number, handle) in handles.into_iter().enumerate() {
            assert_eq!(handle.await?, number);
        }
        let snapshot = scheduler.snapshot();
        assert_eq!(
            snapshot.ready() + snapshot.running() + snapshot.pending(),
            0
        );
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}

/// Retains the scheduler driver for deterministic polling
#[derive(Default)]
struct ControlledExecutor(Mutex<Option<ExecutorFuture>>);

impl TaskExecutor for ControlledExecutor {
    fn spawn(&self, future: ExecutorFuture) -> Result<(), SchedulerError> {
        *self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(future);
        Ok(())
    }
}

impl ControlledExecutor {
    fn take(&self) -> Result<ExecutorFuture, SchedulerError> {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .ok_or(SchedulerError::Stopped)
    }
}

#[derive(Default)]
struct WakeCount(AtomicUsize);

impl ArcWake for WakeCount {
    fn wake_by_ref(value: &Arc<Self>) {
        value.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn fifo_turns_coalesce_wakes_and_bound_all_admitted_work() -> TestResult {
    let executor = ControlledExecutor::default();
    let scheduler = GlobalScheduler::start(config(2, 1), &executor)?;
    let mut driver = executor.take()?;
    let wakes = Arc::new(WakeCount::default());
    let executor_waker = waker(Arc::clone(&wakes));
    let mut cx = Context::from_waker(&executor_waker);
    assert!(driver.as_mut().poll(&mut cx).is_pending());
    let (trace, mut turns) = mpsc::unbounded_channel();
    let saved_wake = Arc::new(Mutex::new(None::<Waker>));
    let task_wake = Arc::clone(&saved_wake);
    let trace_a = trace.clone();
    let first = scheduler.submit(TaskRequest::new(ExecutionDomain::Global, move |_| {
        let mut polled = false;
        poll_fn(move |cx| {
            let _ = trace_a.send("A");
            *task_wake
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(cx.waker().clone());
            for _ in 0..1_000 {
                cx.waker().wake_by_ref();
            }
            if polled {
                Poll::Ready(Ok(7))
            } else {
                polled = true;
                Poll::Pending
            }
        })
    }))?;
    let second = scheduler.submit(TaskRequest::new(
        ExecutionDomain::Global,
        move |_| async move {
            let _ = trace.send("B");
            Ok(9)
        },
    ))?;
    assert!(wakes.0.load(Ordering::SeqCst) > 0);
    assert!(matches!(
        scheduler.submit(TaskRequest::new(ExecutionDomain::Global, |_| async {
            Ok(())
        })),
        Err(SchedulerError::QueueFull { .. })
    ));
    for expected in ["A", "B", "A"] {
        assert!(driver.as_mut().poll(&mut cx).is_pending());
        assert_eq!(turns.try_recv()?, expected);
        assert!(turns.try_recv().is_err());
        assert!(scheduler.snapshot().ready() <= 2);
    }
    assert_eq!(first.await?, 7);
    assert_eq!(second.await?, 9);
    assert_eq!(scheduler.snapshot().ready(), 0);
    let stale = saved_wake
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    if let Some(stale) = stale {
        stale.wake();
    }
    assert_eq!(scheduler.snapshot().ready(), 0);

    let (release, released) = oneshot::channel::<()>();
    let pending = scheduler.submit(TaskRequest::new(
        ExecutionDomain::Global,
        move |_| async move { released.await.map_err(|_| SchedulerError::Stopped) },
    ))?;
    assert!(driver.as_mut().poll(&mut cx).is_pending());
    let another = scheduler.submit(TaskRequest::new(ExecutionDomain::Global, |_| {
        std::future::pending::<Result<(), SchedulerError>>()
    }))?;
    assert!(driver.as_mut().poll(&mut cx).is_pending());
    assert_eq!(scheduler.snapshot().pending(), 2);
    assert!(matches!(
        scheduler.submit(TaskRequest::child(pending.context(), |_| async { Ok(()) })),
        Err(SchedulerError::QueueFull { .. })
    ));
    let before = wakes.0.load(Ordering::SeqCst);
    let _ = release.send(());
    assert!(wakes.0.load(Ordering::SeqCst) > before);
    assert!(driver.as_mut().poll(&mut cx).is_pending());
    pending.await?;
    let detached = scheduler.submit(TaskRequest::new(ExecutionDomain::Global, |_| async {
        Ok(())
    }))?;
    drop(detached);
    assert!(driver.as_mut().poll(&mut cx).is_pending());
    assert_eq!(scheduler.snapshot().pending(), 1);
    drop(driver);
    assert!(matches!(another.await, Err(SchedulerError::Stopped)));
    assert_eq!(scheduler.state(), SchedulerState::Stopped);
    assert!(matches!(
        scheduler.submit(TaskRequest::new(ExecutionDomain::Global, |_| async {
            Ok(())
        })),
        Err(SchedulerError::Stopped)
    ));
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failures_and_invalid_contexts_do_not_poison_the_scheduler() -> TestResult {
    tokio::time::timeout(Duration::from_secs(5), async {
        let scheduler = GlobalScheduler::start(config(4, 2), &TokioExecutor)?;
        let failing = scheduler.submit(TaskRequest::new(ExecutionDomain::Global, |_| async {
            std::panic::resume_unwind(Box::new("intentional task panic"));
            #[allow(unreachable_code)]
            Ok(())
        }))?;
        assert!(matches!(
            failing.await,
            Err(SchedulerError::TaskPanicked { .. })
        ));
        let factory_failure =
            scheduler.submit(TaskRequest::new(ExecutionDomain::Global, |_| {
                std::panic::resume_unwind(Box::new("intentional factory panic"));
                #[allow(unreachable_code)]
                async {
                    Ok(())
                }
            }))?;
        assert!(matches!(
            factory_failure.await,
            Err(SchedulerError::TaskPanicked { .. })
        ));
        let application_error = scheduler
            .submit(TaskRequest::new(ExecutionDomain::Global, |_| async {
                Err::<(), _>(SchedulerError::Backend("work failed".into()))
            }))?;
        assert!(matches!(
            application_error.await,
            Err(SchedulerError::Backend(_))
        ));
        let other = GlobalScheduler::start(config(2, 1), &TokioExecutor)?;
        let checked = scheduler.submit(TaskRequest::new(
            ExecutionDomain::Global,
            move |context| async move {
                assert!(matches!(
                    other.submit(TaskRequest::child(&context, |_| async { Ok(()) })),
                    Err(SchedulerError::ForeignContext)
                ));
                Ok(42)
            },
        ))?;
        assert_eq!(checked.await?, 42);
        assert!(matches!(
            scheduler.submit(TaskRequest::new(
                ExecutionDomain::World(pumpkin_scheduler::WorldDomainId::new(1)),
                |_| async { Ok(()) }
            )),
            Err(SchedulerError::UnsupportedDomain { .. })
        ));
        assert_eq!(scheduler.state(), SchedulerState::Accepting);
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}

fn drive(driver: &mut ExecutorFuture) {
    assert!(
        driver
            .as_mut()
            .poll(&mut Context::from_waker(noop_waker_ref()))
            .is_pending()
    );
}

/// Checks scheduler access and panic containment during task cleanup
struct CleanupProbe {
    scheduler: GlobalScheduler,
    drops: Arc<AtomicUsize>,
    panic: bool,
}

impl Drop for CleanupProbe {
    fn drop(&mut self) {
        let _ = self.scheduler.snapshot();
        self.drops.fetch_add(1, Ordering::SeqCst);
        if self.panic {
            std::panic::resume_unwind(Box::new("intentional cleanup panic"));
        }
    }
}

fn cancel_during_final_poll(
    scheduler: &GlobalScheduler,
    mut driver: ExecutorFuture,
) -> Result<ExecutorFuture, Box<dyn std::error::Error>> {
    // Hold the poll in progress until cancellation is accepted, then return its result
    let (entered, observed) = std::sync::mpsc::channel();
    let (release, resumed) = std::sync::mpsc::channel();
    let racing = scheduler.submit(TaskRequest::new(
        ExecutionDomain::Global,
        move |_| async move {
            let _ = entered.send(());
            resumed
                .recv_timeout(Duration::from_secs(5))
                .map(|()| 42)
                .map_err(|error| SchedulerError::Backend(error.to_string()))
        },
    ))?;
    let running = std::thread::spawn(move || {
        drive(&mut driver);
        driver
    });
    let arrived = observed.recv_timeout(Duration::from_secs(5));
    let accepted = racing.cancellation_handle().cancel();
    let _ = release.send(());
    let driver = running.join().map_err(|_| "driver panicked")?;
    arrived?;
    assert!(accepted);
    assert!(matches!(
        racing.now_or_never().ok_or("racing task did not settle")?,
        Err(SchedulerError::Cancelled { .. })
    ));
    Ok(driver)
}

#[test]
fn cancellation_settles_once_and_releases_admission() -> TestResult {
    let executor = ControlledExecutor::default();
    let scheduler = GlobalScheduler::start(config(1, 1), &executor)?;
    let mut driver = executor.take()?;
    let starts = Arc::new(AtomicUsize::new(0));
    let started = Arc::clone(&starts);
    let queued = scheduler.submit(TaskRequest::new(ExecutionDomain::Global, move |_| {
        started.fetch_add(1, Ordering::SeqCst);
        async { Ok(()) }
    }))?;
    let cancel = queued.cancellation_handle();
    assert!(cancel.cancel());
    assert!(!cancel.cancel());
    assert!(matches!(
        scheduler.submit(TaskRequest::new(ExecutionDomain::Global, |_| async {
            Ok(())
        })),
        Err(SchedulerError::QueueFull { .. })
    ));
    drive(&mut driver);
    assert!(matches!(
        queued.now_or_never().ok_or("queued task did not settle")?,
        Err(SchedulerError::Cancelled { .. })
    ));
    assert_eq!(starts.load(Ordering::SeqCst), 0);
    assert!(cancel.is_requested());
    assert!(!cancel.cancel());

    let drops = Arc::new(AtomicUsize::new(0));
    let probe = CleanupProbe {
        scheduler: scheduler.clone(),
        drops: Arc::clone(&drops),
        panic: false,
    };
    let (saved, mut wake_receiver) = mpsc::unbounded_channel();
    let pending = scheduler.submit(TaskRequest::new(ExecutionDomain::Global, move |_| {
        poll_fn(move |cx| {
            let _keep_alive = &probe;
            let _ = saved.send(cx.waker().clone());
            Poll::<Result<(), SchedulerError>>::Pending
        })
    }))?;
    let wakes = Arc::new(WakeCount::default());
    let driver_wake = waker(Arc::clone(&wakes));
    assert!(
        driver
            .as_mut()
            .poll(&mut Context::from_waker(&driver_wake))
            .is_pending()
    );
    let stale = wake_receiver.try_recv()?;
    let before = wakes.0.load(Ordering::SeqCst);
    assert!(pending.cancellation_handle().cancel());
    assert!(wakes.0.load(Ordering::SeqCst) > before);
    drive(&mut driver);
    assert!(matches!(
        pending
            .now_or_never()
            .ok_or("pending task did not settle")?,
        Err(SchedulerError::Cancelled { .. })
    ));
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert!(wake_receiver.try_recv().is_err());
    stale.wake();
    assert_eq!(scheduler.snapshot().ready(), 0);

    driver = cancel_during_final_poll(&scheduler, driver)?;
    let success = scheduler.submit(TaskRequest::new(ExecutionDomain::Global, |_| async {
        Ok(7)
    }))?;
    let late = success.cancellation_handle();
    drive(&mut driver);
    assert!(!late.cancel());
    assert_eq!(success.now_or_never().ok_or("success did not settle")??, 7);
    let snapshot = scheduler.snapshot();
    assert_eq!(
        (
            snapshot.completed(),
            snapshot.cancelled(),
            snapshot.failed()
        ),
        (1, 3, 0)
    );
    assert_eq!(
        snapshot.ready() + snapshot.running() + snapshot.pending(),
        0
    );
    Ok(())
}

#[test]
fn cancellation_follows_active_descendants_without_touching_other_chains() -> TestResult {
    let executor = ControlledExecutor::default();
    let scheduler = GlobalScheduler::start(config(8, 16), &executor)?;
    let mut driver = executor.take()?;
    let pending = |_| std::future::pending::<Result<(), SchedulerError>>();
    let root = scheduler.submit(TaskRequest::new(ExecutionDomain::Global, pending))?;
    let branch = scheduler.submit(TaskRequest::child(root.context(), pending))?;
    let leaf = scheduler.submit(TaskRequest::child(branch.context(), pending))?;
    let sibling = scheduler.submit(TaskRequest::child(root.context(), pending))?;
    let unrelated = scheduler.submit(TaskRequest::new(ExecutionDomain::Global, pending))?;
    let middle = scheduler.submit(TaskRequest::child(root.context(), |_| async { Ok(()) }))?;
    let orphan = scheduler.submit(TaskRequest::child(middle.context(), pending))?;
    let original_parent = orphan.context().parent();
    let original_chain = orphan.context().chain();
    drive(&mut driver);
    middle
        .now_or_never()
        .ok_or("middle task did not settle")??;
    assert_eq!(scheduler.snapshot().pending(), 6);

    assert!(branch.cancellation_handle().cancel());
    assert!(matches!(
        scheduler.submit(TaskRequest::child(branch.context(), pending)),
        Err(SchedulerError::Cancelled { .. })
    ));
    assert!(leaf.cancellation_handle().is_requested());
    assert!(!sibling.cancellation_handle().is_requested());
    assert!(!root.cancellation_handle().is_requested());
    drive(&mut driver);
    for handle in [branch, leaf] {
        let id = handle.id();
        assert!(
            matches!(handle.now_or_never().ok_or("descendant did not settle")?, Err(SchedulerError::Cancelled { task }) if task == id)
        );
    }
    assert_eq!(scheduler.snapshot().pending(), 4);

    // A completed intermediate task must not disconnect its active descendants
    assert!(root.cancellation_handle().cancel());
    assert!(orphan.cancellation_handle().is_requested());
    assert_eq!(orphan.context().parent(), original_parent);
    assert_eq!(orphan.context().chain(), original_chain);
    assert!(!unrelated.cancellation_handle().is_requested());
    drive(&mut driver);
    for handle in [root, sibling, orphan] {
        assert!(matches!(
            handle.now_or_never().ok_or("root subtree did not settle")?,
            Err(SchedulerError::Cancelled { .. })
        ));
    }
    assert_eq!(scheduler.snapshot().pending(), 1);
    drop(driver);
    assert!(matches!(
        unrelated
            .now_or_never()
            .ok_or("driver loss did not settle")?,
        Err(SchedulerError::Stopped)
    ));
    let snapshot = scheduler.snapshot();
    assert_eq!(
        (
            snapshot.completed(),
            snapshot.cancelled(),
            snapshot.failed()
        ),
        (1, 5, 1)
    );
    Ok(())
}

#[test]
fn driver_loss_settles_all_tasks_even_when_cleanup_panics() -> TestResult {
    let executor = ControlledExecutor::default();
    let scheduler = GlobalScheduler::start(config(3, 4), &executor)?;
    let mut driver = executor.take()?;
    let drops = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();
    for panic in [false, true, false] {
        let probe = CleanupProbe {
            scheduler: scheduler.clone(),
            drops: Arc::clone(&drops),
            panic,
        };
        handles.push(scheduler.submit(TaskRequest::new(
            ExecutionDomain::Global,
            move |_| async move {
                let _probe = probe;
                std::future::pending::<Result<(), SchedulerError>>().await
            },
        ))?);
    }
    drive(&mut driver);
    assert!(handles[0].cancellation_handle().cancel());
    assert!(handles[1].cancellation_handle().cancel());
    let stopped = handles[2].cancellation_handle();
    drop(driver);
    assert_eq!(drops.load(Ordering::SeqCst), 3);
    assert!(!stopped.cancel());
    for (index, handle) in handles.into_iter().enumerate() {
        let result = handle
            .now_or_never()
            .ok_or("driver loss left a task unresolved")?;
        match index {
            0 => assert!(matches!(result, Err(SchedulerError::Cancelled { .. }))),
            1 => assert!(matches!(result, Err(SchedulerError::TaskPanicked { .. }))),
            _ => assert!(matches!(result, Err(SchedulerError::Stopped))),
        }
    }
    let snapshot = scheduler.snapshot();
    assert_eq!(
        (
            snapshot.completed(),
            snapshot.cancelled(),
            snapshot.failed()
        ),
        (0, 1, 2)
    );
    assert_eq!(
        snapshot.ready() + snapshot.running() + snapshot.pending(),
        0
    );
    assert_eq!(scheduler.state(), SchedulerState::Stopped);
    Ok(())
}
