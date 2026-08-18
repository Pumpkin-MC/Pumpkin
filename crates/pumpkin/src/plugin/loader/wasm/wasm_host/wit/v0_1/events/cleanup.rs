use crate::plugin::loader::wasm::wasm_host::{
    state::{
        EntityResource, ItemStackResource, PlayerResource, PluginHostState, ServerResource,
        TextComponentResource, WorldResource,
    },
    wit::v0_1::pumpkin::plugin::{
        entity::Entity, event::Event, item_stack::ItemStack, player::Player, server::Server,
        text::TextComponent, world::World,
    },
};
use wasmtime::component::Resource;

pub fn cleanup_player(state: &mut PluginHostState, player: &Resource<Player>) {
    let _ = state
        .resource_table
        .delete::<PlayerResource>(Resource::new_own(player.rep()));
}

pub fn cleanup_world(state: &mut PluginHostState, world: &Resource<World>) {
    let _ = state
        .resource_table
        .delete::<WorldResource>(Resource::new_own(world.rep()));
}

pub fn cleanup_text_component(
    state: &mut PluginHostState,
    text_component: &Resource<TextComponent>,
) {
    let _ = state
        .resource_table
        .delete::<TextComponentResource>(Resource::new_own(text_component.rep()));
}

pub fn cleanup_item_stack(state: &mut PluginHostState, item: &Resource<ItemStack>) {
    let _ = state
        .resource_table
        .delete::<ItemStackResource>(Resource::new_own(item.rep()));
}

pub fn cleanup_entity(state: &mut PluginHostState, entity: &Resource<Entity>) {
    let _ = state
        .resource_table
        .delete::<EntityResource>(Resource::new_own(entity.rep()));
}

pub fn cleanup_server(state: &mut PluginHostState, server: &Resource<Server>) {
    let _ = state
        .resource_table
        .delete::<ServerResource>(Resource::new_own(server.rep()));
}

#[allow(clippy::too_many_lines, clippy::match_same_arms)]
pub fn cleanup_event(event: &Event, state: &mut PluginHostState) {
    match event {
        Event::PlayerJoinEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.join_message);
        }
        Event::PlayerLeaveEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.leave_message);
        }
        Event::PlayerLoginEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.kick_message);
        }
        Event::PlayerChatEvent(data) => {
            cleanup_player(state, &data.player);
            for res in &data.recipients {
                cleanup_player(state, res);
            }
        }
        Event::PlayerCommandSendEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerPermissionCheckEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerMoveEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerTeleportEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerChangeWorldEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.previous_world);
            cleanup_world(state, &data.new_world);
        }
        Event::PlayerRespawnEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.previous_world);
            cleanup_world(state, &data.respawned_world);
        }
        Event::PlayerExpChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerItemHeldEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerChangedMainHandEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerGamemodeChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerCustomPayloadEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerFishEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerEggThrowEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerInteractUnknownEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerInteractEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerInteractEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerToggleSneakEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerToggleFlightEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerToggleSprintEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::InventoryClickEvent(data) => {
            cleanup_player(state, &data.player);
            if let Some(res) = &data.clicked_item {
                cleanup_item_stack(state, res);
            }
            if let Some(res) = &data.cursor {
                cleanup_item_stack(state, res);
            }
        }
        Event::InventoryCloseEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::BlockRedstoneEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::BlockBreakEvent(data) => {
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        Event::BlockBurnEvent(_) => {}
        Event::BlockCanBuildEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::BlockGrowEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::BlockPlaceEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::BedrockFormResponseEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::CustomClickActionEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::ServerCommandEvent(_) => {}
        Event::ServerListPingEvent(data) => {
            cleanup_text_component(state, &data.motd);
        }
        Event::ServerLoadEvent(_) => {}
        Event::SpawnChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::ServerBroadcastEvent(data) => {
            cleanup_text_component(state, &data.message);
            cleanup_text_component(state, &data.sender);
        }
        Event::ServerTickStartEvent(_) => {}
        Event::ServerTickEndEvent(_) => {}
        Event::PacketReceivedEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PacketSentEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::ChunkLoadEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::ChunkSaveEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::ChunkSendEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::EntityDamageEvent(_) => {}
        Event::EntityDeathEvent(_) => {}
        Event::PlayerDeathEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.death_message);
        }
        Event::EntitySpawnEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::EntityCombustEvent(_) => {}
        Event::EntityRegainHealthEvent(_) => {}
        Event::EntityAirChangeEvent(_) => {}
        Event::EntityBreedEvent(_) => {}
        Event::EntityDismountEvent(_) => {}
        Event::EntityDyeEvent(data) => {
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        Event::EntityEnterLoveModeEvent(_) => {}
        Event::EntityExplodeEvent(_) => {}
        Event::EntityMountEvent(_) => {}
        Event::EntityPickupItemEvent(_) => {}
        Event::EntityPortalEvent(_) => {}
        Event::EntityResurrectEvent(_) => {}
        Event::EntityShootBowEvent(_) => {}
        Event::EntityTameEvent(data) => {
            cleanup_player(state, &data.owner);
        }
        Event::EntityTargetEvent(_) => {}
        Event::EntityTeleportEvent(_) => {}
        Event::EntityToggleGlideEvent(_) => {}
        Event::EntityTransformEvent(_) => {}
        Event::PlayerItemConsumeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerItemDamageEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerDropItemEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerBedEnterEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerBedLeaveEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerBucketEmptyEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerBucketFillEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::BlockDamageEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::BlockIgniteEvent(_) => {}
        Event::BlockFromToEvent(_) => {}
        Event::BlockFormEvent(_) => {}
        Event::BlockFadeEvent(_) => {}
        Event::BlockDispenseEvent(_) => {}
        Event::BlockExplodeEvent(_) => {}
        Event::BlockPhysicsEvent(_) => {}
        Event::BlockPistonExtendEvent(_) => {}
        Event::BlockPistonRetractEvent(_) => {}
        Event::NotePlayEvent(_) => {}
        Event::SignChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::SpongeAbsorbEvent(_) => {}
        Event::TntPrimeEvent(_) => {}
        Event::WeatherChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::ThunderChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::WorldLoadEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::WorldUnloadEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::AsyncStructureGenerateEvent(_) => {}
        Event::AsyncStructureSpawnEvent(_) => {}
        Event::ChunkPopulateEvent(_) => {}
        Event::ChunkUnloadEvent(_) => {}
        Event::EntitiesLoadEvent(_) => {}
        Event::EntitiesUnloadEvent(_) => {}
        Event::GenericGameEvent(_) => {}
        Event::LootGenerateEvent(_) => {}
        Event::PortalCreateEvent(_) => {}
        Event::StructureGrowEvent(_) => {}
        Event::TimeSkipEvent(_) => {}
        Event::WorldInitEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::WorldSaveEvent(_) => {}
        Event::InventoryOpenEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::InventoryDragEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::CraftItemEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::FurnaceSmeltEvent(_) => {}
        Event::BrewEvent(_) => {}
        Event::BrewingStandFuelEvent(_) => {}
        Event::FurnaceBurnEvent(_) => {}
        Event::FurnaceExtractEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::FurnaceStartSmeltEvent(_) => {}
        Event::HopperInventorySearchEvent(_) => {}
        Event::InventoryCreativeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::InventoryInteractEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::InventoryMoveItemEvent(_) => {}
        Event::InventoryPickupItemEvent(_) => {}
        Event::PrepareAnvilEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PrepareGrindstoneEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PrepareInventoryResultEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PrepareItemCraftEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PrepareSmithingEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::SmithItemEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::TradeSelectEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::VehicleBlockCollisionEvent(_) => {}
        Event::VehicleCollisionEvent(_) => {}
        Event::VehicleCreateEvent(_) => {}
        Event::VehicleDamageEvent(_) => {}
        Event::VehicleDestroyEvent(_) => {}
        Event::VehicleEnterEvent(_) => {}
        Event::VehicleEntityCollisionEvent(_) => {}
        Event::VehicleExitEvent(_) => {}
        Event::VehicleMoveEvent(_) => {}
        Event::VehicleUpdateEvent(_) => {}
        Event::PrepareItemEnchantEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_item_stack(state, &data.item);
        }
        Event::EnchantItemEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_item_stack(state, &data.item);
        }
        Event::MapInitializeEvent(_) => {}
        Event::HangingBreakEvent(_) => {}
        Event::HangingBreakByEntityEvent(_) => {}
        Event::HangingPlaceEvent(data) => {
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        Event::BellResonateEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::BellRingEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::BlockBrushEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_player(state, &data.player);
            cleanup_item_stack(state, &data.item);
        }
        Event::BlockCookEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.source);
            cleanup_item_stack(state, &data.result);
        }
        Event::BlockDamageAbortEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.item_in_hand);
        }
        Event::BlockDispenseArmorEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.item);
        }
        Event::BlockDispenseLootEvent(data) => {
            cleanup_world(state, &data.target_world);
            for res in &data.items {
                cleanup_item_stack(state, res);
            }
        }
        Event::BlockDropItemEvent(data) => {
            cleanup_world(state, &data.target_world);
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
            for res in &data.items {
                cleanup_item_stack(state, res);
            }
        }
        Event::BlockExpEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::BlockFertilizeEvent(data) => {
            cleanup_world(state, &data.target_world);
            if let Some(res) = &data.player {
                cleanup_player(state, res);
            }
        }
        Event::BlockMultiPlaceEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.target_world);
        }
        Event::BlockReceiveGameEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::BlockShearEntityEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.item);
        }
        Event::BlockSpreadEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::BrewingStartEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::CampfireStartEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.item);
        }
        Event::CauldronLevelChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::CrafterCraftEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.result);
        }
        Event::EntityBlockFormEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::FluidLevelChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::InventoryBlockStartEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::LeavesDecayEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::MoistureChangeEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::SculkBloomEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::VaultDisplayItemEvent(data) => {
            cleanup_world(state, &data.target_world);
            cleanup_item_stack(state, &data.item);
        }
        Event::CreatureSpawnEvent(data) => {
            cleanup_world(state, &data.target_world);
        }
        Event::EnderDragonChangePhaseEvent(_) => {}
        Event::EntityBreakDoorEvent(_) => {}
        Event::EntityChangeBlockEvent(_) => {}
        Event::EntityDamageByBlockEvent(_) => {}
        Event::EntityDamageByEntityEvent(_) => {}
        Event::EntityDropItemEvent(_) => {}
        Event::EntityEnterBlockEvent(_) => {}
        Event::EntityExhaustionEvent(_) => {}
        Event::EntityInteractEvent(_) => {}
        Event::EntityKnockbackEvent(_) => {}
        Event::EntityPlaceEvent(_) => {}
        Event::EntityPoseChangeEvent(_) => {}
        Event::EntityPotionEffectEvent(_) => {}
        Event::EntitySpellCastEvent(_) => {}
        Event::EntityTargetLivingEntityEvent(_) => {}
        Event::EntityToggleSwimEvent(_) => {}
        Event::ExplosionPrimeEvent(_) => {}
        Event::FireworkExplodeEvent(_) => {}
        Event::FoodLevelChangeEvent(_) => {}
        Event::ItemDespawnEvent(_) => {}
        Event::ItemMergeEvent(_) => {}
        Event::ItemSpawnEvent(_) => {}
        Event::PiglinBarterEvent(data) => {
            cleanup_item_stack(state, &data.input_item);
            for res in &data.outcome {
                cleanup_item_stack(state, res);
            }
        }
        Event::ProjectileHitEvent(_) => {}
        Event::ProjectileLaunchEvent(_) => {}
        Event::SheepDyeWoolEvent(_) => {}
        Event::SheepRegrowWoolEvent(_) => {}
        Event::SlimeSplitEvent(_) => {}
        Event::StriderTemperatureChangeEvent(_) => {}
        Event::VillagerAcquireTradeEvent(_) => {}
        Event::VillagerCareerChangeEvent(_) => {}
        Event::VillagerReplenishTradeEvent(_) => {}
        Event::WardenAngerChangeEvent(_) => {}
        Event::AreaEffectCloudApplyEvent(_) => {}
        Event::ArrowBodyCountChangeEvent(_) => {}
        Event::BatToggleSleepEvent(_) => {}
        Event::CreeperPowerEvent(_) => {}
        Event::EntityCombustByBlockEvent(_) => {}
        Event::EntityCombustByEntityEvent(_) => {}
        Event::EntityKnockbackByEntityEvent(_) => {}
        Event::EntityPortalEnterEvent(_) => {}
        Event::EntityPortalExitEvent(_) => {}
        Event::EntityRemoveEvent(_) => {}
        Event::EntityTargetBlockEvent(_) => {}
        Event::EntityUnleashEvent(_) => {}
        Event::ExpBottleEvent(_) => {}
        Event::HorseJumpEvent(_) => {}
        Event::LingeringPotionSplashEvent(_) => {}
        Event::PigZapEvent(_) => {}
        Event::PigZombieAngerEvent(_) => {}
        Event::PotionSplashEvent(_) => {}
        Event::SpawnerSpawnEvent(_) => {}
        Event::TrialSpawnerSpawnEvent(_) => {}
        Event::VillagerReputationChangeEvent(_) => {}
        Event::AsyncPlayerChatEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.format);
        }
        Event::AsyncPlayerPreLoginEvent(data) => {
            cleanup_text_component(state, &data.kick_message);
        }
        Event::PlayerAdvancementDoneEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerAnimationEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerArmorStandManipulateEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerBucketEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerChangedWorldEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_world(state, &data.from_world);
            cleanup_world(state, &data.to_world);
        }
        Event::PlayerChannelEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerCommandPreprocessEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerEditBookEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerElytraBoostEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerExpCooldownChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerHarvestBlockEvent(data) => {
            cleanup_player(state, &data.player);
            for res in &data.harvested_items {
                cleanup_item_stack(state, res);
            }
        }
        Event::PlayerHideEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerItemBreakEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerItemMendEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerKickEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerLeashEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerLevelChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerLocaleChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerNameEntityEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_text_component(state, &data.name);
        }
        Event::PlayerOpenSignEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerPortalEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerPreLoginEvent(data) => {
            cleanup_text_component(state, &data.kick_message);
        }
        Event::PlayerRiptideEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerShearEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerShowEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerSpawnChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerStatisticIncrementEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerSwapHandsEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerTakeLecternBookEvent(data) => {
            cleanup_player(state, &data.player);
            cleanup_item_stack(state, &data.book);
        }
        Event::PlayerUnleashEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerVelocityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerInputEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerInteractAtEntityEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerLinksSendEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerPickupArrowEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerRecipeBookClickEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerRecipeBookSettingsChangeEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerRecipeDiscoverEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerRegisterChannelEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerResourcePackStatusEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerSpawnLocationEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::PlayerUnregisterChannelEvent(data) => {
            cleanup_player(state, &data.player);
        }
        Event::RaidFinishEvent(_) => {}
        Event::RaidSpawnWaveEvent(_) => {}
        Event::RaidStopEvent(_) => {}
        Event::RaidTriggerEvent(_) => {}
        Event::LightningStrikeEvent(_) => {}
    }
}
