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

#[derive(Default)]
struct State {
    ids: std::sync::Mutex<std::collections::BTreeMap<&'static str, u32>>,
    calls: std::sync::Mutex<Vec<String>>,
    custom_data: std::sync::Mutex<Option<Val>>,
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

    // Recipe builders use HashMap; a deterministic WASI hash seed keeps the fixture reproducible.
    linker
        .instance("wasi:random/insecure-seed@0.2.12")
        .expect("WASI seed")
        .func_wrap("insecure-seed", |_, (): ()| Ok(((1u64, 2u64),)))
        .expect("hash seed");
    install_sdk_host(&mut linker);

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
            ..State::default()
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

struct ContextResource;
struct ServerResource;
struct CommandResource;
struct NodeResource;
struct SenderResource;
struct ArgsResource;
struct EntityResource;
struct ChunkResource;
struct ItemResource;
struct TextResource;
struct RecipeResource;
struct EnchantmentResource;

#[expect(
    clippy::too_many_lines,
    reason = "fixture host definitions mirror the interfaces exercised by one component"
)]
fn install_sdk_host(linker: &mut Linker<Arc<State>>) {
    macro_rules! interface {
        ($name:literal, $($resource:literal => $ty:ty),* $(,)?) => {{
            let mut interface = linker.instance(concat!("pumpkin:plugin/", $name, "@0.2.0")).expect("interface");
            $(interface.resource($resource, wasmtime::component::ResourceType::host::<$ty>(), |_, _| Ok(())).expect("resource");)*
            interface
        }};
    }
    use wasmtime::component::Resource;
    let mut context = interface!("context", "context" => ContextResource);
    for name in [
        "[method]context.register-event",
        "[method]context.register-command",
    ] {
        context
            .func_new(name, |mut store, _, params, _| {
                release_values(&mut store, params)
            })
            .expect("context registration");
    }
    context
        .func_wrap(
            "[method]context.get-server",
            |_, (_context,): (Resource<ContextResource>,)| {
                Ok((Resource::<ServerResource>::new_own(0),))
            },
        )
        .expect("context server");
    let mut command = interface!("command", "command" => CommandResource, "command-node" => NodeResource,
        "command-sender" => SenderResource, "consumed-args" => ArgsResource);
    command
        .func_wrap(
            "[constructor]command",
            |_, (_names, _description): (Vec<String>, String)| {
                Ok((Resource::<CommandResource>::new_own(0),))
            },
        )
        .expect("command constructor");
    command
        .func_wrap("[static]command-node.literal", |_, (_name,): (String,)| {
            Ok((Resource::<NodeResource>::new_own(0),))
        })
        .expect("literal node");
    for (name, key) in [
        ("[method]command.execute-with-handler-id", "command"),
        ("[method]command-node.suggest-with-handler-id", "suggestion"),
    ] {
        command
            .func_new(name, move |mut store, _, params, _| {
                let Val::U32(id) = params[1] else {
                    return Err(wasmtime::Error::msg("expected a handler id"));
                };
                store.data().ids.lock().expect("ids").insert(key, id);
                release_values(&mut store, params)
            })
            .expect("handler registration");
    }
    command
        .func_new("[method]command.then", |mut store, _, params, _| {
            release_values(&mut store, params)
        })
        .expect("command child");
    command
        .func_wrap_concurrent(
            "[method]command-sender.has-permission",
            |_, (_sender, permission): (Resource<SenderResource>, String)| {
                Box::pin(async move {
                    assert_eq!(permission, "fixture.ping");
                    Ok((true,))
                })
            },
        )
        .expect("command permission");
    let mut scheduler = linker.instance(SCHEDULER_INSTANCE).expect("scheduler");
    scheduler
        .func_wrap("schedule-delayed-task", |store, (id, delay): (u32, u64)| {
            assert_eq!(delay, 20);
            store.data().ids.lock().expect("ids").insert("task", id);
            Ok((400u32,))
        })
        .expect("schedule task");
    let mut server = interface!("server", "server" => ServerResource);
    server
        .func_wrap_concurrent(
            "[method]server.broadcast",
            |accessor: &Accessor<Arc<State>>,
             (_server, text): (Resource<ServerResource>, String)| {
                Box::pin(async move {
                    accessor
                        .with(|mut store| store.data_mut().calls.lock().expect("calls").push(text));
                    Ok(())
                })
            },
        )
        .expect("broadcast");
    server
        .func_wrap(
            "[method]server.get-recipe-manager",
            |_, (_server,): (Resource<ServerResource>,)| {
                Ok((Resource::<RecipeResource>::new_own(0),))
            },
        )
        .expect("recipe manager");
    server
        .func_wrap(
            "[method]server.get-enchantment-manager",
            |_, (_server,): (Resource<ServerResource>,)| {
                Ok((Resource::<EnchantmentResource>::new_own(0),))
            },
        )
        .expect("enchantment manager");
    let mut recipes = interface!("recipe", "recipe-manager" => RecipeResource);
    recipes
        .func_new_concurrent(
            "[method]recipe-manager.register-shaped",
            |accessor, _, params, _| {
                Box::pin(async move {
                    assert_eq!(params[1], Val::String("fixture:diamond".into()));
                    accessor.with(|mut store| {
                        store
                            .data_mut()
                            .calls
                            .lock()
                            .expect("calls")
                            .push("recipe".into());
                        release_values(&mut store, params)
                    })
                })
            },
        )
        .expect("register recipe");
    let mut enchantments = interface!("enchantments", "enchantment-manager" => EnchantmentResource);
    enchantments
        .func_new_concurrent(
            "[method]enchantment-manager.register-enchantment",
            |accessor, _, params, results| {
                Box::pin(async move {
                    accessor.with(|mut store| {
                        store
                            .data_mut()
                            .calls
                            .lock()
                            .expect("calls")
                            .push("enchantment".into());
                        release_values(&mut store, params)
                    })?;
                    results[0] = Val::Result(Ok(None));
                    Ok(())
                })
            },
        )
        .expect("register enchantment");
    let mut world =
        interface!("world", "entity" => EntityResource, "chunk-buffer" => ChunkResource);
    world
        .func_wrap(
            "[method]chunk-buffer.get-min-y",
            |_, (_chunk,): (Resource<ChunkResource>,)| Ok((-64i32,)),
        )
        .expect("min y");
    world
        .func_wrap(
            "[method]chunk-buffer.fill-layer",
            |store, (_chunk, y, block): (Resource<ChunkResource>, i32, u16)| {
                assert_eq!((y, block), (-64, 1));
                store
                    .data()
                    .calls
                    .lock()
                    .expect("calls")
                    .push("surface".into());
                Ok(())
            },
        )
        .expect("fill layer");
    let mut display = linker
        .instance("pumpkin:plugin/display@0.2.0")
        .expect("display");
    for name in [
        "[static]display-entity.from-entity",
        "[static]interaction-entity.from-entity",
    ] {
        display
            .func_new(name, |mut store, _, params, results| {
                results[0] = Val::Option(None);
                release_values(&mut store, params)
            })
            .expect("display cast");
    }
    let mut text = interface!("text", "text-component" => TextResource);
    text.func_wrap("[static]text-component.text", |_, (_text,): (String,)| {
        Ok((Resource::<TextResource>::new_own(0),))
    })
    .expect("text constructor");
    let mut items = interface!("item-stack", "item-stack" => ItemResource);
    items
        .func_wrap(
            "[constructor]item-stack",
            |_, (key, count): (String, u8)| {
                assert_eq!((key.as_str(), count), ("minecraft:diamond", 2));
                Ok((Resource::<ItemResource>::new_own(0),))
            },
        )
        .expect("item constructor");
    items
        .func_wrap_concurrent(
            "[method]item-stack.get-registry-key",
            |_, (_item,): (Resource<ItemResource>,)| {
                Box::pin(async { Ok(("minecraft:diamond".to_string(),)) })
            },
        )
        .expect("item key");
    items
        .func_new_concurrent(
            "[method]item-stack.set-custom-data",
            |accessor, _, params, _| {
                Box::pin(async move {
                    accessor.with(|mut store| {
                        *store.data_mut().custom_data.lock().expect("custom data") =
                            Some(params[3].clone());
                        release_values(&mut store, params)
                    })
                })
            },
        )
        .expect("set data");
    items
        .func_new_concurrent(
            "[method]item-stack.get-custom-data",
            |accessor, _, params, results| {
                Box::pin(async move {
                    let data = accessor.with(|mut store| {
                        store
                            .data_mut()
                            .custom_data
                            .lock()
                            .expect("custom data")
                            .clone()
                    });
                    results[0] = Val::Option(data.map(Box::new));
                    accessor.with(|mut store| release_values(&mut store, params))
                })
            },
        )
        .expect("get data");
    let mut logging = linker
        .instance("pumpkin:plugin/logging@0.2.0")
        .expect("logging");
    logging
        .func_wrap("log-tracing", |store, (bytes,): (Vec<u8>,)| {
            assert!(!bytes.is_empty());
            store
                .data()
                .calls
                .lock()
                .expect("calls")
                .push("tracing".into());
            Ok(())
        })
        .expect("tracing");
}

fn owned<T: 'static>(accessor: &Accessor<Arc<State>>) -> Val {
    accessor.with(|mut store| {
        Val::Resource(
            wasmtime::component::ResourceAny::try_from_resource(
                wasmtime::component::Resource::<T>::new_own(0),
                &mut store,
            )
            .expect("host resource"),
        )
    })
}

#[tokio::test]
async fn sdk_helpers_and_registered_callbacks_run_in_a_component() {
    let engine = engine();
    let component = load(&engine);
    let (mut store, instance) = instantiate(&engine, &component, State::default()).await;
    let state = Arc::clone(store.data());
    let init = export_func(&mut store, &instance, "init-plugin");
    let on_load = export_func(&mut store, &instance, "on-load");
    let on_unload = export_func(&mut store, &instance, "on-unload");
    let ipc = export_func(&mut store, &instance, "handle-ipc-message");
    let command = export_func(&mut store, &instance, "handle-command");
    let suggest = export_func(&mut store, &instance, "handle-command-suggestion");
    let task = export_func(&mut store, &instance, "handle-task");
    let generate = export_func(&mut store, &instance, "handle-generate-phase");
    let mut ai = Vec::new();
    for hook in ["can-start", "should-continue", "start", "tick", "stop"] {
        ai.push(export_func(
            &mut store,
            &instance,
            &format!("handle-ai-goal-{hook}"),
        ));
    }
    store.run_concurrent(async |accessor| {
        init.call_concurrent(accessor, &[], &mut []).await.expect("init");
        let mut result = [Val::Bool(false)];
        ipc.call_concurrent(accessor, &ipc_params("helpers", b""), &mut result).await.expect("helpers");
        assert_eq!(ipc_reply(&result[0]).as_deref(), Some(&b"helpers passed"[..]));
        on_load.call_concurrent(accessor, &[owned::<ContextResource>(accessor)], &mut result).await.expect("load");
        assert!(matches!(&result[0], Val::Result(Ok(None))));
        let id = |key| *state.ids.lock().expect("ids").get(key).expect("registered id");
        command.call_concurrent(accessor, &[Val::U32(id("command")), owned::<SenderResource>(accessor), owned::<ServerResource>(accessor), owned::<ArgsResource>(accessor)], &mut result).await.expect("command");
        assert_eq!(result[0], Val::Result(Ok(Some(Box::new(Val::S32(1))))));
        let request = Val::Record(vec![("input".into(), Val::String("fixture f".into())), ("cursor".into(), Val::U32(9)), ("start".into(), Val::U32(8)), ("remaining".into(), Val::String("f".into()))]);
        suggest.call_concurrent(accessor, &[Val::U32(id("suggestion")), owned::<SenderResource>(accessor), owned::<ServerResource>(accessor), request], &mut result).await.expect("suggestions");
        assert!(matches!(&result[0], Val::Record(fields) if fields[0].1 == Val::U32(8) && fields[1].1 == Val::U32(1)));
        task.call_concurrent(accessor, &[Val::U32(id("task")), owned::<ServerResource>(accessor)], &mut []).await.expect("task");
        for (index, hook) in ai.iter().enumerate() {
            let params = [Val::U32(0), owned::<ServerResource>(accessor), owned::<EntityResource>(accessor)];
            if index < 2 {
                hook.call_concurrent(accessor, &params, &mut result).await.expect("AI predicate");
                assert_eq!(result[0], Val::Bool(index == 0));
            } else {
                hook.call_concurrent(accessor, &params, &mut []).await.expect("AI action");
            }
        }
        for phase in ["biomes", "noise", "surface", "features"] {
            generate.call_concurrent(accessor, &[Val::U32(0), Val::Enum(phase.into()), owned::<ChunkResource>(accessor)], &mut []).await.expect("generation");
        }
        on_unload.call_concurrent(accessor, &[owned::<ContextResource>(accessor)], &mut result).await.expect("unload");
        assert!(matches!(&result[0], Val::Result(Ok(None))));
    }).await.expect("SDK exercise");
    assert_eq!(state.cancelled_tasks.load(Ordering::SeqCst), 4);
    assert_eq!(
        *state.calls.lock().expect("calls"),
        [
            "recipe",
            "enchantment",
            "tracing",
            "fixture task ran",
            "surface"
        ]
    );
}

// Converting the fixture's stateless host handles releases dynamic borrow-table entries.
fn release_values(
    store: &mut impl wasmtime::AsContextMut<Data = Arc<State>>,
    values: &[Val],
) -> wasmtime::Result<()> {
    for value in values {
        release_value(store, value)?;
    }
    Ok(())
}

fn release_value(
    store: &mut impl wasmtime::AsContextMut<Data = Arc<State>>,
    value: &Val,
) -> wasmtime::Result<()> {
    match value {
        Val::Resource(resource) => {
            macro_rules! release {
                ($($ty:ty),*) => { $(
                    if resource.ty() == wasmtime::component::ResourceType::host::<$ty>() {
                        let _resource = resource.try_into_resource::<$ty>(store)?;
                        return Ok(());
                    }
                )* };
            }
            release!(
                ContextResource,
                ServerResource,
                CommandResource,
                NodeResource,
                SenderResource,
                ArgsResource,
                EntityResource,
                ChunkResource,
                ItemResource,
                TextResource,
                RecipeResource,
                EnchantmentResource
            );
            Err(wasmtime::Error::msg("unrecognized fixture resource"))
        }
        Val::List(values) | Val::Tuple(values) => release_values(store, values),
        Val::Record(fields) => {
            for (_, value) in fields {
                release_value(store, value)?;
            }
            Ok(())
        }
        Val::Option(Some(value))
        | Val::Variant(_, Some(value))
        | Val::Result(Ok(Some(value)) | Err(Some(value))) => release_value(store, value),
        _ => Ok(()),
    }
}
