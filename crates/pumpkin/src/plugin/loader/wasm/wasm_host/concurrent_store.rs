use std::{
    collections::HashMap,
    future::{Future, poll_fn},
    pin::Pin,
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use futures::{StreamExt, stream::FuturesUnordered};
use tokio::sync::{Mutex, mpsc, oneshot, watch};
use wasmtime::{
    AsContextMut, Store, StoreContextMut,
    component::{Accessor, AccessorTask, ComponentNamedList, Lift, Lower, TypedFunc},
};

use super::state::PluginHostState;

const STORE_QUEUE_CAPACITY: usize = 64;
const REENTRY_QUEUE_CAPACITY: usize = 64;
const MAX_SYNC_REENTRY_DEPTH: usize = 64;

static NEXT_REENTRY_CHAIN_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct ReentryContext {
    chain_id: u64,
    depth: usize,
}

impl ReentryContext {
    fn root() -> Self {
        Self {
            chain_id: NEXT_REENTRY_CHAIN_ID.fetch_add(1, Ordering::Relaxed),
            depth: 0,
        }
    }

    fn current() -> Option<Self> {
        REENTRY_CONTEXT.try_with(|context| *context).ok()
    }

    fn child(self) -> wasmtime::Result<Self> {
        if self.depth >= MAX_SYNC_REENTRY_DEPTH {
            return Err(wasmtime::Error::msg(format!(
                "Wasm plugin synchronous reentry exceeded the maximum depth of {MAX_SYNC_REENTRY_DEPTH}"
            )));
        }

        Ok(Self {
            chain_id: self.chain_id,
            depth: self.depth + 1,
        })
    }
}

tokio::task_local! {
    static REENTRY_CONTEXT: ReentryContext;
}

pub(crate) type StoreFuture<'a, T> = Pin<Box<dyn Future<Output = wasmtime::Result<T>> + Send + 'a>>;

trait StoreJob: Send {
    fn run(self: Box<Self>, accessor: &Accessor<PluginHostState>) -> StoreFuture<'_, ()>;
}

trait GuestStoreJob: Send {
    fn run_concurrent(self: Box<Self>, accessor: &Accessor<PluginHostState>)
    -> StoreFuture<'_, ()>;

    fn run_reentrant(
        self: Box<Self>,
        store: StoreContextMut<'_, PluginHostState>,
    ) -> StoreFuture<'_, ()>;
}

struct StoreCall<F, R> {
    call: F,
    result: oneshot::Sender<wasmtime::Result<R>>,
}

impl<F, R> StoreJob for StoreCall<F, R>
where
    F: for<'a> FnOnce(&'a Accessor<PluginHostState>) -> StoreFuture<'a, R> + Send + 'static,
    R: Send + 'static,
{
    fn run(self: Box<Self>, accessor: &Accessor<PluginHostState>) -> StoreFuture<'_, ()> {
        let Self { call, result } = *self;
        let future = call(accessor);
        Box::pin(async move {
            let _ = result.send(future.await);
            Ok(())
        })
    }
}

struct GuestStoreCall<F, R> {
    call: F,
    result: oneshot::Sender<wasmtime::Result<R>>,
    context: ReentryContext,
    reentry: Arc<ReentryState>,
}

impl<F, R> GuestStoreJob for GuestStoreCall<F, R>
where
    F: for<'a> FnOnce(GuestCallContext<'a>) -> StoreFuture<'a, R> + Send + 'static,
    R: Send + 'static,
{
    fn run_concurrent(
        self: Box<Self>,
        accessor: &Accessor<PluginHostState>,
    ) -> StoreFuture<'_, ()> {
        let Self {
            call,
            result,
            context,
            reentry,
        } = *self;
        Box::pin(REENTRY_CONTEXT.scope(context, async move {
            let _active_context = reentry.enter_active_context(context);
            let output = call(GuestCallContext::Concurrent(accessor)).await;
            let _ = result.send(output);
            Ok(())
        }))
    }

    fn run_reentrant(
        self: Box<Self>,
        store: StoreContextMut<'_, PluginHostState>,
    ) -> StoreFuture<'_, ()> {
        let Self {
            call,
            result,
            context,
            reentry,
        } = *self;
        Box::pin(REENTRY_CONTEXT.scope(context, async move {
            let _active_context = reentry.enter_active_context(context);
            let output = call(GuestCallContext::Reentrant(store)).await;
            let _ = result.send(output);
            Ok(())
        }))
    }
}

/// Store access for a host-to-guest call.
///
/// Normal calls enter through Wasmtime's stackless concurrent API. If a guest
/// calls through the host into another plugin and that plugin calls this store
/// again, the nested call instead reuses the active store context. The latter
/// is required for plain synchronous WIT functions, which cannot be linked to
/// a concurrent host function.
pub(crate) enum GuestCallContext<'a> {
    Concurrent(&'a Accessor<PluginHostState>),
    Reentrant(StoreContextMut<'a, PluginHostState>),
}

impl GuestCallContext<'_> {
    #[cfg(test)]
    const fn is_reentrant(&self) -> bool {
        matches!(self, Self::Reentrant(_))
    }

    pub(crate) fn with<R>(
        &mut self,
        call: impl FnOnce(StoreContextMut<'_, PluginHostState>) -> R,
    ) -> R {
        match self {
            Self::Concurrent(accessor) => accessor.with(|mut access| call(access.as_context_mut())),
            Self::Reentrant(store) => call(store.as_context_mut()),
        }
    }

    pub(crate) async fn call<Params, Return>(
        &mut self,
        function: TypedFunc<Params, Return>,
        params: Params,
    ) -> wasmtime::Result<Return>
    where
        Params: ComponentNamedList + Lower + 'static,
        Return: ComponentNamedList + Lift + 'static,
    {
        match self {
            Self::Concurrent(accessor) => function.call_concurrent(&**accessor, params).await,
            Self::Reentrant(store) => function.call_async(store.as_context_mut(), params).await,
        }
    }
}

struct ConcurrentTask(Box<dyn StoreJob>);

impl AccessorTask<PluginHostState> for ConcurrentTask {
    fn run(
        self,
        accessor: &Accessor<PluginHostState>,
    ) -> impl Future<Output = wasmtime::Result<()>> + Send {
        self.0.run(accessor)
    }
}

enum StoreMessage {
    Call(Box<dyn StoreJob>),
    GuestCall(Box<dyn GuestStoreJob>),
    Shutdown(Box<dyn StoreJob>),
}

type ReentrySender = mpsc::Sender<Box<dyn GuestStoreJob>>;

struct ReentryScope {
    id: u64,
    sender: ReentrySender,
}

struct ReentryState {
    next_scope_id: AtomicU64,
    scopes: StdMutex<HashMap<u64, Vec<ReentryScope>>>,
    active_contexts: StdMutex<Vec<(u64, ReentryContext)>>,
}

impl ReentryState {
    fn new() -> Self {
        Self {
            next_scope_id: AtomicU64::new(0),
            scopes: StdMutex::new(HashMap::new()),
            active_contexts: StdMutex::new(Vec::new()),
        }
    }

    fn enter_active_context(
        self: &Arc<Self>,
        context: ReentryContext,
    ) -> ActiveContextRegistration {
        let id = self.next_scope_id.fetch_add(1, Ordering::Relaxed);
        self.active_contexts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push((id, context));
        ActiveContextRegistration {
            state: Arc::clone(self),
            id,
        }
    }

    fn current_context(&self) -> Option<ReentryContext> {
        self.active_contexts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .last()
            .map(|(_, context)| *context)
    }

    fn remove_active_context(&self, id: u64) {
        let mut contexts = self
            .active_contexts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(index) = contexts
            .iter()
            .rposition(|(context_id, _)| *context_id == id)
        {
            contexts.remove(index);
        }
    }

    fn register(
        self: &Arc<Self>,
        context: ReentryContext,
    ) -> (ReentryRegistration, mpsc::Receiver<Box<dyn GuestStoreJob>>) {
        let scope_id = self.next_scope_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = mpsc::channel(REENTRY_QUEUE_CAPACITY);
        let mut scopes = self
            .scopes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        scopes
            .entry(context.chain_id)
            .or_default()
            .push(ReentryScope {
                id: scope_id,
                sender,
            });
        (
            ReentryRegistration {
                state: Arc::clone(self),
                chain_id: context.chain_id,
                scope_id,
                active: true,
            },
            receiver,
        )
    }

    fn sender_for(&self, chain_id: u64) -> Option<ReentrySender> {
        let scopes = self
            .scopes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        scopes
            .get(&chain_id)
            .and_then(|scopes| scopes.last())
            .map(|scope| scope.sender.clone())
    }

    fn remove(&self, chain_id: u64, scope_id: u64) {
        let mut scopes = self
            .scopes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remove_chain = scopes.get_mut(&chain_id).is_some_and(|chain_scopes| {
            if let Some(index) = chain_scopes.iter().rposition(|scope| scope.id == scope_id) {
                chain_scopes.remove(index);
            }
            chain_scopes.is_empty()
        });
        if remove_chain {
            scopes.remove(&chain_id);
        }
    }
}

struct ActiveContextRegistration {
    state: Arc<ReentryState>,
    id: u64,
}

impl Drop for ActiveContextRegistration {
    fn drop(&mut self) {
        self.state.remove_active_context(self.id);
    }
}

struct ReentryRegistration {
    state: Arc<ReentryState>,
    chain_id: u64,
    scope_id: u64,
    active: bool,
}

impl ReentryRegistration {
    fn close(&mut self) {
        if self.active {
            self.state.remove(self.chain_id, self.scope_id);
            self.active = false;
        }
    }
}

impl Drop for ReentryRegistration {
    fn drop(&mut self) {
        self.close();
    }
}

/// Owns a Wasmtime store and runs all guest calls in one concurrent event loop.
///
/// Keeping the store in this driver, rather than behind a mutex, lets callers
/// queue work without exclusively borrowing the store themselves. Wasmtime
/// still blocks ordinary recursive entry while a sync-lifted export or a
/// store-blocking host import is active. Callback-capable host imports use the
/// call-scoped reentry pump below to service only calls from the same chain on
/// the active store context.
#[derive(Clone)]
pub struct ConcurrentStore {
    sender: mpsc::Sender<StoreMessage>,
    accepting: Arc<AtomicBool>,
    send_gate: Arc<Mutex<()>>,
    stopped: watch::Receiver<bool>,
    reentry: Arc<ReentryState>,
}

impl ConcurrentStore {
    #[must_use]
    pub fn new(mut store: Store<PluginHostState>) -> Self {
        let (sender, mut receiver) = mpsc::channel::<StoreMessage>(STORE_QUEUE_CAPACITY);
        let accepting = Arc::new(AtomicBool::new(true));
        let send_gate = Arc::new(Mutex::new(()));
        let (stopped_sender, stopped) = watch::channel(false);

        tokio::spawn(async move {
            let result = store
                .run_concurrent(async move |accessor| -> wasmtime::Result<()> {
                    let mut active_calls = FuturesUnordered::new();

                    loop {
                        let message = tokio::select! {
                            Some(()) = active_calls.next(), if !active_calls.is_empty() => {
                                continue;
                            }
                            message = receiver.recv() => message,
                        };
                        let Some(message) = message else {
                            while active_calls.next().await.is_some() {}
                            poll_fn(|cx| accessor.poll_no_interesting_tasks(cx)).await;
                            return Ok(());
                        };

                        match message {
                            StoreMessage::Call(job) => {
                                active_calls.push(accessor.spawn(ConcurrentTask(job))?);
                                tokio::task::yield_now().await;
                            }
                            StoreMessage::GuestCall(job) => {
                                // Plugin callbacks are intentionally serialized. A callback
                                // that belongs to the active synchronous chain bypasses this
                                // queue through `pump_reentry`; unrelated work waits here.
                                job.run_concurrent(accessor).await?;
                            }
                            StoreMessage::Shutdown(job) => {
                                receiver.close();
                                while active_calls.next().await.is_some() {}
                                let result = job.run(accessor).await;
                                poll_fn(|cx| accessor.poll_no_interesting_tasks(cx)).await;
                                return result;
                            }
                        }
                    }
                })
                .await;
            drop(store);
            let _ = stopped_sender.send(true);

            match result {
                Ok(Ok(())) => {}
                Ok(Err(error)) | Err(error) => {
                    tracing::error!(%error, "Wasm plugin store driver stopped");
                }
            }
        });

        Self {
            sender,
            accepting,
            send_gate,
            stopped,
            reentry: Arc::new(ReentryState::new()),
        }
    }

    /// Queues a store operation and waits for its result.
    ///
    /// Accepted operations run to completion even if the waiting future is
    /// dropped, so the operation must own every value captured by its closure.
    pub async fn call<R, F>(&self, call: F) -> wasmtime::Result<R>
    where
        F: for<'a> FnOnce(&'a Accessor<PluginHostState>) -> StoreFuture<'a, R> + Send + 'static,
        R: Send + 'static,
    {
        let (result, receiver) = oneshot::channel();
        let send_guard = self.send_gate.lock().await;
        if !self.accepting.load(Ordering::Acquire) {
            return Err(wasmtime::Error::msg("Wasm plugin store is shutting down"));
        }
        let permit = self
            .sender
            .reserve()
            .await
            .map_err(|_| wasmtime::Error::msg("Wasm plugin store driver is not running"))?;
        permit.send(StoreMessage::Call(Box::new(StoreCall { call, result })));
        drop(send_guard);

        receiver
            .await
            .map_err(|_| wasmtime::Error::msg("Wasm plugin store call was cancelled"))?
    }

    /// Queues a guest call, or routes it through the currently active host
    /// frame when this store appears earlier in a synchronous plugin cycle.
    pub(crate) async fn call_guest<R, F>(&self, call: F) -> wasmtime::Result<R>
    where
        F: for<'a> FnOnce(GuestCallContext<'a>) -> StoreFuture<'a, R> + Send + 'static,
        R: Send + 'static,
    {
        let (result, receiver) = oneshot::channel();
        let inherited_context = ReentryContext::current();
        let context = inherited_context.unwrap_or_else(ReentryContext::root);
        let mut job: Box<dyn GuestStoreJob> = Box::new(GuestStoreCall {
            call,
            result,
            context,
            reentry: Arc::clone(&self.reentry),
        });

        if let Some(context) = inherited_context
            && let Some(sender) = self.reentry.sender_for(context.chain_id)
        {
            match sender.send(job).await {
                Ok(()) => {
                    return receiver.await.map_err(|_| {
                        wasmtime::Error::msg("Wasm plugin guest call was cancelled")
                    })?;
                }
                Err(error) => job = error.0,
            }
        }

        let send_guard = self.send_gate.lock().await;
        if !self.accepting.load(Ordering::Acquire) {
            return Err(wasmtime::Error::msg("Wasm plugin store is shutting down"));
        }

        let permit = self
            .sender
            .reserve()
            .await
            .map_err(|_| wasmtime::Error::msg("Wasm plugin store driver is not running"))?;
        permit.send(StoreMessage::GuestCall(job));
        drop(send_guard);

        receiver
            .await
            .map_err(|_| wasmtime::Error::msg("Wasm plugin guest call was cancelled"))?
    }

    /// Waits for an outbound host operation while servicing any calls routed
    /// back into this store through the active synchronous call chain.
    pub(crate) async fn pump_reentry<R, S, F>(
        &self,
        store: &mut S,
        future: F,
    ) -> wasmtime::Result<R>
    where
        S: AsContextMut<Data = PluginHostState> + Send,
        F: Future<Output = R> + Send,
    {
        let context = self.next_reentry_context()?;
        self.pump_reentry_with_context(store, context, future).await
    }

    fn next_reentry_context(&self) -> wasmtime::Result<ReentryContext> {
        self.reentry
            .current_context()
            .or_else(ReentryContext::current)
            .unwrap_or_else(ReentryContext::root)
            .child()
    }

    async fn pump_reentry_with_context<R, S, F>(
        &self,
        store: &mut S,
        context: ReentryContext,
        future: F,
    ) -> wasmtime::Result<R>
    where
        S: AsContextMut<Data = PluginHostState> + Send,
        F: Future<Output = R> + Send,
    {
        let (mut registration, mut receiver) = self.reentry.register(context);
        let scoped_future = REENTRY_CONTEXT.scope(context, future);
        tokio::pin!(scoped_future);

        let output = loop {
            tokio::select! {
                Some(job) = receiver.recv() => {
                    job.run_reentrant(store.as_context_mut()).await?;
                }
                output = &mut scoped_future => break output,
            }
        };

        registration.close();
        receiver.close();
        while let Ok(job) = receiver.try_recv() {
            if let Err(error) = job.run_reentrant(store.as_context_mut()).await {
                tracing::error!(%error, "Accepted reentrant Wasm plugin call failed while draining");
            }
        }
        Ok(output)
    }

    /// Runs a synchronous host operation away from the async runtime while
    /// continuing to service calls routed back into this plugin. This is used
    /// for game APIs that deliberately expose synchronous behavior and may
    /// call `PluginManager::fire_blocking` internally.
    pub(crate) async fn pump_blocking<R, S, F>(
        &self,
        store: &mut S,
        operation: F,
    ) -> wasmtime::Result<R>
    where
        R: Send + 'static,
        S: AsContextMut<Data = PluginHostState> + Send,
        F: FnOnce() -> R + Send + 'static,
    {
        let context = self.next_reentry_context()?;
        let operation =
            tokio::task::spawn_blocking(move || REENTRY_CONTEXT.sync_scope(context, operation));
        self.pump_reentry_with_context(store, context, operation)
            .await?
            .map_err(|error| {
                wasmtime::Error::msg(format!(
                    "Synchronous Wasm plugin host operation failed: {error}"
                ))
            })
    }

    /// Stops accepting new work, waits for every accepted call, runs one final
    /// lifecycle operation, then drains background work and drops the store.
    pub async fn shutdown<R, F>(&self, call: F) -> wasmtime::Result<R>
    where
        F: for<'a> FnOnce(&'a Accessor<PluginHostState>) -> StoreFuture<'a, R> + Send + 'static,
        R: Send + 'static,
    {
        let (result, receiver) = oneshot::channel();
        let send_guard = self.send_gate.lock().await;
        if !self.accepting.load(Ordering::Acquire) {
            return Err(wasmtime::Error::msg(
                "Wasm plugin store is already shutting down",
            ));
        }
        let permit = self
            .sender
            .reserve()
            .await
            .map_err(|_| wasmtime::Error::msg("Wasm plugin store driver is not running"))?;
        self.accepting.store(false, Ordering::Release);
        permit.send(StoreMessage::Shutdown(Box::new(StoreCall { call, result })));
        drop(send_guard);

        let result = receiver.await;
        let mut stopped = self.stopped.clone();
        if !*stopped.borrow() {
            stopped.changed().await.map_err(|_| {
                wasmtime::Error::msg("Wasm plugin store driver stopped unexpectedly")
            })?;
        }
        result.map_err(|_| wasmtime::Error::msg("Wasm plugin store shutdown was cancelled"))?
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    };

    use tokio::{
        sync::Notify,
        time::{Duration, timeout},
    };
    use wasm_encoder::{
        CodeSection, ComponentBuilder, ComponentExportKind, ComponentTypeRef, EntityType,
        ExportKind, ExportSection, Function, FunctionSection, ImportSection, Instruction, Module,
        ModuleArg, PrimitiveValType, TypeSection,
    };
    use wasmtime::{
        Config, Engine, Store,
        component::{Component, Linker, TypedFunc},
    };

    use super::{
        ConcurrentStore, MAX_SYNC_REENTRY_DEPTH, PluginHostState, ReentryContext, StoreFuture,
        StoreJob, StoreMessage,
    };

    struct NoopJob;

    impl StoreJob for NoopJob {
        fn run(
            self: Box<Self>,
            _accessor: &wasmtime::component::Accessor<PluginHostState>,
        ) -> StoreFuture<'_, ()> {
            Box::pin(async move { Ok(()) })
        }
    }

    struct BackgroundTask {
        started: Arc<Notify>,
        release: Arc<Notify>,
    }

    impl wasmtime::component::AccessorTask<PluginHostState> for BackgroundTask {
        async fn run(
            self,
            _accessor: &wasmtime::component::Accessor<PluginHostState>,
        ) -> wasmtime::Result<()> {
            self.started.notify_one();
            self.release.notified().await;
            Ok(())
        }
    }

    fn test_store() -> Store<PluginHostState> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.wasm_component_model_async(true);
        config.concurrency_support(true);
        let engine = Engine::new(&config).expect("test engine");
        Store::new(&engine, PluginHostState::new())
    }

    #[test]
    fn synchronous_reentry_depth_is_bounded_and_recovers() {
        let root = ReentryContext::root();
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

    #[test]
    fn reentry_scopes_only_match_their_causal_chain() {
        let state = Arc::new(super::ReentryState::new());
        let active_context = ReentryContext::root();
        let unrelated_context = ReentryContext::root();
        let (registration, _receiver) = state.register(active_context);

        assert!(state.sender_for(active_context.chain_id).is_some());
        assert!(state.sender_for(unrelated_context.chain_id).is_none());

        drop(registration);
        assert!(state.sender_for(active_context.chain_id).is_none());
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn dispatches_nested_store_jobs_without_deadlocking() {
        // This verifies the store driver, not reentry into a sync-lifted guest
        // export. Wasmtime currently keeps that export locked until it returns.
        let plugin_a = ConcurrentStore::new(test_store());
        let plugin_b = ConcurrentStore::new(test_store());
        let trace = Arc::new(Mutex::new(Vec::new()));

        let nested_a = plugin_a.clone();
        let nested_trace = Arc::clone(&trace);
        let result = timeout(
            Duration::from_secs(2),
            plugin_a.call(move |_| {
                let plugin_b = plugin_b.clone();
                let nested_a = nested_a.clone();
                let trace = Arc::clone(&nested_trace);
                Box::pin(async move {
                    trace.lock().expect("trace lock").push("a:outer");
                    plugin_b
                        .call(move |_| {
                            let nested_a = nested_a.clone();
                            let trace = Arc::clone(&trace);
                            Box::pin(async move {
                                trace.lock().expect("trace lock").push("b");
                                nested_a
                                    .call(move |_| {
                                        Box::pin(async move {
                                            trace.lock().expect("trace lock").push("a:inner");
                                            Ok(())
                                        })
                                    })
                                    .await
                            })
                        })
                        .await
                })
            }),
        )
        .await
        .expect("reentrant dispatch timed out");

        result.expect("reentrant dispatch failed");
        assert_eq!(
            *trace.lock().expect("trace lock"),
            ["a:outer", "b", "a:inner"]
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_drains_accepted_calls_and_rejects_new_work() {
        let store = ConcurrentStore::new(test_store());
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
        while store.accepting.load(Ordering::Acquire) {
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
        assert!(*store.stopped.borrow());
    }

    #[tokio::test]
    async fn cancelled_shutdown_does_not_close_the_store_before_enqueue() {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        sender
            .send(StoreMessage::Call(Box::new(NoopJob)))
            .await
            .expect("fill store queue");
        let (_stopped_sender, stopped) = tokio::sync::watch::channel(false);
        let store = ConcurrentStore {
            sender,
            accepting: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            send_gate: Arc::new(tokio::sync::Mutex::new(())),
            stopped,
            reentry: Arc::new(super::ReentryState::new()),
        };

        let shutdown_store = store.clone();
        let shutdown = tokio::spawn(async move {
            shutdown_store
                .shutdown(|_| Box::pin(async move { Ok(()) }))
                .await
        });
        timeout(Duration::from_secs(1), async {
            while let Ok(guard) = store.send_gate.try_lock() {
                drop(guard);
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("shutdown did not wait for queue capacity");

        shutdown.abort();
        assert!(
            shutdown
                .await
                .expect_err("shutdown should be aborted")
                .is_cancelled()
        );
        assert!(store.accepting.load(Ordering::Acquire));
        assert!(matches!(receiver.recv().await, Some(StoreMessage::Call(_))));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_waits_for_store_background_tasks() {
        let store = ConcurrentStore::new(test_store());
        let started = Arc::new(Notify::new());
        let release = Arc::new(Notify::new());
        let task_started = Arc::clone(&started);
        let task_release = Arc::clone(&release);

        store
            .call(move |accessor| {
                let spawned = accessor.spawn(BackgroundTask {
                    started: task_started,
                    release: task_release,
                });
                Box::pin(async move {
                    spawned?;
                    Ok(())
                })
            })
            .await
            .expect("spawn background store task");
        started.notified().await;

        let shutdown_store = store.clone();
        let shutdown = tokio::spawn(async move {
            shutdown_store
                .shutdown(|_| Box::pin(async move { Ok(()) }))
                .await
        });
        while store.accepting.load(Ordering::Acquire) {
            tokio::task::yield_now().await;
        }
        assert!(!shutdown.is_finished());

        release.notify_one();
        shutdown
            .await
            .expect("shutdown task")
            .expect("shutdown result");
    }

    /// Acceptance test for plain synchronous WIT recursion through the
    /// call-scoped reentry pump.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

        let mut linker = Linker::<PluginHostState>::new(&engine);
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

        let mut store = Store::new(&engine, PluginHostState::new());
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let run = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .expect("run export");
        assert!(run_slot.set(run).is_ok());

        let driver = ConcurrentStore::new(store);
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
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

        let mut linker = Linker::<PluginHostState>::new(&engine);
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

        let mut store = Store::new(&engine, PluginHostState::new());
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let run = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .expect("run export");
        assert!(run_slot.set(run).is_ok());

        let driver = ConcurrentStore::new(store);
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
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

        let mut linker = Linker::<PluginHostState>::new(&engine);
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

        let mut store = Store::new(&engine, PluginHostState::new());
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let run = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .expect("run export");
        let driver = ConcurrentStore::new(store);
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn shutdown_allows_a_callback_needed_by_an_accepted_call() {
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

        let mut linker = Linker::<PluginHostState>::new(&engine);
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

        let mut store = Store::new(&engine, PluginHostState::new());
        let instance = linker
            .instantiate_async(&mut store, &component)
            .await
            .expect("instantiate test component");
        let run = instance
            .get_typed_func::<(), ()>(&mut store, "run")
            .expect("run export");
        assert!(run_slot.set(run).is_ok());
        let driver = ConcurrentStore::new(store);
        assert!(driver_slot.set(driver.clone()).is_ok());

        let outer_driver = driver.clone();
        let outer = tokio::spawn(async move {
            outer_driver
                .call_guest(move |mut context| Box::pin(async move { context.call(run, ()).await }))
                .await
        });
        entered.notified().await;

        let shutdown_driver = driver.clone();
        let shutdown = tokio::spawn(async move {
            shutdown_driver
                .shutdown(|_| Box::pin(async move { Ok(()) }))
                .await
        });
        while driver.accepting.load(Ordering::Acquire) {
            tokio::task::yield_now().await;
        }

        start_callback.notify_one();
        timeout(Duration::from_secs(2), callback_completed.notified())
            .await
            .expect("callback was rejected during shutdown");
        outer.await.expect("outer task").expect("outer guest call");
        shutdown
            .await
            .expect("shutdown task")
            .expect("shutdown store");
        assert_eq!(host_calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[allow(clippy::too_many_lines)]
    async fn sync_lifted_cross_plugin_reentry_completes() {
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

        let mut a_linker = Linker::<PluginHostState>::new(&engine);
        let a_driver_for_a = Arc::clone(&a_driver_slot);
        let b_driver_for_a = Arc::clone(&b_driver_slot);
        let b_run_for_a = Arc::clone(&b_run_slot);
        let a_calls_for_host = Arc::clone(&a_calls);
        let trace_for_a = Arc::clone(&trace);
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
                Box::new(async move {
                    match calls.fetch_add(1, Ordering::SeqCst) {
                        0 => {
                            trace.lock().expect("trace lock").push("a:outer");
                            let outbound = b_driver.call_guest(move |mut context| {
                                assert!(
                                    !context.is_reentrant(),
                                    "first call into B must use its concurrent store path"
                                );
                                Box::pin(async move { context.call(b_run, ()).await })
                            });
                            a_driver.pump_reentry(&mut store, outbound).await??;
                            trace.lock().expect("trace lock").push("a:resumed");
                        }
                        1 => trace.lock().expect("trace lock").push("a:inner"),
                        _ => return Err(wasmtime::Error::msg("unexpected extra call into A")),
                    }
                    Ok(())
                })
            })
            .expect("link A host function");

        let mut b_linker = Linker::<PluginHostState>::new(&engine);
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
                    if calls.fetch_add(1, Ordering::SeqCst) != 0 {
                        return Err(wasmtime::Error::msg("unexpected extra call into B"));
                    }
                    trace.lock().expect("trace lock").push("b:middle");
                    let outbound = a_driver.call_guest(move |mut context| {
                        assert!(
                            context.is_reentrant(),
                            "B to A callback must use A's active store frame"
                        );
                        Box::pin(async move { context.call(a_run, ()).await })
                    });
                    b_driver.pump_reentry(&mut store, outbound).await??;
                    trace.lock().expect("trace lock").push("b:resumed");
                    Ok(())
                })
            })
            .expect("link B host function");

        let mut a_store = Store::new(&engine, PluginHostState::new());
        let a_instance = a_linker
            .instantiate_async(&mut a_store, &component)
            .await
            .expect("instantiate A");
        let a_run = a_instance
            .get_typed_func::<(), ()>(&mut a_store, "run")
            .expect("get A run export");
        assert!(a_run_slot.set(a_run).is_ok());

        let mut b_store = Store::new(&engine, PluginHostState::new());
        let b_instance = b_linker
            .instantiate_async(&mut b_store, &component)
            .await
            .expect("instantiate B");
        let b_run = b_instance
            .get_typed_func::<(), ()>(&mut b_store, "run")
            .expect("get B run export");
        assert!(b_run_slot.set(b_run).is_ok());

        let a_driver = ConcurrentStore::new(a_store);
        let b_driver = ConcurrentStore::new(b_store);
        assert!(a_driver_slot.set(a_driver.clone()).is_ok());
        assert!(b_driver_slot.set(b_driver.clone()).is_ok());

        timeout(
            Duration::from_secs(2),
            a_driver.call_guest(move |mut context| {
                assert!(
                    !context.is_reentrant(),
                    "top-level call into A must use its concurrent store path"
                );
                Box::pin(async move { context.call(a_run, ()).await })
            }),
        )
        .await
        .expect("cross-plugin sync reentry timed out")
        .expect("cross-plugin sync reentry failed");

        assert_eq!(a_calls.load(Ordering::SeqCst), 2);
        assert_eq!(b_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            *trace.lock().expect("trace lock"),
            ["a:outer", "b:middle", "a:inner", "b:resumed", "a:resumed"]
        );
        assert!(
            a_driver
                .reentry
                .scopes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        );
        assert!(
            a_driver
                .reentry
                .active_contexts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        );
        assert!(
            b_driver
                .reentry
                .scopes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
        );
        assert!(
            b_driver
                .reentry
                .active_contexts
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .is_empty()
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
}
