use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub const MAX_SYNC_REENTRY_DEPTH: usize = 64;

static NEXT_ROOT_ADMISSION_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug)]
pub struct RootAdmission {
    state: Arc<RootAdmissionState>,
}

#[derive(Debug)]
struct RootAdmissionState {
    id: u64,
    next_chain_id: AtomicU64,
    gate: Arc<Semaphore>,
}

pub struct RootAdmissionGuard {
    context: ReentryContext,
    acquired_at: Instant,
    permit: Option<OwnedSemaphorePermit>,
}

impl RootAdmission {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RootAdmissionState {
                id: NEXT_ROOT_ADMISSION_ID.fetch_add(1, Ordering::Relaxed),
                next_chain_id: AtomicU64::new(1),
                gate: Arc::new(Semaphore::new(1)),
            }),
        }
    }

    pub fn root_context(&self) -> ReentryContext {
        ReentryContext {
            admission_id: self.state.id,
            chain_id: self.state.next_chain_id.fetch_add(1, Ordering::Relaxed),
            depth: 0,
        }
    }

    pub fn admits(&self, context: ReentryContext) -> bool {
        context.admission_id == self.state.id
    }

    pub async fn acquire_root(&self) -> wasmtime::Result<RootAdmissionGuard> {
        let context = self.root_context();
        let waiting_since = Instant::now();
        tracing::trace!(
            wasm_plugin_admission_id = context.admission_id,
            wasm_plugin_chain_id = context.chain_id,
            "Waiting for legacy Wasm plugin root admission"
        );

        let permit = Arc::clone(&self.state.gate)
            .acquire_owned()
            .await
            .map_err(|_| wasmtime::Error::msg("Wasm plugin root admission is closed"))?;
        let acquired_at = Instant::now();
        tracing::trace!(
            wasm_plugin_admission_id = context.admission_id,
            wasm_plugin_chain_id = context.chain_id,
            wasm_plugin_admission_wait_micros =
                acquired_at.duration_since(waiting_since).as_micros(),
            "Acquired legacy Wasm plugin root admission"
        );

        Ok(RootAdmissionGuard {
            context,
            acquired_at,
            permit: Some(permit),
        })
    }
}

impl RootAdmissionGuard {
    pub const fn context(&self) -> ReentryContext {
        self.context
    }
}

impl Drop for RootAdmissionGuard {
    fn drop(&mut self) {
        tracing::trace!(
            wasm_plugin_admission_id = self.context.admission_id,
            wasm_plugin_chain_id = self.context.chain_id,
            wasm_plugin_admission_held_micros = self.acquired_at.elapsed().as_micros(),
            "Releasing legacy Wasm plugin root admission"
        );
        drop(self.permit.take());
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ReentryContext {
    pub admission_id: u64,
    pub chain_id: u64,
    pub depth: usize,
}

impl ReentryContext {
    pub fn current() -> Option<Self> {
        REENTRY_CONTEXT.try_with(|context| *context).ok()
    }

    pub fn child(self) -> wasmtime::Result<Self> {
        if self.depth >= MAX_SYNC_REENTRY_DEPTH {
            return Err(wasmtime::Error::msg(format!(
                "Wasm plugin synchronous reentry exceeded the maximum depth of {MAX_SYNC_REENTRY_DEPTH}"
            )));
        }

        Ok(Self {
            admission_id: self.admission_id,
            chain_id: self.chain_id,
            depth: self.depth + 1,
        })
    }
}

tokio::task_local! {
    static REENTRY_CONTEXT: ReentryContext;
}

pub async fn scope<T>(context: ReentryContext, future: impl Future<Output = T>) -> T {
    REENTRY_CONTEXT.scope(context, future).await
}

pub fn sync_scope<T>(context: ReentryContext, operation: impl FnOnce() -> T) -> T {
    REENTRY_CONTEXT.sync_scope(context, operation)
}
