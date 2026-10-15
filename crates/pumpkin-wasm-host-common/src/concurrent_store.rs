use std::sync::Arc;

use pumpkin_plugin_runtime::{RuntimeSpawner, SpawnError, SpawnFuture, StoreExecutor};
use wasmtime::Store;

use super::state::PluginHostState;

pub use pumpkin_plugin_runtime::LegacySyncReentry;

pub type LegacyStore = StoreExecutor<PluginHostState, LegacySyncReentry>;

pub struct TokioSpawner {
    runtime: tokio::runtime::Handle,
}

impl TokioSpawner {
    #[must_use]
    pub const fn new(runtime: tokio::runtime::Handle) -> Self {
        Self { runtime }
    }
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

pub async fn start_legacy_store(
    store: Store<PluginHostState>,
    policy: LegacySyncReentry,
    spawner: Arc<dyn RuntimeSpawner>,
) -> wasmtime::Result<LegacyStore> {
    StoreExecutor::start(store, policy, spawner).await
}
