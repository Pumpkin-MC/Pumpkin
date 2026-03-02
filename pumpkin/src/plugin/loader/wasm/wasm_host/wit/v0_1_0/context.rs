use std::sync::Arc;

use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    state::{CommandResource, ContextResource, PluginHostState},
    wit::v0_1_0::{
        events::WasmPluginV0_1_0EventHandler,
        pumpkin::{
            self,
            plugin::{
                command::Command,
                context::Context,
                event::{EventPriority, EventType},
                server::Server,
            },
        },
    },
};

impl pumpkin::plugin::context::Host for PluginHostState {}

impl pumpkin::plugin::context::HostContext for PluginHostState {
    async fn drop(&mut self, rep: Resource<Context>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ContextResource>(Resource::new_own(rep.rep()));
        Ok(())
    }

    async fn get_server(&mut self, context: Resource<Context>) -> Resource<Server> {
        let resource = self
            .resource_table
            .get_any_mut(context.rep())
            .expect("invalid context resource handle")
            .downcast_ref::<ContextResource>()
            .expect("resource type mismatch");
        let server_provider = resource.provider.server.clone();
        self.add_server(server_provider)
            .expect("failed to add server resource")
    }

    async fn register_event(
        &mut self,
        context: Resource<Context>,
        handler_id: u32,
        event_type: EventType,
        event_priority: EventPriority,
        blocking: bool,
    ) {
        let resource = self
            .resource_table
            .get_any_mut(context.rep())
            .expect("invalid context resource handle")
            .downcast_ref::<ContextResource>()
            .expect("resource type mismatch");

        let priority = match event_priority {
            EventPriority::Highest => crate::plugin::EventPriority::Highest,
            EventPriority::High => crate::plugin::EventPriority::High,
            EventPriority::Normal => crate::plugin::EventPriority::Normal,
            EventPriority::Low => crate::plugin::EventPriority::Low,
            EventPriority::Lowest => crate::plugin::EventPriority::Lowest,
        };

        let plugin = self
            .plugin
            .as_ref()
            .expect("plugin should always be initialized here")
            .upgrade()
            .expect("plugin has been dropped");

        let handler = Arc::new(WasmPluginV0_1_0EventHandler { handler_id, plugin });

        match event_type {
            EventType::PlayerJoinEvent => {
                resource
                    .provider
                    .register_event::<crate::plugin::player::player_join::PlayerJoinEvent, _>(
                        handler, priority, blocking,
                    )
                    .await;
            }
        }
    }

    async fn register_command(
        &mut self,
        context: Resource<Context>,
        command: Resource<Command>,
        permission: String,
    ) {
        let command = self
            .resource_table
            .delete::<CommandResource>(Resource::new_own(command.rep()))
            .expect("invalid command resource handle")
            .provider;

        let context_resource = self
            .resource_table
            .get_any_mut(context.rep())
            .expect("invalid context resource handle")
            .downcast_ref::<ContextResource>()
            .expect("resource type mismatch");

        context_resource
            .provider
            .register_command(command, permission)
            .await;
    }
}
