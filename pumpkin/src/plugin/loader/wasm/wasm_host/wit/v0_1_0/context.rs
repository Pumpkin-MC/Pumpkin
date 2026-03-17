use std::{collections::HashMap, sync::Arc};
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    DowncastResourceExt,
    state::{CommandResource, ContextResource, PluginHostState},
    wit::v0_1_0::{
        events::{ToFromV0_1_0WasmEvent, WasmPluginV0_1_0EventHandler},
        pumpkin::{
            self,
            plugin::{
                command::Command,
                context::Context,
                event::{EventPriority, EventType},
                permission::{Permission, PermissionDefault, PermissionLevel},
                server::Server,
            },
        },
    },
};

macro_rules! register_host_event {
    ($resource:expr, $handler:expr, $priority:expr, $blocking:expr, $event_ty:ty) => {
        $resource
            .provider
            .register_event::<$event_ty, _>(Arc::clone($handler), $priority, $blocking)
            .await
    };
}

async fn register_typed_event<E: crate::plugin::Payload + ToFromV0_1_0WasmEvent + 'static>(
    resource: &ContextResource,
    handler: &Arc<WasmPluginV0_1_0EventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
) {
    register_host_event!(resource, handler, priority, blocking, E);
}

async fn register_player_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginV0_1_0EventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::player::{
        changed_main_hand::PlayerChangedMainHandEvent, exp_change::PlayerExpChangeEvent,
        fish::PlayerFishEvent, item_held::PlayerItemHeldEvent, player_chat::PlayerChatEvent,
        player_command_send::PlayerCommandSendEvent,
        player_custom_payload::PlayerCustomPayloadEvent,
        player_gamemode_change::PlayerGamemodeChangeEvent, player_join::PlayerJoinEvent,
        player_leave::PlayerLeaveEvent, player_login::PlayerLoginEvent,
        player_move::PlayerMoveEvent, player_permission_check::PlayerPermissionCheckEvent,
        player_teleport::PlayerTeleportEvent,
    };

    match event_type {
        EventType::PlayerJoinEvent => {
            register_typed_event::<PlayerJoinEvent>(resource, handler, priority, blocking).await;
        }
        EventType::PlayerLeaveEvent => {
            register_typed_event::<PlayerLeaveEvent>(resource, handler, priority, blocking).await;
        }
        EventType::PlayerLoginEvent => {
            register_typed_event::<PlayerLoginEvent>(resource, handler, priority, blocking).await;
        }
        EventType::PlayerChatEvent => {
            register_typed_event::<PlayerChatEvent>(resource, handler, priority, blocking).await;
        }
        EventType::PlayerCommandSendEvent => {
            register_typed_event::<PlayerCommandSendEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerPermissionCheckEvent => {
            register_typed_event::<PlayerPermissionCheckEvent>(
                resource, handler, priority, blocking,
            )
            .await;
        }
        EventType::PlayerMoveEvent => {
            register_typed_event::<PlayerMoveEvent>(resource, handler, priority, blocking).await;
        }
        EventType::PlayerTeleportEvent => {
            register_typed_event::<PlayerTeleportEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerExpChangeEvent => {
            register_typed_event::<PlayerExpChangeEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerItemHeldEvent => {
            register_typed_event::<PlayerItemHeldEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerChangedMainHandEvent => {
            register_typed_event::<PlayerChangedMainHandEvent>(
                resource, handler, priority, blocking,
            )
            .await;
        }
        EventType::PlayerGamemodeChangeEvent => {
            register_typed_event::<PlayerGamemodeChangeEvent>(
                resource, handler, priority, blocking,
            )
            .await;
        }
        EventType::PlayerCustomPayloadEvent => {
            register_typed_event::<PlayerCustomPayloadEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerFishEvent => {
            register_typed_event::<PlayerFishEvent>(resource, handler, priority, blocking).await;
        }
        _ => unreachable!("non-player event should not be routed to register_player_event"),
    }
}

async fn register_server_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginV0_1_0EventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::server::{
        server_broadcast::ServerBroadcastEvent, server_command::ServerCommandEvent,
    };

    match event_type {
        EventType::ServerCommandEvent => {
            register_typed_event::<ServerCommandEvent>(resource, handler, priority, blocking).await;
        }
        EventType::ServerBroadcastEvent => {
            register_typed_event::<ServerBroadcastEvent>(resource, handler, priority, blocking)
                .await;
        }
        _ => unreachable!("non-server event should not be routed to register_server_event"),
    }
}

impl DowncastResourceExt<ContextResource> for Resource<Context> {
    fn downcast_ref<'a>(&'a self, state: &'a mut PluginHostState) -> &'a ContextResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid context resource handle")
            .downcast_ref()
            .expect("resource type mismatch")
    }

    fn downcast_mut<'a>(&'a self, state: &'a mut PluginHostState) -> &'a mut ContextResource {
        state
            .resource_table
            .get_any_mut(self.rep())
            .expect("invalid context resource handle")
            .downcast_mut()
            .expect("resource type mismatch")
    }

    fn consume(self, state: &mut PluginHostState) -> ContextResource {
        state
            .resource_table
            .delete(Resource::new_own(self.rep()))
            .expect("invalid context resource handle")
    }
}

impl pumpkin::plugin::context::Host for PluginHostState {}

impl pumpkin::plugin::context::HostContext for PluginHostState {
    async fn register_event(
        &mut self,
        context: Resource<Context>,
        handler_id: u32,
        event_type: EventType,
        event_priority: EventPriority,
        blocking: bool,
    ) {
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

        let resource = context.downcast_ref(self);
        let handler = Arc::new(WasmPluginV0_1_0EventHandler { handler_id, plugin });

        match event_type {
            event_type @ (EventType::ServerCommandEvent | EventType::ServerBroadcastEvent) => {
                register_server_event(resource, &handler, priority, blocking, event_type).await;
            }
            event_type => {
                register_player_event(resource, &handler, priority, blocking, event_type).await;
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

    async fn register_permission(
        &mut self,
        context: Resource<Context>,
        permission: Permission,
    ) -> Result<(), String> {
        let mut children: HashMap<String, bool> = HashMap::with_capacity(permission.children.len());
        for child in permission.children {
            children.insert(child.node, child.value);
        }

        let permission = pumpkin_util::permission::Permission {
            node: permission.node,
            description: permission.description,
            default: match permission.default {
                PermissionDefault::Deny => pumpkin_util::permission::PermissionDefault::Deny,
                PermissionDefault::Allow => pumpkin_util::permission::PermissionDefault::Allow,
                PermissionDefault::Op(permission_level) => {
                    pumpkin_util::permission::PermissionDefault::Op(match permission_level {
                        PermissionLevel::Zero => pumpkin_util::permission::PermissionLvl::Zero,
                        PermissionLevel::One => pumpkin_util::permission::PermissionLvl::One,
                        PermissionLevel::Two => pumpkin_util::permission::PermissionLvl::Two,
                        PermissionLevel::Three => pumpkin_util::permission::PermissionLvl::Three,
                        PermissionLevel::Four => pumpkin_util::permission::PermissionLvl::Four,
                    })
                }
            },
            children,
        };

        context
            .downcast_mut(self)
            .provider
            .register_permission(permission)
            .await
    }

    async fn get_data_folder(&mut self, context: Resource<Context>) -> String {
        context
            .downcast_ref(self)
            .provider
            .get_data_folder()
            .to_string_lossy()
            .into_owned()
    }

    async fn get_server(&mut self, context: Resource<Context>) -> Resource<Server> {
        let server_provider = context.downcast_ref(self).provider.server.clone();
        self.add_server(server_provider)
            .expect("failed to add server resource")
    }

    async fn drop(&mut self, rep: Resource<Context>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ContextResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
