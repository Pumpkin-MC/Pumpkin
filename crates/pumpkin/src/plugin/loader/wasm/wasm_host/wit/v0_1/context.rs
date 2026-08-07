use std::{collections::HashMap, sync::Arc};
use wasmtime::component::Resource;

use crate::plugin::loader::wasm::wasm_host::{
    DowncastResourceExt,
    state::{CommandResource, ContextResource, PluginHostState},
    wit::v0_1::{
        events::{ToFromWasmEvent, WasmPluginEventHandler},
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

async fn register_typed_event<E: crate::plugin::Payload + ToFromWasmEvent + 'static>(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
) {
    register_host_event!(resource, handler, priority, blocking, E);
}

#[expect(clippy::too_many_lines)]
async fn register_player_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::player::{
        bedrock_form_response::BedrockFormResponseEvent,
        changed_main_hand::PlayerChangedMainHandEvent, custom_click_action::CustomClickActionEvent,
        egg_throw::PlayerEggThrowEvent, exp_change::PlayerExpChangeEvent, fish::PlayerFishEvent,
        inventory_close::InventoryCloseEvent, inventory_interact::InventoryClickEvent,
        item_held::PlayerItemHeldEvent, player_change_world::PlayerChangeWorldEvent,
        player_chat::PlayerChatEvent, player_command_send::PlayerCommandSendEvent,
        player_custom_payload::PlayerCustomPayloadEvent,
        player_gamemode_change::PlayerGamemodeChangeEvent,
        player_interact_entity_event::PlayerInteractEntityEvent,
        player_interact_event::PlayerInteractEvent,
        player_interact_unknown_entity_event::PlayerInteractUnknownEntityEvent,
        player_join::PlayerJoinEvent, player_leave::PlayerLeaveEvent,
        player_login::PlayerLoginEvent, player_move::PlayerMoveEvent,
        player_permission_check::PlayerPermissionCheckEvent, player_respawn::PlayerRespawnEvent,
        player_teleport::PlayerTeleportEvent, player_toggle_flight_event::PlayerToggleFlightEvent,
        player_toggle_sneak_event::PlayerToggleSneakEvent,
        player_toggle_sprint_event::PlayerToggleSprintEvent,
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
        EventType::PlayerChangeWorldEvent => {
            register_typed_event::<PlayerChangeWorldEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerRespawnEvent => {
            register_typed_event::<PlayerRespawnEvent>(resource, handler, priority, blocking).await;
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
        EventType::PlayerEggThrowEvent => {
            register_typed_event::<PlayerEggThrowEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerInteractUnknownEntityEvent => {
            register_typed_event::<PlayerInteractUnknownEntityEvent>(
                resource, handler, priority, blocking,
            )
            .await;
        }
        EventType::PlayerInteractEntityEvent => {
            register_typed_event::<PlayerInteractEntityEvent>(
                resource, handler, priority, blocking,
            )
            .await;
        }
        EventType::PlayerInteractEvent => {
            register_typed_event::<PlayerInteractEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerToggleSneakEvent => {
            register_typed_event::<PlayerToggleSneakEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerToggleFlightEvent => {
            register_typed_event::<PlayerToggleFlightEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerToggleSprintEvent => {
            register_typed_event::<PlayerToggleSprintEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::InventoryClickEvent => {
            register_typed_event::<InventoryClickEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::InventoryCloseEvent => {
            register_typed_event::<InventoryCloseEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::BedrockFormResponseEvent => {
            register_typed_event::<BedrockFormResponseEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::CustomClickActionEvent => {
            register_typed_event::<CustomClickActionEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PlayerItemConsumeEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_item_consume::PlayerItemConsumeEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::PlayerItemDamageEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_item_damage::PlayerItemDamageEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::PlayerDropItemEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_drop_item::PlayerDropItemEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::PlayerBedEnterEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_bed::PlayerBedEnterEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::PlayerBedLeaveEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_bed::PlayerBedLeaveEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::PlayerBucketEmptyEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_bucket::PlayerBucketEmptyEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::PlayerBucketFillEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_bucket::PlayerBucketFillEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        _ => {
            tracing::error!("non-player event should not be routed to register_player_event");
        }
    }
}

impl PluginHostState {
    fn get_context(&self, res: &Resource<Context>) -> wasmtime::Result<&ContextResource> {
        self.resource_table
            .get::<ContextResource>(&Resource::new_own(res.rep()))
            .map_err(wasmtime::Error::from)
    }

    fn take_command(&mut self, res: &Resource<Command>) -> wasmtime::Result<CommandResource> {
        self.resource_table
            .delete::<CommandResource>(Resource::new_own(res.rep()))
            // Convert ResourceTableError -> wasmtime::Error
            .map_err(wasmtime::Error::from)
    }
}

async fn register_entity_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::entity::{
        entity_combust::EntityCombustEvent,
        entity_damage::EntityDamageEvent,
        entity_death::{EntityDeathEvent, PlayerDeathEvent},
        entity_regain_health::EntityRegainHealthEvent,
        entity_spawn::EntitySpawnEvent,
    };

    match event_type {
        EventType::EntityDamageEvent => {
            register_typed_event::<EntityDamageEvent>(resource, handler, priority, blocking).await;
        }
        EventType::EntityDeathEvent => {
            register_typed_event::<EntityDeathEvent>(resource, handler, priority, blocking).await;
        }
        EventType::PlayerDeathEvent => {
            register_typed_event::<PlayerDeathEvent>(resource, handler, priority, blocking).await;
        }
        EventType::EntitySpawnEvent => {
            register_typed_event::<EntitySpawnEvent>(resource, handler, priority, blocking).await;
        }
        EventType::EntityCombustEvent => {
            register_typed_event::<EntityCombustEvent>(resource, handler, priority, blocking).await;
        }
        EventType::EntityRegainHealthEvent => {
            register_typed_event::<EntityRegainHealthEvent>(resource, handler, priority, blocking)
                .await;
        }
        _ => {
            tracing::error!("non-entity event should not be routed to register_entity_event");
        }
    }
}

async fn register_inventory_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::inventory::{
        craft_item::CraftItemEvent, furnace_smelt::FurnaceSmeltEvent,
        inventory_drag::InventoryDragEvent, inventory_open::InventoryOpenEvent,
    };

    match event_type {
        EventType::InventoryOpenEvent => {
            register_typed_event::<InventoryOpenEvent>(resource, handler, priority, blocking).await;
        }
        EventType::InventoryDragEvent => {
            register_typed_event::<InventoryDragEvent>(resource, handler, priority, blocking).await;
        }
        EventType::CraftItemEvent => {
            register_typed_event::<CraftItemEvent>(resource, handler, priority, blocking).await;
        }
        EventType::FurnaceSmeltEvent => {
            register_typed_event::<FurnaceSmeltEvent>(resource, handler, priority, blocking).await;
        }
        _ => {
            tracing::error!("non-inventory event should not be routed to register_inventory_event");
        }
    }
}

async fn register_world_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::world::{
        chunk_load::ChunkLoad, chunk_save::ChunkSave, chunk_send::ChunkSend,
        spawn_change::SpawnChangeEvent,
    };

    match event_type {
        EventType::SpawnChangeEvent => {
            register_typed_event::<SpawnChangeEvent>(resource, handler, priority, blocking).await;
        }
        EventType::ChunkLoadEvent => {
            register_typed_event::<ChunkLoad>(resource, handler, priority, blocking).await;
        }
        EventType::ChunkSaveEvent => {
            register_typed_event::<ChunkSave>(resource, handler, priority, blocking).await;
        }
        EventType::ChunkSendEvent => {
            register_typed_event::<ChunkSend>(resource, handler, priority, blocking).await;
        }
        EventType::WeatherChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::weather_change::WeatherChangeEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::ThunderChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::weather_change::ThunderChangeEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::WorldLoadEvent => {
            register_typed_event::<crate::plugin::api::events::world::world_load::WorldLoadEvent>(
                resource, handler, priority, blocking,
            )
            .await;
        }
        EventType::WorldUnloadEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::world_load::WorldUnloadEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        _ => {
            tracing::error!("non-world event should not be routed to register_world_event");
        }
    }
}

async fn register_block_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::block::{
        block_break::BlockBreakEvent, block_burn::BlockBurnEvent,
        block_can_build::BlockCanBuildEvent, block_grow::BlockGrowEvent,
        block_place::BlockPlaceEvent, block_redstone::BlockRedstoneEvent,
    };

    match event_type {
        EventType::BlockRedstoneEvent => {
            register_typed_event::<BlockRedstoneEvent>(resource, handler, priority, blocking).await;
        }
        EventType::BlockBreakEvent => {
            register_typed_event::<BlockBreakEvent>(resource, handler, priority, blocking).await;
        }
        EventType::BlockBurnEvent => {
            register_typed_event::<BlockBurnEvent>(resource, handler, priority, blocking).await;
        }
        EventType::BlockCanBuildEvent => {
            register_typed_event::<BlockCanBuildEvent>(resource, handler, priority, blocking).await;
        }
        EventType::BlockGrowEvent => {
            register_typed_event::<BlockGrowEvent>(resource, handler, priority, blocking).await;
        }
        EventType::BlockPlaceEvent => {
            register_typed_event::<BlockPlaceEvent>(resource, handler, priority, blocking).await;
        }
        EventType::BlockDamageEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_damage::BlockDamageEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::BlockIgniteEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_ignite::BlockIgniteEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::BlockFromToEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_from_to::BlockFromToEvent,
            >(resource, handler, priority, blocking)
            .await;
        }
        EventType::BlockFormEvent => {
            register_typed_event::<crate::plugin::api::events::block::block_form::BlockFormEvent>(
                resource, handler, priority, blocking,
            )
            .await;
        }
        EventType::BlockFadeEvent => {
            register_typed_event::<crate::plugin::api::events::block::block_fade::BlockFadeEvent>(
                resource, handler, priority, blocking,
            )
            .await;
        }

        _ => {
            tracing::error!("non-block event should not be routed to register_block_event");
        }
    }
}
async fn register_server_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::server::{
        packet::{PacketReceivedEvent, PacketSentEvent},
        server_broadcast::ServerBroadcastEvent,
        server_command::ServerCommandEvent,
        server_load::ServerLoadEvent,
        server_tick_end::ServerTickEndEvent,
        server_tick_start::ServerTickStartEvent,
    };

    match event_type {
        EventType::PacketReceivedEvent => {
            register_typed_event::<PacketReceivedEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::PacketSentEvent => {
            register_typed_event::<PacketSentEvent>(resource, handler, priority, blocking).await;
        }
        EventType::ServerCommandEvent => {
            register_typed_event::<ServerCommandEvent>(resource, handler, priority, blocking).await;
        }
        EventType::ServerBroadcastEvent => {
            register_typed_event::<ServerBroadcastEvent>(resource, handler, priority, blocking)
                .await;
        }
        EventType::ServerLoadEvent => {
            register_typed_event::<ServerLoadEvent>(resource, handler, priority, blocking).await;
        }
        EventType::ServerTickEndEvent => {
            register_typed_event::<ServerTickEndEvent>(resource, handler, priority, blocking).await;
        }
        EventType::ServerTickStartEvent => {
            register_typed_event::<ServerTickStartEvent>(resource, handler, priority, blocking)
                .await;
        }
        _ => {
            tracing::error!("non-server event should not be routed to register_server_event");
        }
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
    ) -> wasmtime::Result<()> {
        // Updated return type
        let priority = match event_priority {
            EventPriority::Highest => crate::plugin::EventPriority::Highest,
            EventPriority::High => crate::plugin::EventPriority::High,
            EventPriority::Normal => crate::plugin::EventPriority::Normal,
            EventPriority::Low => crate::plugin::EventPriority::Low,
            EventPriority::Lowest => crate::plugin::EventPriority::Lowest,
        };

        // Use ? to trap if the plugin was dropped or the context handle is dead
        let plugin = self
            .plugin
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Plugin state uninitialized"))?
            .upgrade()
            .ok_or_else(|| wasmtime::Error::msg("Plugin has been dropped"))?;

        let resource = self.get_context(&context)?;
        let handler = Arc::new(WasmPluginEventHandler { handler_id, plugin });

        match event_type {
            event_type @ (EventType::PacketReceivedEvent
            | EventType::PacketSentEvent
            | EventType::ServerCommandEvent
            | EventType::ServerBroadcastEvent
            | EventType::ServerLoadEvent
            | EventType::ServerTickEndEvent
            | EventType::ServerTickStartEvent) => {
                register_server_event(resource, &handler, priority, blocking, event_type).await;
            }
            event_type @ (EventType::SpawnChangeEvent
            | EventType::ChunkLoadEvent
            | EventType::ChunkSaveEvent
            | EventType::ChunkSendEvent
            | EventType::WeatherChangeEvent
            | EventType::ThunderChangeEvent
            | EventType::WorldLoadEvent
            | EventType::WorldUnloadEvent) => {
                register_world_event(resource, &handler, priority, blocking, event_type).await;
            }
            event_type @ (EventType::EntityDamageEvent
            | EventType::EntityDeathEvent
            | EventType::PlayerDeathEvent
            | EventType::EntitySpawnEvent
            | EventType::EntityCombustEvent
            | EventType::EntityRegainHealthEvent) => {
                register_entity_event(resource, &handler, priority, blocking, event_type).await;
            }
            event_type @ (EventType::BlockRedstoneEvent
            | EventType::BlockBreakEvent
            | EventType::BlockBurnEvent
            | EventType::BlockCanBuildEvent
            | EventType::BlockGrowEvent
            | EventType::BlockPlaceEvent
            | EventType::BlockDamageEvent
            | EventType::BlockIgniteEvent
            | EventType::BlockFromToEvent
            | EventType::BlockFormEvent
            | EventType::BlockFadeEvent) => {
                register_block_event(resource, &handler, priority, blocking, event_type).await;
            }
            event_type @ (EventType::InventoryOpenEvent
            | EventType::InventoryDragEvent
            | EventType::CraftItemEvent
            | EventType::FurnaceSmeltEvent) => {
                register_inventory_event(resource, &handler, priority, blocking, event_type).await;
            }
            event_type => {
                register_player_event(resource, &handler, priority, blocking, event_type).await;
            }
        }

        Ok(())
    }

    async fn register_command(
        &mut self,
        context: Resource<Context>,
        command: Resource<Command>,
        permission: String,
    ) -> wasmtime::Result<()> {
        // Updated return type
        // Use your helpers to safely take/get resources
        let command_res = self.take_command(&command)?;
        let context_res = self.get_context(&context)?;

        context_res
            .provider
            .register_command(command_res.provider, permission)
            .await;
        Ok(())
    }

    async fn register_permission(
        &mut self,
        context: Resource<Context>,
        permission: Permission,
    ) -> wasmtime::Result<Result<(), String>> {
        let mut children = HashMap::with_capacity(permission.children.len());
        for child in permission.children {
            children.insert(child.node, child.value);
        }

        let util_permission = pumpkin_util::permission::Permission {
            node: permission.node,
            description: permission.description,
            default: match permission.default {
                PermissionDefault::Deny => pumpkin_util::permission::PermissionDefault::Deny,
                PermissionDefault::Allow => pumpkin_util::permission::PermissionDefault::Allow,
                PermissionDefault::Op(lvl) => {
                    pumpkin_util::permission::PermissionDefault::Op(match lvl {
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

        let context_res = self
            .resource_table
            .get_mut::<ContextResource>(&Resource::new_own(context.rep()))?;
        Ok(context_res
            .provider
            .register_permission(util_permission)
            .await)
    }

    async fn get_data_folder(&mut self, _context: Resource<Context>) -> wasmtime::Result<String> {
        Ok("data".to_string())
    }

    async fn get_server(
        &mut self,
        context: Resource<Context>,
    ) -> wasmtime::Result<Resource<Server>> {
        let server_provider = self.get_context(&context)?.provider.server.clone();
        self.add_server(server_provider)
            .map_err(|_| wasmtime::Error::msg("failed to add server resource"))
    }

    async fn drop(&mut self, rep: Resource<Context>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ContextResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
