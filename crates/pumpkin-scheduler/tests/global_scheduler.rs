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

use futures::task::{ArcWake, waker};
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

/// Holds the real driver so each executor turn is deterministic.
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
