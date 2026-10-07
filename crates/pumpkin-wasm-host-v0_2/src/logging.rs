use crate::pumpkin;
use pumpkin_wasm_host_common::{
    logging::{format_plugin_message, log_tracing},
    state::PluginHostState,
};

impl pumpkin::plugin::logging::Host for PluginHostState {
    fn log(
        &mut self,
        level: pumpkin::plugin::logging::Level,
        message: String,
    ) -> wasmtime::Result<()> {
        let message = format_plugin_message(self, &message);
        match level {
            pumpkin::plugin::logging::Level::Trace => tracing::trace!("{message}"),
            pumpkin::plugin::logging::Level::Debug => tracing::debug!("{message}"),
            pumpkin::plugin::logging::Level::Info => tracing::info!("{message}"),
            pumpkin::plugin::logging::Level::Warn => tracing::warn!("{message}"),
            pumpkin::plugin::logging::Level::Error => tracing::error!("{message}"),
        }
        Ok(())
    }

    fn log_tracing(&mut self, event: Vec<u8>) -> wasmtime::Result<()> {
        log_tracing(&event);
        Ok(())
    }
}
