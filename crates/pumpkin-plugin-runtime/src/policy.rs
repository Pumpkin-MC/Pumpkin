use std::future::Future;

use crate::chain::{ReentryContext, RootAdmission, RootAdmissionGuard, scope};

mod sealed {
    pub trait StorePolicy {}
}

#[doc(hidden)]
pub trait StorePolicy: sealed::StorePolicy + Clone + Send + Sync + 'static {
    const NAME: &'static str;
}

/// Serializes synchronous guest roots while allowing callbacks from the active
/// causal chain to re-enter stores already on that chain.
#[derive(Clone, Debug)]
pub struct LegacySyncReentry {
    root_admission: RootAdmission,
}

impl LegacySyncReentry {
    pub const NAME: &'static str = "LegacySyncReentry";

    /// Creates one admission authority. Clone and share this policy across all
    /// Stores that can participate in the same synchronous plugin call graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root_admission: RootAdmission::new(),
        }
    }

    #[must_use]
    pub(crate) fn inherited_context(&self) -> Option<ReentryContext> {
        ReentryContext::current().filter(|context| self.root_admission.admits(*context))
    }

    pub(crate) async fn acquire_root(&self) -> wasmtime::Result<RootAdmissionGuard> {
        self.root_admission.acquire_root().await
    }

    #[cfg(test)]
    #[must_use]
    pub(crate) fn root_context(&self) -> ReentryContext {
        self.root_admission.root_context()
    }

    /// Runs Store bootstrap work under the same admission authority used by
    /// subsequent guest calls.
    pub async fn scope_bootstrap<T>(
        &self,
        future: impl Future<Output = wasmtime::Result<T>>,
    ) -> wasmtime::Result<T> {
        if let Some(context) = self.inherited_context() {
            tracing::trace!(
                wasm_plugin_admission_id = context.admission_id,
                wasm_plugin_chain_id = context.chain_id,
                wasm_plugin_reentry_depth = context.depth,
                "Inherited legacy Wasm plugin root admission"
            );
            return scope(context, future).await;
        }

        let admission = self.acquire_root().await?;
        let context = admission.context();
        let output = scope(context, future).await;
        drop(admission);
        output
    }
}

impl Default for LegacySyncReentry {
    fn default() -> Self {
        Self::new()
    }
}

impl sealed::StorePolicy for LegacySyncReentry {}

impl StorePolicy for LegacySyncReentry {
    const NAME: &'static str = Self::NAME;
}
