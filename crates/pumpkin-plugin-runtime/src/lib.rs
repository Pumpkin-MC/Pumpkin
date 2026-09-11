mod chain;
mod executor;
mod lifecycle;
mod policy;
mod spawn;

pub use executor::{
    LegacyGuestScope, LegacyStore, StoreDataMut, StoreExecutor, StoreFuture, StoreHandle,
};
pub use lifecycle::{DriverError, DriverJoin, DriverState};
pub use policy::{LegacySyncReentry, StorePolicy};
pub use spawn::{RuntimeSpawner, SpawnError, SpawnFuture};

#[cfg(test)]
pub(crate) use chain::MAX_SYNC_REENTRY_DEPTH;

#[cfg(test)]
#[allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]
mod tests {
    use std::{
        ops::Deref,
        sync::{
            Arc, Mutex, OnceLock,
            atomic::{AtomicBool, AtomicUsize, Ordering},
        },
    };

    use tokio::{
        sync::Notify,
        time::{Duration, timeout},
    };
    use wasm_encoder::{
        CodeSection, ComponentBuilder, ComponentExportKind, ComponentTypeRef, ComponentValType,
        ConstExpr, EntityType, ExportKind, ExportSection, Function, FunctionSection, GlobalSection,
        GlobalType, ImportSection, Instruction, Module, ModuleArg, PrimitiveValType, TypeBounds,
        TypeSection, ValType,
    };
    use wasmtime::{
        Config, Engine, Store,
        component::{Component, Linker, Resource, ResourceType, TypedFunc},
    };

    use super::{
        DriverError, DriverState, LegacyStore, LegacySyncReentry, MAX_SYNC_REENTRY_DEPTH,
        RuntimeSpawner, SpawnError, SpawnFuture,
    };

    struct TestHostState;

    struct BoundaryResource;

    struct BoundaryState {
        resource_drops: Arc<AtomicUsize>,
        store_dropped: Arc<AtomicBool>,
    }

    impl Drop for BoundaryState {
        fn drop(&mut self) {
            self.store_dropped.store(true, Ordering::Release);
        }
    }

    struct TestSpawner {
        runtime: tokio::runtime::Handle,
    }

    impl RuntimeSpawner for TestSpawner {
        fn spawn(&self, task: SpawnFuture) -> Result<(), SpawnError> {
            drop(self.runtime.spawn(task));
            Ok(())
        }

        fn spawn_blocking(
            &self,
            task: Box<dyn FnOnce() + Send + 'static>,
        ) -> Result<(), SpawnError> {
            drop(self.runtime.spawn_blocking(task));
            Ok(())
        }
    }

    struct DroppingSpawner;

    impl RuntimeSpawner for DroppingSpawner {
        fn spawn(&self, task: SpawnFuture) -> Result<(), SpawnError> {
            drop(task);
            Ok(())
        }

        fn spawn_blocking(
            &self,
            task: Box<dyn FnOnce() + Send + 'static>,
        ) -> Result<(), SpawnError> {
            drop(task);
            Ok(())
        }
    }

    struct AbortRecordingSpawner {
        runtime: tokio::runtime::Handle,
        driver_abort: Arc<Mutex<Option<tokio::task::AbortHandle>>>,
    }

    impl RuntimeSpawner for AbortRecordingSpawner {
        fn spawn(&self, task: SpawnFuture) -> Result<(), SpawnError> {
            let task = self.runtime.spawn(task);
            let abort = task.abort_handle();
            drop(task);
            self.driver_abort
                .lock()
                .expect("driver abort lock")
                .replace(abort);
            Ok(())
        }

        fn spawn_blocking(
            &self,
            task: Box<dyn FnOnce() + Send + 'static>,
        ) -> Result<(), SpawnError> {
            drop(self.runtime.spawn_blocking(task));
            Ok(())
        }
    }

    #[derive(Clone)]
    struct ConcurrentStore(Arc<LegacyStore<TestHostState>>);

    impl ConcurrentStore {
        async fn new(store: Store<TestHostState>) -> Self {
            Self::with_policy(store, LegacySyncReentry::new()).await
        }

        async fn with_policy(store: Store<TestHostState>, policy: LegacySyncReentry) -> Self {
            let spawner = Arc::new(TestSpawner {
                runtime: tokio::runtime::Handle::current(),
            });
            let executor = LegacyStore::start(store, policy, spawner)
                .await
                .expect("start test Store driver");
            Self(Arc::new(executor))
        }
    }

    impl Deref for ConcurrentStore {
        type Target = LegacyStore<TestHostState>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    struct BackgroundTask {
        started: Arc<Notify>,
        release: Arc<Notify>,
    }

    impl wasmtime::component::AccessorTask<TestHostState> for BackgroundTask {
        async fn run(
            self,
            _accessor: &wasmtime::component::Accessor<TestHostState>,
        ) -> wasmtime::Result<()> {
            self.started.notify_one();
            self.release.notified().await;
            Ok(())
        }
    }

    fn test_store() -> Store<TestHostState> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let engine = Engine::new(&config).expect("test engine");
        Store::new(&engine, TestHostState)
    }

    async fn legacy_store(
        store: Store<TestHostState>,
        policy: &LegacySyncReentry,
    ) -> ConcurrentStore {
        ConcurrentStore::with_policy(store, policy.clone()).await
    }

    fn assert_synchronous_reentry_depth_is_bounded_and_recovers() {
        let root = LegacySyncReentry::new().root_context();
        let mut context = root;
        for _ in 0..MAX_SYNC_REENTRY_DEPTH {
            context = context.child().expect("create child reentry context");
        }
        assert_eq!(context.depth, MAX_SYNC_REENTRY_DEPTH);

        let error = context
            .child()
            .expect_err("one scope beyond the limit should be rejected");
        assert!(error.to_string().contains("maximum depth"));
        assert!(root.child().is_ok());
    }

    #[allow(clippy::too_many_lines)]
    async fn root_admission_lifecycle_scenario() {
        let policy = LegacySyncReentry::new();
        let holding_store = legacy_store(test_store(), &policy).await;
        let first_store = legacy_store(test_store(), &policy).await;
        let second_store = legacy_store(test_store(), &policy).await;
        let third_store = legacy_store(test_store(), &policy).await;
        let holding_started = Arc::new(Notify::new());
        let release_holding = Arc::new(Notify::new());
        let holding_completed = Arc::new(Notify::new());
        let third_attempted = Arc::new(Notify::new());
        let third_started = Arc::new(Notify::new());
        let order = Arc::new(Mutex::new(Vec::new()));

        let holding_driver = holding_store.clone();
        let started = Arc::clone(&holding_started);
        let release = Arc::clone(&release_holding);
        let completed = Arc::clone(&holding_completed);
        let holding = tokio::spawn(async move {
            holding_driver
                .call_guest(move |_| {
                    Box::pin(async move {
                        started.notify_one();
                        release.notified().await;
                        completed.notify_one();
                        Ok(())
                    })
                })
                .await
        });
        holding_started.notified().await;
        holding.abort();
        assert!(
            holding
                .await
                .expect_err("holding root waiter should be cancelled")
                .is_cancelled()
        );

        let first_order = Arc::clone(&order);
        let first = first_store.call_guest(move |_| {
            Box::pin(async move {
                first_order.lock().expect("order lock").push(1);
                Ok(1)
            })
        });
        tokio::pin!(first);
        assert!(
            timeout(Duration::from_millis(10), &mut first)
                .await
                .is_err()
        );

        let second_order = Arc::clone(&order);
        let second = second_store.call_guest(move |_| {
            Box::pin(async move {
                second_order.lock().expect("order lock").push(2);
                Ok(2)
            })
        });
        tokio::pin!(second);
        assert!(
            timeout(Duration::from_millis(10), &mut second)
                .await
                .is_err()
        );

        let runtime = tokio::runtime::Handle::current();
        let third_driver = third_store.clone();
        let attempted = Arc::clone(&third_attempted);
        let entered = Arc::clone(&third_started);
        let third_order = Arc::clone(&order);
        let third = tokio::task::spawn_blocking(move || {
            attempted.notify_one();
            runtime.block_on(third_driver.call_guest(move |_| {
                Box::pin(async move {
                    entered.notify_one();
                    third_order.lock().expect("order lock").push(3);
                    Ok(3)
                })
            }))
        });
        third_attempted.notified().await;
        assert!(
            timeout(Duration::from_millis(100), third_started.notified())
                .await
                .is_err(),
            "a root from another execution domain entered before the accepted job finished"
        );

        release_holding.notify_one();
        holding_completed.notified().await;
        assert_eq!(first.await.expect("first root"), 1);
        assert_eq!(second.await.expect("second root"), 2);
        assert_eq!(
            third
                .await
                .expect("third root task")
                .expect("third root result"),
            3
        );
        assert_eq!(*order.lock().expect("order lock"), [1, 2, 3]);

        holding_store
            .shutdown(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("shut down holding store");
        first_store
            .shutdown(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("shut down first store");
        second_store
            .shutdown(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("shut down second store");
        third_store
            .shutdown(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("shut down third store");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn legacy_root_admission_is_fifo_and_cancellation_safe_across_domains() {
        timeout(Duration::from_secs(10), root_admission_lifecycle_scenario())
            .await
            .expect("root admission lifecycle scenario timed out");
    }

    fn sync_reentry_component() -> Vec<u8> {
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.ty().function([], []);
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
        body.instruction(&Instruction::Call(0));
        body.instruction(&Instruction::End);
        let mut code = CodeSection::new();
        code.function(&body);
        module.section(&code);

        let mut component = ComponentBuilder::default();
        let (function_type, mut function) = component.type_function(Some("run-type"));
        function
            .params([] as [(&str, PrimitiveValType); 0])
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

    #[allow(clippy::too_many_lines)]
    fn owned_resource_component() -> Vec<u8> {
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.ty().function([ValType::I32], []);
        types.ty().function([ValType::I32], [ValType::I32]);
        types.ty().function([], []);
        module.section(&types);

        let mut imports = ImportSection::new();
        imports.import("", "resource-drop", EntityType::Function(0));
        module.section(&imports);

        let mut functions = FunctionSection::new();
        functions.function(0);
        functions.function(1);
        functions.function(0);
        functions.function(0);
        functions.function(2);
        module.section(&functions);

        let mut globals = GlobalSection::new();
        globals.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i32_const(0),
        );
        module.section(&globals);

        let mut exports = ExportSection::new();
        exports.export("drop", ExportKind::Func, 1);
        exports.export("return", ExportKind::Func, 2);
        exports.export("trap", ExportKind::Func, 3);
        exports.export("retain", ExportKind::Func, 4);
        exports.export("drop-retained", ExportKind::Func, 5);
        module.section(&exports);

        let mut drop_body = Function::new([]);
        drop_body.instruction(&Instruction::LocalGet(0));
        drop_body.instruction(&Instruction::Call(0));
        drop_body.instruction(&Instruction::End);
        let mut return_body = Function::new([]);
        return_body.instruction(&Instruction::LocalGet(0));
        return_body.instruction(&Instruction::End);
        let mut trap_body = Function::new([]);
        trap_body.instruction(&Instruction::Unreachable);
        trap_body.instruction(&Instruction::End);
        let mut retain_body = Function::new([]);
        retain_body.instruction(&Instruction::LocalGet(0));
        retain_body.instruction(&Instruction::GlobalSet(0));
        retain_body.instruction(&Instruction::End);
        let mut drop_retained_body = Function::new([]);
        drop_retained_body.instruction(&Instruction::GlobalGet(0));
        drop_retained_body.instruction(&Instruction::Call(0));
        drop_retained_body.instruction(&Instruction::End);
        let mut code = CodeSection::new();
        code.function(&drop_body);
        code.function(&return_body);
        code.function(&trap_body);
        code.function(&retain_body);
        code.function(&drop_retained_body);
        module.section(&code);

        let mut component = ComponentBuilder::default();
        let resource =
            component.import("resource", ComponentTypeRef::Type(TypeBounds::SubResource));
        let resource_drop = component.resource_drop(resource);
        let (owned_resource, owned) = component.type_defined(Some("owned-resource"));
        owned.own(resource);

        let (drop_type, mut drop_signature) = component.type_function(Some("drop-type"));
        drop_signature
            .params([("resource", ComponentValType::Type(owned_resource))])
            .result(None);
        let (return_type, mut return_signature) = component.type_function(Some("return-type"));
        return_signature
            .params([("resource", ComponentValType::Type(owned_resource))])
            .result(Some(ComponentValType::Type(owned_resource)));
        let (drop_retained_type, mut drop_retained_signature) =
            component.type_function(Some("drop-retained-type"));
        drop_retained_signature
            .params([] as [(&str, PrimitiveValType); 0])
            .result(None);

        let module = component.core_module(Some("guest"), &module);
        let intrinsics = component.core_instantiate_exports(
            Some("intrinsics"),
            [("resource-drop", ExportKind::Func, resource_drop)],
        );
        let instance = component.core_instantiate(
            Some("guest-instance"),
            module,
            [("", ModuleArg::Instance(intrinsics))],
        );
        let drop_core =
            component.core_alias_export(Some("drop-core"), instance, "drop", ExportKind::Func);
        let return_core =
            component.core_alias_export(Some("return-core"), instance, "return", ExportKind::Func);
        let trap_core =
            component.core_alias_export(Some("trap-core"), instance, "trap", ExportKind::Func);
        let retain_core =
            component.core_alias_export(Some("retain-core"), instance, "retain", ExportKind::Func);
        let drop_retained_core = component.core_alias_export(
            Some("drop-retained-core"),
            instance,
            "drop-retained",
            ExportKind::Func,
        );
        let drop_resource = component.lift_func(Some("drop"), drop_core, drop_type, []);
        let return_resource = component.lift_func(Some("return"), return_core, return_type, []);
        let trap = component.lift_func(Some("trap"), trap_core, drop_type, []);
        let retain = component.lift_func(Some("retain"), retain_core, drop_type, []);
        let drop_retained = component.lift_func(
            Some("drop-retained"),
            drop_retained_core,
            drop_retained_type,
            [],
        );
        component.export("drop", ComponentExportKind::Func, drop_resource, None);
        component.export("return", ComponentExportKind::Func, return_resource, None);
        component.export("trap", ComponentExportKind::Func, trap, None);
        component.export("retain", ComponentExportKind::Func, retain, None);
        component.export(
            "drop-retained",
            ComponentExportKind::Func,
            drop_retained,
            None,
        );
        component.finish()
    }

    async fn shutdown_drains_accepted_calls_and_rejects_new_work() {
        let store = ConcurrentStore::new(test_store()).await;
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());

        let call_store = store.clone();
        let call_started = Arc::clone(&started);
        let call_release = Arc::clone(&release);
        let active_call = tokio::spawn(async move {
            call_store
                .call(move |_| {
                    Box::pin(async move {
                        call_started.notify_one();
                        call_release.notified().await;
                        Ok(7u8)
                    })
                })
                .await
        });
        started.notified().await;

        let shutdown_store = store.clone();
        let shutdown = tokio::spawn(async move {
            shutdown_store
                .shutdown(|_| Box::pin(async move { Ok("unloaded") }))
                .await
        });
        while matches!(store.state(), DriverState::Accepting) {
            tokio::task::yield_now().await;
        }

        let rejected = store
            .call(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect_err("new work should be rejected during shutdown");
        assert!(rejected.to_string().contains("shutting down"));
        assert!(!shutdown.is_finished());

        release.notify_one();
        assert_eq!(active_call.await.expect("active call task").unwrap(), 7);
        assert_eq!(shutdown.await.expect("shutdown task").unwrap(), "unloaded");
        assert_eq!(store.state(), DriverState::Stopped);
    }

    async fn shutdown_waits_for_joined_store_background_tasks() {
        let store = ConcurrentStore::new(test_store()).await;
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let task_started = Arc::clone(&started);
        let task_release = Arc::clone(&release);

        let call_store = store.clone();
        let active_call = tokio::spawn(async move {
            call_store
                .call(move |accessor| {
                    let spawned = accessor.spawn(BackgroundTask {
                        started: task_started,
                        release: task_release,
                    });
                    Box::pin(async move {
                        spawned?.await;
                        Ok(())
                    })
                })
                .await
        });
        started.notified().await;

        let shutdown_store = store.clone();
        let shutdown = tokio::spawn(async move {
            shutdown_store
                .shutdown(|_| Box::pin(async move { Ok(()) }))
                .await
        });
        while matches!(store.state(), DriverState::Accepting) {
            tokio::task::yield_now().await;
        }
        assert!(!shutdown.is_finished());

        release.notify_one();
        active_call
            .await
            .expect("active call task")
            .expect("active call result");
        shutdown
            .await
            .expect("shutdown task")
            .expect("shutdown result");
    }

    async fn owner_drop_drains_accepted_work() {
        let spawner = Arc::new(TestSpawner {
            runtime: tokio::runtime::Handle::current(),
        });
        let executor = LegacyStore::start(test_store(), LegacySyncReentry::new(), spawner)
            .await
            .expect("start test Store driver");
        let handle = executor.handle();
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());

        let call_handle = handle.clone();
        let call_started = Arc::clone(&started);
        let call_release = Arc::clone(&release);
        let accepted = tokio::spawn(async move {
            call_handle
                .call(move |_| {
                    Box::pin(async move {
                        call_started.notify_one();
                        call_release.notified().await;
                        Ok(11u8)
                    })
                })
                .await
        });
        started.notified().await;

        drop(executor);
        let rejected = handle
            .call(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect_err("owner drop should close admission");
        assert!(rejected.to_string().contains("shutting down"));

        release.notify_one();
        assert_eq!(accepted.await.expect("accepted call task").unwrap(), 11);

        let mut join = handle.driver_join();
        let error = join
            .wait()
            .await
            .expect_err("owner drop should fail the driver explicitly");
        assert!(
            error.to_string().contains("control was dropped"),
            "unexpected driver error: {error}"
        );

        let executor = LegacyStore::start(
            test_store(),
            LegacySyncReentry::new(),
            Arc::new(TestSpawner {
                runtime: tokio::runtime::Handle::current(),
            }),
        )
        .await
        .expect("start test Store driver");
        let mut join = executor.driver_join();
        drop(executor);
        let error = join
            .wait()
            .await
            .expect_err("dropping the sole owner should fail the driver explicitly");
        assert!(
            error.to_string().contains("control was dropped"),
            "unexpected driver error: {error}"
        );
    }

    /// Acceptance test for plain synchronous WIT recursion through the
    /// call-scoped reentry pump.
    async fn sync_lifted_same_instance_reentry_completes() {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let engine = Engine::new(&config).expect("test engine");
        let component = Component::new(&engine, sync_reentry_component()).expect("test component");

        let run_slot = Arc::new(OnceLock::<TypedFunc<(), ()>>::new());
        let driver_slot = Arc::new(OnceLock::<ConcurrentStore>::new());
        let host_calls = Arc::new(AtomicUsize::new(0));

        let mut linker = Linker::<TestHostState>::new(&engine);
        let run_for_host = Arc::clone(&run_slot);
        let driver_for_host = Arc::clone(&driver_slot);
        let calls_for_host = Arc::clone(&host_calls);
        linker
            .root()
            .func_wrap_async("host", move |mut store, ()| {
                let run = *run_for_host.get().expect("run initialized");
                let driver = driver_for_host
                    .get()
                    .expect("store driver initialized")
                    .clone();
                let host_calls = Arc::clone(&calls_for_host);
                Box::new(async move {
                    if host_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                        let nested_driver = driver.clone();
                        driver
                            .pump_reentry(
                                &mut store,
                                nested_driver.call_guest(move |mut context| {
                                    assert!(
                                        context.is_reentrant(),
                                        "same-instance callback must use the active store frame"
                                    );
                                    Box::pin(async move { context.call(run, ()).await })
                                }),
                            )
                            .await??;
                    }
                    Ok(())
                })
            })
            .expect("link host function");

        let mut store = Store::new(&engine, TestHostState);
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let run = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .expect("run export");
        assert!(run_slot.set(run).is_ok());

        let driver = ConcurrentStore::new(store).await;
        assert!(driver_slot.set(driver.clone()).is_ok());

        timeout(
            Duration::from_secs(2),
            driver.call_guest(move |mut context| {
                assert!(
                    !context.is_reentrant(),
                    "top-level call must use the concurrent store path"
                );
                Box::pin(async move { context.call(run, ()).await })
            }),
        )
        .await
        .expect("sync-lifted guest reentry timed out")
        .expect("sync-lifted guest reentry failed");

        assert_eq!(host_calls.load(Ordering::SeqCst), 2);
        driver
            .shutdown(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("shut down test store");
    }

    /// Mirrors synchronous game methods that call `fire_blocking` from a
    /// blocking worker while the host import keeps the active store pumpable.
    async fn blocking_host_operation_propagates_the_reentry_chain() {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let engine = Engine::new(&config).expect("test engine");
        let component = Component::new(&engine, sync_reentry_component()).expect("test component");

        let run_slot = Arc::new(OnceLock::<TypedFunc<(), ()>>::new());
        let driver_slot = Arc::new(OnceLock::<ConcurrentStore>::new());
        let host_calls = Arc::new(AtomicUsize::new(0));

        let mut linker = Linker::<TestHostState>::new(&engine);
        let run_for_host = Arc::clone(&run_slot);
        let driver_for_host = Arc::clone(&driver_slot);
        let calls_for_host = Arc::clone(&host_calls);
        linker
            .root()
            .func_wrap_async("host", move |mut store, ()| {
                let run = *run_for_host.get().expect("run initialized");
                let driver = driver_for_host
                    .get()
                    .expect("store driver initialized")
                    .clone();
                let host_calls = Arc::clone(&calls_for_host);
                Box::new(async move {
                    if host_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                        let runtime = tokio::runtime::Handle::current();
                        let nested_driver = driver.clone();
                        driver
                            .pump_blocking(&mut store, move || {
                                tokio::task::block_in_place(|| {
                                    runtime.block_on(nested_driver.call_guest(
                                        move |mut context| {
                                            assert!(
                                                context.is_reentrant(),
                                                "blocking callback must use the active store frame"
                                            );
                                            Box::pin(async move { context.call(run, ()).await })
                                        },
                                    ))
                                })
                            })
                            .await??;
                    }
                    Ok(())
                })
            })
            .expect("link host function");

        let mut store = Store::new(&engine, TestHostState);
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let run = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .expect("run export");
        assert!(run_slot.set(run).is_ok());

        let driver = ConcurrentStore::new(store).await;
        assert!(driver_slot.set(driver.clone()).is_ok());

        timeout(
            Duration::from_secs(2),
            driver.call_guest(move |mut context| {
                Box::pin(async move { context.call(run, ()).await })
            }),
        )
        .await
        .expect("blocking sync-lifted guest reentry timed out")
        .expect("blocking sync-lifted guest reentry failed");

        assert_eq!(host_calls.load(Ordering::SeqCst), 2);
        driver
            .shutdown(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("shut down test store");
    }

    async fn unrelated_guest_call_waits_for_the_active_chain() {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let engine = Engine::new(&config).expect("test engine");
        let component = Component::new(&engine, sync_reentry_component()).expect("test component");

        let driver_slot = Arc::new(OnceLock::<ConcurrentStore>::new());
        let entered = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let unrelated_entered = Arc::new(Notify::new());
        let host_calls = Arc::new(AtomicUsize::new(0));

        let mut linker = Linker::<TestHostState>::new(&engine);
        let driver_for_host = Arc::clone(&driver_slot);
        let entered_for_host = Arc::clone(&entered);
        let release_for_host = Arc::clone(&release);
        let unrelated_for_host = Arc::clone(&unrelated_entered);
        let calls_for_host = Arc::clone(&host_calls);
        linker
            .root()
            .func_wrap_async("host", move |mut store, ()| {
                let driver = driver_for_host
                    .get()
                    .expect("store driver initialized")
                    .clone();
                let entered = Arc::clone(&entered_for_host);
                let release = Arc::clone(&release_for_host);
                let unrelated_entered = Arc::clone(&unrelated_for_host);
                let host_calls = Arc::clone(&calls_for_host);
                Box::new(async move {
                    if host_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                        entered.notify_one();
                        driver.pump_reentry(&mut store, release.notified()).await?;
                    } else {
                        unrelated_entered.notify_one();
                    }
                    Ok(())
                })
            })
            .expect("link host function");

        let mut store = Store::new(&engine, TestHostState);
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let run = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .expect("run export");
        let driver = ConcurrentStore::new(store).await;
        assert!(driver_slot.set(driver.clone()).is_ok());

        let outer_driver = driver.clone();
        let outer = tokio::spawn(async move {
            outer_driver
                .call_guest(move |mut context| Box::pin(async move { context.call(run, ()).await }))
                .await
        });
        entered.notified().await;

        let unrelated_driver = driver.clone();
        let unrelated = tokio::spawn(async move {
            unrelated_driver
                .call_guest(move |mut context| Box::pin(async move { context.call(run, ()).await }))
                .await
        });

        assert!(
            timeout(Duration::from_millis(100), unrelated_entered.notified())
                .await
                .is_err(),
            "an unrelated root call was routed into the active reentry scope"
        );

        release.notify_one();
        outer.await.expect("outer task").expect("outer guest call");
        unrelated
            .await
            .expect("unrelated task")
            .expect("unrelated guest call");
        assert_eq!(host_calls.load(Ordering::SeqCst), 2);

        driver
            .shutdown(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("shut down test store");
    }

    #[allow(clippy::too_many_lines)]
    async fn shutdown_reentry_scenario() {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let engine = Engine::new(&config).expect("test engine");
        let component = Component::new(&engine, sync_reentry_component()).expect("test component");

        let run_slot = Arc::new(OnceLock::<TypedFunc<(), ()>>::new());
        let driver_slot = Arc::new(OnceLock::<ConcurrentStore>::new());
        let entered = Arc::new(Notify::new());
        let start_callback = Arc::new(Notify::new());
        let callback_completed = Arc::new(Notify::new());
        let host_calls = Arc::new(AtomicUsize::new(0));
        let waiting_root_ran = Arc::new(AtomicBool::new(false));

        let mut linker = Linker::<TestHostState>::new(&engine);
        let run_for_host = Arc::clone(&run_slot);
        let driver_for_host = Arc::clone(&driver_slot);
        let entered_for_host = Arc::clone(&entered);
        let start_for_host = Arc::clone(&start_callback);
        let completed_for_host = Arc::clone(&callback_completed);
        let calls_for_host = Arc::clone(&host_calls);
        linker
            .root()
            .func_wrap_async("host", move |mut store, ()| {
                let run = *run_for_host.get().expect("run initialized");
                let driver = driver_for_host
                    .get()
                    .expect("store driver initialized")
                    .clone();
                let entered = Arc::clone(&entered_for_host);
                let start_callback = Arc::clone(&start_for_host);
                let callback_completed = Arc::clone(&completed_for_host);
                let host_calls = Arc::clone(&calls_for_host);
                Box::new(async move {
                    if host_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                        entered.notify_one();
                        let callback_driver = driver.clone();
                        let outbound = async move {
                            start_callback.notified().await;
                            callback_driver
                                .call_guest(move |mut context| {
                                    Box::pin(async move { context.call(run, ()).await })
                                })
                                .await
                        };
                        driver.pump_reentry(&mut store, outbound).await??;
                    } else {
                        callback_completed.notify_one();
                    }
                    Ok(())
                })
            })
            .expect("link host function");

        let mut store = Store::new(&engine, TestHostState);
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let run = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .expect("run export");
        assert!(run_slot.set(run).is_ok());
        let driver = ConcurrentStore::new(store).await;
        assert!(driver_slot.set(driver.clone()).is_ok());

        let outer_driver = driver.clone();
        let outer = tokio::spawn(async move {
            outer_driver
                .call_guest(move |mut context| Box::pin(async move { context.call(run, ()).await }))
                .await
        });
        entered.notified().await;

        let shutdown_driver = driver.clone();
        let shutdown = shutdown_driver.shutdown(|_| Box::pin(async move { Ok(()) }));
        tokio::pin!(shutdown);
        assert!(
            timeout(Duration::from_millis(10), &mut shutdown)
                .await
                .is_err()
        );
        assert_eq!(driver.state(), DriverState::Accepting);

        let root_ran = Arc::clone(&waiting_root_ran);
        let waiting_root = driver.call_guest(move |_| {
            Box::pin(async move {
                root_ran.store(true, Ordering::Release);
                Ok(())
            })
        });
        tokio::pin!(waiting_root);
        assert!(
            timeout(Duration::from_millis(10), &mut waiting_root)
                .await
                .is_err()
        );

        start_callback.notify_one();
        timeout(Duration::from_secs(2), callback_completed.notified())
            .await
            .expect("callback was rejected during shutdown");
        outer.await.expect("outer task").expect("outer guest call");
        shutdown.await.expect("shutdown store");
        let error = waiting_root
            .await
            .expect_err("root queued behind shutdown should be rejected");
        assert!(error.to_string().contains("shutting down"), "{error}");
        assert!(!waiting_root_ran.load(Ordering::Acquire));
        assert_eq!(host_calls.load(Ordering::SeqCst), 2);
    }

    #[allow(clippy::too_many_lines)]
    async fn opposing_plugin_roots_scenario() {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let engine = Engine::new(&config).expect("test engine");
        let component = Component::new(&engine, sync_reentry_component()).expect("test component");

        let a_run_slot = Arc::new(OnceLock::<TypedFunc<(), ()>>::new());
        let b_run_slot = Arc::new(OnceLock::<TypedFunc<(), ()>>::new());
        let a_driver_slot = Arc::new(OnceLock::<ConcurrentStore>::new());
        let b_driver_slot = Arc::new(OnceLock::<ConcurrentStore>::new());
        let a_calls = Arc::new(AtomicUsize::new(0));
        let b_calls = Arc::new(AtomicUsize::new(0));
        let trace = Arc::new(Mutex::new(Vec::new()));
        let first_root_entered = Arc::new(Notify::new());
        let release_first_root = Arc::new(Notify::new());
        let second_root_started = Arc::new(Notify::new());

        let mut a_linker = Linker::<TestHostState>::new(&engine);
        let a_driver_for_a = Arc::clone(&a_driver_slot);
        let b_driver_for_a = Arc::clone(&b_driver_slot);
        let b_run_for_a = Arc::clone(&b_run_slot);
        let a_calls_for_host = Arc::clone(&a_calls);
        let trace_for_a = Arc::clone(&trace);
        let entered_for_a = Arc::clone(&first_root_entered);
        let release_for_a = Arc::clone(&release_first_root);
        a_linker
            .root()
            .func_wrap_async("host", move |mut store, ()| {
                let a_driver = a_driver_for_a
                    .get()
                    .expect("A store driver initialized")
                    .clone();
                let b_driver = b_driver_for_a
                    .get()
                    .expect("B store driver initialized")
                    .clone();
                let b_run = *b_run_for_a.get().expect("B run initialized");
                let calls = Arc::clone(&a_calls_for_host);
                let trace = Arc::clone(&trace_for_a);
                let entered = Arc::clone(&entered_for_a);
                let release = Arc::clone(&release_for_a);
                Box::new(async move {
                    match calls.fetch_add(1, Ordering::SeqCst) {
                        0 => {
                            trace.lock().expect("trace lock").push("a:first-outer");
                            entered.notify_one();
                            a_driver
                                .pump_reentry(&mut store, release.notified())
                                .await?;
                            let outbound = b_driver.call_guest(move |mut context| {
                                assert!(
                                    !context.is_reentrant(),
                                    "first call into B must use its concurrent store path"
                                );
                                Box::pin(async move { context.call(b_run, ()).await })
                            });
                            a_driver.pump_reentry(&mut store, outbound).await??;
                        }
                        1 => trace.lock().expect("trace lock").push("a:first-inner"),
                        2 => {
                            trace.lock().expect("trace lock").push("a:second-middle");
                            let outbound = b_driver.call_guest(move |mut context| {
                                assert!(
                                    context.is_reentrant(),
                                    "A to B callback must use B's active store frame"
                                );
                                Box::pin(async move { context.call(b_run, ()).await })
                            });
                            a_driver.pump_reentry(&mut store, outbound).await??;
                        }
                        _ => return Err(wasmtime::Error::msg("unexpected extra call into A")),
                    }
                    Ok(())
                })
            })
            .expect("link A host function");

        let mut b_linker = Linker::<TestHostState>::new(&engine);
        let a_driver_for_b = Arc::clone(&a_driver_slot);
        let b_driver_for_b = Arc::clone(&b_driver_slot);
        let a_run_for_b = Arc::clone(&a_run_slot);
        let b_calls_for_host = Arc::clone(&b_calls);
        let trace_for_b = Arc::clone(&trace);
        b_linker
            .root()
            .func_wrap_async("host", move |mut store, ()| {
                let a_driver = a_driver_for_b
                    .get()
                    .expect("A store driver initialized")
                    .clone();
                let b_driver = b_driver_for_b
                    .get()
                    .expect("B store driver initialized")
                    .clone();
                let a_run = *a_run_for_b.get().expect("A run initialized");
                let calls = Arc::clone(&b_calls_for_host);
                let trace = Arc::clone(&trace_for_b);
                Box::new(async move {
                    match calls.fetch_add(1, Ordering::SeqCst) {
                        0 => {
                            trace
                                .lock()
                                .expect("trace lock")
                                .push("b:first-middle");
                            let outbound = a_driver.call_guest(move |mut context| {
                                assert!(
                                    context.is_reentrant(),
                                    "B to A callback must use A's active store frame"
                                );
                                Box::pin(async move { context.call(a_run, ()).await })
                            });
                            b_driver.pump_reentry(&mut store, outbound).await??;
                        }
                        1 => {
                            trace
                                .lock()
                                .expect("trace lock")
                                .push("b:second-outer");
                            let outbound = a_driver.call_guest(move |mut context| {
                                assert!(
                                    !context.is_reentrant(),
                                    "first call into A from the second root must use its concurrent store path"
                                );
                                Box::pin(async move { context.call(a_run, ()).await })
                            });
                            b_driver.pump_reentry(&mut store, outbound).await??;
                        }
                        2 => trace
                            .lock()
                            .expect("trace lock")
                            .push("b:second-inner"),
                        _ => return Err(wasmtime::Error::msg("unexpected extra call into B")),
                    }
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
            .get_typed_func::<(), ()>(&mut a_store, "run")
            .expect("get A run export");
        assert!(a_run_slot.set(a_run).is_ok());

        let mut b_store = Store::new(&engine, TestHostState);
        let b_instance = b_linker
            .instantiate_async(&mut b_store, &component)
            .await
            .expect("instantiate B");
        let b_run = b_instance
            .get_typed_func::<(), ()>(&mut b_store, "run")
            .expect("get B run export");
        assert!(b_run_slot.set(b_run).is_ok());

        let policy = LegacySyncReentry::new();
        let a_driver = legacy_store(a_store, &policy).await;
        let b_driver = legacy_store(b_store, &policy).await;
        assert!(a_driver_slot.set(a_driver.clone()).is_ok());
        assert!(b_driver_slot.set(b_driver.clone()).is_ok());

        let first_driver = a_driver.clone();
        let first_root = tokio::spawn(async move {
            first_driver
                .call_guest(move |mut context| {
                    assert!(
                        !context.is_reentrant(),
                        "top-level call into A must use its concurrent store path"
                    );
                    Box::pin(async move { context.call(a_run, ()).await })
                })
                .await
        });
        first_root_entered.notified().await;

        let started = Arc::clone(&second_root_started);
        let second_root = b_driver.call_guest(move |mut context| {
            Box::pin(async move {
                started.notify_one();
                assert!(
                    !context.is_reentrant(),
                    "top-level call into B must use its concurrent store path"
                );
                context.call(b_run, ()).await
            })
        });
        tokio::pin!(second_root);
        assert!(
            timeout(Duration::from_millis(10), &mut second_root)
                .await
                .is_err()
        );
        assert!(
            timeout(Duration::from_millis(100), second_root_started.notified())
                .await
                .is_err(),
            "the opposing root entered B while the first root still held admission"
        );

        release_first_root.notify_one();
        timeout(Duration::from_secs(2), async {
            first_root
                .await
                .expect("first root task")
                .expect("first root result");
            second_root.await.expect("second root result");
        })
        .await
        .expect("opposing synchronous roots timed out");

        assert_eq!(a_calls.load(Ordering::SeqCst), 3);
        assert_eq!(b_calls.load(Ordering::SeqCst), 3);
        assert_eq!(
            *trace.lock().expect("trace lock"),
            [
                "a:first-outer",
                "b:first-middle",
                "a:first-inner",
                "b:second-outer",
                "a:second-middle",
                "b:second-inner"
            ]
        );
        a_driver
            .shutdown(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("shut down A store");
        b_driver
            .shutdown(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("shut down B store");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn legacy_reentry_routes_complete() {
        assert_synchronous_reentry_depth_is_bounded_and_recovers();
        timeout(Duration::from_secs(10), async {
            sync_lifted_same_instance_reentry_completes().await;
            blocking_host_operation_propagates_the_reentry_chain().await;
            unrelated_guest_call_waits_for_the_active_chain().await;
            opposing_plugin_roots_scenario().await;
        })
        .await
        .expect("legacy reentry routing scenarios timed out");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_drains_accepted_work() {
        timeout(Duration::from_secs(10), async {
            shutdown_drains_accepted_calls_and_rejects_new_work().await;
            shutdown_waits_for_joined_store_background_tasks().await;
            owner_drop_drains_accepted_work().await;
            shutdown_reentry_scenario().await;
        })
        .await
        .expect("Store shutdown scenarios timed out");
    }

    async fn startup_task_loss_is_reported() {
        let started = LegacyStore::start(
            test_store(),
            LegacySyncReentry::new(),
            Arc::new(DroppingSpawner),
        )
        .await;
        let Err(error) = started else {
            panic!("a discarded driver task should not start");
        };
        assert!(error.to_string().contains("ended without reporting"));
    }

    async fn active_driver_task_loss_is_reported() {
        let driver_abort = Arc::new(Mutex::new(None));
        let spawner = Arc::new(AbortRecordingSpawner {
            runtime: tokio::runtime::Handle::current(),
            driver_abort: Arc::clone(&driver_abort),
        });
        let executor = LegacyStore::start(test_store(), LegacySyncReentry::new(), spawner)
            .await
            .expect("start test Store driver");
        let handle = executor.handle();
        let started = Arc::new(Notify::new());
        let never_release = Arc::new(Notify::new());

        let call_handle = handle.clone();
        let call_started = Arc::clone(&started);
        let call_release = Arc::clone(&never_release);
        let active = tokio::spawn(async move {
            call_handle
                .call(move |_| {
                    Box::pin(async move {
                        call_started.notify_one();
                        call_release.notified().await;
                        Ok(())
                    })
                })
                .await
        });
        started.notified().await;

        driver_abort
            .lock()
            .expect("driver abort lock")
            .take()
            .expect("driver abort handle")
            .abort();
        let call_error = active
            .await
            .expect("active call task")
            .expect_err("driver cancellation should fail the active call");
        assert!(call_error.to_string().contains("ended without reporting"));

        let retained = match executor.state() {
            DriverState::Failed(error) => error,
            state => panic!("expected a failed driver, got {state:?}"),
        };
        assert!(retained.to_string().contains("ended without reporting"));
        let mut join = handle.driver_join();
        assert_eq!(
            join.wait().await.expect_err("driver join should fail"),
            retained
        );
    }

    async fn driver_panic_is_reported() {
        let store = ConcurrentStore::new(test_store()).await;
        let handle = store.handle();
        let call_error = timeout(
            Duration::from_secs(2),
            handle.call_guest::<(), _>(|_| panic!("intentional Store driver panic")),
        )
        .await
        .expect("panicked Store driver call timed out")
        .expect_err("panicked Store driver call should fail");
        assert!(call_error.to_string().contains("ended without reporting"));

        let retained = match store.state() {
            DriverState::Failed(error) => error,
            state => panic!("expected a failed driver, got {state:?}"),
        };
        let mut join = store.driver_join();
        assert_eq!(
            join.wait().await.expect_err("driver join should fail"),
            retained
        );
        let later_error = handle
            .call(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect_err("later calls should retain the driver failure");
        assert!(later_error.to_string().contains("ended without reporting"));
    }

    #[allow(clippy::too_many_lines)]
    async fn caught_reentrant_guest_trap_stops_driver_and_queued_work() {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let engine = Engine::new(&config).expect("test engine");
        let component = Component::new(&engine, sync_reentry_component()).expect("test component");
        let run_slot = Arc::new(OnceLock::<TypedFunc<(), ()>>::new());
        let driver_slot = Arc::new(OnceLock::<ConcurrentStore>::new());
        let host_calls = Arc::new(AtomicUsize::new(0));
        let queued_call_ran = Arc::new(AtomicBool::new(false));

        let mut linker = Linker::<TestHostState>::new(&engine);
        let run_for_host = Arc::clone(&run_slot);
        let driver_for_host = Arc::clone(&driver_slot);
        let calls_for_host = Arc::clone(&host_calls);
        let queued_for_host = Arc::clone(&queued_call_ran);
        linker
            .root()
            .func_wrap_async("host", move |mut store, ()| {
                let run = *run_for_host.get().expect("run initialized");
                let driver = driver_for_host
                    .get()
                    .expect("store driver initialized")
                    .clone();
                let host_calls = Arc::clone(&calls_for_host);
                let queued_call_ran = Arc::clone(&queued_for_host);
                Box::new(async move {
                    if host_calls.fetch_add(1, Ordering::SeqCst) != 0 {
                        return Err(wasmtime::Error::msg("intentional reentrant guest failure"));
                    }

                    let first_driver = driver.clone();
                    let second_driver = driver.clone();
                    let queued = async move {
                        tokio::join!(
                            biased;
                            first_driver.call_guest(move |mut guest| {
                                Box::pin(async move { guest.call(run, ()).await })
                            }),
                            second_driver.call_guest(move |_| {
                                queued_call_ran.store(true, Ordering::Release);
                                Box::pin(async move { Ok(()) })
                            }),
                        )
                    };
                    assert!(driver.pump_reentry(&mut store, queued).await.is_err());
                    Ok(())
                })
            })
            .expect("link host function");

        let mut store = Store::new(&engine, TestHostState);
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let run = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .expect("run export");
        assert!(run_slot.set(run).is_ok());
        let driver = ConcurrentStore::new(store).await;
        assert!(driver_slot.set(driver.clone()).is_ok());

        let error = timeout(
            Duration::from_secs(2),
            driver.call_guest(move |mut guest| Box::pin(async move { guest.call(run, ()).await })),
        )
        .await
        .expect("reentrant guest trap timed out")
        .expect_err("reentrant guest call should fail");
        assert!(error.to_string().contains("wasm backtrace"), "{error}");

        let mut join = driver.driver_join();
        let retained = join.wait().await.expect_err("driver should fail");
        assert!(retained.to_string().contains("wasm backtrace"));
        assert!(!queued_call_ran.load(Ordering::Acquire));
        assert_eq!(host_calls.load(Ordering::SeqCst), 2);
    }

    #[allow(clippy::too_many_lines)]
    async fn owned_resource_lifecycle_retires_store_on_trap() {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let engine = Engine::new(&config).expect("test engine");
        let component =
            Component::new(&engine, owned_resource_component()).expect("test component");
        let resource_drops = Arc::new(AtomicUsize::new(0));
        let store_dropped = Arc::new(AtomicBool::new(false));
        let mut linker = Linker::<BoundaryState>::new(&engine);
        linker
            .root()
            .resource(
                "resource",
                ResourceType::host::<BoundaryResource>(),
                |store, rep| {
                    assert!(matches!(rep, 11 | 22 | 33 | 44));
                    store.data().resource_drops.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                },
            )
            .expect("link host resource");
        let mut store = Store::new(
            &engine,
            BoundaryState {
                resource_drops: Arc::clone(&resource_drops),
                store_dropped: Arc::clone(&store_dropped),
            },
        );
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let drop_resource = instance
            .get_typed_func::<(Resource<BoundaryResource>,), ()>(&mut store, "drop")
            .expect("drop export");
        let return_resource = instance
            .get_typed_func::<(Resource<BoundaryResource>,), (Resource<BoundaryResource>,)>(
                &mut store, "return",
            )
            .expect("return export");
        let trap = instance
            .get_typed_func::<(Resource<BoundaryResource>,), ()>(&mut store, "trap")
            .expect("trap export");
        let retain = instance
            .get_typed_func::<(Resource<BoundaryResource>,), ()>(&mut store, "retain")
            .expect("retain export");
        let drop_retained = instance
            .get_typed_func::<(), ()>(&mut store, "drop-retained")
            .expect("drop-retained export");
        let driver = LegacyStore::start(
            store,
            LegacySyncReentry::new(),
            Arc::new(TestSpawner {
                runtime: tokio::runtime::Handle::current(),
            }),
        )
        .await
        .expect("start test Store driver");
        let handle = driver.handle();

        let setup_error = driver
            .call_guest(|_| {
                Box::pin(async move { Err::<(), _>(wasmtime::Error::msg("setup failed")) })
            })
            .await
            .expect_err("pre-call setup should fail");
        assert!(setup_error.to_string().contains("setup failed"));
        assert_eq!(driver.state(), DriverState::Accepting);
        driver
            .call_guest(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect("pre-call setup failure should leave the store usable");

        driver
            .call_guest(move |mut context| {
                Box::pin(async move { context.call(drop_resource, (Resource::new_own(11),)).await })
            })
            .await
            .expect("guest should drop the owned resource");
        assert_eq!(resource_drops.load(Ordering::SeqCst), 1);

        let (returned,) = driver
            .call_guest(move |mut context| {
                Box::pin(async move {
                    context
                        .call(return_resource, (Resource::new_own(22),))
                        .await
                })
            })
            .await
            .expect("guest should return the owned resource");
        assert_eq!(returned.rep(), 22);
        assert!(returned.owned());
        assert_eq!(resource_drops.load(Ordering::SeqCst), 1);

        driver
            .call_guest(move |mut context| {
                Box::pin(async move { context.call(drop_resource, (returned,)).await })
            })
            .await
            .expect("returned ownership should be droppable");
        assert_eq!(resource_drops.load(Ordering::SeqCst), 2);

        driver
            .call_guest(move |mut context| {
                Box::pin(async move { context.call(retain, (Resource::new_own(44),)).await })
            })
            .await
            .expect("guest should retain ownership across calls");
        assert_eq!(resource_drops.load(Ordering::SeqCst), 2);
        driver
            .call_guest(move |mut context| {
                Box::pin(async move { context.call(drop_retained, ()).await })
            })
            .await
            .expect("guest should drop retained ownership later");
        assert_eq!(resource_drops.load(Ordering::SeqCst), 3);

        let call_error = driver
            .call_guest(move |mut context| {
                Box::pin(async move { context.call(trap, (Resource::new_own(33),)).await })
            })
            .await
            .expect_err("guest call should fail");
        assert!(
            call_error.to_string().contains("wasm backtrace"),
            "{call_error}"
        );

        let later_call_ran = Arc::new(AtomicBool::new(false));
        let ran = Arc::clone(&later_call_ran);
        let immediate_error = handle
            .call_guest(move |_| {
                ran.store(true, Ordering::Release);
                Box::pin(async move { Ok(()) })
            })
            .await
            .expect_err("a call after a guest trap should be rejected");
        assert!(
            immediate_error.to_string().contains("guest call")
                || immediate_error.to_string().contains("wasm backtrace"),
            "{immediate_error}"
        );
        assert!(!later_call_ran.load(Ordering::Acquire));

        let mut join = driver.driver_join();
        let retained = join.wait().await.expect_err("driver should fail");
        assert!(
            retained.to_string().contains("wasm backtrace"),
            "{retained}"
        );
        // Trapping does not run the destructor for a guest-held handle. Its
        // backing host data is released when the failed Store is dropped.
        assert!(store_dropped.load(Ordering::Acquire));
        assert_eq!(resource_drops.load(Ordering::SeqCst), 3);
        assert_eq!(driver.state(), DriverState::Failed(Arc::clone(&retained)));

        let later_error = handle
            .call(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect_err("later calls should retain the guest failure");
        assert!(
            later_error.to_string().contains("wasm backtrace"),
            "{later_error}"
        );
    }

    fn shutdown_does_not_hide_terminal_failure() {
        let error = super::executor::reconcile_shutdown_result(
            Ok(Ok("finalized")),
            Err(Arc::new(DriverError::new("late driver failure"))),
        )
        .expect_err("terminal driver failure should override finalizer success");
        assert!(error.to_string().contains("driver failed"), "{error}");
        assert!(error.to_string().contains("late driver failure"), "{error}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn terminal_failure_is_retained() {
        startup_task_loss_is_reported().await;
        active_driver_task_loss_is_reported().await;
        driver_panic_is_reported().await;
        caught_reentrant_guest_trap_stops_driver_and_queued_work().await;
        owned_resource_lifecycle_retires_store_on_trap().await;
        shutdown_does_not_hide_terminal_failure();

        let store = ConcurrentStore::new(test_store()).await;
        let handle = store.handle();
        let shutdown_error = store
            .shutdown(|_| {
                Box::pin(async move { Err::<(), _>(wasmtime::Error::msg("finalizer failed")) })
            })
            .await
            .expect_err("finalizer failure should fail shutdown");
        assert!(shutdown_error.to_string().contains("finalizer failed"));

        let retained = match store.state() {
            DriverState::Failed(error) => error,
            state => panic!("expected a failed driver, got {state:?}"),
        };
        assert!(retained.to_string().contains("finalizer failed"));

        let mut join = store.driver_join();
        let joined = join.wait().await.expect_err("driver join should fail");
        assert_eq!(joined, retained);

        let call_error = handle
            .call(|_| Box::pin(async move { Ok(()) }))
            .await
            .expect_err("future calls should retain the driver failure");
        assert!(call_error.to_string().contains("finalizer failed"));
    }
}
