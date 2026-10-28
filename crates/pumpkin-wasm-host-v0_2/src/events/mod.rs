use std::{marker::PhantomData, mem, ptr::NonNull, sync::Arc};

use pumpkin_data::{Block, entity::EntityType};
use pumpkin_inventory::screen_handler::ClickType;
use pumpkin_protocol::java::server::play::ActionType;
use pumpkin_util::{
    GameMode, Hand,
    math::{position::BlockPos, vector3::Vector3},
};
use wasmtime::component::Resource;

use crate::pumpkin::{self, plugin::event::Event as WasmEvent};
use pumpkin_core::{
    entity::player::Player,
    plugin::{BoxFuture, EventHandler, Payload},
    server::Server,
    world::World,
};
use pumpkin_wasm_host_common::{plugin::WasmPlugin, state::PluginHostState};

pub mod block;
pub mod cleanup;
pub mod dialog;
pub mod enchantment;
pub mod entity;
pub mod hanging;
pub mod inventory;
pub mod player;
pub mod raid;
pub mod server;
pub mod vehicle;
pub mod world;

pub use cleanup::*;

impl pumpkin::plugin::event::Host for PluginHostState {}

fn get_event_inner<E: Payload + ToFromWasmEvent + 'static>(
    state: &mut PluginHostState,
    event: Resource<WasmEvent>,
) -> wasmtime::Result<Result<E::Wit, ()>> {
    let handle = {
        let orig = state.get(&event)?;

        EventHandle {
            data: orig.data,
            mutable: orig.mutable,
        }
    };

    let Ok(event_inner) = handle.get_ref::<E>() else {
        return Ok(Err(()));
    };

    Ok(Ok(event_inner.to_wasm_event(state)))
}

fn set_event_inner<E: Payload + ToFromWasmEvent + 'static>(
    state: &mut PluginHostState,
    event: Resource<WasmEvent>,
    data: E::Wit,
) -> wasmtime::Result<Result<(), ()>> {
    let mut handle = {
        let orig = state.get(&event)?;

        EventHandle {
            data: orig.data,
            mutable: orig.mutable,
        }
    };

    let Ok(event_inner) = handle.get_mut::<E>() else {
        return Ok(Err(()));
    };

    event_inner.apply_wasm_event(data, state);
    Ok(Ok(()))
}

macro_rules! impl_event_getter_setter {
    ($event_type:path, $getter:ident, $setter:ident) => {
        fn $getter(
            &mut self,
            event: Resource<WasmEvent>,
        ) -> wasmtime::Result<Result<<$event_type as ToFromWasmEvent>::Wit, ()>> {
            get_event_inner::<$event_type>(self, event)
        }

        fn $setter(
            &mut self,
            event: Resource<pumpkin::plugin::event::Event>,
            data: <$event_type as ToFromWasmEvent>::Wit,
        ) -> wasmtime::Result<Result<(), ()>> {
            set_event_inner::<$event_type>(self, event, data)
        }
    };
}

impl pumpkin::plugin::event::HostEvent for PluginHostState {
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerJoinEvent,
        get_player_join,
        set_player_join
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerLeaveEvent,
        get_player_leave,
        set_player_leave
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerLoginEvent,
        get_player_login,
        set_player_login
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerChatEvent,
        get_player_chat,
        set_player_chat
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerCommandSendEvent,
        get_player_command_send,
        set_player_command_send
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerPermissionCheckEvent,
        get_player_permission_check,
        set_player_permission_check
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerMoveEvent,
        get_player_move,
        set_player_move
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerTeleportEvent,
        get_player_teleport,
        set_player_teleport
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerChangeWorldEvent,
        get_player_change_world,
        set_player_change_world
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerRespawnEvent,
        get_player_respawn,
        set_player_respawn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerExpChangeEvent,
        get_player_exp_change,
        set_player_exp_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerItemHeldEvent,
        get_player_item_held,
        set_player_item_held
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerChangedMainHandEvent,
        get_player_changed_main_hand,
        set_player_changed_main_hand
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerGamemodeChangeEvent,
        get_player_gamemode_change,
        set_player_gamemode_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerCustomPayloadEvent,
        get_player_custom_payload,
        set_player_custom_payload
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerFishEvent,
        get_player_fish,
        set_player_fish
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerEggThrowEvent,
        get_player_egg_throw,
        set_player_egg_throw
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerInteractUnknownEntityEvent,
        get_player_interact_unknown_entity,
        set_player_interact_unknown_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerInteractEntityEvent,
        get_player_interact_entity,
        set_player_interact_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerInteractEvent,
        get_player_interact,
        set_player_interact
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerToggleSneakEvent,
        get_player_toggle_sneak,
        set_player_toggle_sneak
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerToggleFlightEvent,
        get_player_toggle_flight,
        set_player_toggle_flight
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerToggleSprintEvent,
        get_player_toggle_sprint,
        set_player_toggle_sprint
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::InventoryClickEvent,
        get_inventory_click,
        set_inventory_click
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::InventoryCloseEvent,
        get_inventory_close,
        set_inventory_close
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockRedstoneEvent,
        get_block_redstone,
        set_block_redstone
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockBreakEvent,
        get_block_break,
        set_block_break
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockBurnEvent,
        get_block_burn,
        set_block_burn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockCanBuildEvent,
        get_block_can_build,
        set_block_can_build
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockGrowEvent,
        get_block_grow,
        set_block_grow
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockPlaceEvent,
        get_block_place,
        set_block_place
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::BedrockFormResponseEvent,
        get_bedrock_form_response,
        set_bedrock_form_response
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::dialog::DialogClickActionEvent,
        get_dialog_click_action,
        set_dialog_click_action
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::dialog::DialogShowEvent,
        get_dialog_show,
        set_dialog_show
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::dialog::DialogClearEvent,
        get_dialog_clear,
        set_dialog_clear
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::server::ServerCommandEvent,
        get_server_command,
        set_server_command
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::server::ServerListPingEvent,
        get_server_list_ping,
        set_server_list_ping
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::server::ServerLoadEvent,
        get_server_load,
        set_server_load
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::SpawnChangeEvent,
        get_spawn_change,
        set_spawn_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::server::ServerBroadcastEvent,
        get_server_broadcast,
        set_server_broadcast
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::server::ServerTickStartEvent,
        get_server_tick_start,
        set_server_tick_start
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::server::ServerTickEndEvent,
        get_server_tick_end,
        set_server_tick_end
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::server::PacketReceivedEvent,
        get_packet_received,
        set_packet_received
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::server::PacketSentEvent,
        get_packet_sent,
        set_packet_sent
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::ChunkLoad,
        get_chunk_load,
        set_chunk_load
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::ChunkSave,
        get_chunk_save,
        set_chunk_save
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::ChunkSend,
        get_chunk_send,
        set_chunk_send
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityDamageEvent,
        get_entity_damage,
        set_entity_damage
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityDeathEvent,
        get_entity_death,
        set_entity_death
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::PlayerDeathEvent,
        get_player_death,
        set_player_death
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntitySpawnEvent,
        get_entity_spawn,
        set_entity_spawn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityCombustEvent,
        get_entity_combust,
        set_entity_combust
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityRegainHealthEvent,
        get_entity_regain_health,
        set_entity_regain_health
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityAirChangeEvent,
        get_entity_air_change,
        set_entity_air_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityBreedEvent,
        get_entity_breed,
        set_entity_breed
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityDismountEvent,
        get_entity_dismount,
        set_entity_dismount
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityDyeEvent,
        get_entity_dye,
        set_entity_dye
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityEnterLoveModeEvent,
        get_entity_enter_love_mode,
        set_entity_enter_love_mode
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityExplodeEvent,
        get_entity_explode,
        set_entity_explode
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityMountEvent,
        get_entity_mount,
        set_entity_mount
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityPickupItemEvent,
        get_entity_pickup_item,
        set_entity_pickup_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityPortalEvent,
        get_entity_portal,
        set_entity_portal
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityResurrectEvent,
        get_entity_resurrect,
        set_entity_resurrect
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityShootBowEvent,
        get_entity_shoot_bow,
        set_entity_shoot_bow
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityTameEvent,
        get_entity_tame,
        set_entity_tame
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityTargetEvent,
        get_entity_target,
        set_entity_target
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityTeleportEvent,
        get_entity_teleport,
        set_entity_teleport
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityToggleGlideEvent,
        get_entity_toggle_glide,
        set_entity_toggle_glide
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityTransformEvent,
        get_entity_transform,
        set_entity_transform
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerItemConsumeEvent,
        get_player_item_consume,
        set_player_item_consume
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerItemDamageEvent,
        get_player_item_damage,
        set_player_item_damage
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerDropItemEvent,
        get_player_drop_item,
        set_player_drop_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerBedEnterEvent,
        get_player_bed_enter,
        set_player_bed_enter
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerBedLeaveEvent,
        get_player_bed_leave,
        set_player_bed_leave
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerBucketEmptyEvent,
        get_player_bucket_empty,
        set_player_bucket_empty
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerBucketFillEvent,
        get_player_bucket_fill,
        set_player_bucket_fill
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockDamageEvent,
        get_block_damage,
        set_block_damage
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockIgniteEvent,
        get_block_ignite,
        set_block_ignite
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockFromToEvent,
        get_block_from_to,
        set_block_from_to
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockFormEvent,
        get_block_form,
        set_block_form
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockFadeEvent,
        get_block_fade,
        set_block_fade
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockDispenseEvent,
        get_block_dispense,
        set_block_dispense
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockExplodeEvent,
        get_block_explode,
        set_block_explode
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockPhysicsEvent,
        get_block_physics,
        set_block_physics
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockPistonExtendEvent,
        get_block_piston_extend,
        set_block_piston_extend
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockPistonRetractEvent,
        get_block_piston_retract,
        set_block_piston_retract
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::NotePlayEvent,
        get_note_play,
        set_note_play
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::SignChangeEvent,
        get_sign_change,
        set_sign_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::SpongeAbsorbEvent,
        get_sponge_absorb,
        set_sponge_absorb
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::TNTPrimeEvent,
        get_tnt_prime,
        set_tnt_prime
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::WeatherChangeEvent,
        get_weather_change,
        set_weather_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::ThunderChangeEvent,
        get_thunder_change,
        set_thunder_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::WorldLoadEvent,
        get_world_load,
        set_world_load
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::WorldUnloadEvent,
        get_world_unload,
        set_world_unload
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::AsyncStructureGenerateEvent,
        get_async_structure_generate,
        set_async_structure_generate
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::AsyncStructureSpawnEvent,
        get_async_structure_spawn,
        set_async_structure_spawn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::ChunkPopulateEvent,
        get_chunk_populate,
        set_chunk_populate
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::ChunkUnloadEvent,
        get_chunk_unload,
        set_chunk_unload
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::EntitiesLoadEvent,
        get_entities_load,
        set_entities_load
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::EntitiesUnloadEvent,
        get_entities_unload,
        set_entities_unload
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::GenericGameEvent,
        get_generic_game,
        set_generic_game
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::LootGenerateEvent,
        get_loot_generate,
        set_loot_generate
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::PortalCreateEvent,
        get_portal_create,
        set_portal_create
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::StructureGrowEvent,
        get_structure_grow,
        set_structure_grow
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::TimeSkipEvent,
        get_time_skip,
        set_time_skip
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::WorldInitEvent,
        get_world_init,
        set_world_init
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::WorldSaveEvent,
        get_world_save,
        set_world_save
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::InventoryOpenEvent,
        get_inventory_open,
        set_inventory_open
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::InventoryDragEvent,
        get_inventory_drag,
        set_inventory_drag
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::CraftItemEvent,
        get_craft_item,
        set_craft_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::FurnaceSmeltEvent,
        get_furnace_smelt,
        set_furnace_smelt
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::BrewEvent,
        get_brew,
        set_brew
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::BrewingStandFuelEvent,
        get_brewing_stand_fuel,
        set_brewing_stand_fuel
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::FurnaceBurnEvent,
        get_furnace_burn,
        set_furnace_burn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::FurnaceExtractEvent,
        get_furnace_extract,
        set_furnace_extract
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::FurnaceStartSmeltEvent,
        get_furnace_start_smelt,
        set_furnace_start_smelt
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::HopperInventorySearchEvent,
        get_hopper_inventory_search,
        set_hopper_inventory_search
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::InventoryCreativeEvent,
        get_inventory_creative,
        set_inventory_creative
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::InventoryInteractEvent,
        get_inventory_interact,
        set_inventory_interact
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::InventoryMoveItemEvent,
        get_inventory_move_item,
        set_inventory_move_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::InventoryPickupItemEvent,
        get_inventory_pickup_item,
        set_inventory_pickup_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::PrepareAnvilEvent,
        get_prepare_anvil,
        set_prepare_anvil
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::PrepareGrindstoneEvent,
        get_prepare_grindstone,
        set_prepare_grindstone
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::PrepareInventoryResultEvent,
        get_prepare_inventory_result,
        set_prepare_inventory_result
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::PrepareItemCraftEvent,
        get_prepare_item_craft,
        set_prepare_item_craft
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::PrepareSmithingEvent,
        get_prepare_smithing,
        set_prepare_smithing
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::SmithItemEvent,
        get_smith_item,
        set_smith_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::inventory::TradeSelectEvent,
        get_trade_select,
        set_trade_select
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleBlockCollisionEvent,
        get_vehicle_block_collision,
        set_vehicle_block_collision
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleCollisionEvent,
        get_vehicle_collision,
        set_vehicle_collision
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleCreateEvent,
        get_vehicle_create,
        set_vehicle_create
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleDamageEvent,
        get_vehicle_damage,
        set_vehicle_damage
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleDestroyEvent,
        get_vehicle_destroy,
        set_vehicle_destroy
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleEnterEvent,
        get_vehicle_enter,
        set_vehicle_enter
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleEntityCollisionEvent,
        get_vehicle_entity_collision,
        set_vehicle_entity_collision
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleExitEvent,
        get_vehicle_exit,
        set_vehicle_exit
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleMoveEvent,
        get_vehicle_move,
        set_vehicle_move
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::vehicle::VehicleUpdateEvent,
        get_vehicle_update,
        set_vehicle_update
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::enchantment::PrepareItemEnchantEvent,
        get_prepare_item_enchant,
        set_prepare_item_enchant
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::enchantment::EnchantItemEvent,
        get_enchant_item,
        set_enchant_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::server::MapInitializeEvent,
        get_map_initialize,
        set_map_initialize
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::hanging::HangingBreakEvent,
        get_hanging_break,
        set_hanging_break
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::hanging::HangingBreakByEntityEvent,
        get_hanging_break_by_entity,
        set_hanging_break_by_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::hanging::HangingPlaceEvent,
        get_hanging_place,
        set_hanging_place
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BellResonateEvent,
        get_bell_resonate,
        set_bell_resonate
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BellRingEvent,
        get_bell_ring,
        set_bell_ring
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockBrushEvent,
        get_block_brush,
        set_block_brush
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockCookEvent,
        get_block_cook,
        set_block_cook
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockDamageAbortEvent,
        get_block_damage_abort,
        set_block_damage_abort
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockDispenseArmorEvent,
        get_block_dispense_armor,
        set_block_dispense_armor
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockDispenseLootEvent,
        get_block_dispense_loot,
        set_block_dispense_loot
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockDropItemEvent,
        get_block_drop_item,
        set_block_drop_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockExpEvent,
        get_block_exp,
        set_block_exp
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockFertilizeEvent,
        get_block_fertilize,
        set_block_fertilize
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockMultiPlaceEvent,
        get_block_multi_place,
        set_block_multi_place
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockReceiveGameEvent,
        get_block_receive_game,
        set_block_receive_game
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockShearEntityEvent,
        get_block_shear_entity,
        set_block_shear_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BlockSpreadEvent,
        get_block_spread,
        set_block_spread
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::BrewingStartEvent,
        get_brewing_start,
        set_brewing_start
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::CampfireStartEvent,
        get_campfire_start,
        set_campfire_start
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::CauldronLevelChangeEvent,
        get_cauldron_level_change,
        set_cauldron_level_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::CrafterCraftEvent,
        get_crafter_craft,
        set_crafter_craft
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::EntityBlockFormEvent,
        get_entity_block_form,
        set_entity_block_form
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::FluidLevelChangeEvent,
        get_fluid_level_change,
        set_fluid_level_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::InventoryBlockStartEvent,
        get_inventory_block_start,
        set_inventory_block_start
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::LeavesDecayEvent,
        get_leaves_decay,
        set_leaves_decay
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::MoistureChangeEvent,
        get_moisture_change,
        set_moisture_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::SculkBloomEvent,
        get_sculk_bloom,
        set_sculk_bloom
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::block::VaultDisplayItemEvent,
        get_vault_display_item,
        set_vault_display_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::CreatureSpawnEvent,
        get_creature_spawn,
        set_creature_spawn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EnderDragonChangePhaseEvent,
        get_ender_dragon_change_phase,
        set_ender_dragon_change_phase
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityBreakDoorEvent,
        get_entity_break_door,
        set_entity_break_door
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityChangeBlockEvent,
        get_entity_change_block,
        set_entity_change_block
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityDamageByBlockEvent,
        get_entity_damage_by_block,
        set_entity_damage_by_block
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityDamageByEntityEvent,
        get_entity_damage_by_entity,
        set_entity_damage_by_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityDropItemEvent,
        get_entity_drop_item,
        set_entity_drop_item
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityEnterBlockEvent,
        get_entity_enter_block,
        set_entity_enter_block
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityExhaustionEvent,
        get_entity_exhaustion,
        set_entity_exhaustion
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityInteractEvent,
        get_entity_interact,
        set_entity_interact
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityKnockbackEvent,
        get_entity_knockback,
        set_entity_knockback
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityPlaceEvent,
        get_entity_place,
        set_entity_place
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityPoseChangeEvent,
        get_entity_pose_change,
        set_entity_pose_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityPotionEffectEvent,
        get_entity_potion_effect,
        set_entity_potion_effect
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntitySpellCastEvent,
        get_entity_spell_cast,
        set_entity_spell_cast
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityTargetLivingEntityEvent,
        get_entity_target_living_entity,
        set_entity_target_living_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityToggleSwimEvent,
        get_entity_toggle_swim,
        set_entity_toggle_swim
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::ExplosionPrimeEvent,
        get_explosion_prime,
        set_explosion_prime
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::FireworkExplodeEvent,
        get_firework_explode,
        set_firework_explode
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::FoodLevelChangeEvent,
        get_food_level_change,
        set_food_level_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::ItemDespawnEvent,
        get_item_despawn,
        set_item_despawn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::ItemMergeEvent,
        get_item_merge,
        set_item_merge
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::ItemSpawnEvent,
        get_item_spawn,
        set_item_spawn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::PiglinBarterEvent,
        get_piglin_barter,
        set_piglin_barter
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::ProjectileHitEvent,
        get_projectile_hit,
        set_projectile_hit
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::ProjectileLaunchEvent,
        get_projectile_launch,
        set_projectile_launch
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::SheepDyeWoolEvent,
        get_sheep_dye_wool,
        set_sheep_dye_wool
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::SheepRegrowWoolEvent,
        get_sheep_regrow_wool,
        set_sheep_regrow_wool
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::SlimeSplitEvent,
        get_slime_split,
        set_slime_split
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::StriderTemperatureChangeEvent,
        get_strider_temperature_change,
        set_strider_temperature_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::VillagerAcquireTradeEvent,
        get_villager_acquire_trade,
        set_villager_acquire_trade
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::VillagerCareerChangeEvent,
        get_villager_career_change,
        set_villager_career_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::VillagerReplenishTradeEvent,
        get_villager_replenish_trade,
        set_villager_replenish_trade
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::WardenAngerChangeEvent,
        get_warden_anger_change,
        set_warden_anger_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::AreaEffectCloudApplyEvent,
        get_area_effect_cloud_apply,
        set_area_effect_cloud_apply
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::ArrowBodyCountChangeEvent,
        get_arrow_body_count_change,
        set_arrow_body_count_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::BatToggleSleepEvent,
        get_bat_toggle_sleep,
        set_bat_toggle_sleep
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::CreeperPowerEvent,
        get_creeper_power,
        set_creeper_power
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityCombustByBlockEvent,
        get_entity_combust_by_block,
        set_entity_combust_by_block
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityCombustByEntityEvent,
        get_entity_combust_by_entity,
        set_entity_combust_by_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityKnockbackByEntityEvent,
        get_entity_knockback_by_entity,
        set_entity_knockback_by_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityPortalEnterEvent,
        get_entity_portal_enter,
        set_entity_portal_enter
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityPortalExitEvent,
        get_entity_portal_exit,
        set_entity_portal_exit
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityRemoveEvent,
        get_entity_remove,
        set_entity_remove
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityTargetBlockEvent,
        get_entity_target_block,
        set_entity_target_block
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::EntityUnleashEvent,
        get_entity_unleash,
        set_entity_unleash
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::ExpBottleEvent,
        get_exp_bottle,
        set_exp_bottle
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::HorseJumpEvent,
        get_horse_jump,
        set_horse_jump
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::LingeringPotionSplashEvent,
        get_lingering_potion_splash,
        set_lingering_potion_splash
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::PigZapEvent,
        get_pig_zap,
        set_pig_zap
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::PigZombieAngerEvent,
        get_pig_zombie_anger,
        set_pig_zombie_anger
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::PotionSplashEvent,
        get_potion_splash,
        set_potion_splash
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::SpawnerSpawnEvent,
        get_spawner_spawn,
        set_spawner_spawn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::TrialSpawnerSpawnEvent,
        get_trial_spawner_spawn,
        set_trial_spawner_spawn
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::entity::VillagerReputationChangeEvent,
        get_villager_reputation_change,
        set_villager_reputation_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::AsyncPlayerChatEvent,
        get_async_player_chat,
        set_async_player_chat
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::AsyncPlayerPreLoginEvent,
        get_async_player_pre_login,
        set_async_player_pre_login
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerAdvancementDoneEvent,
        get_player_advancement_done,
        set_player_advancement_done
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerAnimationEvent,
        get_player_animation,
        set_player_animation
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerArmorStandManipulateEvent,
        get_player_armor_stand_manipulate,
        set_player_armor_stand_manipulate
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerBucketEntityEvent,
        get_player_bucket_entity,
        set_player_bucket_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerChangedWorldEvent,
        get_player_changed_world,
        set_player_changed_world
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerChannelEvent,
        get_player_channel,
        set_player_channel
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerCommandPreprocessEvent,
        get_player_command_preprocess,
        set_player_command_preprocess
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerEditBookEvent,
        get_player_edit_book,
        set_player_edit_book
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerElytraBoostEvent,
        get_player_elytra_boost,
        set_player_elytra_boost
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerExpCooldownChangeEvent,
        get_player_exp_cooldown_change,
        set_player_exp_cooldown_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerHarvestBlockEvent,
        get_player_harvest_block,
        set_player_harvest_block
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerHideEntityEvent,
        get_player_hide_entity,
        set_player_hide_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerItemBreakEvent,
        get_player_item_break,
        set_player_item_break
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerItemMendEvent,
        get_player_item_mend,
        set_player_item_mend
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerKickEvent,
        get_player_kick,
        set_player_kick
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerLeashEntityEvent,
        get_player_leash_entity,
        set_player_leash_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerLevelChangeEvent,
        get_player_level_change,
        set_player_level_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerLocaleChangeEvent,
        get_player_locale_change,
        set_player_locale_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerNameEntityEvent,
        get_player_name_entity,
        set_player_name_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerOpenSignEvent,
        get_player_open_sign,
        set_player_open_sign
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerPortalEvent,
        get_player_portal,
        set_player_portal
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerPreLoginEvent,
        get_player_pre_login,
        set_player_pre_login
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerRiptideEvent,
        get_player_riptide,
        set_player_riptide
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerShearEntityEvent,
        get_player_shear_entity,
        set_player_shear_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerShowEntityEvent,
        get_player_show_entity,
        set_player_show_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerSpawnChangeEvent,
        get_player_spawn_change,
        set_player_spawn_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerStatisticIncrementEvent,
        get_player_statistic_increment,
        set_player_statistic_increment
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerSwapHandItemsEvent,
        get_player_swap_hands,
        set_player_swap_hands
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerTakeLecternBookEvent,
        get_player_take_lectern_book,
        set_player_take_lectern_book
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerUnleashEntityEvent,
        get_player_unleash_entity,
        set_player_unleash_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerVelocityEvent,
        get_player_velocity,
        set_player_velocity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerInputEvent,
        get_player_input,
        set_player_input
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerInteractAtEntityEvent,
        get_player_interact_at_entity,
        set_player_interact_at_entity
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerLinksSendEvent,
        get_player_links_send,
        set_player_links_send
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerPickupArrowEvent,
        get_player_pickup_arrow,
        set_player_pickup_arrow
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerRecipeBookClickEvent,
        get_player_recipe_book_click,
        set_player_recipe_book_click
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerRecipeBookSettingsChangeEvent,
        get_player_recipe_book_settings_change,
        set_player_recipe_book_settings_change
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerRecipeDiscoverEvent,
        get_player_recipe_discover,
        set_player_recipe_discover
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerRegisterChannelEvent,
        get_player_register_channel,
        set_player_register_channel
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerResourcePackStatusEvent,
        get_player_resource_pack_status,
        set_player_resource_pack_status
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerSpawnLocationEvent,
        get_player_spawn_location,
        set_player_spawn_location
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::player::PlayerUnregisterChannelEvent,
        get_player_unregister_channel,
        set_player_unregister_channel
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::raid::RaidFinishEvent,
        get_raid_finish,
        set_raid_finish
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::raid::RaidSpawnWaveEvent,
        get_raid_spawn_wave,
        set_raid_spawn_wave
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::raid::RaidStopEvent,
        get_raid_stop,
        set_raid_stop
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::raid::RaidTriggerEvent,
        get_raid_trigger,
        set_raid_trigger
    );
    impl_event_getter_setter!(
        pumpkin_core::plugin::events::world::LightningStrikeEvent,
        get_lightning_strike,
        set_lightning_strike
    );
}

pub struct WasmPluginEventHandler {
    pub handler_id: u32,
    pub plugin: Arc<WasmPlugin>,
}

pub trait ToFromWasmEvent {
    type Wit;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit;

    fn from_wasm_event(event: Self::Wit, state: &mut PluginHostState) -> Self;

    fn apply_wasm_event(&mut self, event: Self::Wit, state: &mut PluginHostState)
    where
        Self: Sized,
    {
        *self = Self::from_wasm_event(event, state);
    }
}

pub(super) const fn to_wasm_position(position: Vector3<f64>) -> pumpkin::plugin::common::Position {
    (position.x, position.y, position.z)
}

pub(super) const fn from_wasm_position(
    position: pumpkin::plugin::common::Position,
) -> Vector3<f64> {
    Vector3::new(position.0, position.1, position.2)
}

pub(super) const fn to_wasm_block_position(
    position: BlockPos,
) -> pumpkin::plugin::common::BlockPos {
    pumpkin::plugin::common::BlockPos {
        x: position.0.x,
        y: position.0.y,
        z: position.0.z,
    }
}

pub(super) const fn from_wasm_block_position(
    position: pumpkin::plugin::common::BlockPos,
) -> BlockPos {
    BlockPos::new(position.x, position.y, position.z)
}

pub(super) fn to_wasm_block_name(block: &'static Block) -> String {
    format!("minecraft:{}", block.name)
}

pub(super) fn from_wasm_block_name(block_name: &str) -> &'static Block {
    Block::from_registry_key(block_name.strip_prefix("minecraft:").unwrap_or(block_name))
        .unwrap_or(&Block::AIR)
}

pub(super) fn to_wasm_entity_type(entity_type: &'static EntityType) -> String {
    format!("minecraft:{}", entity_type.resource_name)
}

pub(super) fn from_wasm_entity_type(entity_type: &str) -> &'static EntityType {
    EntityType::from_name(
        entity_type
            .strip_prefix("minecraft:")
            .unwrap_or(entity_type),
    )
    .unwrap_or(&EntityType::PLAYER)
}

pub(super) const fn to_wasm_hand(hand: Hand) -> pumpkin::plugin::common::Hand {
    match hand {
        Hand::Left => pumpkin::plugin::common::Hand::Left,
        Hand::Right => pumpkin::plugin::common::Hand::Right,
    }
}

pub(super) const fn from_wasm_hand(hand: pumpkin::plugin::common::Hand) -> Hand {
    match hand {
        pumpkin::plugin::common::Hand::Left => Hand::Left,
        pumpkin::plugin::common::Hand::Right => Hand::Right,
    }
}

pub(super) const fn to_wasm_entity_interaction_action(
    action: ActionType,
) -> pumpkin::plugin::event::EntityInteractionAction {
    match action {
        ActionType::Interact => pumpkin::plugin::event::EntityInteractionAction::Interact,
        ActionType::Attack => pumpkin::plugin::event::EntityInteractionAction::Attack,
        ActionType::InteractAt => pumpkin::plugin::event::EntityInteractionAction::InteractAt,
    }
}

pub(super) const fn from_wasm_entity_interaction_action(
    action: pumpkin::plugin::event::EntityInteractionAction,
) -> ActionType {
    match action {
        pumpkin::plugin::event::EntityInteractionAction::Interact => ActionType::Interact,
        pumpkin::plugin::event::EntityInteractionAction::Attack => ActionType::Attack,
        pumpkin::plugin::event::EntityInteractionAction::InteractAt => ActionType::InteractAt,
    }
}

pub(super) const fn to_wasm_game_mode(game_mode: GameMode) -> pumpkin::plugin::common::GameMode {
    match game_mode {
        GameMode::Survival => pumpkin::plugin::common::GameMode::Survival,
        GameMode::Creative => pumpkin::plugin::common::GameMode::Creative,
        GameMode::Adventure => pumpkin::plugin::common::GameMode::Adventure,
        GameMode::Spectator => pumpkin::plugin::common::GameMode::Spectator,
    }
}

pub(super) const fn from_wasm_game_mode(game_mode: pumpkin::plugin::common::GameMode) -> GameMode {
    match game_mode {
        pumpkin::plugin::common::GameMode::Survival => GameMode::Survival,
        pumpkin::plugin::common::GameMode::Creative => GameMode::Creative,
        pumpkin::plugin::common::GameMode::Adventure => GameMode::Adventure,
        pumpkin::plugin::common::GameMode::Spectator => GameMode::Spectator,
    }
}

pub(super) const fn to_wasm_click_type(click_type: ClickType) -> pumpkin::plugin::gui::ClickType {
    match click_type {
        ClickType::Left => pumpkin::plugin::gui::ClickType::Left,
        ClickType::Right => pumpkin::plugin::gui::ClickType::Right,
        ClickType::ShiftLeft => pumpkin::plugin::gui::ClickType::ShiftLeft,
        ClickType::ShiftRight => pumpkin::plugin::gui::ClickType::ShiftRight,
        ClickType::Middle => pumpkin::plugin::gui::ClickType::Middle,
        ClickType::Drop => pumpkin::plugin::gui::ClickType::Drop,
        ClickType::ControlDrop => pumpkin::plugin::gui::ClickType::ControlDrop,
        ClickType::DoubleClick => pumpkin::plugin::gui::ClickType::DoubleClick,
        ClickType::NumberKey(_) => pumpkin::plugin::gui::ClickType::NumberKey,
        ClickType::Unknown => pumpkin::plugin::gui::ClickType::Unknown,
    }
}

pub(super) const fn from_wasm_click_type(click_type: pumpkin::plugin::gui::ClickType) -> ClickType {
    match click_type {
        pumpkin::plugin::gui::ClickType::Left => ClickType::Left,
        pumpkin::plugin::gui::ClickType::Right => ClickType::Right,
        pumpkin::plugin::gui::ClickType::ShiftLeft => ClickType::ShiftLeft,
        pumpkin::plugin::gui::ClickType::ShiftRight => ClickType::ShiftRight,
        pumpkin::plugin::gui::ClickType::Middle => ClickType::Middle,
        pumpkin::plugin::gui::ClickType::Drop => ClickType::Drop,
        pumpkin::plugin::gui::ClickType::ControlDrop => ClickType::ControlDrop,
        pumpkin::plugin::gui::ClickType::DoubleClick => ClickType::DoubleClick,
        pumpkin::plugin::gui::ClickType::NumberKey => ClickType::NumberKey(0), // Default to 0
        pumpkin::plugin::gui::ClickType::Unknown => ClickType::Unknown,
    }
}

pub(super) fn consume_player(
    state: &mut PluginHostState,
    player: &Resource<pumpkin::plugin::player::Player>,
) -> Arc<Player> {
    state
        .resource_table
        .delete::<Arc<Player>>(Resource::new_own(player.rep()))
        .expect("invalid player resource handle")
}

pub(super) fn consume_text_component(
    state: &mut PluginHostState,
    text_component: &Resource<pumpkin::plugin::text::TextComponent>,
) -> pumpkin_util::text::TextComponent {
    state
        .resource_table
        .delete::<pumpkin_util::text::TextComponent>(Resource::new_own(text_component.rep()))
        .expect("invalid text-component resource handle")
}

pub(super) fn consume_world(
    state: &mut PluginHostState,
    world: &Resource<pumpkin::plugin::world::World>,
) -> Arc<World> {
    state
        .resource_table
        .delete::<Arc<World>>(Resource::new_own(world.rep()))
        .expect("invalid world resource handle")
}

pub(crate) struct EventHandle {
    data: NonNull<dyn Payload + 'static>,
    mutable: bool,
}

unsafe impl Send for EventHandle {}
unsafe impl Sync for EventHandle {}

impl EventHandle {
    fn from_ref(event: &(dyn Payload + 'static)) -> Self {
        Self {
            data: NonNull::from_ref(event),
            mutable: false,
        }
    }

    fn from_mut(event: &mut (dyn Payload + 'static)) -> Self {
        Self {
            data: NonNull::from_mut(event),
            mutable: true,
        }
    }

    fn get_ref<'a, T: Payload + 'static>(&'a self) -> Result<&'a T, ()> {
        unsafe { self.data.as_ref() }.downcast_ref::<T>().ok_or(())
    }

    fn get_mut<'a, T: Payload + 'static>(&'a mut self) -> Result<&'a mut T, ()> {
        if !self.mutable {
            return Err(());
        }
        unsafe { self.data.as_mut() }.downcast_mut::<T>().ok_or(())
    }
}

struct EventResource<'a> {
    data: Resource<WasmEvent>,
    _phantom: PhantomData<&'a mut (dyn Payload + 'static)>,
}

impl<'a> EventResource<'a> {
    async fn from_ref(
        event: &'a (dyn Payload + 'static),
        plugin: &'a Arc<WasmPlugin>,
    ) -> wasmtime::Result<Self> {
        let static_event: &'static (dyn Payload + 'static) = unsafe { mem::transmute(event) };
        let handle = EventHandle::from_ref(static_event);
        Ok(Self {
            data: plugin
                .store
                .call(move |state| state.add(handle))
                .await
                .flatten()?,
            _phantom: PhantomData,
        })
    }

    async fn from_mut(
        event: &'a mut (dyn Payload + 'static),
        plugin: &'a Arc<WasmPlugin>,
    ) -> wasmtime::Result<Self> {
        let static_event: &'static mut (dyn Payload + 'static) = unsafe { mem::transmute(event) };
        let handle = EventHandle::from_mut(static_event);
        Ok(Self {
            data: plugin
                .store
                .call(|state| state.add(handle))
                .await
                .flatten()?,
            _phantom: PhantomData,
        })
    }

    fn get_borrow(&self) -> Resource<WasmEvent> {
        Resource::new_borrow(self.data.rep())
    }

    async fn destroy(self, plugin: &Arc<WasmPlugin>) -> wasmtime::Result<()> {
        plugin
            .store
            .call(|state| state.drop(self.data))
            .await
            .flatten()
    }
}

impl WasmPluginEventHandler {
    async fn handle_dyn<'a, F, Fut>(&'a self, server: &'a Arc<Server>, create_event: F)
    where
        F: FnOnce(&'a Arc<WasmPlugin>) -> Fut,
        Fut: std::future::Future<Output = wasmtime::Result<EventResource<'a>>>,
    {
        let wasm_event = match create_event(&self.plugin).await {
            Ok(res) => res,
            Err(error) => {
                tracing::error!(
                    handler_id = self.handler_id,
                    %error,
                    "Failed to create resource for event."
                );
                return;
            }
        };

        let event_borrow = wasm_event.get_borrow();
        let server_clone = server.clone();

        let handler_id = self.handler_id;
        let function = *self
            .plugin
            .instance::<crate::Plugin>()
            .func_handle_event()
            .func();

        match self
            .plugin
            .store
            .call_guest(move |mut guest| {
                Box::pin(async move {
                    let server_res = guest.with(|mut store| store.data_mut().add(server_clone))?;

                    guest
                        .call_func::<(
                            u32,
                            Resource<crate::pumpkin::plugin::server::Server>,
                            Resource<WasmEvent>,
                        ), ()>(
                            function, (handler_id, server_res, event_borrow)
                        )
                        .await
                })
            })
            .await
        {
            Ok(_) => {}
            Err(error) => {
                tracing::error!(
                    handler_id = self.handler_id,
                    %error,
                    "Wasm event handler failed"
                );
            }
        }

        match wasm_event.destroy(&self.plugin).await {
            Ok(_) => {}
            Err(error) => {
                tracing::error!(
                    handler_id = self.handler_id,
                    %error,
                    "Failed to cleanup resource for event."
                );
            }
        };
    }
}

impl<E: Payload + ToFromWasmEvent + 'static> EventHandler<E> for WasmPluginEventHandler {
    fn handle<'a>(&'a self, server: &'a Arc<Server>, event: &'a E) -> BoxFuture<'a, ()> {
        let dyn_event: &(dyn Payload + 'static) = event;
        Box::pin(self.handle_dyn(server, move |plugin| {
            EventResource::from_ref(dyn_event, plugin)
        }))
    }

    fn handle_blocking<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut E,
    ) -> BoxFuture<'a, ()> {
        let dyn_event: &mut (dyn Payload + 'static) = event;
        Box::pin(self.handle_dyn(server, move |plugin| {
            EventResource::from_mut(dyn_event, plugin)
        }))
    }
}
