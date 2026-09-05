use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_1::pumpkin::{
        self,
        plugin::ipc::{IpcMessage, PluginId},
    },
};
use wasmtime::component::{Access, HasSelf};

impl pumpkin::plugin::ipc::Host for PluginHostState {}

impl pumpkin::plugin::ipc::HostWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn send_ipc_message(
        mut host: Access<'_, PluginHostState, Self>,
        recipient: PluginId,
        message: IpcMessage,
    ) -> wasmtime::Result<Result<Result<IpcMessage, String>, ()>> {
        let (server, name, plugin) = {
            let state = host.get();
            let server = state
                .server
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            let name = state
                .name
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("Plugin name not available"))?;
            let plugin = state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin instance not available"))?;
            (server, name, plugin)
        };

        let outbound = server
            .plugin_manager
            .send_message(&name, &recipient, &message);
        plugin.store.pump_reentry(&mut host, outbound).await
    }
}
