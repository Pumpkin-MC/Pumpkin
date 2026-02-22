use std::sync::Arc;

use crate::{
    plugin::{
        BoxFuture, EventHandler, Payload,
        loader::wasm::wasm_host::{PluginInstance, WasmPlugin, state::PluginHostState, wit},
    },
    server::Server,
};

pub mod player;

pub struct WasmPluginV0_1_0EventHandler {
    pub handler_id: u32,
    pub plugin: Arc<WasmPlugin>,
}

pub trait IntoV0_1_0WasmEvent {
    fn into_v0_1_0_wasm_event(
        &self,
        state: &mut PluginHostState,
    ) -> wit::v0_1_0::pumpkin::plugin::event::Event;
}

impl<E: Payload + IntoV0_1_0WasmEvent> EventHandler<E> for WasmPluginV0_1_0EventHandler {
    fn handle<'a>(&'a self, server: &'a Arc<Server>, event: &'a E) -> BoxFuture<'a, ()> {
        Box::pin(async {
            let mut store = self.plugin.store.lock().await;
            let event = event.into_v0_1_0_wasm_event(store.data_mut());
            match self.plugin.plugin_instance {
                PluginInstance::V0_1_0(ref plugin) => {
                    let server = store.data_mut().add_server(server.clone()).unwrap();
                    plugin
                        .call_handle_event(&mut *store, self.handler_id, server, &event)
                        .await
                        .unwrap();
                }
            }
        })
    }

    fn handle_blocking<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut E,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {
            let mut store = self.plugin.store.lock().await;
            let event = event.into_v0_1_0_wasm_event(store.data_mut());
            match self.plugin.plugin_instance {
                PluginInstance::V0_1_0(ref plugin) => {
                    let server = store.data_mut().add_server(server.clone()).unwrap();
                    plugin
                        .call_handle_event(&mut *store, self.handler_id, server, &event)
                        .await
                        .unwrap();
                }
            }
        })
    }
}
