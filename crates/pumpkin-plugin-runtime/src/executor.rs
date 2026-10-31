use std::{
    any::Any,
    collections::HashMap,
    future::{Future, poll_fn},
    ops::Deref,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::Pin,
    sync::{
        Arc, Mutex as StdMutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use futures::{StreamExt, stream::FuturesUnordered};
use tokio::sync::{Mutex, mpsc, oneshot};
use wasmtime::{
    AsContext, AsContextMut, Store, StoreContextMut,
    component::{Accessor, ComponentNamedList, Func, Lift, Lower, TypedFunc},
};

use crate::{
    RuntimeSpawner,
    chain::{ReentryContext, RootAdmissionGuard, scope, sync_scope},
    lifecycle::{DriverError, DriverJoin, DriverState, Lifecycle},
    policy::{LegacySyncReentry, StorePolicy},
};

const STORE_QUEUE_CAPACITY: usize = 64;
const REENTRY_QUEUE_CAPACITY: usize = 64;

/// How a Store's guest calls are driven.
///
/// A Store takes its mode from the `Engine` it was created on, so both must agree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreMode {
    /// One call at a time through `call_async`, on an engine without Wasmtime concurrency
    /// support. A host import may re-enter the guest of the Store it is running on, which Wasmtime
    /// rejects once concurrency support is on.
    Sequential,
    /// Calls run as tasks in Wasmtime's concurrent event loop, on an engine with concurrency
    /// support. Host imports reach the guest through the event loop instead of nesting calls.
    Concurrent,
}

pub type StoreFuture<'a, R> = Pin<Box<dyn Future<Output = wasmtime::Result<R>> + Send + 'a>>;

trait StoreJob<T>: Send {
    fn run_concurrent(self: Box<Self>, accessor: &Accessor<T>) -> StoreFuture<'_, ()>;

    fn run_sequential(self: Box<Self>, store: StoreContextMut<'_, T>) -> StoreFuture<'_, ()>;
}

trait GuestStoreJob<T>: Send {
    fn run_concurrent(self: Box<Self>, accessor: &Accessor<T>) -> StoreFuture<'_, ()>;

    fn run_sequential(
        self: Box<Self>,
        store: StoreContextMut<'_, T>,
        reentrant: bool,
    ) -> StoreFuture<'_, ()>;
}

struct StoreCall<F, R> {
    call: F,
    result: oneshot::Sender<wasmtime::Result<R>>,
}

impl<T, F, R> StoreJob<T> for StoreCall<F, R>
where
    T: Send + 'static,
    F: FnOnce(&mut T) -> R + Send + 'static,
    R: Send + 'static,
{
    fn run_concurrent(self: Box<Self>, accessor: &Accessor<T>) -> StoreFuture<'_, ()> {
        let Self { call, result } = *self;
        let output = accessor.with(|mut access| call(access.data_mut()));
        let _ = result.send(Ok(output));
        Box::pin(std::future::ready(Ok(())))
    }

    fn run_sequential(self: Box<Self>, mut store: StoreContextMut<'_, T>) -> StoreFuture<'_, ()> {
        let Self { call, result } = *self;
        let output = call(store.data_mut());
        let _ = result.send(Ok(output));
        Box::pin(std::future::ready(Ok(())))
    }
}

struct ShutdownStoreCall<F, R> {
    call: F,
    result: oneshot::Sender<wasmtime::Result<R>>,
    context: ReentryContext,
}

impl<F, R> ShutdownStoreCall<F, R>
where
    R: Send + 'static,
{
    async fn finish(
        context: ReentryContext,
        result: oneshot::Sender<wasmtime::Result<R>>,
        future: impl Future<Output = wasmtime::Result<R>>,
    ) -> wasmtime::Result<()> {
        match scope(context, future).await {
            Ok(value) => {
                let _ = result.send(Ok(value));
                Ok(())
            }
            Err(error) => {
                let message = error.to_string();
                let _ = result.send(Err(error));
                Err(wasmtime::Error::msg(message))
            }
        }
    }
}

impl<T, F, R> StoreJob<T> for ShutdownStoreCall<F, R>
where
    T: Send + 'static,
    F: for<'a> FnOnce(LegacyGuestScope<'a, T>) -> StoreFuture<'a, R> + Send + 'static,
    R: Send + 'static,
{
    fn run_concurrent(self: Box<Self>, accessor: &Accessor<T>) -> StoreFuture<'_, ()> {
        let Self {
            call,
            result,
            context,
        } = *self;
        let future = call(LegacyGuestScope::concurrent(
            accessor,
            Arc::new(GuestCallFailure::default()),
        ));
        Box::pin(Self::finish(context, result, future))
    }

    fn run_sequential(self: Box<Self>, store: StoreContextMut<'_, T>) -> StoreFuture<'_, ()> {
        let Self {
            call,
            result,
            context,
        } = *self;
        let future = call(LegacyGuestScope::sequential(
            store,
            Arc::new(GuestCallFailure::default()),
            false,
        ));
        Box::pin(Self::finish(context, result, future))
    }
}

struct GuestStoreCall<T, F, R> {
    call: F,
    result: oneshot::Sender<wasmtime::Result<R>>,
    context: ReentryContext,
    reentry: Arc<ReentryState<T>>,
    guest_call_failure: Arc<GuestCallFailure>,
    root_admission: Option<RootAdmissionGuard>,
}

#[derive(Default)]
struct GuestCallFailure {
    message: OnceLock<String>,
}

impl GuestCallFailure {
    fn record(&self, error: &wasmtime::Error) {
        let _ = self.message.set(format!("{error:#}"));
    }

    fn message(&self) -> Option<String> {
        self.message.get().cloned()
    }
}

impl<T, F, R> GuestStoreCall<T, F, R>
where
    T: Send + 'static,
    F: for<'a> FnOnce(LegacyGuestScope<'a, T>) -> StoreFuture<'a, R> + Send + 'static,
    R: Send + 'static,
{
    async fn execute<'a>(
        self,
        make_scope: impl FnOnce(Arc<GuestCallFailure>) -> LegacyGuestScope<'a, T>,
    ) -> wasmtime::Result<()> {
        let Self {
            call,
            result,
            context,
            reentry,
            guest_call_failure,
            root_admission,
        } = self;
        let failure_for_scope = Arc::clone(&guest_call_failure);
        let output = scope(context, async move {
            let _active_context = reentry.enter_active_context(context);
            call(make_scope(failure_for_scope)).await
        })
        .await;
        let guest_call_failure = guest_call_failure.message();
        drop(root_admission);
        if let Some(message) = guest_call_failure {
            let output = match output {
                Ok(_) => Err(wasmtime::Error::msg(message.clone())),
                Err(error) => Err(error),
            };
            let _ = result.send(output);
            Err(wasmtime::Error::msg(message))
        } else {
            let _ = result.send(output);
            Ok(())
        }
    }
}

impl<T, F, R> GuestStoreJob<T> for GuestStoreCall<T, F, R>
where
    T: Send + 'static,
    F: for<'a> FnOnce(LegacyGuestScope<'a, T>) -> StoreFuture<'a, R> + Send + 'static,
    R: Send + 'static,
{
    fn run_concurrent(self: Box<Self>, accessor: &Accessor<T>) -> StoreFuture<'_, ()> {
        Box::pin((*self).execute(move |failure| LegacyGuestScope::concurrent(accessor, failure)))
    }

    fn run_sequential(
        self: Box<Self>,
        store: StoreContextMut<'_, T>,
        reentrant: bool,
    ) -> StoreFuture<'_, ()> {
        Box::pin(
            (*self).execute(move |failure| LegacyGuestScope::sequential(store, failure, reentrant)),
        )
    }
}

enum LegacyGuestAccess<'a, T: 'static> {
    Concurrent(&'a Accessor<T>),
    /// The Store is driven one call at a time. `reentrant` is set when the call was routed into a
    /// host import that is already running on the Store.
    Sequential {
        store: StoreContextMut<'a, T>,
        #[cfg_attr(
            not(test),
            expect(dead_code, reason = "tests assert which path routed the call")
        )]
        reentrant: bool,
    },
}

/// Capability available while one synchronous legacy guest call is active.
///
/// Store data may only be borrowed by a synchronous closure. The underlying
/// Wasmtime Store context is never exposed outside this crate.
pub struct LegacyGuestScope<'a, T: 'static> {
    access: LegacyGuestAccess<'a, T>,
    guest_call_failure: Arc<GuestCallFailure>,
}

impl<'a, T> LegacyGuestScope<'a, T>
where
    T: Send + 'static,
{
    const fn concurrent(
        accessor: &'a Accessor<T>,
        guest_call_failure: Arc<GuestCallFailure>,
    ) -> Self {
        Self {
            access: LegacyGuestAccess::Concurrent(accessor),
            guest_call_failure,
        }
    }

    const fn sequential(
        store: StoreContextMut<'a, T>,
        guest_call_failure: Arc<GuestCallFailure>,
        reentrant: bool,
    ) -> Self {
        Self {
            access: LegacyGuestAccess::Sequential { store, reentrant },
            guest_call_failure,
        }
    }

    #[cfg(test)]
    pub(crate) const fn is_reentrant(&self) -> bool {
        matches!(
            self.access,
            LegacyGuestAccess::Sequential {
                reentrant: true,
                ..
            }
        )
    }

    #[cfg(test)]
    pub(crate) const fn accessor(&self) -> Option<&'a Accessor<T>> {
        match &self.access {
            LegacyGuestAccess::Concurrent(accessor) => Some(*accessor),
            LegacyGuestAccess::Sequential { .. } => None,
        }
    }

    pub fn with<R>(&mut self, call: impl for<'store> FnOnce(StoreDataMut<'store, T>) -> R) -> R {
        match &mut self.access {
            LegacyGuestAccess::Concurrent(accessor) => accessor.with(|mut access| {
                call(StoreDataMut {
                    data: access.data_mut(),
                })
            }),
            LegacyGuestAccess::Sequential { store, .. } => call(StoreDataMut {
                data: store.data_mut(),
            }),
        }
    }

    pub async fn call<Params, Return>(
        &mut self,
        function: TypedFunc<Params, Return>,
        params: Params,
    ) -> wasmtime::Result<Return>
    where
        Params: ComponentNamedList + Lower + 'static,
        Return: ComponentNamedList + Lift + 'static,
    {
        if let Some(message) = self.guest_call_failure.message() {
            return Err(wasmtime::Error::msg(message));
        }
        let result = match &mut self.access {
            LegacyGuestAccess::Concurrent(accessor) => {
                function.call_concurrent(&**accessor, params).await
            }
            LegacyGuestAccess::Sequential { store, .. } => {
                function.call_async(store.as_context_mut(), params).await
            }
        };
        if let Err(error) = &result {
            self.guest_call_failure.record(error);
        }
        result
    }

    /// Like [`Self::call`], for exports whose generated `TypedFunc` borrows its arguments and so
    /// cannot be moved into a `'static` job. Takes the untyped `Func` and re-types it here with
    /// owned arguments.
    pub async fn call_func<Params, Return>(
        &mut self,
        function: Func,
        params: Params,
    ) -> wasmtime::Result<Return>
    where
        Params: ComponentNamedList + Lower + 'static,
        Return: ComponentNamedList + Lift + 'static,
    {
        let function = match &self.access {
            LegacyGuestAccess::Concurrent(accessor) => {
                accessor.with(|access| function.typed::<Params, Return>(access))?
            }
            LegacyGuestAccess::Sequential { store, .. } => {
                function.typed::<Params, Return>(store.as_context())?
            }
        };
        self.call(function, params).await
    }
}

pub struct StoreDataMut<'a, T> {
    data: &'a mut T,
}

impl<T> StoreDataMut<'_, T> {
    pub const fn data_mut(&mut self) -> &mut T {
        self.data
    }
}

enum StoreMessage<T> {
    Call(Box<dyn StoreJob<T>>),
    GuestCall(Box<dyn GuestStoreJob<T>>),
    Shutdown {
        job: Box<dyn StoreJob<T>>,
        root_admission: Option<RootAdmissionGuard>,
    },
}

enum ControlMessage {
    Dropped,
    Discarded,
}

type ReentrySender<T> = mpsc::Sender<Box<dyn GuestStoreJob<T>>>;

struct ReentryScope<T> {
    id: u64,
    sender: ReentrySender<T>,
}

type ReentryScopes<T> = HashMap<(u64, u64), Vec<ReentryScope<T>>>;

struct ReentryState<T> {
    next_scope_id: AtomicU64,
    scopes: StdMutex<ReentryScopes<T>>,
    active_contexts: StdMutex<Vec<(u64, ReentryContext)>>,
}

impl<T> ReentryState<T> {
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
    ) -> ActiveContextRegistration<T> {
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

    fn has_active_context(&self, context: ReentryContext) -> bool {
        self.active_contexts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .any(|(_, active)| {
                active.admission_id == context.admission_id && active.chain_id == context.chain_id
            })
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
    ) -> (
        ReentryRegistration<T>,
        mpsc::Receiver<Box<dyn GuestStoreJob<T>>>,
    ) {
        let scope_id = self.next_scope_id.fetch_add(1, Ordering::Relaxed);
        let (sender, receiver) = mpsc::channel(REENTRY_QUEUE_CAPACITY);
        let mut scopes = self
            .scopes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        scopes
            .entry((context.admission_id, context.chain_id))
            .or_default()
            .push(ReentryScope {
                id: scope_id,
                sender,
            });
        (
            ReentryRegistration {
                state: Arc::clone(self),
                admission_id: context.admission_id,
                chain_id: context.chain_id,
                scope_id,
                active: true,
            },
            receiver,
        )
    }

    fn sender_for(&self, context: ReentryContext) -> Option<ReentrySender<T>> {
        let scopes = self
            .scopes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        scopes
            .get(&(context.admission_id, context.chain_id))
            .and_then(|scopes| scopes.last())
            .map(|scope| scope.sender.clone())
    }

    fn remove(&self, admission_id: u64, chain_id: u64, scope_id: u64) {
        let mut scopes = self
            .scopes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let chain = (admission_id, chain_id);
        let remove_chain = scopes.get_mut(&chain).is_some_and(|chain_scopes| {
            if let Some(index) = chain_scopes.iter().rposition(|scope| scope.id == scope_id) {
                chain_scopes.remove(index);
            }
            chain_scopes.is_empty()
        });
        if remove_chain {
            scopes.remove(&chain);
        }
    }
}

struct ActiveContextRegistration<T> {
    state: Arc<ReentryState<T>>,
    id: u64,
}

impl<T> Drop for ActiveContextRegistration<T> {
    fn drop(&mut self) {
        self.state.remove_active_context(self.id);
    }
}

struct ReentryRegistration<T> {
    state: Arc<ReentryState<T>>,
    admission_id: u64,
    chain_id: u64,
    scope_id: u64,
    active: bool,
}

impl<T> ReentryRegistration<T> {
    fn close(&mut self) {
        if self.active {
            self.state
                .remove(self.admission_id, self.chain_id, self.scope_id);
            self.active = false;
        }
    }
}

impl<T> Drop for ReentryRegistration<T> {
    fn drop(&mut self) {
        self.close();
    }
}

struct StoreShared<T, P> {
    sender: mpsc::Sender<StoreMessage<T>>,
    accepting: Arc<AtomicBool>,
    guest_call_failure: Arc<GuestCallFailure>,
    send_gate: Mutex<()>,
    lifecycle: Arc<Lifecycle>,
    reentry: Arc<ReentryState<T>>,
    policy: P,
    spawner: Arc<dyn RuntimeSpawner>,
    mode: StoreMode,
}

/// Cloneable submission and observation capability for one Wasmtime Store.
pub struct StoreHandle<T, P> {
    shared: Arc<StoreShared<T, P>>,
}

impl<T, P> Clone for StoreHandle<T, P> {
    fn clone(&self) -> Self {
        Self {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl<T, P> StoreHandle<T, P>
where
    T: Send + 'static,
    P: StorePolicy,
{
    #[must_use]
    pub fn state(&self) -> DriverState {
        self.shared.lifecycle.state()
    }

    #[must_use]
    pub fn driver_join(&self) -> DriverJoin {
        self.shared.lifecycle.subscribe()
    }

    /// Queues a short synchronous operation on the Store data. Once accepted,
    /// the operation runs even if the caller stops waiting for its result.
    ///
    /// The closure cannot await, so it cannot delay other plugin work beyond its own runtime. Keep
    /// it brief; anything that needs the guest or an async host operation belongs in
    /// [`StoreHandle::call_guest`].
    pub async fn call<R, F>(&self, call: F) -> wasmtime::Result<R>
    where
        F: FnOnce(&mut T) -> R + Send + 'static,
        R: Send + 'static,
    {
        let (result, receiver) = oneshot::channel();
        let send_guard = self.shared.send_gate.lock().await;
        self.ensure_accepting("Wasm plugin store is shutting down")?;
        let permit = self
            .shared
            .sender
            .reserve()
            .await
            .map_err(|_| self.terminal_error("Wasm plugin store driver is not running"))?;
        permit.send(StoreMessage::Call(Box::new(StoreCall { call, result })));
        drop(send_guard);

        self.receive_result(receiver, "Wasm plugin store call did not complete")
            .await
    }

    fn ensure_accepting(&self, message: &'static str) -> wasmtime::Result<()> {
        match self.state() {
            DriverState::Failed(error) => Err(wasmtime::Error::msg(format!(
                "Wasm plugin store driver failed: {error}"
            ))),
            DriverState::Stopped => Err(wasmtime::Error::msg(message)),
            _ if self.shared.guest_call_failure.message().is_some() => Err(wasmtime::Error::msg(
                "Wasm plugin store failed during a guest call",
            )),
            _ if self.shared.accepting.load(Ordering::Acquire) => Ok(()),
            _ => Err(wasmtime::Error::msg(message)),
        }
    }

    fn terminal_error(&self, fallback: &'static str) -> wasmtime::Error {
        match self.state() {
            DriverState::Failed(error) => {
                wasmtime::Error::msg(format!("Wasm plugin store driver failed: {error}"))
            }
            DriverState::Stopped => wasmtime::Error::msg("Wasm plugin store driver has stopped"),
            _ => wasmtime::Error::msg(fallback),
        }
    }

    async fn receive_result<R>(
        &self,
        receiver: oneshot::Receiver<wasmtime::Result<R>>,
        fallback: &'static str,
    ) -> wasmtime::Result<R> {
        if let Ok(result) = receiver.await {
            result
        } else {
            let mut join = self.driver_join();
            match join.wait().await {
                Ok(()) => Err(wasmtime::Error::msg(fallback)),
                Err(error) => Err(wasmtime::Error::msg(format!(
                    "Wasm plugin store driver failed: {error}"
                ))),
            }
        }
    }
}

impl<T> StoreHandle<T, LegacySyncReentry>
where
    T: Send + 'static,
{
    /// Queues a synchronous guest call, or routes it through the active host
    /// frame when this Store already appears in the current causal chain.
    pub async fn call_guest<R, F>(&self, call: F) -> wasmtime::Result<R>
    where
        F: for<'a> FnOnce(LegacyGuestScope<'a, T>) -> StoreFuture<'a, R> + Send + 'static,
        R: Send + 'static,
    {
        if let Some(message) = self.shared.guest_call_failure.message() {
            return Err(wasmtime::Error::msg(format!(
                "Wasm plugin store failed during a guest call: {message}"
            )));
        }
        let (result, receiver) = oneshot::channel();
        let inherited_context = self.shared.policy.inherited_context();
        let (context, root_admission) = if let Some(context) = inherited_context {
            tracing::trace!(
                wasm_plugin_admission_id = context.admission_id,
                wasm_plugin_chain_id = context.chain_id,
                wasm_plugin_reentry_depth = context.depth,
                "Inherited legacy Wasm plugin root admission"
            );
            (context, None)
        } else {
            self.ensure_accepting("Wasm plugin store is shutting down")?;
            let admission = self.shared.policy.acquire_root().await?;
            (admission.context(), Some(admission))
        };
        let mut job: Box<dyn GuestStoreJob<T>> = Box::new(GuestStoreCall {
            call,
            result,
            context,
            reentry: Arc::clone(&self.shared.reentry),
            guest_call_failure: Arc::clone(&self.shared.guest_call_failure),
            root_admission,
        });

        if let Some(context) = inherited_context
            && let Some(sender) = self.shared.reentry.sender_for(context)
        {
            match sender.send(job).await {
                Ok(()) => {
                    return self
                        .receive_result(receiver, "Wasm plugin guest call did not complete")
                        .await;
                }
                Err(error) => job = error.0,
            }
        }

        let send_guard = self.shared.send_gate.lock().await;
        self.ensure_accepting("Wasm plugin store is shutting down")?;
        let permit = self
            .shared
            .sender
            .reserve()
            .await
            .map_err(|_| self.terminal_error("Wasm plugin store driver is not running"))?;
        permit.send(StoreMessage::GuestCall(job));
        drop(send_guard);

        self.receive_result(receiver, "Wasm plugin guest call did not complete")
            .await
    }

    /// Awaits host work in the active guest call's chain with the reentry depth incremented
    ///
    /// For [`StoreMode::Concurrent`] stores. A sequential store runs one guest call at a time, so
    /// guest calls made from the awaited future would queue behind the active one; use
    /// [`StoreHandle::pump_reentry`] there instead
    ///
    /// Host imports are polled by Wasmtime's own event loop, outside the guest call's task-local
    /// scope, so the context has to come from the Store's active-context registry
    /// Without an active guest call, the future runs without adding a reentry context
    ///
    /// # Errors
    ///
    /// Returns an error before polling the future if the reentry depth limit is reached
    pub async fn in_active_context<R>(
        &self,
        future: impl Future<Output = R>,
    ) -> wasmtime::Result<R> {
        let context = self
            .shared
            .reentry
            .current_context()
            .map(ReentryContext::child)
            .transpose()?;
        Ok(match context {
            Some(context) => scope(context, future).await,
            None => future.await,
        })
    }

    /// Runs synchronous host work on Tokio's blocking pool in the active guest call's chain
    ///
    /// Increments the reentry depth before spawning so callbacks inherit the same chain and
    /// root admission on the blocking thread
    /// Without an active guest call, the operation runs without adding a reentry context
    ///
    /// # Errors
    ///
    /// Returns an error if the reentry depth limit is reached or the blocking task fails
    /// The operation is not spawned when the depth check fails
    pub async fn spawn_blocking_in_active_context<R, F>(&self, operation: F) -> wasmtime::Result<R>
    where
        R: Send + 'static,
        F: FnOnce() -> R + Send + 'static,
    {
        let context = self
            .shared
            .reentry
            .current_context()
            .map(ReentryContext::child)
            .transpose()?;
        tokio::task::spawn_blocking(move || match context {
            Some(context) => sync_scope(context, operation),
            None => operation(),
        })
        .await
        .map_err(|error| {
            wasmtime::Error::msg(format!(
                "Synchronous Wasm plugin host operation failed: {error}"
            ))
        })
    }

    /// Waits for an outbound host operation while servicing callbacks routed
    /// back into this Store from the active synchronous chain.
    pub async fn pump_reentry<R, S, F>(&self, store: &mut S, future: F) -> wasmtime::Result<R>
    where
        S: AsContextMut<Data = T> + Send,
        F: Future<Output = R> + Send,
    {
        self.ensure_sequential()?;
        let context = self.next_reentry_context()?;
        self.pump_reentry_with_context(store, context, future).await
    }

    /// Pumping only works when guest calls run one at a time. Checked before any work starts, so a
    /// rejected call never runs its operation.
    fn ensure_sequential(&self) -> wasmtime::Result<()> {
        if self.shared.mode == StoreMode::Concurrent {
            return Err(wasmtime::Error::msg(
                "Reentrant callbacks can only be pumped on a sequential Wasm plugin store",
            ));
        }
        Ok(())
    }

    fn next_reentry_context(&self) -> wasmtime::Result<ReentryContext> {
        self.shared
            .reentry
            .current_context()
            .or_else(|| self.shared.policy.inherited_context())
            .ok_or_else(|| {
                wasmtime::Error::msg("Wasm plugin reentry requires an active admitted causal chain")
            })?
            .child()
    }

    async fn pump_reentry_with_context<R, S, F>(
        &self,
        store: &mut S,
        context: ReentryContext,
        future: F,
    ) -> wasmtime::Result<R>
    where
        S: AsContextMut<Data = T> + Send,
        F: Future<Output = R> + Send,
    {
        let (mut registration, mut receiver) = self.shared.reentry.register(context);
        let scoped_future = scope(context, future);
        tokio::pin!(scoped_future);

        let output = loop {
            tokio::select! {
                Some(job) = receiver.recv() => {
                    job.run_sequential(store.as_context_mut(), true).await?;
                }
                output = &mut scoped_future => break output,
            }
        };

        registration.close();
        receiver.close();
        while let Ok(job) = receiver.try_recv() {
            job.run_sequential(store.as_context_mut(), true).await?;
        }
        Ok(output)
    }

    /// Runs synchronous host work through the injected blocking spawner while
    /// the active Store continues servicing reentrant callbacks.
    pub async fn pump_blocking<R, S, F>(&self, store: &mut S, operation: F) -> wasmtime::Result<R>
    where
        R: Send + 'static,
        S: AsContextMut<Data = T> + Send,
        F: FnOnce() -> R + Send + 'static,
    {
        self.ensure_sequential()?;
        let context = self.next_reentry_context()?;
        let (result, receiver) = oneshot::channel();
        self.shared
            .spawner
            .spawn_blocking(Box::new(move || {
                let output = catch_unwind(AssertUnwindSafe(|| sync_scope(context, operation)))
                    .map_err(|payload| panic_message(payload.as_ref()));
                let _ = result.send(output);
            }))
            .map_err(|error| {
                wasmtime::Error::msg(format!(
                    "Failed to spawn synchronous Wasm plugin host operation: {error}"
                ))
            })?;

        self.pump_reentry_with_context(store, context, receiver)
            .await?
            .map_err(|_| {
                wasmtime::Error::msg("Synchronous Wasm plugin host operation did not complete")
            })?
            .map_err(|message| {
                wasmtime::Error::msg(format!(
                    "Synchronous Wasm plugin host operation panicked: {message}"
                ))
            })
    }
}

/// Non-cloneable owner of one Store driver's lifecycle capability.
pub struct StoreExecutor<T: 'static, P>
where
    P: StorePolicy,
{
    handle: StoreHandle<T, P>,
    control: mpsc::UnboundedSender<ControlMessage>,
    terminal_admitted: AtomicBool,
}

impl<T, P> StoreExecutor<T, P>
where
    T: Send + 'static,
    P: StorePolicy,
{
    pub async fn start(
        store: Store<T>,
        policy: P,
        spawner: Arc<dyn RuntimeSpawner>,
        mode: StoreMode,
    ) -> wasmtime::Result<Self> {
        let (sender, receiver) = mpsc::channel(STORE_QUEUE_CAPACITY);
        let (control, control_receiver) = mpsc::unbounded_channel();
        let lifecycle = Lifecycle::new();
        let accepting = Arc::new(AtomicBool::new(true));
        let guest_call_failure = Arc::new(GuestCallFailure::default());
        let shared = Arc::new(StoreShared {
            sender,
            accepting: Arc::clone(&accepting),
            guest_call_failure,
            send_gate: Mutex::new(()),
            lifecycle: Arc::clone(&lifecycle),
            reentry: Arc::new(ReentryState::new()),
            policy,
            spawner: Arc::clone(&spawner),
            mode,
        });
        let (ready_sender, ready) = oneshot::channel();
        let policy_name = P::NAME;
        let task_guard = DriverTaskGuard::new(Arc::clone(&lifecycle), accepting);

        tracing::debug!(
            wasm_plugin_policy = policy_name,
            "Starting Wasm plugin store driver"
        );

        spawner
            .spawn(Box::pin(async move {
                let mut task_guard = task_guard;
                run_driver(
                    store,
                    receiver,
                    control_receiver,
                    Arc::clone(&task_guard.lifecycle),
                    ready_sender,
                    policy_name,
                    mode,
                )
                .await;
                task_guard.disarm();
            }))
            .map_err(|error| {
                let error = Arc::new(DriverError::new(format!(
                    "Failed to spawn Wasm plugin store driver: {error}"
                )));
                lifecycle.transition(DriverState::Failed(Arc::clone(&error)));
                wasmtime::Error::msg(error.to_string())
            })?;

        if ready.await.is_err() {
            let mut join = lifecycle.subscribe();
            return match join.wait().await {
                Ok(()) => Err(wasmtime::Error::msg(
                    "Wasm plugin store driver stopped before becoming ready",
                )),
                Err(error) => Err(wasmtime::Error::msg(format!(
                    "Wasm plugin store driver failed before becoming ready: {error}"
                ))),
            };
        }

        Ok(Self {
            handle: StoreHandle { shared },
            control,
            terminal_admitted: AtomicBool::new(false),
        })
    }

    #[must_use]
    pub fn handle(&self) -> StoreHandle<T, P> {
        self.handle.clone()
    }

    #[must_use]
    pub fn driver_join(&self) -> DriverJoin {
        self.handle.driver_join()
    }

    fn claim_terminal(&self) -> bool {
        self.terminal_admitted
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }

    /// Stops admission and drains accepted work without running a final Store
    /// operation.
    ///
    /// This is intended for a Store whose owner was prepared but never admitted
    /// to its higher-level lifecycle. Dropping an executor without either
    /// calling this method or completing [`Self::shutdown`] remains an error.
    pub fn discard(&self) {
        if !self.claim_terminal() {
            return;
        }

        self.handle.shared.accepting.store(false, Ordering::Release);

        let _ = self.control.send(ControlMessage::Discarded);
    }
}

impl<T> StoreExecutor<T, LegacySyncReentry>
where
    T: Send + 'static,
{
    /// Stops admission, drains accepted work, runs one final Store operation,
    /// drains Wasmtime tasks, and drops the Store.
    pub async fn shutdown<R, F>(&self, call: F) -> wasmtime::Result<R>
    where
        F: for<'a> FnOnce(LegacyGuestScope<'a, T>) -> StoreFuture<'a, R> + Send + 'static,
        R: Send + 'static,
    {
        let (result, receiver) = oneshot::channel();
        self.handle
            .ensure_accepting("Wasm plugin store is already shutting down")?;
        let inherited_context = self.handle.shared.policy.inherited_context();
        let (context, root_admission) = if let Some(context) = inherited_context {
            if self.handle.shared.reentry.has_active_context(context) {
                return Err(wasmtime::Error::msg(
                    "Cannot shut down a Wasm plugin store while it is active in the current causal chain",
                ));
            }
            tracing::trace!(
                wasm_plugin_admission_id = context.admission_id,
                wasm_plugin_chain_id = context.chain_id,
                wasm_plugin_reentry_depth = context.depth,
                "Inherited legacy Wasm plugin root admission"
            );
            (context, None)
        } else {
            let admission = self.handle.shared.policy.acquire_root().await?;
            (admission.context(), Some(admission))
        };
        let send_guard = self.handle.shared.send_gate.lock().await;
        self.handle
            .ensure_accepting("Wasm plugin store is already shutting down")?;
        let permit = self.handle.shared.sender.reserve().await.map_err(|_| {
            self.handle
                .terminal_error("Wasm plugin store driver is not running")
        })?;

        if !self.claim_terminal() {
            drop(permit);
            return Err(wasmtime::Error::msg(
                "Already shutting down wasm plugin store",
            ));
        }

        self.handle.shared.accepting.store(false, Ordering::Release);

        permit.send(StoreMessage::Shutdown {
            job: Box::new(ShutdownStoreCall {
                call,
                result,
                context,
            }),
            root_admission,
        });
        drop(send_guard);

        let result = receiver.await;
        let mut join = self.driver_join();
        let terminal = join.wait().await;
        reconcile_shutdown_result(result, terminal)
    }
}

pub fn reconcile_shutdown_result<R>(
    result: Result<wasmtime::Result<R>, oneshot::error::RecvError>,
    terminal: Result<(), Arc<DriverError>>,
) -> wasmtime::Result<R> {
    match (result, terminal) {
        (Ok(Err(error)), _) => Err(error),
        (Ok(Ok(value)), Ok(())) => Ok(value),
        (Ok(Ok(_)) | Err(_), Err(error)) => Err(wasmtime::Error::msg(format!(
            "Wasm plugin store driver failed: {error}"
        ))),
        (Err(_), Ok(())) => Err(wasmtime::Error::msg(
            "Wasm plugin store shutdown did not complete",
        )),
    }
}

impl<T, P> Deref for StoreExecutor<T, P>
where
    P: StorePolicy,
{
    type Target = StoreHandle<T, P>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

impl<T, P> Drop for StoreExecutor<T, P>
where
    P: StorePolicy,
{
    fn drop(&mut self) {
        if self.terminal_admitted.load(Ordering::Acquire)
            || !self.handle.shared.accepting.swap(false, Ordering::AcqRel)
        {
            return;
        }

        let _ = self.control.send(ControlMessage::Dropped);
    }
}

pub type LegacyStore<T> = StoreExecutor<T, LegacySyncReentry>;

fn control_result(message: Option<&ControlMessage>) -> wasmtime::Result<()> {
    match message {
        Some(ControlMessage::Dropped) => Err(wasmtime::Error::msg(
            "Wasm plugin store lifecycle control was dropped before shutdown",
        )),
        Some(ControlMessage::Discarded) => Ok(()),
        None => Err(wasmtime::Error::msg(
            "Wasm plugin store submission channel closed before shutdown",
        )),
    }
}

async fn run_driver<T>(
    store: Store<T>,
    receiver: mpsc::Receiver<StoreMessage<T>>,
    control: mpsc::UnboundedReceiver<ControlMessage>,
    lifecycle: Arc<Lifecycle>,
    ready: oneshot::Sender<()>,
    policy_name: &'static str,
    mode: StoreMode,
) where
    T: Send + 'static,
{
    match mode {
        StoreMode::Concurrent => {
            run_concurrent_driver(store, receiver, control, lifecycle, ready, policy_name).await;
        }
        StoreMode::Sequential => {
            run_sequential_driver(store, receiver, control, lifecycle, ready, policy_name).await;
        }
    }
}

async fn run_sequential_driver<T>(
    mut store: Store<T>,
    mut receiver: mpsc::Receiver<StoreMessage<T>>,
    mut control: mpsc::UnboundedReceiver<ControlMessage>,
    lifecycle: Arc<Lifecycle>,
    ready: oneshot::Sender<()>,
    policy_name: &'static str,
) where
    T: Send + 'static,
{
    lifecycle.transition(DriverState::Accepting);
    let _ = ready.send(());

    let result = drive_store(&mut store, &mut receiver, &mut control, &lifecycle).await;
    drop(store);

    match result {
        Ok(()) => lifecycle.transition(DriverState::Stopped),
        Err(error) => {
            let error = Arc::new(DriverError::new(format!("{error:#}")));
            tracing::error!(
                %error,
                wasm_plugin_policy = policy_name,
                "Wasm plugin store driver stopped"
            );
            lifecycle.transition(DriverState::Failed(error));
        }
    }
}

/// Outcome of handling one queued message.
enum Step {
    Continue,
    /// A shutdown job ran; the driver stops with its result.
    Finished(wasmtime::Result<()>),
}

async fn run_sequential_message<T>(
    store: &mut Store<T>,
    message: StoreMessage<T>,
    lifecycle: &Lifecycle,
) -> wasmtime::Result<Step>
where
    T: Send + 'static,
{
    match message {
        StoreMessage::Call(job) => job.run_sequential(store.as_context_mut()).await?,
        StoreMessage::GuestCall(job) => job.run_sequential(store.as_context_mut(), false).await?,
        StoreMessage::Shutdown {
            job,
            root_admission,
        } => {
            lifecycle.transition(DriverState::Stopping);
            let result = job.run_sequential(store.as_context_mut()).await;
            drop(root_admission);
            return Ok(Step::Finished(result));
        }
    }
    Ok(Step::Continue)
}

async fn drive_store<T>(
    store: &mut Store<T>,
    receiver: &mut mpsc::Receiver<StoreMessage<T>>,
    control: &mut mpsc::UnboundedReceiver<ControlMessage>,
    lifecycle: &Lifecycle,
) -> wasmtime::Result<()>
where
    T: Send + 'static,
{
    loop {
        let message = tokio::select! {
            biased;
            Some(control_message) = control.recv() => {
                lifecycle.transition(DriverState::Draining);
                receiver.close();
                while let Some(message) = receiver.recv().await {
                    if let Step::Finished(result) =
                        run_sequential_message(store, message, lifecycle).await?
                    {
                        return result;
                    }
                }
                lifecycle.transition(DriverState::Stopping);
                return control_result(Some(&control_message));
            }
            message = receiver.recv() => message,
        };
        let Some(message) = message else {
            lifecycle.transition(DriverState::Draining);
            // The owner may have closed the submission channel while its control signal is still pending
            let control_message = control.recv().await;
            lifecycle.transition(DriverState::Stopping);
            return control_result(control_message.as_ref());
        };

        if matches!(message, StoreMessage::Shutdown { .. }) {
            lifecycle.transition(DriverState::Draining);
            receiver.close();
        }
        if let Step::Finished(result) = run_sequential_message(store, message, lifecycle).await? {
            return result;
        }
    }
}

#[allow(clippy::too_many_lines)]
async fn run_concurrent_driver<T>(
    mut store: Store<T>,
    mut receiver: mpsc::Receiver<StoreMessage<T>>,
    mut control: mpsc::UnboundedReceiver<ControlMessage>,
    lifecycle: Arc<Lifecycle>,
    ready: oneshot::Sender<()>,
    policy_name: &'static str,
) where
    T: Send + 'static,
{
    let loop_lifecycle = Arc::clone(&lifecycle);
    let result = store
        .run_concurrent(async move |accessor| -> wasmtime::Result<()> {
            loop_lifecycle.transition(DriverState::Accepting);
            let _ = ready.send(());
            let mut active_calls = FuturesUnordered::new();

            loop {
                let message = tokio::select! {
                    biased;
                    Some(()) = active_calls.next(), if !active_calls.is_empty() => {
                        continue;
                    }
                    Some(control_message) = control.recv() => {
                        loop_lifecycle.transition(DriverState::Draining);
                        receiver.close();
                        loop {
                            let message = tokio::select! {
                                Some(()) = active_calls.next(), if !active_calls.is_empty() => {
                                    continue;
                                }
                                message = receiver.recv() => message,
                            };
                            let Some(message) = message else {
                                break;
                            };
                            match message {
                                StoreMessage::Call(job) => {
                                    job.run_concurrent(accessor).await?;
                                }
                                StoreMessage::GuestCall(job) => {
                                    active_calls.push(accessor.spawn(async move |accessor| job.run_concurrent(accessor).await)?);
                                }
                                StoreMessage::Shutdown {
                                    job,
                                    root_admission,
                                } => {
                                    while active_calls.next().await.is_some() {}
                                    loop_lifecycle.transition(DriverState::Stopping);
                                    let result = job.run_concurrent(accessor).await;
                                    poll_fn(|cx| accessor.poll_no_interesting_tasks(cx)).await;
                                    drop(root_admission);
                                    return result;
                                }
                            }
                        }
                        while active_calls.next().await.is_some() {}
                        loop_lifecycle.transition(DriverState::Stopping);
                        poll_fn(|cx| accessor.poll_no_interesting_tasks(cx)).await;
                        return control_result(Some(&control_message));
                    }
                    message = receiver.recv() => message,
                };
                let Some(message) = message else {
                    loop_lifecycle.transition(DriverState::Draining);
                    while active_calls.next().await.is_some() {}
                    // The owner may have closed the submission channel while its control signal is still pending
                    let control_message = control.recv().await;
                    loop_lifecycle.transition(DriverState::Stopping);
                    poll_fn(|cx| accessor.poll_no_interesting_tasks(cx)).await;
                    return control_result(control_message.as_ref());
                };

                match message {
                    StoreMessage::Call(job) => {
                        job.run_concurrent(accessor).await?;
                    }
                    StoreMessage::GuestCall(job) => {
                        active_calls.push(accessor.spawn(async move |accessor| job.run_concurrent(accessor).await)?);
                    }
                    StoreMessage::Shutdown {
                        job,
                        root_admission,
                    } => {
                        loop_lifecycle.transition(DriverState::Draining);
                        receiver.close();
                        while active_calls.next().await.is_some() {}
                        loop_lifecycle.transition(DriverState::Stopping);
                        let result = job.run_concurrent(accessor).await;
                        poll_fn(|cx| accessor.poll_no_interesting_tasks(cx)).await;
                        drop(root_admission);
                        return result;
                    }
                }
            }
        })
        .await;
    drop(store);

    match result {
        Ok(Ok(())) => lifecycle.transition(DriverState::Stopped),
        Ok(Err(error)) | Err(error) => {
            let error = Arc::new(DriverError::new(error.to_string()));
            tracing::error!(
                %error,
                wasm_plugin_policy = policy_name,
                "Wasm plugin store driver stopped"
            );
            lifecycle.transition(DriverState::Failed(error));
        }
    }
}

struct DriverTaskGuard {
    lifecycle: Arc<Lifecycle>,
    accepting: Arc<AtomicBool>,
    armed: bool,
}

impl DriverTaskGuard {
    const fn new(lifecycle: Arc<Lifecycle>, accepting: Arc<AtomicBool>) -> Self {
        Self {
            lifecycle,
            accepting,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.accepting.store(false, Ordering::Release);
        self.armed = false;
    }
}

impl Drop for DriverTaskGuard {
    fn drop(&mut self) {
        if self.armed {
            self.accepting.store(false, Ordering::Release);
            self.lifecycle
                .transition(DriverState::Failed(Arc::new(DriverError::new(
                    "Wasm plugin store driver task ended without reporting a terminal state",
                ))));
        }
    }
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    payload.downcast_ref::<&str>().map_or_else(
        || {
            payload
                .downcast_ref::<String>()
                .cloned()
                .unwrap_or_else(|| "unknown panic payload".to_owned())
        },
        |message| (*message).to_owned(),
    )
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::sync::Arc;

    use tokio::{
        sync::{mpsc, oneshot},
        time::{Duration, timeout},
    };
    use wasmtime::{Config, Engine, Store};

    use super::{ControlMessage, DriverState, Lifecycle, StoreMode, run_driver};

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn submission_close_waits_for_owner_control() {
        for mode in [StoreMode::Sequential, StoreMode::Concurrent] {
            submission_close_waits_for_owner_control_in(mode).await;
        }
    }

    async fn submission_close_waits_for_owner_control_in(mode: StoreMode) {
        let mut config = Config::new();
        config.wasm_component_model(true);
        if mode == StoreMode::Concurrent {
            config.wasm_component_model_async(true);
        }
        config.concurrency_support(mode == StoreMode::Concurrent);
        let engine = Engine::new(&config).expect("test engine");
        let store = Store::new(&engine, ());
        let (sender, receiver) = mpsc::channel(1);
        let (control_sender, control_receiver) = mpsc::unbounded_channel();
        let lifecycle = Lifecycle::new();
        let (ready_sender, ready) = oneshot::channel();
        let driver = tokio::spawn(run_driver(
            store,
            receiver,
            control_receiver,
            Arc::clone(&lifecycle),
            ready_sender,
            "test",
            mode,
        ));

        timeout(Duration::from_secs(10), async {
            ready.await.expect("driver should become ready");
            drop(sender);
            while lifecycle.state() != DriverState::Draining {
                tokio::task::yield_now().await;
            }
            assert!(control_sender.send(ControlMessage::Dropped).is_ok());
            driver.await.expect("driver task should finish");
        })
        .await
        .expect("owner control should stop the driver");

        let mut join = lifecycle.subscribe();
        let error = join
            .wait()
            .await
            .expect_err("owner drop should fail the driver");
        assert!(error.to_string().contains("control was dropped"));
    }
}
