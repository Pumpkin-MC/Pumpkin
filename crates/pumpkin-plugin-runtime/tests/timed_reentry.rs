use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc, Barrier, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    thread::ThreadId,
    time::{Duration, Instant},
};

use pumpkin_plugin_runtime::{
    LegacyStore, LegacySyncReentry, RuntimeSpawner, SpawnError, SpawnFuture, StoreHandle,
};
use tokio::{runtime::Handle as RuntimeHandle, task::JoinSet, time::timeout};
use wasm_encoder::{
    CodeSection, ComponentBuilder, ComponentExportKind, ComponentTypeRef, EntityType, ExportKind,
    ExportSection, Function, FunctionSection, ImportSection, Instruction, Module, ModuleArg,
    PrimitiveValType, TypeSection, ValType,
};
use wasmtime::{
    Config, Engine, Store,
    component::{Component, Linker, TypedFunc},
};

const ROOTS: usize = 10;
const TEST_TIMEOUT: Duration = Duration::from_secs(10);

struct TestHostState;

type PluginHandle = StoreHandle<TestHostState, LegacySyncReentry>;
type Run = TypedFunc<(u32, u32), ()>;

struct TokioSpawner {
    runtime: RuntimeHandle,
}

impl RuntimeSpawner for TokioSpawner {
    fn spawn(&self, task: SpawnFuture) -> Result<(), SpawnError> {
        drop(self.runtime.spawn(task));
        Ok(())
    }

    fn spawn_blocking(&self, task: Box<dyn FnOnce() + Send + 'static>) -> Result<(), SpawnError> {
        drop(self.runtime.spawn_blocking(task));
        Ok(())
    }
}

#[derive(Clone)]
struct Trace(Arc<TraceInner>);

struct TraceInner {
    started: Instant,
    events: Mutex<Vec<TraceEvent>>,
}

#[derive(Clone)]
struct TraceEvent {
    sequence: usize,
    elapsed: Duration,
    thread: ThreadId,
    message: String,
}

impl Trace {
    fn new() -> Self {
        Self(Arc::new(TraceInner {
            started: Instant::now(),
            events: Mutex::new(Vec::new()),
        }))
    }

    fn record(&self, message: impl Into<String>) {
        let mut events = self.0.events.lock().expect("trace lock");
        let sequence = events.len();
        events.push(TraceEvent {
            sequence,
            elapsed: self.0.started.elapsed(),
            thread: std::thread::current().id(),
            message: message.into(),
        });
    }

    fn elapsed(&self) -> Duration {
        self.0.started.elapsed()
    }

    fn render(&self) -> String {
        let mut events = self.0.events.lock().expect("trace lock").clone();
        events.sort_by_key(|event| event.sequence);
        events
            .into_iter()
            .map(|event| {
                format!(
                    "[+{:>9.3} ms] {} on {:?}",
                    event.elapsed.as_secs_f64() * 1_000.0,
                    event.message,
                    event.thread
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn update_max(maximum: &AtomicUsize, candidate: usize) {
    let mut current = maximum.load(Ordering::Acquire);
    while candidate > current {
        match maximum.compare_exchange_weak(current, candidate, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => break,
            Err(observed) => current = observed,
        }
    }
}

fn relay_component() -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32, ValType::I32], []);
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
    body.instruction(&Instruction::LocalGet(0));
    body.instruction(&Instruction::LocalGet(1));
    body.instruction(&Instruction::Call(0));
    body.instruction(&Instruction::End);
    let mut code = CodeSection::new();
    code.function(&body);
    module.section(&code);

    let mut component = ComponentBuilder::default();
    let (function_type, mut function) = component.type_function(Some("run-type"));
    function
        .params([
            ("root", PrimitiveValType::U32),
            ("step", PrimitiveValType::U32),
        ])
        .result(None);
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

fn boxed<'a, T>(
    future: impl Future<Output = T> + Send + 'a,
) -> Pin<Box<dyn Future<Output = T> + Send + 'a>> {
    Box::pin(future)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn ten_parallel_cross_plugin_reentry_routes_complete() {
    let trace = Trace::new();
    trace.record("Load all plugins...");

    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    config.concurrency_support(true);
    let engine = Engine::new(&config).expect("test engine");
    let component = Component::new(&engine, relay_component()).expect("test component");

    trace.record("Load plugin A");
    let a_handle_slot = Arc::new(OnceLock::<PluginHandle>::new());
    let b_handle_slot = Arc::new(OnceLock::<PluginHandle>::new());
    let a_run_slot = Arc::new(OnceLock::<Run>::new());
    let b_run_slot = Arc::new(OnceLock::<Run>::new());

    let mut a_linker = Linker::<TestHostState>::new(&engine);
    let a_handle_for_a = Arc::clone(&a_handle_slot);
    let b_handle_for_a = Arc::clone(&b_handle_slot);
    let b_run_for_a = Arc::clone(&b_run_slot);
    let trace_for_a = trace.clone();
    a_linker
        .root()
        .func_wrap_async("host", move |mut store, (root, step): (u32, u32)| {
            let a_handle = a_handle_for_a.get().expect("A handle initialized").clone();
            let b_handle = b_handle_for_a.get().expect("B handle initialized").clone();
            let b_run = *b_run_for_a.get().expect("B run initialized");
            let trace = trace_for_a.clone();
            Box::new(async move {
                match step {
                    0 => {
                        trace.record(format!("Host run B for A {root}"));
                        let nested_trace = trace.clone();
                        let outbound = b_handle.call_guest(move |mut context| {
                            nested_trace.record(format!("Lock B for A {root}"));
                            nested_trace.record(format!("Start B for A {root}"));
                            boxed(async move { context.call(b_run, (root, 1)).await })
                        });
                        a_handle.pump_reentry(&mut store, outbound).await??;
                        trace.record(format!("Resume A {root}"));
                    }
                    2 => trace.record(format!("A ({root}): A -> B -> A")),
                    _ => return Err(wasmtime::Error::msg("unexpected step routed to A")),
                }
                Ok(())
            })
        })
        .expect("link A host function");

    trace.record("Load plugin B");
    let mut b_linker = Linker::<TestHostState>::new(&engine);
    let a_handle_for_b = Arc::clone(&a_handle_slot);
    let b_handle_for_b = Arc::clone(&b_handle_slot);
    let a_run_for_b = Arc::clone(&a_run_slot);
    let trace_for_b = trace.clone();
    b_linker
        .root()
        .func_wrap_async("host", move |mut store, (root, step): (u32, u32)| {
            let a_handle = a_handle_for_b.get().expect("A handle initialized").clone();
            let b_handle = b_handle_for_b.get().expect("B handle initialized").clone();
            let a_run = *a_run_for_b.get().expect("A run initialized");
            let trace = trace_for_b.clone();
            Box::new(async move {
                if step != 1 {
                    return Err(wasmtime::Error::msg("unexpected step routed to B"));
                }
                trace.record(format!("Host reenter A for root {root}"));
                let nested_trace = trace.clone();
                let outbound = a_handle.call_guest(move |mut context| {
                    nested_trace.record(format!("Lock A reentry for root {root}"));
                    nested_trace.record(format!("Start A reentry for root {root}"));
                    boxed(async move { context.call(a_run, (root, 2)).await })
                });
                b_handle.pump_reentry(&mut store, outbound).await??;
                trace.record(format!("Resume B for A {root}"));
                Ok(())
            })
        })
        .expect("link B host function");

    let mut a_store = Store::new(&engine, TestHostState);
    let a_instance = a_linker
        .instantiate_async(&mut a_store, &component)
        .await
        .expect("instantiate A");
    let a_run = a_instance
        .get_typed_func::<(u32, u32), ()>(&mut a_store, "run")
        .expect("get A run export");
    assert!(a_run_slot.set(a_run).is_ok(), "set A run");

    let mut b_store = Store::new(&engine, TestHostState);
    let b_instance = b_linker
        .instantiate_async(&mut b_store, &component)
        .await
        .expect("instantiate B");
    let b_run = b_instance
        .get_typed_func::<(u32, u32), ()>(&mut b_store, "run")
        .expect("get B run export");
    assert!(b_run_slot.set(b_run).is_ok(), "set B run");

    let policy = LegacySyncReentry::new();
    let spawner: Arc<dyn RuntimeSpawner> = Arc::new(TokioSpawner {
        runtime: RuntimeHandle::current(),
    });
    let a_store = Arc::new(
        LegacyStore::start(a_store, policy.clone(), Arc::clone(&spawner))
            .await
            .expect("start A driver"),
    );
    let b_store = Arc::new(
        LegacyStore::start(b_store, policy, spawner)
            .await
            .expect("start B driver"),
    );
    assert!(a_handle_slot.set(a_store.handle()).is_ok(), "set A handle");
    assert!(b_handle_slot.set(b_store.handle()).is_ok(), "set B handle");
    trace.record("All plugins loaded.");

    let release = Arc::new(Barrier::new(ROOTS + 1));
    let active_roots = Arc::new(AtomicUsize::new(0));
    let max_active_roots = Arc::new(AtomicUsize::new(0));
    let mut roots = JoinSet::new();

    for root in 0..ROOTS {
        let release = Arc::clone(&release);
        let a_handle = a_store.handle();
        let trace = trace.clone();
        let active_roots = Arc::clone(&active_roots);
        let max_active_roots = Arc::clone(&max_active_roots);
        let runtime = RuntimeHandle::current();
        roots.spawn_blocking(move || {
            trace.record(format!("Host run A {root}"));
            release.wait();
            let root_started = Instant::now();
            let call_trace = trace.clone();
            runtime.block_on(a_handle.call_guest(move |mut context| {
                let active = active_roots.fetch_add(1, Ordering::AcqRel) + 1;
                update_max(&max_active_roots, active);
                call_trace.record(format!("Lock A for root {root}"));
                call_trace.record(format!("Start A {root}"));
                boxed(async move {
                    let result = context.call(a_run, (root as u32, 0)).await;
                    active_roots.fetch_sub(1, Ordering::AcqRel);
                    result
                })
            }))?;
            let elapsed = root_started.elapsed();
            trace.record(format!(
                "Complete A {root} ({:.3} ms)",
                elapsed.as_secs_f64() * 1_000.0
            ));
            Ok::<_, wasmtime::Error>((root, elapsed))
        });
    }

    release.wait();
    let mut durations = timeout(TEST_TIMEOUT, async {
        let mut durations = Vec::with_capacity(ROOTS);
        while let Some(result) = roots.join_next().await {
            durations.push(result.expect("root task").expect("root call"));
        }
        durations
    })
    .await
    .expect("parallel reentry roots deadlocked");

    durations.sort_by_key(|(root, _)| *root);
    assert_eq!(durations.len(), ROOTS);
    assert_eq!(max_active_roots.load(Ordering::Acquire), 1);

    a_store
        .shutdown(|_| boxed(async move { Ok(()) }))
        .await
        .expect("shut down A");
    b_store
        .shutdown(|_| boxed(async move { Ok(()) }))
        .await
        .expect("shut down B");

    let mut sorted = durations
        .iter()
        .map(|(_, duration)| *duration)
        .collect::<Vec<_>>();
    sorted.sort_unstable();
    let min = sorted[0];
    let median = sorted[sorted.len() / 2];
    let max = sorted[sorted.len() - 1];

    println!("{}", trace.render());
    println!(
        "Completed {ROOTS}/{ROOTS} | total={:.3} ms | min={:.3} ms | p50={:.3} ms | max={:.3} ms | max_parallel_roots={} | deadlocks=0",
        trace.elapsed().as_secs_f64() * 1_000.0,
        min.as_secs_f64() * 1_000.0,
        median.as_secs_f64() * 1_000.0,
        max.as_secs_f64() * 1_000.0,
        max_active_roots.load(Ordering::Acquire),
    );
}
