use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1::pumpkin::{
        self,
        plugin::ipc::{IpcMessage, PluginId},
    },
};

impl pumpkin::plugin::ipc::Host for PluginHostState {
    async fn send_ipc_message(
        &mut self,
        recipient: PluginId,
        message: IpcMessage,
    ) -> wasmtime::Result<Result<Result<IpcMessage, String>, ()>> {
        let Some(server) = &self.server else {
            return Err(wasmtime::Error::msg("Server not available"));
        };
        let Some(name) = &self.name else {
            return Err(wasmtime::Error::msg("Plugin name not available"));
        };
        Ok(server
            .plugin_manager
            .send_message(name, &recipient, &message)
            .await)
    }
}
