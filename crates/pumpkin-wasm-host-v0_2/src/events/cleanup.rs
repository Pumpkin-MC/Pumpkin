use crate::pumpkin::plugin::{
    entity::Entity, event as wit_event, item_stack::ItemStack, player::Player, server::Server,
    text::TextComponent, world::World,
};
use pumpkin_wasm_host_common::state::PluginHostState;
use wasmtime::component::Resource;

fn cleanup_player(state: &mut PluginHostState, player: &Resource<Player>) {
    state.discard_to_be_removed(player);
}

fn cleanup_world(state: &mut PluginHostState, world: &Resource<World>) {
    state.discard_to_be_removed(world);
}

fn cleanup_text_component(state: &mut PluginHostState, text_component: &Resource<TextComponent>) {
    state.discard_to_be_removed(text_component);
}

fn cleanup_item_stack(state: &mut PluginHostState, item: &Resource<ItemStack>) {
    state.discard_to_be_removed(item);
}

pub fn cleanup_entity(state: &mut PluginHostState, entity: &Resource<Entity>) {
    state.discard_to_be_removed(entity);
}

pub fn cleanup_server(state: &mut PluginHostState, server: &Resource<Server>) {
    state.discard_to_be_removed(server);
}

pub trait CleanupEvent {
    fn cleanup_event(&self, state: &mut PluginHostState);
}

impl CleanupEvent for wit_event::PlayerJoinEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_text_component(state, &self.join_message);
    }
}

impl CleanupEvent for wit_event::PlayerLeaveEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_text_component(state, &self.leave_message);
    }
}

impl CleanupEvent for wit_event::PlayerLoginEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_text_component(state, &self.kick_message);
    }
}

impl CleanupEvent for wit_event::PlayerChatEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        for res in &self.recipients {
            cleanup_player(state, res);
        }
    }
}

impl CleanupEvent for wit_event::PlayerCommandSendEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerPermissionCheckEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerMoveEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerTeleportEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerChangeWorldEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_world(state, &self.previous_world);
        cleanup_world(state, &self.new_world);
    }
}

impl CleanupEvent for wit_event::PlayerRespawnEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_world(state, &self.previous_world);
        cleanup_world(state, &self.respawned_world);
    }
}

impl CleanupEvent for wit_event::PlayerExpChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerItemHeldEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerChangedMainHandEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerGamemodeChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerCustomPayloadEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerFishEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerEggThrowEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerInteractUnknownEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerInteractEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerInteractEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerToggleSneakEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerToggleFlightEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerToggleSprintEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::InventoryClickEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        if let Some(res) = &self.clicked_item {
            cleanup_item_stack(state, res);
        }
        if let Some(res) = &self.cursor {
            cleanup_item_stack(state, res);
        }
    }
}

impl CleanupEvent for wit_event::InventoryCloseEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::BlockRedstoneEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::BlockBreakEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        if let Some(res) = &self.player {
            cleanup_player(state, res);
        }
    }
}

impl CleanupEvent for wit_event::BlockBurnEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::BlockCanBuildEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::BlockGrowEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::BlockPlaceEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::BedrockFormResponseEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::DialogClickActionEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::DialogClearEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::DialogShowEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_text_component(state, &self.dialog.title);
        for body in &self.dialog.body {
            match body {
                crate::pumpkin::plugin::java_dialogs::DialogBody::PlainMessage(c) => {
                    cleanup_text_component(state, c);
                }
                crate::pumpkin::plugin::java_dialogs::DialogBody::Item(item) => {
                    cleanup_item_stack(state, item);
                }
            }
        }
        for input in &self.dialog.inputs {
            use crate::pumpkin::plugin::java_dialogs::DialogInput;
            match input {
                DialogInput::Bool(b) => cleanup_text_component(state, &b.label),
                DialogInput::Text(t) => {
                    cleanup_text_component(state, &t.label);
                    cleanup_text_component(state, &t.placeholder);
                }
                DialogInput::NumberRange(n) => cleanup_text_component(state, &n.label),
                DialogInput::SingleOption(s) => {
                    cleanup_text_component(state, &s.label);
                    for opt in &s.options {
                        cleanup_text_component(state, opt);
                    }
                }
            }
        }
        for button in &self.dialog.buttons {
            cleanup_text_component(state, &button.text);
            if let Some(tooltip) = &button.tooltip {
                cleanup_text_component(state, tooltip);
            }
        }
        for link in &self.dialog.links {
            if let crate::pumpkin::plugin::java_dialogs::LinkLabel::Custom(c) = &link.label {
                cleanup_text_component(state, c);
            }
        }
        if let Some(ext_title) = &self.dialog.external_title {
            cleanup_text_component(state, ext_title);
        }
    }
}

impl CleanupEvent for wit_event::ServerCommandEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ServerListPingEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_text_component(state, &self.motd);
    }
}

impl CleanupEvent for wit_event::ServerLoadEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::SpawnChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::ServerBroadcastEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_text_component(state, &self.message);
        cleanup_text_component(state, &self.sender);
    }
}

impl CleanupEvent for wit_event::ServerTickStartEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ServerTickEndEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PacketReceivedEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PacketSentEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::ChunkLoadEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::ChunkSaveEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::ChunkSendEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::EntityDamageEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityDeathEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PlayerDeathEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_text_component(state, &self.death_message);
    }
}

impl CleanupEvent for wit_event::EntitySpawnEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::EntityCombustEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityRegainHealthEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityAirChangeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityBreedEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityDismountEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityDyeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        if let Some(res) = &self.player {
            cleanup_player(state, res);
        }
    }
}

impl CleanupEvent for wit_event::EntityEnterLoveModeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityExplodeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityMountEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityPickupItemEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityPortalEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityResurrectEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityShootBowEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityTameEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.owner);
    }
}

impl CleanupEvent for wit_event::EntityTargetEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityTeleportEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityToggleGlideEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityTransformEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PlayerItemConsumeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerItemDamageEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerDropItemEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerBedEnterEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerBedLeaveEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerBucketEmptyEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerBucketFillEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::BlockDamageEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::BlockIgniteEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BlockFromToEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BlockFormEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BlockFadeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BlockDispenseEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BlockExplodeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BlockPhysicsEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BlockPistonExtendEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BlockPistonRetractEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::NotePlayEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::SignChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::SpongeAbsorbEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::TntPrimeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::WeatherChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::ThunderChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::WorldLoadEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::WorldUnloadEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::AsyncStructureGenerateEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::AsyncStructureSpawnEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ChunkPopulateEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ChunkUnloadEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntitiesLoadEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntitiesUnloadEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::GenericGameEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::LootGenerateEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PortalCreateEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::StructureGrowEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::TimeSkipEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::WorldInitEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::WorldSaveEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::InventoryOpenEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::InventoryDragEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::CraftItemEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::FurnaceSmeltEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BrewEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BrewingStandFuelEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::FurnaceBurnEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::FurnaceExtractEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::FurnaceStartSmeltEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::HopperInventorySearchEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::InventoryCreativeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::InventoryInteractEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::InventoryMoveItemEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::InventoryPickupItemEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PrepareAnvilEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PrepareGrindstoneEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PrepareInventoryResultEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PrepareItemCraftEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PrepareSmithingEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::SmithItemEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::TradeSelectEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::VehicleBlockCollisionEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VehicleCollisionEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VehicleCreateEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VehicleDamageEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VehicleDestroyEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VehicleEnterEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VehicleEntityCollisionEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VehicleExitEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VehicleMoveEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VehicleUpdateEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PrepareItemEnchantEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_item_stack(state, &self.item);
    }
}

impl CleanupEvent for wit_event::EnchantItemEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_item_stack(state, &self.item);
    }
}

impl CleanupEvent for wit_event::MapInitializeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::HangingBreakEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::HangingBreakByEntityEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::HangingPlaceEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        if let Some(res) = &self.player {
            cleanup_player(state, res);
        }
    }
}

impl CleanupEvent for wit_event::BellResonateEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::BellRingEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::BlockBrushEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        cleanup_player(state, &self.player);
        cleanup_item_stack(state, &self.item);
    }
}

impl CleanupEvent for wit_event::BlockCookEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        cleanup_item_stack(state, &self.source);
        cleanup_item_stack(state, &self.result);
    }
}

impl CleanupEvent for wit_event::BlockDamageAbortEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_world(state, &self.target_world);
        cleanup_item_stack(state, &self.item_in_hand);
    }
}

impl CleanupEvent for wit_event::BlockDispenseArmorEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        cleanup_item_stack(state, &self.item);
    }
}

impl CleanupEvent for wit_event::BlockDispenseLootEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        for res in &self.items {
            cleanup_item_stack(state, res);
        }
    }
}

impl CleanupEvent for wit_event::BlockDropItemEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        if let Some(res) = &self.player {
            cleanup_player(state, res);
        }
        for res in &self.items {
            cleanup_item_stack(state, res);
        }
    }
}

impl CleanupEvent for wit_event::BlockExpEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::BlockFertilizeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        if let Some(res) = &self.player {
            cleanup_player(state, res);
        }
    }
}

impl CleanupEvent for wit_event::BlockMultiPlaceEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::BlockReceiveGameEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::BlockShearEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        cleanup_item_stack(state, &self.item);
    }
}

impl CleanupEvent for wit_event::BlockSpreadEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::BrewingStartEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::CampfireStartEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        cleanup_item_stack(state, &self.item);
    }
}

impl CleanupEvent for wit_event::CauldronLevelChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::CrafterCraftEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        cleanup_item_stack(state, &self.result);
    }
}

impl CleanupEvent for wit_event::EntityBlockFormEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::FluidLevelChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::InventoryBlockStartEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::LeavesDecayEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::MoistureChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::SculkBloomEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
    }
}

impl CleanupEvent for wit_event::VaultDisplayItemEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        cleanup_item_stack(state, &self.item);
    }
}

impl CleanupEvent for wit_event::CreatureSpawnEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_world(state, &self.target_world);
        if let Some(res) = &self.player {
            cleanup_player(state, res);
        }
    }
}

impl CleanupEvent for wit_event::EnderDragonChangePhaseEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityBreakDoorEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityChangeBlockEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityDamageByBlockEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityDamageByEntityEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityDropItemEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityEnterBlockEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityExhaustionEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityInteractEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityKnockbackEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityPlaceEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityPoseChangeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityPotionEffectEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntitySpellCastEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityTargetLivingEntityEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityToggleSwimEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ExplosionPrimeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::FireworkExplodeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::FoodLevelChangeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ItemDespawnEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ItemMergeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ItemSpawnEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PiglinBarterEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_item_stack(state, &self.input_item);
        for res in &self.outcome {
            cleanup_item_stack(state, res);
        }
    }
}

impl CleanupEvent for wit_event::ProjectileHitEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ProjectileLaunchEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::SheepDyeWoolEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::SheepRegrowWoolEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::SlimeSplitEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::StriderTemperatureChangeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VillagerAcquireTradeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VillagerCareerChangeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VillagerReplenishTradeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::WardenAngerChangeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::AreaEffectCloudApplyEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ArrowBodyCountChangeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::BatToggleSleepEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::CreeperPowerEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityCombustByBlockEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityCombustByEntityEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityKnockbackByEntityEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityPortalEnterEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityPortalExitEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityRemoveEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityTargetBlockEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::EntityUnleashEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::ExpBottleEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::HorseJumpEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::LingeringPotionSplashEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PigZapEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PigZombieAngerEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::PotionSplashEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::SpawnerSpawnEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::TrialSpawnerSpawnEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::VillagerReputationChangeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::AsyncPlayerChatEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_text_component(state, &self.format);
    }
}

impl CleanupEvent for wit_event::AsyncPlayerPreLoginEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_text_component(state, &self.kick_message);
    }
}

impl CleanupEvent for wit_event::PlayerAdvancementDoneEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerAnimationEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerArmorStandManipulateEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerBucketEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerChangedWorldEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_world(state, &self.from_world);
        cleanup_world(state, &self.to_world);
    }
}

impl CleanupEvent for wit_event::PlayerChannelEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerCommandPreprocessEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerEditBookEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerElytraBoostEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerExpCooldownChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerHarvestBlockEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        for res in &self.harvested_items {
            cleanup_item_stack(state, res);
        }
    }
}

impl CleanupEvent for wit_event::PlayerHideEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerItemBreakEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerItemMendEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerKickEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerLeashEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerLevelChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerLocaleChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerNameEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_text_component(state, &self.name);
    }
}

impl CleanupEvent for wit_event::PlayerOpenSignEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerPortalEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerPreLoginEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_text_component(state, &self.kick_message);
    }
}

impl CleanupEvent for wit_event::PlayerRiptideEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerShearEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerShowEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerSpawnChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerStatisticIncrementEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerSwapHandsEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerTakeLecternBookEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
        cleanup_item_stack(state, &self.book);
    }
}

impl CleanupEvent for wit_event::PlayerUnleashEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerVelocityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerInputEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerInteractAtEntityEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerLinksSendEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerPickupArrowEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerRecipeBookClickEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerRecipeBookSettingsChangeEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerRecipeDiscoverEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerRegisterChannelEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerResourcePackStatusEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerSpawnLocationEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::PlayerUnregisterChannelEvent {
    fn cleanup_event(&self, state: &mut PluginHostState) {
        cleanup_player(state, &self.player);
    }
}

impl CleanupEvent for wit_event::RaidFinishEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::RaidSpawnWaveEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::RaidStopEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::RaidTriggerEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}

impl CleanupEvent for wit_event::LightningStrikeEvent {
    fn cleanup_event(&self, _state: &mut PluginHostState) {}
}
