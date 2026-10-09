//! Builds the fixture in `tests/fixture` into a component and runs it on Wasmtime
//!
//! The fixture is compiled with the real `wasm32-wasip2` toolchain, so the first run needs
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

const ASYNC_CALLBACKS: [&str; 14] = [
    "init-plugin",
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

// Unoptimized codegen for the lifted `event` variant overflows the Wasm locals limit
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
            .expect("cargo should start");
        assert!(status.success(), "building the fixture component failed");
        target_dir.join("wasm32-wasip2/debug/examples/component_fixture.wasm")
    })
}

fn engine() -> Engine {
    let mut config = Config::new();
    config.wasm_component_model(true);
    config.wasm_component_model_async(true);
    config.concurrency_support(true);
    Engine::new(&config).expect("engine")
}

fn load(engine: &Engine) -> Component {
    Component::from_file(engine, fixture_path()).expect("fixture should be a valid component")
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
        .expect("function should exist")
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

    // Metadata and the non-async GameTest callbacks keep their synchronous signatures
    let metadata = instance_funcs(&engine, &component, METADATA_INSTANCE, false);
    assert!(!is_async(&metadata, "get-metadata"));
    let gametest = instance_funcs(&engine, &component, GAMETEST_INSTANCE, false);
    assert!(is_async(&gametest, "invoke-async-test"));
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
    assert!(is_async(&ipc, "send-ipc-message"));
    let scheduler = instance_funcs(&engine, &component, SCHEDULER_INSTANCE, true);
    assert!(!is_async(&scheduler, "cancel-task"));
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
        .expect("scheduler instance")
        .func_wrap("cancel-task", |store, (_task,): (u32,)| {
            store.data().cancelled_tasks.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .expect("cancel-task");

    // The reply is produced only after the test releases the host side, so the guest has to
    // suspend at its await and resume when the host completes the import
    linker
        .instance(IPC_INSTANCE)
        .expect("ipc instance")
        .func_wrap_concurrent(
            "send-ipc-message",
            |accessor: &Accessor<Arc<State>>, (recipient, mut message): (String, Vec<u8>)| {
                Box::pin(async move {
                    let state = accessor.with(|mut store| Arc::clone(store.data_mut()));
                    if recipient == "slow" {
                        let release = state
                            .release
                            .lock()
                            .expect("release lock")
                            .take()
                            .expect("slow send should happen once");
                        state.entered.notify_one();
                        release.await.expect("test should release the host import");
                    }
                    message.extend_from_slice(b"!");
                    Ok((Ok::<_, ()>(Ok::<_, String>(message)),))
                })
            },
        )
        .expect("send-ipc-message");

    linker
        .define_unknown_imports_as_traps(component)
        .expect("remaining imports");
    let instance = linker
        .instantiate_async(&mut store, component)
        .await
        .expect("instantiate");
    (store, instance)
}

fn export_func(store: &mut Store<Arc<State>>, instance: &Instance, name: &str) -> Func {
    instance.get_func(store, name).expect("export should exist")
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
        .expect("metadata instance");
    let get_metadata_index = instance
        .get_export_index(&mut store, Some(&metadata_instance), "get-metadata")
        .expect("get-metadata");
    let get_metadata = instance
        .get_func(&mut store, get_metadata_index)
        .expect("get-metadata func");

    store
        .run_concurrent(async |accessor| {
            init.call_concurrent(accessor, &[], &mut [])
                .await
                .expect("init-plugin");

            let mut metadata = [Val::Bool(false)];
            get_metadata
                .call_concurrent(accessor, &[], &mut metadata)
                .await
                .expect("get-metadata");
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
                    .expect("fast call");
                assert_eq!(ipc_reply(&result[0]).as_deref(), Some(&b"fast!"[..]));
                release.send(()).expect("slow import is still waiting");
            };
            let (slow, ()) = tokio::join!(slow, fast);
            slow.expect("slow call");
            assert_eq!(ipc_reply(&slow_result[0]).as_deref(), Some(&b"slow!"[..]));
        })
        .await
        .expect("run_concurrent");

    // The synchronous import ran once per handled message
    assert_eq!(state.cancelled_tasks.load(Ordering::SeqCst), 2);
}
