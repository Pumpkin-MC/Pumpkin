use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState,
    wit::v0_2::pumpkin::{
        self,
        plugin::ipc::{IpcMessage, PluginId},
    },
};
use wasmtime::component::Accessor;
use wasmtime::component::HasSelf;

impl pumpkin::plugin::ipc::Host for PluginHostState {}

impl pumpkin::plugin::ipc::HostWithStore<PluginHostState> for HasSelf<PluginHostState> {
    async fn send_ipc_message(
        accessor: &Accessor<PluginHostState, Self>,
        recipient: PluginId,
        message: IpcMessage,
    ) -> wasmtime::Result<Result<Result<IpcMessage, String>, ()>> {
        let (server, name) = accessor.with(|mut host| -> wasmtime::Result<_> {
            let state = host.get();
            let server = state
                .server
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("Server not available"))?;
            let name = state
                .name
                .clone()
                .ok_or_else(|| wasmtime::Error::msg("Plugin name not available"))?;
            state
                .plugin
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
                .ok_or_else(|| wasmtime::Error::msg("Plugin instance not available"))?;
            Ok((server, name))
        })?;

        let outbound = server
            .plugin_manager
            .send_message(&name, &recipient, &message);
        Ok(outbound.await)
    }
}
