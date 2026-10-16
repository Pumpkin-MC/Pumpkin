//! Builds the fixture in `tests/fixture` into a component and runs it on Wasmtime
//!
//! The fixture is compiled for `wasm32-wasip2`, so the first run needs
//! `rustup target add wasm32-wasip2`

// The Android emulator job has no cargo to build the fixture with
#![cfg(not(target_os = "android"))]
#![expect(
    clippy::expect_used,
    reason = "test setup failures should stop the test"
)]

use std::{
    path::PathBuf,
    process::Command,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicU32, Ordering},
    },
};

use tokio::sync::{Notify, oneshot};
use wasmtime::{
    Config, Engine, Store,
    component::{Accessor, Component, Func, Instance, Linker, Val, types::ComponentItem},
};

const METADATA_INSTANCE: &str = "pumpkin:plugin/metadata@0.2.0";
const GAMETEST_INSTANCE: &str = "pumpkin:plugin/gametest-callbacks@0.2.0";
const IPC_INSTANCE: &str = "pumpkin:plugin/ipc@0.2.0";
const SCHEDULER_INSTANCE: &str = "pumpkin:plugin/scheduler@0.2.0";

const ASYNC_CALLBACKS: [&str; 13] = [
    "on-load",
    "on-unload",
    "handle-event",
    "handle-command",
    "handle-command-suggestion",
    "handle-task",
    "handle-ipc-message",
    "handle-ai-goal-can-start",
    "handle-ai-goal-should-continue",
    "handle-ai-goal-start",
    "handle-ai-goal-tick",
    "handle-ai-goal-stop",
    "handle-generate-phase",
];

// Opt-level 1 because an unoptimized build of the lifted `event` variant exceeds the Wasm locals
// limit and the component fails validation
fn fixture_path() -> &'static PathBuf {
    static FIXTURE: OnceLock<PathBuf> = OnceLock::new();
    FIXTURE.get_or_init(|| {
        let target_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("component-fixture");
        let status = Command::new(env!("CARGO"))
            .args([
                "build",
                "--package",
                "pumpkin-plugin-api-v0_2",
                "--example",
                "component_fixture",
                "--target",
                "wasm32-wasip2",
                "--target-dir",
            ])
            .arg(&target_dir)
            .env("CARGO_PROFILE_DEV_OPT_LEVEL", "1")
            .status()
            .expect("failed to run cargo to build the fixture");
        assert!(status.success(), "building the fixture component failed");
        target_dir.join("wasm32-wasip2/debug/examples/component_fixture.wasm")
    })
}

fn engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    config.concurrency_support(true);
    Engine::new(&config).expect("failed to create the Wasmtime engine")
}

fn load(engine: &Engine) -> Component {
    Component::from_file(engine, fixture_path())
        .expect("the built fixture is not a valid component")
}

fn instance_funcs(
    engine: &Engine,
    component: &Component,
    instance: &str,
    imports: bool,
) -> Vec<(String, bool)> {
    let ty = component.component_type();
    let mut items: Box<dyn Iterator<Item = (&str, ComponentItem)>> = if imports {
        Box::new(ty.imports(engine).map(|(name, item)| (name, item.ty)))
    } else {
        Box::new(ty.exports(engine).map(|(name, item)| (name, item.ty)))
    };
    let Some((_, ComponentItem::ComponentInstance(instance))) =
        items.find(|(name, _)| *name == instance)
    else {
        return Vec::new();
    };
    instance
        .exports(engine)
        .filter_map(|(name, item)| match item.ty {
            ComponentItem::ComponentFunc(func) => Some((name.to_string(), func.async_())),
            _ => None,
        })
        .collect()
}

fn is_async(funcs: &[(String, bool)], name: &str) -> bool {
    funcs
        .iter()
        .find(|(candidate, _)| candidate == name)
        .map(|(_, is_async)| *is_async)
        .expect("the function is missing from the component type")
}

#[test]
fn component_declares_the_mixed_v0_2_world() {
    let engine = engine();
    let component = load(&engine);
    let ty = component.component_type();

    for name in ASYNC_CALLBACKS {
        let export = ty.get_export(&engine, name).map(|export| export.ty);
        assert!(
            matches!(export, Some(ComponentItem::ComponentFunc(func)) if func.async_()),
            "{name} should be an async export"
        );
    }

    let init = ty
        .get_export(&engine, "init-plugin")
        .map(|export| export.ty);
    assert!(
        matches!(init, Some(ComponentItem::ComponentFunc(func)) if !func.async_()),
        "bootstrap initialization should stay synchronous"
    );

    // Metadata and the non-async GameTest callbacks keep their synchronous signatures
    let metadata = instance_funcs(&engine, &component, METADATA_INSTANCE, false);
    assert!(
        !is_async(&metadata, "get-metadata"),
        "get-metadata should stay synchronous"
    );
    let gametest = instance_funcs(&engine, &component, GAMETEST_INSTANCE, false);
    assert!(
        is_async(&gametest, "invoke-async-test"),
        "invoke-async-test should be async"
    );
    for sync in [
        "invoke-void",
        "invoke-test",
        "invoke-block-predicate",
        "invoke-entity-predicate",
    ] {
        assert!(!is_async(&gametest, sync), "{sync} should stay synchronous");
    }

    // The fixture imports one async host function and one synchronous one
    let ipc = instance_funcs(&engine, &component, IPC_INSTANCE, true);
    assert!(
        is_async(&ipc, "send-ipc-message"),
        "send-ipc-message should be an async import"
    );
    let scheduler = instance_funcs(&engine, &component, SCHEDULER_INSTANCE, true);
    assert!(
        !is_async(&scheduler, "cancel-task"),
        "cancel-task should be a synchronous import"
    );
}

struct State {
    cancelled_tasks: AtomicU32,
    // Signals that the guest entered the host import and is now waiting on it
    entered: Notify,
    // Held by the host import until the test lets the awaited work finish
    release: std::sync::Mutex<Option<oneshot::Receiver<()>>>,
}

async fn instantiate(
    engine: &Engine,
    component: &Component,
    state: State,
) -> (Store<Arc<State>>, Instance) {
    let state = Arc::new(state);
    let mut store = Store::new(engine, Arc::clone(&state));
    let mut linker = Linker::<Arc<State>>::new(engine);

    linker
        .instance(SCHEDULER_INSTANCE)
        .expect("failed to define the scheduler import instance")
        .func_wrap("cancel-task", |store, (_task,): (u32,)| {
            store.data().cancelled_tasks.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("failed to define cancel-task");

    // The reply is produced only after the test releases the host side, so the guest has to
    // suspend at its await and resume when the host completes the import
    linker
        .instance(IPC_INSTANCE)
        .expect("failed to define the ipc import instance")
        .func_wrap_concurrent(
            "send-ipc-message",
            |accessor: &Accessor<Arc<State>>, (recipient, mut message): (String, Vec<u8>)| {
                Box::pin(async move {
                    let state = accessor.with(|mut store| Arc::clone(store.data_mut()));
                    if recipient == "slow" {
                        let release = state
                            .release
                            .lock()
                            .expect("release lock was poisoned")
                            .take()
                            .expect("only one send should wait on the release");
                        state.entered.notify_one();
                        release.await.expect(
                            "the test dropped the release sender before the host import finished",
                        );
                    }
                    message.extend_from_slice(b"!");
                    Ok((Ok::<_, ()>(Ok::<_, String>(message)),))
                })
            },
        )
        .expect("failed to define send-ipc-message");

    linker
        .define_unknown_imports_as_traps(component)
        .expect("failed to stub the remaining imports");
    let instance = linker
        .instantiate_async(&mut store, component)
        .await
        .expect("failed to instantiate the component");
    (store, instance)
}

fn export_func(store: &mut Store<Arc<State>>, instance: &Instance, name: &str) -> Func {
    instance
        .get_func(store, name)
        .expect("the export is missing from the component")
}

fn ipc_params(sender: &str, bytes: &[u8]) -> [Val; 2] {
    [
        Val::String(sender.into()),
        Val::List(bytes.iter().map(|byte| Val::U8(*byte)).collect()),
    ]
}

fn ipc_reply(result: &Val) -> Option<Vec<u8>> {
    let Val::Result(Ok(Some(reply))) = result else {
        return None;
    };
    let Val::List(bytes) = &**reply else {
        return None;
    };
    bytes
        .iter()
        .map(|byte| match byte {
            Val::U8(byte) => Some(*byte),
            _ => None,
        })
        .collect()
}

#[tokio::test]
async fn async_handler_awaits_a_host_import_while_other_calls_proceed() {
    let engine = engine();
    let component = load(&engine);
    let (release, released) = oneshot::channel();
    let (mut store, instance) = instantiate(
        &engine,
        &component,
        State {
            cancelled_tasks: AtomicU32::new(0),
            entered: Notify::new(),
            release: std::sync::Mutex::new(Some(released)),
        },
    )
    .await;
    let state = Arc::clone(store.data());

    let init = export_func(&mut store, &instance, "init-plugin");
    let handle_ipc = export_func(&mut store, &instance, "handle-ipc-message");
    let metadata_instance = instance
        .get_export_index(&mut store, None, METADATA_INSTANCE)
        .expect("the metadata instance export is missing");
    let get_metadata_index = instance
        .get_export_index(&mut store, Some(&metadata_instance), "get-metadata")
        .expect("the get-metadata export is missing");
    let get_metadata = instance
        .get_func(&mut store, get_metadata_index)
        .expect("failed to look up get-metadata");

    store
        .run_concurrent(async |accessor| {
            init.call_concurrent(accessor, &[], &mut [])
                .await
                .expect("calling init-plugin failed");

            let mut metadata = [Val::Bool(false)];
            get_metadata
                .call_concurrent(accessor, &[], &mut metadata)
                .await
                .expect("calling get-metadata failed");
            let name = ("name".to_string(), Val::String("fixture".into()));
            assert!(
                matches!(&metadata[0], Val::Record(fields) if fields.first() == Some(&name)),
                "metadata should start with the fixture name"
            );

            // The slow call stays suspended in the guest on its host import while a second
            // call into the same instance completes, which a blocking wait could not allow
            let slow_params = ipc_params("slow", b"slow");
            let mut slow_result = [Val::Bool(false)];
            let slow = handle_ipc.call_concurrent(accessor, &slow_params, &mut slow_result);
            let fast = async {
                state.entered.notified().await;
                let params = ipc_params("fast", b"fast");
                let mut result = [Val::Bool(false)];
                handle_ipc
                    .call_concurrent(accessor, &params, &mut result)
                    .await
                    .expect("the fast handle-ipc-message call failed");
                assert_eq!(ipc_reply(&result[0]).as_deref(), Some(&b"fast!"[..]));
                release
                    .send(())
                    .expect("the slow import was no longer waiting for the release");
            };
            let (slow, ()) = tokio::join!(slow, fast);
            slow.expect("the slow handle-ipc-message call failed");
            assert_eq!(ipc_reply(&slow_result[0]).as_deref(), Some(&b"slow!"[..]));
        })
        .await
        .expect("running the guest calls failed");

    // The synchronous import ran once per handled message
    assert_eq!(state.cancelled_tasks.load(Ordering::SeqCst), 2);
}
