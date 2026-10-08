use super::super::{
    Context, EventHandler, EventPriority, ManagerError, Plugin, PluginFuture, PluginMetadata,
    PluginState,
    loader::{LoaderError, PluginLoadFuture, PluginLoader, PluginUnloadFuture},
};
use crate::{
    LOGGER_IMPL,
    command::{
        CommandSource,
        argument_builder::{ArgumentBuilder, command},
        context::command_context::CommandContext,
        errors::error_types::DISPATCHER_UNKNOWN_COMMAND,
        node::CommandExecutor,
    },
    data::VanillaData,
    plugin::api::events::world::world_load::WorldUnloadEvent,
    server::Server,
};
use pumpkin_command::node::CommandExecutorResult;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use std::{
    any::Any,
    future::{Future, poll_fn},
    path::Path,
    sync::{
        Arc, Mutex, RwLock, Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::Poll,
    time::Duration,
};
use tokio::{sync::oneshot, task::JoinHandle, time::timeout};

const STEP_TIMEOUT: Duration = Duration::from_secs(5);
const OTHER: &str = "cleanup-unrelated";
const UNOWNED: &str = "cleanup-unowned";
const LOAD_ERROR: &str = "Initialization failed: load-sentinel";

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    LateOnly,
    EarlyAndLate,
    Success,
}

#[derive(Debug, PartialEq, Eq)]
enum Stage {
    OnLoad,
    UnloadEntry,
    LateRegistration,
    Loader,
    Initialized,
}

#[derive(Debug, PartialEq, Eq)]
struct CommandObservation {
    allocated: bool,
    source_matches: bool,
    disabled: bool,
    listed: bool,
    value: Option<i32>,
    unknown_command: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    early: [CommandObservation; 2],
    late: [CommandObservation; 2],
    handlers: usize,
    primary_tags: usize,
    unrelated: [CommandObservation; 2],
    unowned: [CommandObservation; 2],
    unrelated_handlers: usize,
    unowned_handlers: usize,
}

type Checkpoint = (Stage, Snapshot, usize);

#[derive(Debug, PartialEq, Eq)]
struct Calls {
    load: usize,
    hook: usize,
    unload: usize,
    drops: usize,
    marker_type: bool,
}

#[derive(Debug, PartialEq, Eq)]
struct CaseReport {
    checkpoints: Vec<Checkpoint>,
    state: Option<PluginState>,
    waiter_pending: bool,
    waiter: Result<(), String>,
    active: bool,
    loaded: bool,
    before_cleanup: Calls,
    after_cleanup: Calls,
    errors: Vec<String>,
}

struct Executor;

impl CommandExecutor for Executor {
    fn execute(&self, _context: &CommandContext) -> CommandExecutorResult {
        Ok(1)
    }
}

struct Handler;
impl EventHandler<WorldUnloadEvent> for Handler {}

fn metadata(name: &str) -> PluginMetadata {
    PluginMetadata {
        name: name.to_owned(),
        version: "1".to_owned(),
        authors: Vec::new(),
        description: "Returned-hook cleanup regression".to_owned(),
        dependencies: Vec::new(),
        permissions: Vec::new(),
    }
}

fn context(server: &Arc<Server>, name: &str) -> Context {
    Context::new(
        metadata(name),
        server.clone(),
        server.plugin_manager.handlers.clone(),
        server.plugin_manager.clone(),
        Arc::clone(&LOGGER_IMPL),
    )
}

fn names(owner: &str, late: bool) -> [String; 2] {
    let phase = if late { "late" } else { "early" };
    [format!("{owner}-{phase}"), format!("{owner}-{phase}-alias")]
}

fn register_command(context: &Context, late: bool) {
    let [primary, alias] = names(&context.get_metadata().name, late);
    context.register_command_with_aliases(
        command(primary, "Cleanup regression").executes(Executor),
        &[alias.replace("-alias", "-AlIaS")],
        "run",
    );
}

fn register_handler(context: &Context) {
    context.register_event::<WorldUnloadEvent, _>(Arc::new(Handler), EventPriority::Normal, true);
}

fn snapshot(server: &Server, owner: &str) -> Snapshot {
    let dispatcher = server.command_dispatcher.load();
    let source = CommandSource::dummy();
    let listed = dispatcher.get_all_commands();
    let observe = |owner: &str, late| {
        names(owner, late).map(|name| {
            let node = dispatcher.tree.get(&name);
            let result = dispatcher.execute_input(&name, &source);
            let unknown_command = result
                .as_ref()
                .is_err_and(|error| error.error_type == &DISPATCHER_UNKNOWN_COMMAND);
            CommandObservation {
                allocated: node.is_some(),
                source_matches: node
                    .is_some_and(|id| dispatcher.tree[id].meta.source.as_deref() == Some(owner)),
                disabled: dispatcher.is_disabled(&name),
                listed: listed.contains_key(name.as_str()),
                value: result.ok(),
                unknown_command,
            }
        })
    };
    let handlers = server.plugin_manager.handlers.load();
    let count = |source| {
        handlers
            .values()
            .flatten()
            .filter(|handler| handler.source() == source)
            .count()
    };
    Snapshot {
        early: observe(owner, false),
        late: observe(owner, true),
        handlers: count(Some(owner)),
        primary_tags: dispatcher
            .tree
            .get_root_children()
            .into_iter()
            .filter(|id| dispatcher.tree[*id].meta.source.as_deref() == Some(owner))
            .count(),
        unrelated: observe(OTHER, false),
        unowned: observe(UNOWNED, false),
        unrelated_handlers: count(Some(OTHER)),
        unowned_handlers: count(None),
    }
}

struct Shared {
    server: Weak<Server>,
    name: &'static str,
    checkpoints: Mutex<Vec<Checkpoint>>,
    errors: Mutex<Vec<String>>,
    load: AtomicUsize,
    hook: AtomicUsize,
    unload: AtomicUsize,
    drops: AtomicUsize,
    marker_type: AtomicBool,
}

impl Shared {
    fn error(&self, message: impl Into<String>) {
        self.errors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(message.into());
    }

    fn record(&self, stage: Stage) {
        if let Some(server) = self.server.upgrade() {
            let observation = snapshot(&server, self.name);
            self.checkpoints
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((stage, observation, self.drops.load(Ordering::SeqCst)));
        } else {
            self.error("Server disappeared before an observation");
        }
    }

    fn calls(&self) -> Calls {
        Calls {
            load: self.load.load(Ordering::SeqCst),
            hook: self.hook.load(Ordering::SeqCst),
            unload: self.unload.load(Ordering::SeqCst),
            drops: self.drops.load(Ordering::SeqCst),
            marker_type: self.marker_type.load(Ordering::SeqCst),
        }
    }
}

struct TestPlugin {
    shared: Arc<Shared>,
    mode: Mode,
    unload_error: bool,
    ready: Mutex<Option<oneshot::Sender<()>>>,
    release: Mutex<Option<oneshot::Receiver<()>>>,
}

impl Plugin for TestPlugin {
    fn on_load(&self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            if self.mode != Mode::LateOnly {
                register_command(&context, false);
                register_handler(&context);
            }
            self.shared.record(Stage::OnLoad);
            let ready = self
                .ready
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            if ready.is_none_or(|sender| sender.send(()).is_err()) {
                self.shared.error("on_load ready channel unavailable");
            }
            let release = self
                .release
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            if let Some(receiver) = release {
                if !matches!(timeout(STEP_TIMEOUT, receiver).await, Ok(Ok(()))) {
                    self.shared.error("on_load release failed or timed out");
                }
            } else {
                self.shared.error("on_load release receiver missing");
            }
            if self.mode == Mode::Success {
                Ok(())
            } else {
                Err("load-sentinel".to_owned())
            }
        })
    }

    fn on_unload(&self, context: Arc<Context>) -> PluginFuture<'_, Result<(), String>> {
        Box::pin(async move {
            self.shared.hook.fetch_add(1, Ordering::SeqCst);
            self.shared.record(Stage::UnloadEntry);
            if self.mode != Mode::Success {
                if self.mode == Mode::EarlyAndLate {
                    register_command(&context, false);
                }
                register_command(&context, true);
                register_handler(&context);
                self.shared.record(Stage::LateRegistration);
            }
            if self.unload_error {
                Err("unload-sentinel".to_owned())
            } else {
                Ok(())
            }
        })
    }
}

struct Marker(Arc<Shared>);

impl Drop for Marker {
    fn drop(&mut self) {
        self.0.drops.fetch_add(1, Ordering::SeqCst);
    }
}

struct CountingLoader(Arc<Shared>);

impl PluginLoader for CountingLoader {
    fn load<'a>(&'a self, _path: &'a Path) -> PluginLoadFuture<'a> {
        self.0.load.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            Err(LoaderError::InitializationFailed(
                "Unexpected load".to_owned(),
            ))
        })
    }

    fn can_load(&self, _path: &Path) -> bool {
        false
    }

    fn can_unload(&self) -> bool {
        true
    }

    fn unload(&self, data: Box<dyn Any + Send + Sync>) -> PluginUnloadFuture<'_> {
        Box::pin(async move {
            self.0.unload.fetch_add(1, Ordering::SeqCst);
            self.0
                .marker_type
                .store(data.is::<Marker>(), Ordering::SeqCst);
            self.0.record(Stage::Loader);
            drop(data);
            Ok(())
        })
    }
}

#[derive(Clone, Copy)]
enum Availability {
    Missing,
    Active,
    Inactive,
}

fn expected_pair(availability: Availability, tagged: bool) -> [CommandObservation; 2] {
    let (allocated, disabled, active) = match availability {
        Availability::Missing => (false, false, false),
        Availability::Active => (true, false, true),
        Availability::Inactive => (true, true, false),
    };
    [true, false].map(|primary| CommandObservation {
        allocated,
        source_matches: allocated && primary && tagged,
        disabled,
        listed: active,
        value: active.then_some(1),
        unknown_command: !active,
    })
}

fn expected_snapshot(
    early: Availability,
    late: Availability,
    handlers: usize,
    primary_tags: usize,
) -> Snapshot {
    Snapshot {
        early: expected_pair(early, true),
        late: expected_pair(late, true),
        handlers,
        primary_tags,
        unrelated: expected_pair(Availability::Active, true),
        unowned: expected_pair(Availability::Active, false),
        unrelated_handlers: 1,
        unowned_handlers: 1,
    }
}

fn expected_calls(finished: bool) -> Calls {
    let count = usize::from(finished);
    Calls {
        load: 0,
        hook: count,
        unload: count,
        drops: count,
        marker_type: finished,
    }
}

fn expected_report(mode: Mode) -> CaseReport {
    use Availability::{Active, Inactive, Missing};
    let checkpoint = |stage, early, late, handlers, tags, drops| {
        (stage, expected_snapshot(early, late, handlers, tags), drops)
    };
    let checkpoints = match mode {
        Mode::LateOnly => vec![
            checkpoint(Stage::OnLoad, Missing, Missing, 0, 0, 0),
            checkpoint(Stage::UnloadEntry, Missing, Missing, 0, 0, 0),
            checkpoint(Stage::LateRegistration, Missing, Active, 1, 1, 0),
            checkpoint(Stage::Loader, Missing, Inactive, 0, 1, 0),
            checkpoint(Stage::Initialized, Missing, Inactive, 0, 1, 1),
        ],
        Mode::EarlyAndLate => vec![
            checkpoint(Stage::OnLoad, Active, Missing, 1, 1, 0),
            checkpoint(Stage::UnloadEntry, Active, Missing, 1, 1, 0),
            checkpoint(Stage::LateRegistration, Active, Active, 2, 2, 0),
            checkpoint(Stage::Loader, Inactive, Inactive, 0, 2, 0),
            checkpoint(Stage::Initialized, Inactive, Inactive, 0, 2, 1),
        ],
        Mode::Success => vec![
            checkpoint(Stage::OnLoad, Active, Missing, 1, 1, 0),
            checkpoint(Stage::Initialized, Active, Missing, 1, 1, 0),
            checkpoint(Stage::UnloadEntry, Inactive, Missing, 0, 1, 0),
            checkpoint(Stage::Loader, Inactive, Missing, 0, 1, 0),
        ],
    };
    let success = mode == Mode::Success;
    CaseReport {
        checkpoints,
        state: Some(if success {
            PluginState::Loaded
        } else {
            PluginState::Failed(LOAD_ERROR.to_owned())
        }),
        waiter_pending: true,
        waiter: if success {
            Ok(())
        } else {
            Err(LOAD_ERROR.to_owned())
        },
        active: success,
        loaded: success,
        before_cleanup: expected_calls(!success),
        after_cleanup: expected_calls(true),
        errors: Vec::new(),
    }
}

async fn join_initialization(mut task: JoinHandle<()>, shared: &Shared) -> Option<JoinHandle<()>> {
    match timeout(STEP_TIMEOUT, &mut task).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => shared.error(format!("Initialization join failed: {error}")),
        Err(_) => {
            task.abort();
            let joined = timeout(STEP_TIMEOUT, &mut task).await;
            shared.error(format!(
                "Initialization timed out; abort/join result: {joined:?}"
            ));
            if joined.is_err() {
                return Some(task);
            }
        }
    }
    None
}

#[expect(
    clippy::too_many_lines,
    reason = "Keep one ordered initialization, observation and cleanup sequence together"
)]
async fn run_case(
    server: &Arc<Server>,
    name: &'static str,
    mode: Mode,
    unload_error: bool,
    failures: &mut Vec<String>,
) -> Option<JoinHandle<()>> {
    let shared = Arc::new(Shared {
        server: Arc::downgrade(server),
        name,
        checkpoints: Mutex::default(),
        errors: Mutex::default(),
        load: AtomicUsize::new(0),
        hook: AtomicUsize::new(0),
        unload: AtomicUsize::new(0),
        drops: AtomicUsize::new(0),
        marker_type: AtomicBool::new(false),
    });
    let (ready_tx, ready_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let manager = &server.plugin_manager;
    let started = timeout(
        STEP_TIMEOUT,
        manager.spawn_plugin_initialization(
            server.clone(),
            Arc::new(TestPlugin {
                shared: shared.clone(),
                mode,
                unload_error,
                ready: Mutex::new(Some(ready_tx)),
                release: Mutex::new(Some(release_rx)),
            }),
            metadata(name),
            Box::new(Marker(shared.clone())),
            Arc::new(CountingLoader(shared.clone())),
            server
                .basic_config
                .get_world_path()
                .join(format!("{name}.inert")),
        ),
    )
    .await;
    if !matches!(timeout(STEP_TIMEOUT, ready_rx).await, Ok(Ok(()))) {
        shared.error("Driver ready wait failed or timed out");
    }
    let waiter = manager.wait_for_plugin(name);
    tokio::pin!(waiter);
    let first_poll = poll_fn(|cx| Poll::Ready(waiter.as_mut().poll(cx))).await;
    let waiter_pending = first_poll.is_pending();
    if release_tx.send(()).is_err() {
        shared.error("Driver release channel closed");
    }
    let waited = match first_poll {
        Poll::Ready(result) => Ok(result),
        Poll::Pending => timeout(STEP_TIMEOUT, waiter.as_mut()).await,
    };
    let waiter = match waited {
        Ok(Ok(())) => Ok(()),
        Ok(Err(ManagerError::LoaderError(LoaderError::InitializationFailed(error)))) => Err(error),
        other => Err(format!("Unexpected waiter outcome: {other:?}")),
    };
    let unfinished = match started {
        Ok(Ok(task)) => join_initialization(task, &shared).await,
        other => {
            shared.error(format!("Initialization start failed: {other:?}"));
            None
        }
    };
    shared.record(Stage::Initialized);
    let state = manager.get_plugin_state(name).await;
    let active = manager.is_plugin_active(name);
    let loaded = manager.is_plugin_loaded(name);
    let before_cleanup = shared.calls();

    // Observe the success control before explicit unload; shutdown does not unload plugins.
    if loaded
        && !matches!(
            timeout(STEP_TIMEOUT, manager.unload_plugin(name)).await,
            Ok(Ok(()))
        )
    {
        shared.error("Explicit plugin cleanup failed or timed out");
    }
    manager.unregister_handlers(name);
    context(server, name).unregister_commands();
    let report = CaseReport {
        checkpoints: std::mem::take(
            &mut *shared
                .checkpoints
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        ),
        state,
        waiter_pending,
        waiter,
        active,
        loaded,
        before_cleanup,
        after_cleanup: shared.calls(),
        errors: std::mem::take(
            &mut *shared
                .errors
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
        ),
    };
    let expected = expected_report(mode);
    if report != expected {
        failures.push(format!("{name}: observed {report:?}"));
    }
    unfinished
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_load_cleans_returned_hook_registrations_before_loader_teardown()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let basic = BasicConfiguration {
        default_level_name: directory
            .path()
            .to_str()
            .ok_or("non-UTF8 test directory")?
            .to_owned(),
        allow_nether: false,
        allow_end: false,
        allow_chat_reports: false,
        use_favicon: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.bedrock.online_mode = false;
    // Real boot performs temporary world I/O, starts workers and generates an ephemeral key.
    // Tokio deadlines do not preempt synchronous boot work; the runner supplies the outer limit.
    let server = timeout(
        Duration::from_secs(60),
        Server::new(
            basic,
            advanced,
            TelemetryConfig::default(),
            VanillaData {
                banned_ip_list: RwLock::default(),
                banned_player_list: RwLock::default(),
                operator_config: RwLock::default(),
                user_cache: RwLock::default(),
                whitelist_config: RwLock::default(),
            },
            Vec::new(),
        ),
    )
    .await??;
    let original_dispatcher = server.command_dispatcher.load_full();
    let original_handlers = server.plugin_manager.handlers.load_full();
    let unrelated = context(&server, OTHER);
    register_command(&unrelated, false);
    register_handler(&unrelated);
    server.command_dispatcher.rcu(|dispatcher| {
        let mut next = (**dispatcher).clone();
        let [primary, alias] = names(UNOWNED, false);
        next.register_with_aliases(
            command(primary, "Unowned control").executes(Executor),
            &[alias.replace("-alias", "-AlIaS")],
        );
        Arc::new(next)
    });
    server.plugin_manager.register::<WorldUnloadEvent, _>(
        Arc::new(Handler),
        EventPriority::Normal,
        true,
    );
    let mut failures = Vec::new();
    let mut unfinished = None;
    for (name, mode, unload_error) in [
        ("cleanup-late-ok", Mode::LateOnly, false),
        ("cleanup-late-err", Mode::LateOnly, true),
        ("cleanup-early-ok", Mode::EarlyAndLate, false),
        ("cleanup-early-err", Mode::EarlyAndLate, true),
        ("cleanup-success", Mode::Success, false),
    ] {
        unfinished = run_case(&server, name, mode, unload_error, &mut failures).await;
        if unfinished.is_some() {
            break;
        }
    }
    server.command_dispatcher.store(original_dispatcher);
    server.plugin_manager.handlers.store(original_handlers);
    if timeout(Duration::from_secs(30), server.shutdown())
        .await
        .is_err()
    {
        failures.push("Server shutdown timed out; task drain is not established".to_owned());
    }
    if let Some(mut task) = unfinished {
        task.abort();
        if timeout(STEP_TIMEOUT, &mut task).await.is_err() {
            failures.push("Aborted initialization still unjoined after shutdown".to_owned());
        }
    }
    for world in server.worlds.load().iter() {
        world.level.world_portal.store(Arc::new(None));
    }
    server.worlds.store(Arc::new(Vec::new()));
    drop(unrelated);
    drop(server);
    drop(directory);

    // RED must still reach late registration, loader observation and fixture cleanup.
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
