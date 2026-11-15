use std::{collections::HashMap, sync::Arc};
use wasmtime::component::Resource;

use crate::{
    events::{ToFromWasmEvent, WasmPluginEventHandler},
    pumpkin::{
        self,
        plugin::{
            command::Command,
            context::{Context as WitContext, MarketplaceMetadata},
            event::{EventPriority, EventType},
            permission::{Permission, PermissionDefault, PermissionLevel},
            server::Server,
        },
    },
};
use pumpkin_core::plugin::Context;
use pumpkin_wasm_host_common::state::PluginHostState;

fn register_typed_event<E: pumpkin_core::plugin::Payload + ToFromWasmEvent + Clone + 'static>(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
) {
    ctx.register_event::<E, _>(Arc::clone(handler), priority, blocking);
}

#[expect(clippy::too_many_lines)]
fn register_player_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{
        BedrockFormResponseEvent, InventoryClickEvent, InventoryCloseEvent, PlayerChangeWorldEvent,
        PlayerChangedMainHandEvent, PlayerChatEvent, PlayerCommandSendEvent,
        PlayerCustomPayloadEvent, PlayerEggThrowEvent, PlayerExpChangeEvent, PlayerFishEvent,
        PlayerGamemodeChangeEvent, PlayerInteractEntityEvent, PlayerInteractEvent,
        PlayerInteractUnknownEntityEvent, PlayerItemHeldEvent, PlayerJoinEvent, PlayerLeaveEvent,
        PlayerLoginEvent, PlayerMoveEvent, PlayerPermissionCheckEvent, PlayerRespawnEvent,
        PlayerTeleportEvent, PlayerToggleFlightEvent, PlayerToggleSneakEvent,
        PlayerToggleSprintEvent,
    };

    match event_type {
        EventType::PlayerJoinEvent => {
            register_typed_event::<PlayerJoinEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerLeaveEvent => {
            register_typed_event::<PlayerLeaveEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerLoginEvent => {
            register_typed_event::<PlayerLoginEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerChatEvent => {
            register_typed_event::<PlayerChatEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerCommandSendEvent => {
            register_typed_event::<PlayerCommandSendEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerPermissionCheckEvent => {
            register_typed_event::<PlayerPermissionCheckEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerMoveEvent => {
            register_typed_event::<PlayerMoveEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerTeleportEvent => {
            register_typed_event::<PlayerTeleportEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerChangeWorldEvent => {
            register_typed_event::<PlayerChangeWorldEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerRespawnEvent => {
            register_typed_event::<PlayerRespawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerExpChangeEvent => {
            register_typed_event::<PlayerExpChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerItemHeldEvent => {
            register_typed_event::<PlayerItemHeldEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerChangedMainHandEvent => {
            register_typed_event::<PlayerChangedMainHandEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerGamemodeChangeEvent => {
            register_typed_event::<PlayerGamemodeChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerCustomPayloadEvent => {
            register_typed_event::<PlayerCustomPayloadEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerFishEvent => {
            register_typed_event::<PlayerFishEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerEggThrowEvent => {
            register_typed_event::<PlayerEggThrowEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerInteractUnknownEntityEvent => {
            register_typed_event::<PlayerInteractUnknownEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerInteractEntityEvent => {
            register_typed_event::<PlayerInteractEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerInteractEvent => {
            register_typed_event::<PlayerInteractEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerToggleSneakEvent => {
            register_typed_event::<PlayerToggleSneakEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerToggleFlightEvent => {
            register_typed_event::<PlayerToggleFlightEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerToggleSprintEvent => {
            register_typed_event::<PlayerToggleSprintEvent>(ctx, handler, priority, blocking);
        }
        EventType::InventoryClickEvent => {
            register_typed_event::<InventoryClickEvent>(ctx, handler, priority, blocking);
        }
        EventType::InventoryCloseEvent => {
            register_typed_event::<InventoryCloseEvent>(ctx, handler, priority, blocking);
        }
        EventType::BedrockFormResponseEvent => {
            register_typed_event::<BedrockFormResponseEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerItemConsumeEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerItemConsumeEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerItemDamageEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerItemDamageEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerDropItemEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerDropItemEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerBedEnterEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerBedEnterEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerBedLeaveEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerBedLeaveEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerBucketEmptyEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerBucketEmptyEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerBucketFillEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerBucketFillEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::AsyncPlayerChatEvent => {
            register_typed_event::<pumpkin_core::plugin::AsyncPlayerChatEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::AsyncPlayerPreLoginEvent => {
            register_typed_event::<pumpkin_core::plugin::AsyncPlayerPreLoginEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerAdvancementDoneEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerAdvancementDoneEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerAnimationEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerAnimationEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerArmorStandManipulateEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerArmorStandManipulateEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerBucketEntityEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerBucketEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerChangedWorldEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerChangedWorldEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerChannelEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerChannelEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerCommandPreprocessEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerCommandPreprocessEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerEditBookEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerEditBookEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerElytraBoostEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerElytraBoostEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerExpCooldownChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerExpCooldownChangeEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerHarvestBlockEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerHarvestBlockEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerHideEntityEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerHideEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerItemBreakEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerItemBreakEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerItemMendEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerItemMendEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerKickEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerKickEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerLeashEntityEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerLeashEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerLevelChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerLevelChangeEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerLocaleChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerLocaleChangeEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerNameEntityEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerNameEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerOpenSignEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerOpenSignEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerPortalEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerPortalEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerPreLoginEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerPreLoginEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerRiptideEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerRiptideEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerShearEntityEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerShearEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerShowEntityEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerShowEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerSpawnChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerSpawnChangeEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerStatisticIncrementEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerStatisticIncrementEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerSwapHandsEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerSwapHandsEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerTakeLecternBookEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerTakeLecternBookEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerUnleashEntityEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerUnleashEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerVelocityEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerVelocityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerInputEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerInputEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerInteractAtEntityEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerInteractAtEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerLinksSendEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerLinksSendEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerPickupArrowEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerPickupArrowEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerRecipeBookClickEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerRecipeBookClickEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerRecipeBookSettingsChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerRecipeBookSettingsChangeEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerRecipeDiscoverEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerRecipeDiscoverEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerRegisterChannelEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerRegisterChannelEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerResourcePackStatusEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerResourcePackStatusEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerSpawnLocationEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerSpawnLocationEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerUnregisterChannelEvent => {
            register_typed_event::<pumpkin_core::plugin::PlayerUnregisterChannelEvent>(
                ctx, handler, priority, blocking,
            );
        }
        _ => {
            tracing::error!("non-player event should not be routed to register_player_event");
        }
    }
}

#[allow(clippy::too_many_lines)]
fn register_entity_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{
        AreaEffectCloudApplyEvent, ArrowBodyCountChangeEvent, BatToggleSleepEvent,
        CreatureSpawnEvent, CreeperPowerEvent, EnderDragonChangePhaseEvent, EntityAirChangeEvent,
        EntityBreakDoorEvent, EntityBreedEvent, EntityChangeBlockEvent, EntityCombustByBlockEvent,
        EntityCombustByEntityEvent, EntityCombustEvent, EntityDamageByBlockEvent,
        EntityDamageByEntityEvent, EntityDamageEvent, EntityDeathEvent, EntityDismountEvent,
        EntityDropItemEvent, EntityDyeEvent, EntityEnterBlockEvent, EntityEnterLoveModeEvent,
        EntityExhaustionEvent, EntityExplodeEvent, EntityInteractEvent,
        EntityKnockbackByEntityEvent, EntityKnockbackEvent, EntityMountEvent,
        EntityPickupItemEvent, EntityPlaceEvent, EntityPortalEnterEvent, EntityPortalEvent,
        EntityPortalExitEvent, EntityPoseChangeEvent, EntityPotionEffectEvent,
        EntityRegainHealthEvent, EntityRemoveEvent, EntityResurrectEvent, EntityShootBowEvent,
        EntitySpawnEvent, EntitySpellCastEvent, EntityTameEvent, EntityTargetBlockEvent,
        EntityTargetEvent, EntityTargetLivingEntityEvent, EntityTeleportEvent,
        EntityToggleGlideEvent, EntityToggleSwimEvent, EntityTransformEvent, EntityUnleashEvent,
        ExpBottleEvent, ExplosionPrimeEvent, FireworkExplodeEvent, FoodLevelChangeEvent,
        HorseJumpEvent, ItemDespawnEvent, ItemMergeEvent, ItemSpawnEvent,
        LingeringPotionSplashEvent, PigZapEvent, PigZombieAngerEvent, PiglinBarterEvent,
        PlayerDeathEvent, PotionSplashEvent, ProjectileHitEvent, ProjectileLaunchEvent,
        SheepDyeWoolEvent, SheepRegrowWoolEvent, SlimeSplitEvent, SpawnerSpawnEvent,
        StriderTemperatureChangeEvent, TrialSpawnerSpawnEvent, VillagerAcquireTradeEvent,
        VillagerCareerChangeEvent, VillagerReplenishTradeEvent, VillagerReputationChangeEvent,
        WardenAngerChangeEvent,
    };

    match event_type {
        EventType::EntityDamageEvent => {
            register_typed_event::<EntityDamageEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDeathEvent => {
            register_typed_event::<EntityDeathEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerDeathEvent => {
            register_typed_event::<PlayerDeathEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntitySpawnEvent => {
            register_typed_event::<EntitySpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityCombustEvent => {
            register_typed_event::<EntityCombustEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityRegainHealthEvent => {
            register_typed_event::<EntityRegainHealthEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityAirChangeEvent => {
            register_typed_event::<EntityAirChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityBreedEvent => {
            register_typed_event::<EntityBreedEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDismountEvent => {
            register_typed_event::<EntityDismountEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDyeEvent => {
            register_typed_event::<EntityDyeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityEnterLoveModeEvent => {
            register_typed_event::<EntityEnterLoveModeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityExplodeEvent => {
            register_typed_event::<EntityExplodeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityMountEvent => {
            register_typed_event::<EntityMountEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPickupItemEvent => {
            register_typed_event::<EntityPickupItemEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPortalEvent => {
            register_typed_event::<EntityPortalEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityResurrectEvent => {
            register_typed_event::<EntityResurrectEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityShootBowEvent => {
            register_typed_event::<EntityShootBowEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTameEvent => {
            register_typed_event::<EntityTameEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTargetEvent => {
            register_typed_event::<EntityTargetEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTeleportEvent => {
            register_typed_event::<EntityTeleportEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityToggleGlideEvent => {
            register_typed_event::<EntityToggleGlideEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTransformEvent => {
            register_typed_event::<EntityTransformEvent>(ctx, handler, priority, blocking);
        }
        EventType::CreatureSpawnEvent => {
            register_typed_event::<CreatureSpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::EnderDragonChangePhaseEvent => {
            register_typed_event::<EnderDragonChangePhaseEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityBreakDoorEvent => {
            register_typed_event::<EntityBreakDoorEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityChangeBlockEvent => {
            register_typed_event::<EntityChangeBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDamageByBlockEvent => {
            register_typed_event::<EntityDamageByBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDamageByEntityEvent => {
            register_typed_event::<EntityDamageByEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDropItemEvent => {
            register_typed_event::<EntityDropItemEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityEnterBlockEvent => {
            register_typed_event::<EntityEnterBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityExhaustionEvent => {
            register_typed_event::<EntityExhaustionEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityInteractEvent => {
            register_typed_event::<EntityInteractEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityKnockbackEvent => {
            register_typed_event::<EntityKnockbackEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPlaceEvent => {
            register_typed_event::<EntityPlaceEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPoseChangeEvent => {
            register_typed_event::<EntityPoseChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPotionEffectEvent => {
            register_typed_event::<EntityPotionEffectEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntitySpellCastEvent => {
            register_typed_event::<EntitySpellCastEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTargetLivingEntityEvent => {
            register_typed_event::<EntityTargetLivingEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityToggleSwimEvent => {
            register_typed_event::<EntityToggleSwimEvent>(ctx, handler, priority, blocking);
        }
        EventType::ExplosionPrimeEvent => {
            register_typed_event::<ExplosionPrimeEvent>(ctx, handler, priority, blocking);
        }
        EventType::FireworkExplodeEvent => {
            register_typed_event::<FireworkExplodeEvent>(ctx, handler, priority, blocking);
        }
        EventType::FoodLevelChangeEvent => {
            register_typed_event::<FoodLevelChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::ItemDespawnEvent => {
            register_typed_event::<ItemDespawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::ItemMergeEvent => {
            register_typed_event::<ItemMergeEvent>(ctx, handler, priority, blocking);
        }
        EventType::ItemSpawnEvent => {
            register_typed_event::<ItemSpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::PiglinBarterEvent => {
            register_typed_event::<PiglinBarterEvent>(ctx, handler, priority, blocking);
        }
        EventType::ProjectileHitEvent => {
            register_typed_event::<ProjectileHitEvent>(ctx, handler, priority, blocking);
        }
        EventType::ProjectileLaunchEvent => {
            register_typed_event::<ProjectileLaunchEvent>(ctx, handler, priority, blocking);
        }
        EventType::SheepDyeWoolEvent => {
            register_typed_event::<SheepDyeWoolEvent>(ctx, handler, priority, blocking);
        }
        EventType::SheepRegrowWoolEvent => {
            register_typed_event::<SheepRegrowWoolEvent>(ctx, handler, priority, blocking);
        }
        EventType::SlimeSplitEvent => {
            register_typed_event::<SlimeSplitEvent>(ctx, handler, priority, blocking);
        }
        EventType::StriderTemperatureChangeEvent => {
            register_typed_event::<StriderTemperatureChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::VillagerAcquireTradeEvent => {
            register_typed_event::<VillagerAcquireTradeEvent>(ctx, handler, priority, blocking);
        }
        EventType::VillagerCareerChangeEvent => {
            register_typed_event::<VillagerCareerChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::VillagerReplenishTradeEvent => {
            register_typed_event::<VillagerReplenishTradeEvent>(ctx, handler, priority, blocking);
        }
        EventType::WardenAngerChangeEvent => {
            register_typed_event::<WardenAngerChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::AreaEffectCloudApplyEvent => {
            register_typed_event::<AreaEffectCloudApplyEvent>(ctx, handler, priority, blocking);
        }
        EventType::ArrowBodyCountChangeEvent => {
            register_typed_event::<ArrowBodyCountChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::BatToggleSleepEvent => {
            register_typed_event::<BatToggleSleepEvent>(ctx, handler, priority, blocking);
        }
        EventType::CreeperPowerEvent => {
            register_typed_event::<CreeperPowerEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityCombustByBlockEvent => {
            register_typed_event::<EntityCombustByBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityCombustByEntityEvent => {
            register_typed_event::<EntityCombustByEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityKnockbackByEntityEvent => {
            register_typed_event::<EntityKnockbackByEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPortalEnterEvent => {
            register_typed_event::<EntityPortalEnterEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPortalExitEvent => {
            register_typed_event::<EntityPortalExitEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityRemoveEvent => {
            register_typed_event::<EntityRemoveEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTargetBlockEvent => {
            register_typed_event::<EntityTargetBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityUnleashEvent => {
            register_typed_event::<EntityUnleashEvent>(ctx, handler, priority, blocking);
        }
        EventType::ExpBottleEvent => {
            register_typed_event::<ExpBottleEvent>(ctx, handler, priority, blocking);
        }
        EventType::HorseJumpEvent => {
            register_typed_event::<HorseJumpEvent>(ctx, handler, priority, blocking);
        }
        EventType::LingeringPotionSplashEvent => {
            register_typed_event::<LingeringPotionSplashEvent>(ctx, handler, priority, blocking);
        }
        EventType::PigZapEvent => {
            register_typed_event::<PigZapEvent>(ctx, handler, priority, blocking);
        }
        EventType::PigZombieAngerEvent => {
            register_typed_event::<PigZombieAngerEvent>(ctx, handler, priority, blocking);
        }
        EventType::PotionSplashEvent => {
            register_typed_event::<PotionSplashEvent>(ctx, handler, priority, blocking);
        }
        EventType::SpawnerSpawnEvent => {
            register_typed_event::<SpawnerSpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::TrialSpawnerSpawnEvent => {
            register_typed_event::<TrialSpawnerSpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::VillagerReputationChangeEvent => {
            register_typed_event::<VillagerReputationChangeEvent>(ctx, handler, priority, blocking);
        }
        _ => {
            tracing::error!("non-entity event should not be routed to register_entity_event");
        }
    }
}

fn register_inventory_event(
    resource: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{
        BrewEvent, BrewingStandFuelEvent, CraftItemEvent, FurnaceBurnEvent, FurnaceExtractEvent,
        FurnaceSmeltEvent, FurnaceStartSmeltEvent, HopperInventorySearchEvent,
        InventoryCreativeEvent, InventoryDragEvent, InventoryInteractEvent, InventoryMoveItemEvent,
        InventoryOpenEvent, InventoryPickupItemEvent, PrepareAnvilEvent, PrepareGrindstoneEvent,
        PrepareInventoryResultEvent, PrepareItemCraftEvent, PrepareSmithingEvent, SmithItemEvent,
        TradeSelectEvent,
    };

    match event_type {
        EventType::InventoryOpenEvent => {
            register_typed_event::<InventoryOpenEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryDragEvent => {
            register_typed_event::<InventoryDragEvent>(resource, handler, priority, blocking);
        }
        EventType::CraftItemEvent => {
            register_typed_event::<CraftItemEvent>(resource, handler, priority, blocking);
        }
        EventType::FurnaceSmeltEvent => {
            register_typed_event::<FurnaceSmeltEvent>(resource, handler, priority, blocking);
        }
        EventType::BrewEvent => {
            register_typed_event::<BrewEvent>(resource, handler, priority, blocking);
        }
        EventType::BrewingStandFuelEvent => {
            register_typed_event::<BrewingStandFuelEvent>(resource, handler, priority, blocking);
        }
        EventType::FurnaceBurnEvent => {
            register_typed_event::<FurnaceBurnEvent>(resource, handler, priority, blocking);
        }
        EventType::FurnaceExtractEvent => {
            register_typed_event::<FurnaceExtractEvent>(resource, handler, priority, blocking);
        }
        EventType::FurnaceStartSmeltEvent => {
            register_typed_event::<FurnaceStartSmeltEvent>(resource, handler, priority, blocking);
        }
        EventType::HopperInventorySearchEvent => {
            register_typed_event::<HopperInventorySearchEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::InventoryCreativeEvent => {
            register_typed_event::<InventoryCreativeEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryInteractEvent => {
            register_typed_event::<InventoryInteractEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryMoveItemEvent => {
            register_typed_event::<InventoryMoveItemEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryPickupItemEvent => {
            register_typed_event::<InventoryPickupItemEvent>(resource, handler, priority, blocking);
        }
        EventType::PrepareAnvilEvent => {
            register_typed_event::<PrepareAnvilEvent>(resource, handler, priority, blocking);
        }
        EventType::PrepareGrindstoneEvent => {
            register_typed_event::<PrepareGrindstoneEvent>(resource, handler, priority, blocking);
        }
        EventType::PrepareInventoryResultEvent => {
            register_typed_event::<PrepareInventoryResultEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PrepareItemCraftEvent => {
            register_typed_event::<PrepareItemCraftEvent>(resource, handler, priority, blocking);
        }
        EventType::PrepareSmithingEvent => {
            register_typed_event::<PrepareSmithingEvent>(resource, handler, priority, blocking);
        }
        EventType::SmithItemEvent => {
            register_typed_event::<SmithItemEvent>(resource, handler, priority, blocking);
        }
        EventType::TradeSelectEvent => {
            register_typed_event::<TradeSelectEvent>(resource, handler, priority, blocking);
        }
        _ => {
            tracing::error!("non-inventory event should not be routed to register_inventory_event");
        }
    }
}

fn register_vehicle_event(
    resource: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{
        VehicleBlockCollisionEvent, VehicleCollisionEvent, VehicleCreateEvent, VehicleDamageEvent,
        VehicleDestroyEvent, VehicleEnterEvent, VehicleEntityCollisionEvent, VehicleExitEvent,
        VehicleMoveEvent, VehicleUpdateEvent,
    };

    match event_type {
        EventType::VehicleBlockCollisionEvent => {
            register_typed_event::<VehicleBlockCollisionEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::VehicleCollisionEvent => {
            register_typed_event::<VehicleCollisionEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleCreateEvent => {
            register_typed_event::<VehicleCreateEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleDamageEvent => {
            register_typed_event::<VehicleDamageEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleDestroyEvent => {
            register_typed_event::<VehicleDestroyEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleEnterEvent => {
            register_typed_event::<VehicleEnterEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleEntityCollisionEvent => {
            register_typed_event::<VehicleEntityCollisionEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::VehicleExitEvent => {
            register_typed_event::<VehicleExitEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleMoveEvent => {
            register_typed_event::<VehicleMoveEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleUpdateEvent => {
            register_typed_event::<VehicleUpdateEvent>(resource, handler, priority, blocking);
        }
        _ => {
            tracing::error!("non-vehicle event should not be routed to register_vehicle_event");
        }
    }
}

fn register_enchantment_event(
    resource: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{EnchantItemEvent, PrepareItemEnchantEvent};

    match event_type {
        EventType::PrepareItemEnchantEvent => {
            register_typed_event::<PrepareItemEnchantEvent>(resource, handler, priority, blocking);
        }
        EventType::EnchantItemEvent => {
            register_typed_event::<EnchantItemEvent>(resource, handler, priority, blocking);
        }
        _ => {
            tracing::error!(
                "non-enchantment event should not be routed to register_enchantment_event"
            );
        }
    }
}

#[allow(clippy::too_many_lines)]
fn register_world_event(
    resource: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{ChunkLoadEvent, ChunkSaveEvent, ChunkSendEvent, SpawnChangeEvent};

    match event_type {
        EventType::SpawnChangeEvent => {
            register_typed_event::<SpawnChangeEvent>(resource, handler, priority, blocking);
        }
        EventType::ChunkLoadEvent => {
            register_typed_event::<ChunkLoadEvent>(resource, handler, priority, blocking);
        }
        EventType::ChunkSaveEvent => {
            register_typed_event::<ChunkSaveEvent>(resource, handler, priority, blocking);
        }
        EventType::ChunkSendEvent => {
            register_typed_event::<ChunkSendEvent>(resource, handler, priority, blocking);
        }
        EventType::WeatherChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::WeatherChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::ThunderChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::ThunderChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::WorldLoadEvent => {
            register_typed_event::<pumpkin_core::plugin::WorldLoadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::WorldUnloadEvent => {
            register_typed_event::<pumpkin_core::plugin::WorldUnloadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::AsyncStructureGenerateEvent => {
            register_typed_event::<pumpkin_core::plugin::AsyncStructureGenerateEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::AsyncStructureSpawnEvent => {
            register_typed_event::<pumpkin_core::plugin::AsyncStructureSpawnEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::ChunkPopulateEvent => {
            register_typed_event::<pumpkin_core::plugin::ChunkPopulateEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::ChunkUnloadEvent => {
            register_typed_event::<pumpkin_core::plugin::ChunkUnloadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntitiesLoadEvent => {
            register_typed_event::<pumpkin_core::plugin::EntitiesLoadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntitiesUnloadEvent => {
            register_typed_event::<pumpkin_core::plugin::EntitiesUnloadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::GenericGameEvent => {
            register_typed_event::<pumpkin_core::plugin::GenericGameEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::LootGenerateEvent => {
            register_typed_event::<pumpkin_core::plugin::LootGenerateEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PortalCreateEvent => {
            register_typed_event::<pumpkin_core::plugin::PortalCreateEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::StructureGrowEvent => {
            register_typed_event::<pumpkin_core::plugin::StructureGrowEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::TimeSkipEvent => {
            register_typed_event::<pumpkin_core::plugin::TimeSkipEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::WorldInitEvent => {
            register_typed_event::<pumpkin_core::plugin::WorldInitEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::WorldSaveEvent => {
            register_typed_event::<pumpkin_core::plugin::WorldSaveEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::LightningStrikeEvent => {
            register_typed_event::<pumpkin_core::plugin::LightningStrikeEvent>(
                resource, handler, priority, blocking,
            );
        }
        _ => {
            tracing::error!("non-world event should not be routed to register_world_event");
        }
    }
}

#[allow(clippy::too_many_lines)]
fn register_block_event(
    resource: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{
        BlockBreakEvent, BlockBurnEvent, BlockCanBuildEvent, BlockGrowEvent, BlockPlaceEvent,
        BlockRedstoneEvent,
    };

    match event_type {
        EventType::BlockRedstoneEvent => {
            register_typed_event::<BlockRedstoneEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockBreakEvent => {
            register_typed_event::<BlockBreakEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockBurnEvent => {
            register_typed_event::<BlockBurnEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockCanBuildEvent => {
            register_typed_event::<BlockCanBuildEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockGrowEvent => {
            register_typed_event::<BlockGrowEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockPlaceEvent => {
            register_typed_event::<BlockPlaceEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockDamageEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockDamageEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockIgniteEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockIgniteEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockFromToEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockFromToEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockFormEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockFormEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockFadeEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockFadeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockDispenseEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockDispenseEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockExplodeEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockExplodeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockPhysicsEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockPhysicsEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockPistonExtendEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockPistonExtendEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockPistonRetractEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockPistonRetractEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::NotePlayEvent => {
            register_typed_event::<pumpkin_core::plugin::NotePlayEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::SignChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::SignChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::SpongeAbsorbEvent => {
            register_typed_event::<pumpkin_core::plugin::SpongeAbsorbEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::TntPrimeEvent => {
            register_typed_event::<pumpkin_core::plugin::TntPrimeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BellResonateEvent => {
            register_typed_event::<pumpkin_core::plugin::BellResonateEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BellRingEvent => {
            register_typed_event::<pumpkin_core::plugin::BellRingEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockBrushEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockBrushEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockCookEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockCookEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockDamageAbortEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockDamageAbortEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockDispenseArmorEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockDispenseArmorEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockDispenseLootEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockDispenseLootEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockDropItemEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockDropItemEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockExpEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockExpEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockFertilizeEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockFertilizeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockMultiPlaceEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockMultiPlaceEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockReceiveGameEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockReceiveGameEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockShearEntityEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockShearEntityEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockSpreadEvent => {
            register_typed_event::<pumpkin_core::plugin::BlockSpreadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BrewingStartEvent => {
            register_typed_event::<pumpkin_core::plugin::BrewingStartEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::CampfireStartEvent => {
            register_typed_event::<pumpkin_core::plugin::CampfireStartEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::CauldronLevelChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::CauldronLevelChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::CrafterCraftEvent => {
            register_typed_event::<pumpkin_core::plugin::CrafterCraftEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntityBlockFormEvent => {
            register_typed_event::<pumpkin_core::plugin::EntityBlockFormEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::FluidLevelChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::FluidLevelChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::InventoryBlockStartEvent => {
            register_typed_event::<pumpkin_core::plugin::InventoryBlockStartEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::LeavesDecayEvent => {
            register_typed_event::<pumpkin_core::plugin::LeavesDecayEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::MoistureChangeEvent => {
            register_typed_event::<pumpkin_core::plugin::MoistureChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::SculkBloomEvent => {
            register_typed_event::<pumpkin_core::plugin::SculkBloomEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::VaultDisplayItemEvent => {
            register_typed_event::<pumpkin_core::plugin::VaultDisplayItemEvent>(
                resource, handler, priority, blocking,
            );
        }

        _ => {
            tracing::error!("non-block event should not be routed to register_block_event");
        }
    }
}

fn register_raid_event(
    resource: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{
        RaidFinishEvent, RaidSpawnWaveEvent, RaidStopEvent, RaidTriggerEvent,
    };

    match event_type {
        EventType::RaidFinishEvent => {
            register_typed_event::<RaidFinishEvent>(resource, handler, priority, blocking);
        }
        EventType::RaidSpawnWaveEvent => {
            register_typed_event::<RaidSpawnWaveEvent>(resource, handler, priority, blocking);
        }
        EventType::RaidStopEvent => {
            register_typed_event::<RaidStopEvent>(resource, handler, priority, blocking);
        }
        EventType::RaidTriggerEvent => {
            register_typed_event::<RaidTriggerEvent>(resource, handler, priority, blocking);
        }
        _ => {
            tracing::error!("non-raid event should not be routed to register_raid_event");
        }
    }
}

fn register_dialog_event(
    resource: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{DialogClearEvent, DialogClickActionEvent, DialogShowEvent};

    match event_type {
        EventType::DialogClickActionEvent => {
            register_typed_event::<DialogClickActionEvent>(resource, handler, priority, blocking);
        }
        EventType::DialogShowEvent => {
            register_typed_event::<DialogShowEvent>(resource, handler, priority, blocking);
        }
        EventType::DialogClearEvent => {
            register_typed_event::<DialogClearEvent>(resource, handler, priority, blocking);
        }
        _ => {
            tracing::error!("non-dialog event should not be routed to register_dialog_event");
        }
    }
}

fn register_server_event(
    resource: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{
        PacketReceivedEvent, PacketSentEvent, ServerBroadcastEvent, ServerCommandEvent,
        ServerListPingEvent, ServerLoadEvent, ServerTickEndEvent, ServerTickStartEvent,
    };

    match event_type {
        EventType::PacketReceivedEvent => {
            register_typed_event::<PacketReceivedEvent>(resource, handler, priority, blocking);
        }
        EventType::PacketSentEvent => {
            register_typed_event::<PacketSentEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerCommandEvent => {
            register_typed_event::<ServerCommandEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerListPingEvent => {
            register_typed_event::<ServerListPingEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerBroadcastEvent => {
            register_typed_event::<ServerBroadcastEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerLoadEvent => {
            register_typed_event::<ServerLoadEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerTickEndEvent => {
            register_typed_event::<ServerTickEndEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerTickStartEvent => {
            register_typed_event::<ServerTickStartEvent>(resource, handler, priority, blocking);
        }
        EventType::MapInitializeEvent => {
            register_typed_event::<pumpkin_core::plugin::MapInitializeEvent>(
                resource, handler, priority, blocking,
            );
        }
        _ => {
            tracing::error!("non-server event should not be routed to register_server_event");
        }
    }
}

fn register_hanging_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use pumpkin_core::plugin::{HangingBreakByEntityEvent, HangingBreakEvent, HangingPlaceEvent};

    match event_type {
        EventType::HangingBreakEvent => {
            register_typed_event::<HangingBreakEvent>(ctx, handler, priority, blocking);
        }
        EventType::HangingBreakByEntityEvent => {
            register_typed_event::<HangingBreakByEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::HangingPlaceEvent => {
            register_typed_event::<HangingPlaceEvent>(ctx, handler, priority, blocking);
        }
        _ => {
            tracing::error!("non-hanging event should not be routed to register_hanging_event");
        }
    }
}

impl pumpkin::plugin::context::Host for PluginHostState {}

impl pumpkin::plugin::context::HostContext for PluginHostState {
    #[allow(clippy::too_many_lines)]
    fn register_event(
        &mut self,
        context: Resource<WitContext>,
        handler_id: u32,
        event_type: EventType,
        event_priority: EventPriority,
        blocking: bool,
    ) -> wasmtime::Result<()> {
        // Updated return type
        let priority = match event_priority {
            EventPriority::Highest => pumpkin_core::plugin::EventPriority::Highest,
            EventPriority::High => pumpkin_core::plugin::EventPriority::High,
            EventPriority::Normal => pumpkin_core::plugin::EventPriority::Normal,
            EventPriority::Low => pumpkin_core::plugin::EventPriority::Low,
            EventPriority::Lowest => pumpkin_core::plugin::EventPriority::Lowest,
        };

        // Use ? to trap if the plugin was dropped or the context handle is dead
        let plugin = self
            .plugin
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("Plugin state uninitialized"))?
            .upgrade()
            .ok_or_else(|| wasmtime::Error::msg("Plugin has been dropped"))?;

        let ctx = self.get(&context)?.as_ref();
        let handler = Arc::new(WasmPluginEventHandler { handler_id, plugin });

        match event_type {
            event_type @ (EventType::PacketReceivedEvent
            | EventType::PacketSentEvent
            | EventType::ServerCommandEvent
            | EventType::ServerListPingEvent
            | EventType::ServerBroadcastEvent
            | EventType::ServerLoadEvent
            | EventType::ServerTickEndEvent
            | EventType::ServerTickStartEvent
            | EventType::MapInitializeEvent) => {
                register_server_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::HangingBreakEvent
            | EventType::HangingBreakByEntityEvent
            | EventType::HangingPlaceEvent) => {
                register_hanging_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::SpawnChangeEvent
            | EventType::ChunkLoadEvent
            | EventType::ChunkSaveEvent
            | EventType::ChunkSendEvent
            | EventType::WeatherChangeEvent
            | EventType::ThunderChangeEvent
            | EventType::WorldLoadEvent
            | EventType::WorldUnloadEvent
            | EventType::AsyncStructureGenerateEvent
            | EventType::AsyncStructureSpawnEvent
            | EventType::ChunkPopulateEvent
            | EventType::ChunkUnloadEvent
            | EventType::EntitiesLoadEvent
            | EventType::EntitiesUnloadEvent
            | EventType::GenericGameEvent
            | EventType::LootGenerateEvent
            | EventType::PortalCreateEvent
            | EventType::StructureGrowEvent
            | EventType::TimeSkipEvent
            | EventType::WorldInitEvent
            | EventType::WorldSaveEvent
            | EventType::LightningStrikeEvent) => {
                register_world_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::EntityDamageEvent
            | EventType::EntityDeathEvent
            | EventType::PlayerDeathEvent
            | EventType::EntitySpawnEvent
            | EventType::EntityCombustEvent
            | EventType::EntityRegainHealthEvent
            | EventType::EntityAirChangeEvent
            | EventType::EntityBreedEvent
            | EventType::EntityDismountEvent
            | EventType::EntityDyeEvent
            | EventType::EntityEnterLoveModeEvent
            | EventType::EntityExplodeEvent
            | EventType::EntityMountEvent
            | EventType::EntityPickupItemEvent
            | EventType::EntityPortalEvent
            | EventType::EntityResurrectEvent
            | EventType::EntityShootBowEvent
            | EventType::EntityTameEvent
            | EventType::EntityTargetEvent
            | EventType::EntityTeleportEvent
            | EventType::EntityToggleGlideEvent
            | EventType::EntityTransformEvent
            | EventType::CreatureSpawnEvent
            | EventType::EnderDragonChangePhaseEvent
            | EventType::EntityBreakDoorEvent
            | EventType::EntityChangeBlockEvent
            | EventType::EntityDamageByBlockEvent
            | EventType::EntityDamageByEntityEvent
            | EventType::EntityDropItemEvent
            | EventType::EntityEnterBlockEvent
            | EventType::EntityExhaustionEvent
            | EventType::EntityInteractEvent
            | EventType::EntityKnockbackEvent
            | EventType::EntityPlaceEvent
            | EventType::EntityPoseChangeEvent
            | EventType::EntityPotionEffectEvent
            | EventType::EntitySpellCastEvent
            | EventType::EntityTargetLivingEntityEvent
            | EventType::EntityToggleSwimEvent
            | EventType::ExplosionPrimeEvent
            | EventType::FireworkExplodeEvent
            | EventType::FoodLevelChangeEvent
            | EventType::ItemDespawnEvent
            | EventType::ItemMergeEvent
            | EventType::ItemSpawnEvent
            | EventType::PiglinBarterEvent
            | EventType::ProjectileHitEvent
            | EventType::ProjectileLaunchEvent
            | EventType::SheepDyeWoolEvent
            | EventType::SheepRegrowWoolEvent
            | EventType::SlimeSplitEvent
            | EventType::StriderTemperatureChangeEvent
            | EventType::VillagerAcquireTradeEvent
            | EventType::VillagerCareerChangeEvent
            | EventType::VillagerReplenishTradeEvent
            | EventType::WardenAngerChangeEvent
            | EventType::AreaEffectCloudApplyEvent
            | EventType::ArrowBodyCountChangeEvent
            | EventType::BatToggleSleepEvent
            | EventType::CreeperPowerEvent
            | EventType::EntityCombustByBlockEvent
            | EventType::EntityCombustByEntityEvent
            | EventType::EntityKnockbackByEntityEvent
            | EventType::EntityPortalEnterEvent
            | EventType::EntityPortalExitEvent
            | EventType::EntityRemoveEvent
            | EventType::EntityTargetBlockEvent
            | EventType::EntityUnleashEvent
            | EventType::ExpBottleEvent
            | EventType::HorseJumpEvent
            | EventType::LingeringPotionSplashEvent
            | EventType::PigZapEvent
            | EventType::PigZombieAngerEvent
            | EventType::PotionSplashEvent
            | EventType::SpawnerSpawnEvent
            | EventType::TrialSpawnerSpawnEvent
            | EventType::VillagerReputationChangeEvent) => {
                register_entity_event(ctx, &handler, priority, blocking, event_type);
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
            | EventType::BlockFadeEvent
            | EventType::BlockDispenseEvent
            | EventType::BlockExplodeEvent
            | EventType::BlockPhysicsEvent
            | EventType::BlockPistonExtendEvent
            | EventType::BlockPistonRetractEvent
            | EventType::NotePlayEvent
            | EventType::SignChangeEvent
            | EventType::SpongeAbsorbEvent
            | EventType::TntPrimeEvent
            | EventType::BellResonateEvent
            | EventType::BellRingEvent
            | EventType::BlockBrushEvent
            | EventType::BlockCookEvent
            | EventType::BlockDamageAbortEvent
            | EventType::BlockDispenseArmorEvent
            | EventType::BlockDispenseLootEvent
            | EventType::BlockDropItemEvent
            | EventType::BlockExpEvent
            | EventType::BlockFertilizeEvent
            | EventType::BlockMultiPlaceEvent
            | EventType::BlockReceiveGameEvent
            | EventType::BlockShearEntityEvent
            | EventType::BlockSpreadEvent
            | EventType::BrewingStartEvent
            | EventType::CampfireStartEvent
            | EventType::CauldronLevelChangeEvent
            | EventType::CrafterCraftEvent
            | EventType::EntityBlockFormEvent
            | EventType::FluidLevelChangeEvent
            | EventType::InventoryBlockStartEvent
            | EventType::LeavesDecayEvent
            | EventType::MoistureChangeEvent
            | EventType::SculkBloomEvent
            | EventType::VaultDisplayItemEvent) => {
                register_block_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::InventoryOpenEvent
            | EventType::InventoryDragEvent
            | EventType::CraftItemEvent
            | EventType::FurnaceSmeltEvent
            | EventType::BrewEvent
            | EventType::BrewingStandFuelEvent
            | EventType::FurnaceBurnEvent
            | EventType::FurnaceExtractEvent
            | EventType::FurnaceStartSmeltEvent
            | EventType::HopperInventorySearchEvent
            | EventType::InventoryCreativeEvent
            | EventType::InventoryInteractEvent
            | EventType::InventoryMoveItemEvent
            | EventType::InventoryPickupItemEvent
            | EventType::PrepareAnvilEvent
            | EventType::PrepareGrindstoneEvent
            | EventType::PrepareInventoryResultEvent
            | EventType::PrepareItemCraftEvent
            | EventType::PrepareSmithingEvent
            | EventType::SmithItemEvent
            | EventType::TradeSelectEvent) => {
                register_inventory_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::PrepareItemEnchantEvent | EventType::EnchantItemEvent) => {
                register_enchantment_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::VehicleBlockCollisionEvent
            | EventType::VehicleCollisionEvent
            | EventType::VehicleCreateEvent
            | EventType::VehicleDamageEvent
            | EventType::VehicleDestroyEvent
            | EventType::VehicleEnterEvent
            | EventType::VehicleEntityCollisionEvent
            | EventType::VehicleExitEvent
            | EventType::VehicleMoveEvent
            | EventType::VehicleUpdateEvent) => {
                register_vehicle_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::RaidFinishEvent
            | EventType::RaidSpawnWaveEvent
            | EventType::RaidStopEvent
            | EventType::RaidTriggerEvent) => {
                register_raid_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::DialogClickActionEvent
            | EventType::DialogShowEvent
            | EventType::DialogClearEvent) => {
                register_dialog_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type => {
                register_player_event(ctx, &handler, priority, blocking, event_type);
            }
        }

        Ok(())
    }

    fn register_command(
        &mut self,
        context: Resource<WitContext>,
        command: Resource<Command>,
        permission: String,
    ) -> wasmtime::Result<()> {
        use pumpkin_core::command::argument_builder::ArgumentBuilder;

        let command = self.take(command)?;
        let context = self.get(&context)?.clone();
        let aliases = if command.names.len() > 1 {
            command.names[1..].to_vec()
        } else {
            Vec::new()
        };
        let node = command.builder.build();
        context.register_command_with_aliases(node, &aliases, permission);

        Ok(())
    }

    fn register_permission(
        &mut self,
        context: Resource<WitContext>,
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

        let context_res = self.get(&context)?;
        Ok(context_res.register_permission(util_permission))
    }

    fn get_data_folder(&mut self, _context: Resource<WitContext>) -> wasmtime::Result<String> {
        Ok("data".to_string())
    }

    fn get_server(&mut self, context: Resource<WitContext>) -> wasmtime::Result<Resource<Server>> {
        self.add(self.get(&context)?.server.clone())
    }

    fn get_marketplace_metadata(
        &mut self,
        _context: Resource<WitContext>,
    ) -> wasmtime::Result<Option<MarketplaceMetadata>> {
        Ok(self
            .marketplace_metadata
            .clone()
            .map(|metadata| MarketplaceMetadata {
                marketplace_url: metadata.marketplace_url,
                plugin_id: metadata.plugin_id,
                plugin_name: metadata.plugin_name,
                version: metadata.version,
                dev_id: metadata.dev_id,
                dev_name: metadata.dev_name,
                is_paid: metadata.is_paid,
                user_id: metadata.user_id,
                license_key: metadata.license_key,
                issued_at: metadata.issued_at,
            }))
    }

    fn drop(&mut self, rep: Resource<WitContext>) -> wasmtime::Result<()> {
        self.drop(rep)
    }
}
