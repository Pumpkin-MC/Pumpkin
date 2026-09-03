use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt::{self, Write as _},
    future::Future,
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    task::Poll,
    time::{Duration, Instant},
};

use futures::future::BoxFuture;
use sysinfo::{ProcessesToUpdate, System, get_current_pid};
use tokio::{
    runtime::Builder,
    sync::{Barrier, Notify, Semaphore},
    task::JoinSet,
    time::timeout,
};
use tokio_util::sync::CancellationToken;
use wasm_encoder::{
    CodeSection, ComponentBuilder, ComponentExportKind, ComponentTypeRef, EntityType, ExportKind,
    ExportSection, Function, FunctionSection, ImportSection, Instruction, Module, ModuleArg,
    PrimitiveValType, TypeSection, ValType,
};
use wasmtime::{
    Config, Engine, Store,
    component::{Component, Linker, TypedFunc},
};

use super::{
    GuestCallContext, LegacyStore as ConcurrentStore, MAX_SYNC_REENTRY_DEPTH, PluginHostState,
    STORE_QUEUE_CAPACITY, StoreFuture, StoreJob, StoreMessage,
};

const SCENARIO_TIMEOUT: Duration = Duration::from_secs(2);
const OPPOSING_ROOT_TIMEOUT: Duration = Duration::from_secs(1);
const OPPOSING_ROOT_PROCESS_TIMEOUT: Duration = Duration::from_secs(8);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(1);
const OPPOSING_ROOT_WORKER: &str =
    "plugin::loader::wasm::wasm_host::concurrent_store::conformance::opposing_roots_legacy_worker";
const OPPOSING_ROOT_WORKER_COMPLETE: &str = "PWR002_OPPOSING_ROOT_WORKER_COMPLETE";

pub(super) fn test_engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    config.concurrency_support(true);
    Engine::new(&config).expect("test engine")
}

pub(super) fn test_store() -> Store<PluginHostState> {
    Store::new(&test_engine(), PluginHostState::new())
}

pub(super) fn sync_reentry_component() -> Vec<u8> {
    component_relay([], [], None)
}

fn route_component() -> Vec<u8> {
    component_relay([ValType::I32], [("step", PrimitiveValType::U32)], Some(0))
}

fn component_relay<const CORE: usize, const COMPONENT: usize>(
    core_params: [ValType; CORE],
    component_params: [(&str, PrimitiveValType); COMPONENT],
    forwarded_local: Option<u32>,
) -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function(core_params, []);
    module.section(&types);

    let mut imports = ImportSection::new();
    imports.import("", "host", EntityType::Function(0));
    module.section(&imports);

    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);

    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, 1);
    module.section(&exports);

    let mut body = Function::new([]);
    if let Some(local) = forwarded_local {
        body.instruction(&Instruction::LocalGet(local));
    }
    body.instruction(&Instruction::Call(0));
    body.instruction(&Instruction::End);
    let mut code = CodeSection::new();
    code.function(&body);
    module.section(&code);

    let mut component = ComponentBuilder::default();
    let (function_type, mut function) = component.type_function(Some("run-type"));
    function.params(component_params).result(None);
    let imported = component.import("host", ComponentTypeRef::Func(function_type));
    let lowered = component.lower_func(Some("host-lowered"), imported, []);
    let module = component.core_module(Some("guest"), &module);
    let host_instance = component
        .core_instantiate_exports(Some("host-instance"), [("host", ExportKind::Func, lowered)]);
    let guest_instance = component.core_instantiate(
        Some("guest-instance"),
        module,
        [("", ModuleArg::Instance(host_instance))],
    );
    let run_core =
        component.core_alias_export(Some("run-core"), guest_instance, "run", ExportKind::Func);
    let run = component.lift_func(Some("run"), run_core, function_type, []);
    component.export("run", ComponentExportKind::Func, run, None);
    component.finish()
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
enum StoreName {
    A,
    B,
    C,
}

impl fmt::Display for StoreName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Transition {
    Declared,
    Enqueue,
    Start,
    Yield,
    Resume,
    Complete,
    Trap,
    Shutdown,
}

impl fmt::Display for Transition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Declared => "not-started",
            Self::Enqueue => "enqueue",
            Self::Start => "start",
            Self::Yield => "yield",
            Self::Resume => "resume",
            Self::Complete => "completion",
            Self::Trap => "trap",
            Self::Shutdown => "shutdown",
        })
    }
}

#[derive(Clone, Debug)]
struct TraceEvent {
    sequence: usize,
    elapsed: Duration,
    chain: Arc<str>,
    store: StoreName,
    depth: usize,
    transition: Transition,
    queue_depth: usize,
    detail: Arc<str>,
}

#[derive(Default)]
struct TraceState {
    events: Vec<TraceEvent>,
    queue_depths: HashMap<StoreName, usize>,
    max_queue_depth: usize,
}

struct TraceInner {
    started: Instant,
    state: Mutex<TraceState>,
}

#[derive(Clone)]
struct TraceRecorder(Arc<TraceInner>);

impl TraceRecorder {
    fn new() -> Self {
        Self(Arc::new(TraceInner {
            started: Instant::now(),
            state: Mutex::new(TraceState::default()),
        }))
    }

    fn record(
        &self,
        chain: impl Into<Arc<str>>,
        store: StoreName,
        depth: usize,
        transition: Transition,
        detail: impl Into<Arc<str>>,
    ) {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let sequence = state.events.len();
        let queue_depth = {
            let current = state.queue_depths.entry(store).or_default();
            match transition {
                Transition::Enqueue => *current += 1,
                Transition::Start => *current = current.saturating_sub(1),
                _ => {}
            }
            *current
        };
        state.max_queue_depth = state.max_queue_depth.max(queue_depth);
        state.events.push(TraceEvent {
            sequence,
            elapsed: self.0.started.elapsed(),
            chain: chain.into(),
            store,
            depth,
            transition,
            queue_depth,
            detail: detail.into(),
        });
    }

    fn record_queue_depth(
        &self,
        chain: impl Into<Arc<str>>,
        store: StoreName,
        depth: usize,
        transition: Transition,
        queue_depth: usize,
        detail: impl Into<Arc<str>>,
    ) {
        let mut state = self
            .0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let sequence = state.events.len();
        state.queue_depths.insert(store, queue_depth);
        state.max_queue_depth = state.max_queue_depth.max(queue_depth);
        state.events.push(TraceEvent {
            sequence,
            elapsed: self.0.started.elapsed(),
            chain: chain.into(),
            store,
            depth,
            transition,
            queue_depth,
            detail: detail.into(),
        });
    }

    fn max_queue_depth(&self) -> usize {
        self.0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .max_queue_depth
    }

    fn register_route(&self, root: &RouteRoot) {
        for (depth, store) in root.route.iter().copied().enumerate() {
            self.register_pair(Arc::clone(&root.chain), store, depth);
        }
    }

    fn register_pair(&self, chain: impl Into<Arc<str>>, store: StoreName, depth: usize) {
        self.record(chain, store, depth, Transition::Declared, "route-declared");
    }

    fn has_transition(&self, chain: &str, store: StoreName, transition: Transition) -> bool {
        self.0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .events
            .iter()
            .any(|event| {
                event.chain.as_ref() == chain
                    && event.store == store
                    && event.transition == transition
            })
    }

    fn has_detail(&self, chain: &str, detail: &str) -> bool {
        self.0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .events
            .iter()
            .any(|event| event.chain.as_ref() == chain && event.detail.contains(detail))
    }

    fn last_transitions(&self) -> BTreeMap<(Arc<str>, StoreName), TraceEvent> {
        let state = self
            .0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut last = BTreeMap::new();
        for event in &state.events {
            last.insert((Arc::clone(&event.chain), event.store), event.clone());
        }
        last
    }

    fn timeout_report(&self, scenario: &str) -> String {
        let state = self
            .0
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut last = BTreeMap::<(Arc<str>, StoreName), &TraceEvent>::new();
        for event in &state.events {
            last.insert((Arc::clone(&event.chain), event.store), event);
        }

        let mut report = format!("scenario {scenario:?} reached its deadline\nlast transitions:\n");
        for ((chain, store), event) in last {
            let _ = writeln!(
                report,
                "  chain={chain} store={store} depth={} last={} queue_depth={} detail={} at={}us",
                event.depth,
                event.transition,
                event.queue_depth,
                compact_detail(&event.detail),
                event.elapsed.as_micros()
            );
        }
        report.push_str("last 32 trace events:\n");
        let first_event = state.events.len().saturating_sub(32);
        for event in &state.events[first_event..] {
            let _ = writeln!(
                report,
                "  #{:03} {}us chain={} store={} depth={} transition={} queue_depth={} detail={}",
                event.sequence,
                event.elapsed.as_micros(),
                event.chain,
                event.store,
                event.depth,
                event.transition,
                event.queue_depth,
                compact_detail(&event.detail)
            );
        }
        report
    }
}

fn compact_detail(detail: &str) -> &str {
    detail.lines().next().unwrap_or(detail)
}

#[derive(Clone)]
struct RouteFrame {
    chain: Arc<str>,
    route: Arc<[StoreName]>,
    index: usize,
}

impl RouteFrame {
    fn new(chain: impl Into<Arc<str>>, route: impl Into<Arc<[StoreName]>>) -> Self {
        Self {
            chain: chain.into(),
            route: route.into(),
            index: 0,
        }
    }

    fn store(&self) -> StoreName {
        self.route[self.index]
    }

    fn next(&self) -> Option<Self> {
        (self.index + 1 < self.route.len()).then(|| Self {
            chain: Arc::clone(&self.chain),
            route: Arc::clone(&self.route),
            index: self.index + 1,
        })
    }
}

#[derive(Clone)]
struct HostHold {
    chain: Arc<str>,
    store: StoreName,
    depth: usize,
    entered: Arc<Notify>,
    release: Arc<Notify>,
}

impl HostHold {
    fn matches(&self, frame: &RouteFrame) -> bool {
        self.chain == frame.chain && self.store == frame.store() && self.depth == frame.index
    }
}

#[derive(Clone)]
struct LegacyEndpoint {
    driver: ConcurrentStore,
    run: TypedFunc<(u32,), ()>,
}

struct LegacyNetwork {
    endpoints: OnceLock<HashMap<StoreName, LegacyEndpoint>>,
    frames: Mutex<HashMap<u32, RouteFrame>>,
    next_step: AtomicUsize,
    trace: TraceRecorder,
    cancellation: CancellationToken,
    initial_host_barrier: Option<Arc<Barrier>>,
    cross_enqueue_signal: Option<Arc<Semaphore>>,
    hold: Option<HostHold>,
}

impl LegacyNetwork {
    #[allow(clippy::too_many_lines)]
    async fn build(
        stores: &BTreeSet<StoreName>,
        trace: TraceRecorder,
        initial_host_barrier: Option<usize>,
        cross_enqueue_count: Option<usize>,
        hold: Option<HostHold>,
    ) -> wasmtime::Result<Arc<Self>> {
        let engine = test_engine();
        let component = Component::new(&engine, route_component())?;
        let network = Arc::new(Self {
            endpoints: OnceLock::new(),
            frames: Mutex::new(HashMap::new()),
            next_step: AtomicUsize::new(0),
            trace,
            cancellation: CancellationToken::new(),
            initial_host_barrier: initial_host_barrier
                .map(|parties| Arc::new(Barrier::new(parties))),
            cross_enqueue_signal: cross_enqueue_count.map(|_| Arc::new(Semaphore::new(0))),
            hold,
        });
        let mut prepared = Vec::with_capacity(stores.len());

        for &store_name in stores {
            let mut linker = Linker::<PluginHostState>::new(&engine);
            let network_for_host = Arc::clone(&network);
            linker.root().func_wrap_async(
                "host",
                move |mut store, (step,): (u32,)| {
                    let network = Arc::clone(&network_for_host);
                    Box::new(async move {
                        let frame = network.frame(step)?;
                        if frame.store() != store_name {
                            network.trace.record(
                                Arc::clone(&frame.chain),
                                store_name,
                                frame.index,
                                Transition::Trap,
                                "route frame did not match guest step",
                            );
                            return Err(wasmtime::Error::msg("conformance route mismatch"));
                        }

                        if frame.index == 0
                            && let Some(barrier) = &network.initial_host_barrier
                        {
                            network.trace.record(
                                Arc::clone(&frame.chain),
                                store_name,
                                frame.index,
                                Transition::Yield,
                                "initial-host-barrier",
                            );
                            tokio::select! {
                                _ = barrier.wait() => {}
                                () = network.cancellation.cancelled() => {
                                    network.trace.record(
                                        Arc::clone(&frame.chain),
                                        store_name,
                                        frame.index,
                                        Transition::Trap,
                                        "cancelled-at-initial-host-barrier",
                                    );
                                    return Err(wasmtime::Error::msg("conformance scenario cancelled"));
                                }
                            }
                            network.trace.record(
                                Arc::clone(&frame.chain),
                                store_name,
                                frame.index,
                                Transition::Resume,
                                "initial-host-barrier",
                            );
                        }

                        if let Some(hold) = &network.hold
                            && hold.matches(&frame)
                        {
                            let current = network.endpoint(store_name)?.driver;
                            network.trace.record(
                                Arc::clone(&frame.chain),
                                store_name,
                                frame.index,
                                Transition::Yield,
                                "host-hold",
                            );
                            hold.entered.notify_one();
                            let wait = async {
                                tokio::select! {
                                    () = hold.release.notified() => Ok(()),
                                    () = network.cancellation.cancelled() => {
                                        Err(wasmtime::Error::msg("conformance scenario cancelled"))
                                    }
                                }
                            };
                            current.pump_reentry(&mut store, wait).await??;
                            network.trace.record(
                                Arc::clone(&frame.chain),
                                store_name,
                                frame.index,
                                Transition::Resume,
                                "host-hold",
                            );
                        }

                        if let Some(next) = frame.next() {
                            let current = network.endpoint(store_name)?.driver;
                            network.trace.record(
                                Arc::clone(&frame.chain),
                                store_name,
                                frame.index,
                                Transition::Yield,
                                format!("outbound-to-{}", next.store()),
                            );
                            let outbound = Arc::clone(&network).invoke(next);
                            let nested = tokio::select! {
                                () = network.cancellation.cancelled() => {
                                    Err(wasmtime::Error::msg("conformance scenario cancelled"))
                                }
                                result = current.pump_reentry(&mut store, outbound) => {
                                    match result {
                                        Ok(nested) => nested,
                                        Err(error) => {
                                            network.trace.record(
                                                Arc::clone(&frame.chain),
                                                store_name,
                                                frame.index,
                                                Transition::Trap,
                                                error.to_string(),
                                            );
                                            return Err(error);
                                        }
                                    }
                                }
                            };
                            if let Err(error) = nested {
                                network.trace.record(
                                    Arc::clone(&frame.chain),
                                    store_name,
                                    frame.index,
                                    Transition::Trap,
                                    error.to_string(),
                                );
                                return Err(error);
                            }
                            network.trace.record(
                                Arc::clone(&frame.chain),
                                store_name,
                                frame.index,
                                Transition::Resume,
                                "outbound-complete",
                            );
                        }
                        Ok(())
                    })
                },
            )?;

            let mut store = Store::new(&engine, PluginHostState::new());
            let instance = linker.instantiate_async(&mut store, &component).await?;
            let run = instance.get_typed_func::<(u32,), ()>(&mut store, "run")?;
            prepared.push((store_name, store, run));
        }

        let endpoints = prepared
            .into_iter()
            .map(|(name, store, run)| {
                (
                    name,
                    LegacyEndpoint {
                        driver: ConcurrentStore::new(store),
                        run,
                    },
                )
            })
            .collect();
        network
            .endpoints
            .set(endpoints)
            .map_err(|_| wasmtime::Error::msg("conformance endpoints initialized twice"))?;
        Ok(network)
    }

    fn endpoint(&self, store: StoreName) -> wasmtime::Result<LegacyEndpoint> {
        self.endpoints
            .get()
            .and_then(|endpoints| endpoints.get(&store))
            .cloned()
            .ok_or_else(|| wasmtime::Error::msg(format!("missing conformance Store {store}")))
    }

    fn register_frame(&self, frame: RouteFrame) -> wasmtime::Result<u32> {
        let step = u32::try_from(self.next_step.fetch_add(1, Ordering::SeqCst))
            .map_err(|_| wasmtime::Error::msg("conformance step ID overflow"))?;
        self.frames
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(step, frame);
        Ok(step)
    }

    fn frame(&self, step: u32) -> wasmtime::Result<RouteFrame> {
        self.frames
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&step)
            .cloned()
            .ok_or_else(|| wasmtime::Error::msg(format!("missing conformance step {step}")))
    }

    fn remove_frame(&self, step: u32) {
        self.frames
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&step);
    }

    fn invoke(self: Arc<Self>, frame: RouteFrame) -> BoxFuture<'static, wasmtime::Result<()>> {
        Box::pin(async move {
            let endpoint = self.endpoint(frame.store())?;
            let step = self.register_frame(frame.clone())?;
            self.trace.record(
                Arc::clone(&frame.chain),
                frame.store(),
                frame.index,
                Transition::Enqueue,
                "guest-call",
            );
            if frame.index == 1
                && let Some(signal) = &self.cross_enqueue_signal
            {
                signal.add_permits(1);
            }
            let trace = self.trace.clone();
            let cancellation = self.cancellation.clone();
            let call_frame = frame.clone();
            let result = endpoint
                .driver
                .call_guest(move |mut context: GuestCallContext<'_>| {
                    Box::pin(async move {
                        trace.record(
                            Arc::clone(&call_frame.chain),
                            call_frame.store(),
                            call_frame.index,
                            Transition::Start,
                            if context.is_reentrant() {
                                "legacy-reentrant"
                            } else {
                                "legacy-concurrent"
                            },
                        );
                        if cancellation.is_cancelled() {
                            trace.record(
                                Arc::clone(&call_frame.chain),
                                call_frame.store(),
                                call_frame.index,
                                Transition::Trap,
                                "cancelled-before-guest-entry",
                            );
                            return Err(wasmtime::Error::msg("conformance scenario cancelled"));
                        }
                        match context.call(endpoint.run, (step,)).await {
                            Ok(()) => {
                                trace.record(
                                    Arc::clone(&call_frame.chain),
                                    call_frame.store(),
                                    call_frame.index,
                                    Transition::Complete,
                                    "guest-call",
                                );
                                Ok(())
                            }
                            Err(error) => {
                                trace.record(
                                    Arc::clone(&call_frame.chain),
                                    call_frame.store(),
                                    call_frame.index,
                                    Transition::Trap,
                                    error.to_string(),
                                );
                                Err(error)
                            }
                        }
                    })
                })
                .await;
            self.remove_frame(step);
            if let Err(error) = &result {
                self.trace.record(
                    Arc::clone(&frame.chain),
                    frame.store(),
                    frame.index,
                    Transition::Trap,
                    error.to_string(),
                );
            }
            result
        })
    }

    async fn shutdown_all(&self) -> wasmtime::Result<()> {
        let endpoints = self
            .endpoints
            .get()
            .ok_or_else(|| wasmtime::Error::msg("conformance endpoints not initialized"))?;
        let mut names = endpoints.keys().copied().collect::<Vec<_>>();
        names.sort_unstable();
        for name in names {
            self.trace
                .record("lifecycle", name, 0, Transition::Shutdown, "begin");
            endpoints[&name]
                .driver
                .shutdown(|_| Box::pin(async move { Ok(()) }))
                .await?;
            self.trace
                .record("lifecycle", name, 0, Transition::Shutdown, "complete");
        }
        Ok(())
    }
}

#[derive(Clone)]
struct RouteRoot {
    chain: Arc<str>,
    route: Arc<[StoreName]>,
}

impl RouteRoot {
    fn new(chain: impl Into<Arc<str>>, route: impl Into<Arc<[StoreName]>>) -> Self {
        Self {
            chain: chain.into(),
            route: route.into(),
        }
    }

    fn frame(&self) -> RouteFrame {
        RouteFrame::new(Arc::clone(&self.chain), Arc::clone(&self.route))
    }
}

struct RouteScenario {
    name: &'static str,
    roots: Vec<RouteRoot>,
    synchronize_initial_hosts: bool,
    deadline: Duration,
}

impl RouteScenario {
    fn single(name: &'static str, chain: &'static str, route: &[StoreName]) -> Self {
        Self {
            name,
            roots: vec![RouteRoot::new(chain, Arc::<[StoreName]>::from(route))],
            synchronize_initial_hosts: false,
            deadline: SCENARIO_TIMEOUT,
        }
    }

    fn stores(&self) -> BTreeSet<StoreName> {
        self.roots
            .iter()
            .flat_map(|root| root.route.iter().copied())
            .collect()
    }
}

enum ScenarioOutcome {
    Completed,
    TimedOut(TimedOutScenario),
}

struct TimedOutScenario {
    report: String,
    cleanup_completed: bool,
    last_transitions: BTreeMap<(Arc<str>, StoreName), TraceEvent>,
}

impl ScenarioOutcome {
    fn expect_completed(self) {
        match self {
            Self::Completed => {}
            Self::TimedOut(timeout) => panic!("{}", timeout.report),
        }
    }

    fn expect_timeout(self) -> TimedOutScenario {
        match self {
            Self::Completed => panic!("scenario unexpectedly completed"),
            Self::TimedOut(timeout) => timeout,
        }
    }
}

trait ReentryExecutionPolicy: Send + Sync + 'static {
    const NAME: &'static str;

    fn run_routes(
        &self,
        scenario: RouteScenario,
        trace: TraceRecorder,
    ) -> BoxFuture<'_, wasmtime::Result<ScenarioOutcome>>;
}

#[derive(Clone, Copy)]
struct LegacySyncReentry;

impl ReentryExecutionPolicy for LegacySyncReentry {
    const NAME: &'static str = "LegacySyncReentry";

    #[allow(clippy::too_many_lines)]
    fn run_routes(
        &self,
        scenario: RouteScenario,
        trace: TraceRecorder,
    ) -> BoxFuture<'_, wasmtime::Result<ScenarioOutcome>> {
        Box::pin(async move {
            for root in &scenario.roots {
                trace.register_route(root);
            }
            let host_barrier = scenario
                .synchronize_initial_hosts
                .then_some(scenario.roots.len());
            let network = timeout(
                scenario.deadline,
                LegacyNetwork::build(
                    &scenario.stores(),
                    trace.clone(),
                    host_barrier,
                    host_barrier,
                    None,
                ),
            )
            .await
            .map_err(|_| {
                wasmtime::Error::msg(format!(
                    "conformance setup timed out\n{}",
                    trace.timeout_report(scenario.name)
                ))
            })??;
            let root_release = Arc::new(Barrier::new(scenario.roots.len() + 1));
            let mut roots = JoinSet::new();
            for root in scenario.roots {
                let network = Arc::clone(&network);
                let release = Arc::clone(&root_release);
                let frame = root.frame();
                roots.spawn(async move {
                    network.trace.record(
                        Arc::clone(&frame.chain),
                        frame.store(),
                        frame.index,
                        Transition::Yield,
                        "root-release-barrier",
                    );
                    release.wait().await;
                    network.trace.record(
                        Arc::clone(&frame.chain),
                        frame.store(),
                        frame.index,
                        Transition::Resume,
                        "root-release-barrier",
                    );
                    network.invoke(frame).await
                });
            }

            let coordinate = async {
                root_release.wait().await;
                if let Some(expected) = host_barrier {
                    let expected = u32::try_from(expected).map_err(|_| {
                        wasmtime::Error::msg("cross-enqueue coordination count overflow")
                    })?;
                    let signal = network
                        .cross_enqueue_signal
                        .as_ref()
                        .ok_or_else(|| {
                            wasmtime::Error::msg("missing cross-enqueue coordination signal")
                        })?
                        .clone();
                    let _permits = signal.acquire_many_owned(expected).await.map_err(|_| {
                        wasmtime::Error::msg("cross-enqueue coordination signal closed")
                    })?;
                }
                Ok::<(), wasmtime::Error>(())
            };
            let coordination = timeout(scenario.deadline, coordinate)
                .await
                .map_err(|_| {
                    wasmtime::Error::msg(format!(
                        "conformance coordination timed out\n{}",
                        trace.timeout_report(scenario.name)
                    ))
                })
                .and_then(std::convert::identity);
            if let Err(error) = coordination {
                network.cancellation.cancel();
                let _ = timeout(CLEANUP_TIMEOUT, async {
                    while roots.join_next().await.is_some() {}
                })
                .await;
                let _ = timeout(CLEANUP_TIMEOUT, network.shutdown_all()).await;
                return Err(error);
            }

            let execute = async {
                while let Some(result) = roots.join_next().await {
                    result.map_err(|error| {
                        wasmtime::Error::msg(format!("conformance root task failed: {error}"))
                    })??;
                }
                Ok::<(), wasmtime::Error>(())
            };

            if let Ok(result) = timeout(scenario.deadline, execute).await {
                if let Err(error) = result {
                    network.cancellation.cancel();
                    let _ = timeout(CLEANUP_TIMEOUT, async {
                        while roots.join_next().await.is_some() {}
                    })
                    .await;
                    let _ = timeout(CLEANUP_TIMEOUT, network.shutdown_all()).await;
                    return Err(error);
                }
                timeout(CLEANUP_TIMEOUT, network.shutdown_all())
                    .await
                    .map_err(|_| wasmtime::Error::msg("conformance shutdown timed out"))??;
                Ok(ScenarioOutcome::Completed)
            } else {
                let report = trace.timeout_report(scenario.name);
                let last_transitions = trace.last_transitions();
                network.cancellation.cancel();
                let roots_cleaned = timeout(CLEANUP_TIMEOUT, async {
                    while roots.join_next().await.is_some() {}
                })
                .await
                .is_ok();
                let stores_cleaned = timeout(CLEANUP_TIMEOUT, network.shutdown_all())
                    .await
                    .is_ok_and(|result| result.is_ok());
                Ok(ScenarioOutcome::TimedOut(TimedOutScenario {
                    report,
                    cleanup_completed: roots_cleaned && stores_cleaned,
                    last_transitions,
                }))
            }
        })
    }
}

struct ConformanceHarness<P: ReentryExecutionPolicy> {
    policy: P,
    trace: TraceRecorder,
    _policy: PhantomData<fn() -> P>,
}

impl<P: ReentryExecutionPolicy> ConformanceHarness<P> {
    fn new(policy: P) -> Self {
        Self {
            policy,
            trace: TraceRecorder::new(),
            _policy: PhantomData,
        }
    }

    async fn run_routes(&self, scenario: RouteScenario) -> wasmtime::Result<ScenarioOutcome> {
        self.policy.run_routes(scenario, self.trace.clone()).await
    }
}

fn run_in_runtime(future: impl Future<Output = ()> + Send + 'static) {
    let runtime = Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("build isolated conformance runtime");
    let outcome = catch_unwind(AssertUnwindSafe(|| runtime.block_on(future)));
    runtime.shutdown_timeout(CLEANUP_TIMEOUT);
    if let Err(payload) = outcome {
        resume_unwind(payload);
    }
}

fn resident_memory_bytes(system: &mut System) -> u64 {
    let pid = get_current_pid().expect("resolve conformance process ID");
    system.refresh_processes(ProcessesToUpdate::Some(&[pid]), false);
    system
        .process(pid)
        .expect("read conformance process metrics")
        .memory()
}

async fn build_test_network(
    stores: &BTreeSet<StoreName>,
    trace: TraceRecorder,
    initial_host_barrier: Option<usize>,
    hold: Option<HostHold>,
    expected_pairs: &[(&str, StoreName)],
    scenario: &str,
) -> Arc<LegacyNetwork> {
    for &(chain, store) in expected_pairs {
        trace.register_pair(chain, store, 0);
    }
    timeout(
        SCENARIO_TIMEOUT,
        LegacyNetwork::build(stores, trace.clone(), initial_host_barrier, None, hold),
    )
    .await
    .unwrap_or_else(|_| panic!("{}", trace.timeout_report(scenario)))
    .unwrap_or_else(|error| panic!("build {scenario} conformance network: {error:#}"))
}

fn assert_route_completes<P: ReentryExecutionPolicy>(policy: P, scenario: RouteScenario) {
    run_in_runtime(async move {
        let harness = ConformanceHarness::new(policy);
        let name = scenario.name;
        match harness.run_routes(scenario).await {
            Ok(outcome) => outcome.expect_completed(),
            Err(error) => panic!(
                "legacy conformance scenario failed: {error}\n{}",
                harness.trace.timeout_report(name)
            ),
        }
    });
}

#[test]
fn reentry_conformance_basic_root() {
    assert_route_completes(
        LegacySyncReentry,
        RouteScenario::single("basic-root", "root-1", &[StoreName::A]),
    );
}

#[test]
fn reentry_conformance_same_instance_cycle() {
    assert_route_completes(
        LegacySyncReentry,
        RouteScenario::single("same-instance", "root-1", &[StoreName::A, StoreName::A]),
    );
}

#[test]
fn reentry_conformance_cross_plugin_cycle() {
    assert_route_completes(
        LegacySyncReentry,
        RouteScenario::single(
            "cross-plugin",
            "root-1",
            &[StoreName::A, StoreName::B, StoreName::A],
        ),
    );
}

#[test]
fn reentry_conformance_three_store_cycle() {
    assert_route_completes(
        LegacySyncReentry,
        RouteScenario::single(
            "three-store",
            "root-1",
            &[StoreName::A, StoreName::B, StoreName::C, StoreName::A],
        ),
    );
}

#[test]
#[expect(clippy::print_stdout)]
fn reentry_conformance_records_short_legacy_baseline() {
    const WARMUP_CALLS: usize = 32;
    const LATENCY_CALLS: usize = 128;
    const THROUGHPUT_BATCHES: usize = 4;
    const CALLS_PER_BATCH: usize = STORE_QUEUE_CAPACITY;

    run_in_runtime(async move {
        let trace = TraceRecorder::new();
        let network = build_test_network(
            &BTreeSet::from([StoreName::A]),
            trace.clone(),
            None,
            None,
            &[("baseline-root", StoreName::A)],
            "short-baseline-setup",
        )
        .await;
        let mut system = System::new();
        let rss_before = resident_memory_bytes(&mut system);

        let measurement = async {
            for index in 0..WARMUP_CALLS {
                Arc::clone(&network)
                    .invoke(RouteFrame::new(
                        format!("baseline-warmup-{index}"),
                        [StoreName::A],
                    ))
                    .await?;
            }
            let rss_warm = resident_memory_bytes(&mut system);
            let mut peak_rss = rss_warm;

            let mut latencies = Vec::with_capacity(LATENCY_CALLS);
            for index in 0..LATENCY_CALLS {
                let started = Instant::now();
                Arc::clone(&network)
                    .invoke(RouteFrame::new(
                        format!("baseline-latency-{index}"),
                        [StoreName::A],
                    ))
                    .await?;
                latencies.push(started.elapsed());
            }
            latencies.sort_unstable();
            let p50_index = (latencies.len() * 50).div_ceil(100) - 1;
            let p95_index = (latencies.len() * 95).div_ceil(100) - 1;

            let throughput_started = Instant::now();
            for batch in 0..THROUGHPUT_BATCHES {
                let mut calls = JoinSet::new();
                for index in 0..CALLS_PER_BATCH {
                    let network = Arc::clone(&network);
                    calls.spawn(async move {
                        network
                            .invoke(RouteFrame::new(
                                format!("baseline-throughput-{batch}-{index}"),
                                [StoreName::A],
                            ))
                            .await
                    });
                }
                tokio::task::yield_now().await;
                peak_rss = peak_rss.max(resident_memory_bytes(&mut system));
                while let Some(result) = calls.join_next().await {
                    result.map_err(|error| {
                        wasmtime::Error::msg(format!("baseline root task failed: {error}"))
                    })??;
                }
                peak_rss = peak_rss.max(resident_memory_bytes(&mut system));
            }
            let throughput_elapsed = throughput_started.elapsed();
            let throughput_calls = THROUGHPUT_BATCHES * CALLS_PER_BATCH;
            let calls_per_second =
                (throughput_calls as u128 * 1_000_000_000) / throughput_elapsed.as_nanos().max(1);

            network.shutdown_all().await?;
            let rss_after_shutdown = resident_memory_bytes(&mut system);
            Ok::<_, wasmtime::Error>((
                latencies[p50_index].as_micros(),
                latencies[p95_index].as_micros(),
                calls_per_second,
                rss_warm,
                peak_rss,
                rss_after_shutdown,
            ))
        };

        let (p50_us, p95_us, calls_per_second, rss_warm, peak_rss, rss_after_shutdown) =
            timeout(Duration::from_secs(10), measurement)
                .await
                .unwrap_or_else(|_| panic!("{}", trace.timeout_report("short-baseline")))
                .expect("measure legacy conformance baseline");
        println!(
            "policy={} warmup_calls={WARMUP_CALLS} latency_calls={LATENCY_CALLS} throughput_calls={} concurrency={CALLS_PER_BATCH} p50_us={p50_us} p95_us={p95_us} calls_per_second={calls_per_second} max_pending_submission_depth={} queue_capacity={STORE_QUEUE_CAPACITY} rss_before_bytes={rss_before} rss_warm_bytes={rss_warm} peak_rss_bytes={peak_rss} rss_after_shutdown_bytes={rss_after_shutdown}",
            LegacySyncReentry::NAME,
            THROUGHPUT_BATCHES * CALLS_PER_BATCH,
            trace.max_queue_depth(),
        );
    });
}

#[test]
#[expect(clippy::print_stdout)]
fn reentry_conformance_opposing_roots_is_process_bounded() {
    let executable = std::env::current_exe().expect("resolve current test executable");
    let mut child = Command::new(executable)
        .args([
            "--exact",
            OPPOSING_ROOT_WORKER,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("PUMPKIN_REENTRY_CONFORMANCE_CHILD", "1")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn opposing-root conformance worker");
    let deadline = Instant::now() + OPPOSING_ROOT_PROCESS_TIMEOUT;

    loop {
        if let Some(status) = child.try_wait().expect("poll conformance worker") {
            let output = child.wait_with_output().expect("collect worker output");
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(
                status.success(),
                "opposing-root worker failed\nstdout:\n{}\nstderr:\n{}",
                stdout,
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                stdout.contains(OPPOSING_ROOT_WORKER_COMPLETE),
                "opposing-root worker exited without running its assertion path\nstdout:\n{stdout}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stderr)
            );
            println!("{stdout}");
            break;
        }
        if Instant::now() >= deadline {
            child.kill().expect("kill stuck conformance worker");
            let output = child.wait_with_output().expect("reap stuck worker");
            panic!(
                "opposing-root worker exceeded hard process deadline\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[ignore = "invoked by the process-bounded opposing-root parent test"]
#[expect(clippy::print_stdout)]
fn opposing_roots_legacy_worker() {
    assert_eq!(
        std::env::var("PUMPKIN_REENTRY_CONFORMANCE_CHILD").as_deref(),
        Ok("1"),
        "worker must only run in the isolated child process"
    );
    run_in_runtime(async move {
        let harness = ConformanceHarness::new(LegacySyncReentry);
        let outcome = harness
            .run_routes(RouteScenario {
                name: "opposing-roots",
                roots: vec![
                    RouteRoot::new(
                        "root-1",
                        Arc::<[StoreName]>::from([StoreName::A, StoreName::B, StoreName::A]),
                    ),
                    RouteRoot::new(
                        "root-2",
                        Arc::<[StoreName]>::from([StoreName::B, StoreName::A, StoreName::B]),
                    ),
                ],
                synchronize_initial_hosts: true,
                deadline: OPPOSING_ROOT_TIMEOUT,
            })
            .await
            .expect("run opposing-root scenario");
        let TimedOutScenario {
            report,
            cleanup_completed,
            last_transitions,
        } = outcome.expect_timeout();
        for (chain, store, depth, transition, detail) in [
            (
                "root-1",
                StoreName::A,
                0,
                Transition::Yield,
                "outbound-to-B",
            ),
            ("root-1", StoreName::B, 1, Transition::Enqueue, "guest-call"),
            (
                "root-2",
                StoreName::B,
                0,
                Transition::Yield,
                "outbound-to-A",
            ),
            ("root-2", StoreName::A, 1, Transition::Enqueue, "guest-call"),
        ] {
            let event = last_transitions
                .get(&(Arc::<str>::from(chain), store))
                .unwrap_or_else(|| {
                    panic!("missing terminal event for chain={chain} store={store}")
                });
            assert_eq!(
                event.depth, depth,
                "wrong terminal depth for {chain}/{store}"
            );
            assert_eq!(
                event.transition, transition,
                "wrong terminal transition for {chain}/{store}\n{report}"
            );
            assert_eq!(
                event.detail.as_ref(),
                detail,
                "wrong terminal detail for {chain}/{store}\n{report}"
            );
        }
        println!(
            "policy={} expected=known-legacy-limitation cleanup_completed={cleanup_completed}\n{report}",
            LegacySyncReentry::NAME
        );
        println!("{OPPOSING_ROOT_WORKER_COMPLETE}");
    });
}

struct NoopJob;

impl StoreJob for NoopJob {
    fn run(
        self: Box<Self>,
        _accessor: &wasmtime::component::Accessor<PluginHostState>,
    ) -> StoreFuture<'_, ()> {
        Box::pin(async move { Ok(()) })
    }
}

#[test]
fn reentry_conformance_unrelated_root_waits_for_active_chain() {
    run_in_runtime(async move {
        let trace = TraceRecorder::new();
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let hold = HostHold {
            chain: Arc::from("active-root"),
            store: StoreName::A,
            depth: 0,
            entered: Arc::clone(&entered),
            release: Arc::clone(&release),
        };
        let network = build_test_network(
            &BTreeSet::from([StoreName::A]),
            trace.clone(),
            None,
            Some(hold),
            &[
                ("active-root", StoreName::A),
                ("unrelated-root", StoreName::A),
            ],
            "unrelated-root-setup",
        )
        .await;
        let active_network = Arc::clone(&network);
        let active = tokio::spawn(async move {
            active_network
                .invoke(RouteFrame::new("active-root", [StoreName::A]))
                .await
        });
        timeout(SCENARIO_TIMEOUT, entered.notified())
            .await
            .unwrap_or_else(|_| panic!("{}", trace.timeout_report("unrelated-root-enter")));

        let unrelated =
            Arc::clone(&network).invoke(RouteFrame::new("unrelated-root", [StoreName::A]));
        tokio::pin!(unrelated);
        assert!(
            matches!(futures::poll!(unrelated.as_mut()), Poll::Pending),
            "unrelated root must remain pending behind the active sync chain"
        );
        assert!(
            trace.has_transition("unrelated-root", StoreName::A, Transition::Enqueue)
                && !trace.has_transition("unrelated-root", StoreName::A, Transition::Start),
            "unrelated root entered the active Store before release:\n{}",
            trace.timeout_report("unrelated-root-order")
        );

        release.notify_one();
        timeout(SCENARIO_TIMEOUT, async {
            active
                .await
                .expect("active root task")
                .expect("active root result");
            unrelated.await.expect("unrelated root result");
            network
                .shutdown_all()
                .await
                .expect("shutdown unrelated network");
        })
        .await
        .unwrap_or_else(|_| panic!("{}", trace.timeout_report("unrelated-root-finish")));
    });
}

#[test]
fn reentry_conformance_queue_saturation_is_deterministic() {
    run_in_runtime(async move {
        let trace = TraceRecorder::new();
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let hold = HostHold {
            chain: Arc::from("queue-owner"),
            store: StoreName::A,
            depth: 0,
            entered: Arc::clone(&entered),
            release: Arc::clone(&release),
        };
        let network = build_test_network(
            &BTreeSet::from([StoreName::A]),
            trace.clone(),
            None,
            Some(hold),
            &[
                ("queue-owner", StoreName::A),
                ("blocked-root", StoreName::A),
            ],
            "queue-saturation-setup",
        )
        .await;
        let active_network = Arc::clone(&network);
        let active = tokio::spawn(async move {
            active_network
                .invoke(RouteFrame::new("queue-owner", [StoreName::A]))
                .await
        });
        timeout(SCENARIO_TIMEOUT, entered.notified())
            .await
            .unwrap_or_else(|_| panic!("{}", trace.timeout_report("queue-owner-enter")));

        let driver = network
            .endpoint(StoreName::A)
            .expect("queue endpoint")
            .driver;
        let scenario = async {
            for index in 0..STORE_QUEUE_CAPACITY {
                driver
                    .sender
                    .send(StoreMessage::Call(Box::new(NoopJob)))
                    .await
                    .expect("fill live conformance Store queue");
                let depth = driver.sender.max_capacity() - driver.sender.capacity();
                trace.record_queue_depth(
                    format!("fill-{index}"),
                    StoreName::A,
                    0,
                    Transition::Enqueue,
                    depth,
                    "live-main-queue",
                );
            }
            assert_eq!(driver.sender.capacity(), 0);

            let blocked = driver.call(|_| Box::pin(async move { Ok("recovered") }));
            tokio::pin!(blocked);
            assert!(
                matches!(futures::poll!(blocked.as_mut()), Poll::Pending),
                "the call beyond production queue capacity must wait"
            );
            trace.record_queue_depth(
                "blocked-root",
                StoreName::A,
                0,
                Transition::Yield,
                STORE_QUEUE_CAPACITY,
                "waiting-for-main-queue-capacity",
            );

            release.notify_one();
            active
                .await
                .expect("queue owner task")
                .expect("queue owner result");
            assert_eq!(
                blocked.await.expect("call admitted after drain"),
                "recovered"
            );
            trace.record_queue_depth(
                "blocked-root",
                StoreName::A,
                0,
                Transition::Resume,
                driver.sender.max_capacity() - driver.sender.capacity(),
                "completed-after-live-queue-drain",
            );
            assert_eq!(trace.max_queue_depth(), STORE_QUEUE_CAPACITY);
            network
                .shutdown_all()
                .await
                .expect("shutdown queue saturation network");
        };
        timeout(SCENARIO_TIMEOUT, scenario)
            .await
            .unwrap_or_else(|_| panic!("{}", trace.timeout_report("queue-saturation")));
    });
}

#[test]
fn reentry_conformance_dropped_accepted_receiver_still_completes() {
    run_in_runtime(async move {
        let trace = TraceRecorder::new();
        let store = ConcurrentStore::new(test_store());
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let completed = Arc::new(Notify::new());
        let side_effect = Arc::new(AtomicUsize::new(0));

        let call_store = store.clone();
        let call_trace = trace.clone();
        let call_started = Arc::clone(&started);
        let call_release = Arc::clone(&release);
        let call_completed = Arc::clone(&completed);
        let call_side_effect = Arc::clone(&side_effect);
        let waiter = tokio::spawn(async move {
            call_trace.record(
                "dropped-waiter",
                StoreName::A,
                0,
                Transition::Enqueue,
                "store-call",
            );
            call_store
                .call(move |_| {
                    Box::pin(async move {
                        call_trace.record(
                            "dropped-waiter",
                            StoreName::A,
                            0,
                            Transition::Start,
                            "accepted-store-call",
                        );
                        call_started.notify_one();
                        call_release.notified().await;
                        call_side_effect.fetch_add(1, Ordering::SeqCst);
                        call_trace.record(
                            "dropped-waiter",
                            StoreName::A,
                            0,
                            Transition::Complete,
                            "result-receiver-dropped",
                        );
                        call_completed.notify_one();
                        Ok(())
                    })
                })
                .await
        });

        timeout(SCENARIO_TIMEOUT, started.notified())
            .await
            .unwrap_or_else(|_| panic!("{}", trace.timeout_report("dropped-waiter-start")));
        waiter.abort();
        assert!(
            waiter
                .await
                .expect_err("caller task must be aborted")
                .is_cancelled()
        );
        release.notify_one();
        timeout(SCENARIO_TIMEOUT, completed.notified())
            .await
            .unwrap_or_else(|_| panic!("{}", trace.timeout_report("dropped-waiter-complete")));

        timeout(SCENARIO_TIMEOUT, async {
            let observed = store
                .call(move |_| Box::pin(async move { Ok(side_effect.load(Ordering::SeqCst)) }))
                .await
                .expect("follow-up Store call");
            assert_eq!(observed, 1, "accepted work must outlive its waiter");
            trace.record("lifecycle", StoreName::A, 0, Transition::Shutdown, "begin");
            store
                .shutdown(|_| Box::pin(async move { Ok(()) }))
                .await
                .expect("shutdown dropped-waiter Store");
        })
        .await
        .unwrap_or_else(|_| panic!("{}", trace.timeout_report("dropped-waiter-finish")));
    });
}

#[test]
fn reentry_conformance_shutdown_admits_required_callback() {
    run_in_runtime(async move {
        let trace = TraceRecorder::new();
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let hold = HostHold {
            chain: Arc::from("shutdown-root"),
            store: StoreName::A,
            depth: 0,
            entered: Arc::clone(&entered),
            release: Arc::clone(&release),
        };
        let network = build_test_network(
            &BTreeSet::from([StoreName::A]),
            trace.clone(),
            None,
            Some(hold),
            &[("shutdown-root", StoreName::A), ("lifecycle", StoreName::A)],
            "shutdown-setup",
        )
        .await;
        let active_network = Arc::clone(&network);
        let active = tokio::spawn(async move {
            active_network
                .invoke(RouteFrame::new(
                    "shutdown-root",
                    [StoreName::A, StoreName::A],
                ))
                .await
        });
        timeout(SCENARIO_TIMEOUT, entered.notified())
            .await
            .unwrap_or_else(|_| panic!("{}", trace.timeout_report("shutdown-enter")));

        let driver = network
            .endpoint(StoreName::A)
            .expect("shutdown endpoint")
            .driver;
        trace.record(
            "lifecycle",
            StoreName::A,
            0,
            Transition::Shutdown,
            "admission-requested",
        );
        let shutdown = driver.shutdown(|_| Box::pin(async move { Ok(()) }));
        tokio::pin!(shutdown);
        assert!(
            matches!(futures::poll!(shutdown.as_mut()), Poll::Pending),
            "shutdown must wait for the active accepted chain"
        );
        assert!(
            !driver.accepting.load(Ordering::Acquire),
            "one poll must commit shutdown admission before callback release"
        );
        trace.record(
            "lifecycle",
            StoreName::A,
            0,
            Transition::Shutdown,
            "admission-committed",
        );

        release.notify_one();
        timeout(SCENARIO_TIMEOUT, async {
            active
                .await
                .expect("shutdown root task")
                .expect("shutdown root result");
            shutdown.await.expect("shutdown result");
        })
        .await
        .unwrap_or_else(|_| panic!("{}", trace.timeout_report("shutdown-finish")));
        assert!(
            trace.has_transition("shutdown-root", StoreName::A, Transition::Complete),
            "nested callback did not complete during drain"
        );
    });
}

#[test]
fn reentry_conformance_recursion_limit_records_legacy_store_loss() {
    run_in_runtime(async move {
        let trace = TraceRecorder::new();
        let network = build_test_network(
            &BTreeSet::from([StoreName::A]),
            trace.clone(),
            None,
            None,
            &[
                ("recursion-limit", StoreName::A),
                ("recovery-root", StoreName::A),
            ],
            "recursion-setup",
        )
        .await;
        let route = vec![StoreName::A; MAX_SYNC_REENTRY_DEPTH + 2];
        let error = timeout(
            SCENARIO_TIMEOUT,
            Arc::clone(&network).invoke(RouteFrame::new("recursion-limit", route)),
        )
        .await
        .unwrap_or_else(|_| panic!("{}", trace.timeout_report("recursion-limit")))
        .expect_err("one call beyond the recursion limit must fail");
        assert!(
            trace.has_detail("recursion-limit", "maximum depth"),
            "unexpected recursion error: {error:#}\n{}",
            trace.timeout_report("recursion-limit-error")
        );

        let driver = network
            .endpoint(StoreName::A)
            .expect("recursion endpoint")
            .driver;
        assert!(
            driver
                .reentry
                .scopes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty(),
            "reentry scopes must be cleaned after the limit trap"
        );
        assert!(
            driver
                .reentry
                .active_contexts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty(),
            "active contexts must be cleaned after the limit trap"
        );

        let recovery_error = timeout(
            SCENARIO_TIMEOUT,
            Arc::clone(&network).invoke(RouteFrame::new("recovery-root", [StoreName::A])),
        )
        .await
        .unwrap_or_else(|_| panic!("{}", trace.timeout_report("recursion-recovery")))
        .expect_err("legacy Store driver currently stops after the recursion trap");
        assert!(
            recovery_error
                .to_string()
                .contains("store driver is not running"),
            "unexpected legacy recovery result: {recovery_error:#}"
        );

        let recovered_trace = TraceRecorder::new();
        let recovered_network = build_test_network(
            &BTreeSet::from([StoreName::A]),
            recovered_trace.clone(),
            None,
            None,
            &[("replacement-root", StoreName::A)],
            "recursion-replacement-setup",
        )
        .await;
        timeout(
            SCENARIO_TIMEOUT,
            Arc::clone(&recovered_network)
                .invoke(RouteFrame::new("replacement-root", [StoreName::A])),
        )
        .await
        .unwrap_or_else(|_| panic!("{}", recovered_trace.timeout_report("replacement-root")))
        .expect("replacement Store must accept a fresh root");
        timeout(CLEANUP_TIMEOUT, recovered_network.shutdown_all())
            .await
            .unwrap_or_else(|_| {
                panic!(
                    "{}",
                    recovered_trace.timeout_report("recursion-replacement-shutdown")
                )
            })
            .expect("shutdown replacement recursion network");
    });
}
