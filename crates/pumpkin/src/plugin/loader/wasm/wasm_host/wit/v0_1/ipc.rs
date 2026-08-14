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
        Ok(self
            .server
            .as_ref()
            .unwrap()
            .plugin_manager
            .send_message(self.name.as_ref().unwrap(), &recipient, &message)
            .await)
    }
}
