use crate::plugin::loader::wasm::wasm_host::{
    logging::log_tracing, state::PluginHostState, wit::v0_1::pumpkin,
};
use pumpkin_util::translation::localized_log_format;

impl pumpkin::plugin::logging::Host for PluginHostState {
    async fn log(
        &mut self,
        level: pumpkin::plugin::logging::Level,
        message: String,
    ) -> wasmtime::Result<()> {
        match level {
            pumpkin::plugin::logging::Level::Trace => tracing::trace!(
                "{}",
                localized_log_format("server.log.plugin_wasm_log", &[message])
            ),
            pumpkin::plugin::logging::Level::Debug => tracing::debug!(
                "{}",
                localized_log_format("server.log.plugin_wasm_log", &[message])
            ),
            pumpkin::plugin::logging::Level::Info => tracing::info!(
                "{}",
                localized_log_format("server.log.plugin_wasm_log", &[message])
            ),
            pumpkin::plugin::logging::Level::Warn => tracing::warn!(
                "{}",
                localized_log_format("server.log.plugin_wasm_log", &[message])
            ),
            pumpkin::plugin::logging::Level::Error => tracing::error!(
                "{}",
                localized_log_format("server.log.plugin_wasm_log", &[message])
            ),
        }
        Ok(())
    }

    async fn log_tracing(&mut self, event: Vec<u8>) -> wasmtime::Result<()> {
        log_tracing(event).await;
        Ok(())
    }
}
