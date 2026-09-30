use crate::plugin::loader::wasm::wasm_host::{
    logging::log_tracing, state::PluginHostState, wit::v0_2::pumpkin,
};
use wasmtime::component::Accessor;
use wasmtime::component::HasSelf;

impl pumpkin::plugin::logging::Host for PluginHostState {}

impl pumpkin::plugin::logging::HostWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn log(
        _accessor: &Accessor<PluginHostState, Self>,
        level: pumpkin::plugin::logging::Level,
        message: String,
    ) -> wasmtime::Result<()> {
        match level {
            pumpkin::plugin::logging::Level::Trace => tracing::trace!("[plugin] {message}"),
            pumpkin::plugin::logging::Level::Debug => tracing::debug!("[plugin] {message}"),
            pumpkin::plugin::logging::Level::Info => tracing::info!("[plugin] {message}"),
            pumpkin::plugin::logging::Level::Warn => tracing::warn!("[plugin] {message}"),
            pumpkin::plugin::logging::Level::Error => tracing::error!("[plugin] {message}"),
        }
        Ok(())
    }

    async fn log_tracing(
        _accessor: &Accessor<PluginHostState, Self>,
        event: Vec<u8>,
    ) -> wasmtime::Result<()> {
        log_tracing(event).await;
        Ok(())
    }
}
