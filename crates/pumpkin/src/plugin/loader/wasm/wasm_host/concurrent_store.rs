use std::sync::Arc;

use pumpkin_plugin_runtime::{RuntimeSpawner, SpawnError, SpawnFuture, StoreExecutor};
use wasmtime::Store;

use super::state::PluginHostState;

pub(crate) use pumpkin_plugin_runtime::LegacySyncReentry;

pub(crate) type LegacyStore = StoreExecutor<PluginHostState, LegacySyncReentry>;

pub(crate) struct TokioSpawner {
    runtime: tokio::runtime::Handle,
}

impl TokioSpawner {
    pub(crate) const fn new(runtime: tokio::runtime::Handle) -> Self {
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

pub(crate) async fn start_legacy_store(
    store: Store<PluginHostState>,
    policy: LegacySyncReentry,
    spawner: Arc<dyn RuntimeSpawner>,
) -> wasmtime::Result<LegacyStore> {
    StoreExecutor::start(store, policy, spawner).await
}
