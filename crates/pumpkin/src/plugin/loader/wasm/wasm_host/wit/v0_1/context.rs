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
                context::{Context, MarketplaceMetadata},
                event::{EventPriority, EventType},
                permission::{Permission, PermissionDefault, PermissionLevel},
                server::Server,
            },
        },
    },
};

macro_rules! register_host_event {
    ($resource:expr, $handler:expr, $priority:expr, $blocking:expr, $event_ty:ty) => {
        $resource.provider.register_event::<$event_ty, _>(
            Arc::clone($handler),
            $priority,
            $blocking,
        )
    };
}

fn register_typed_event<E: crate::plugin::Payload + ToFromWasmEvent + 'static>(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
) {
    register_host_event!(resource, handler, priority, blocking, E);
}

#[expect(clippy::too_many_lines)]
fn register_player_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::player::{
        bedrock_form_response::BedrockFormResponseEvent,
        changed_main_hand::PlayerChangedMainHandEvent, egg_throw::PlayerEggThrowEvent,
        exp_change::PlayerExpChangeEvent, fish::PlayerFishEvent,
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
            register_typed_event::<PlayerJoinEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerLeaveEvent => {
            register_typed_event::<PlayerLeaveEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerLoginEvent => {
            register_typed_event::<PlayerLoginEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerChatEvent => {
            register_typed_event::<PlayerChatEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerCommandSendEvent => {
            register_typed_event::<PlayerCommandSendEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerPermissionCheckEvent => {
            register_typed_event::<PlayerPermissionCheckEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PlayerMoveEvent => {
            register_typed_event::<PlayerMoveEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerTeleportEvent => {
            register_typed_event::<PlayerTeleportEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerChangeWorldEvent => {
            register_typed_event::<PlayerChangeWorldEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerRespawnEvent => {
            register_typed_event::<PlayerRespawnEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerExpChangeEvent => {
            register_typed_event::<PlayerExpChangeEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerItemHeldEvent => {
            register_typed_event::<PlayerItemHeldEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerChangedMainHandEvent => {
            register_typed_event::<PlayerChangedMainHandEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PlayerGamemodeChangeEvent => {
            register_typed_event::<PlayerGamemodeChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PlayerCustomPayloadEvent => {
            register_typed_event::<PlayerCustomPayloadEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerFishEvent => {
            register_typed_event::<PlayerFishEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerEggThrowEvent => {
            register_typed_event::<PlayerEggThrowEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerInteractUnknownEntityEvent => {
            register_typed_event::<PlayerInteractUnknownEntityEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PlayerInteractEntityEvent => {
            register_typed_event::<PlayerInteractEntityEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PlayerInteractEvent => {
            register_typed_event::<PlayerInteractEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerToggleSneakEvent => {
            register_typed_event::<PlayerToggleSneakEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerToggleFlightEvent => {
            register_typed_event::<PlayerToggleFlightEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerToggleSprintEvent => {
            register_typed_event::<PlayerToggleSprintEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryClickEvent => {
            register_typed_event::<InventoryClickEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryCloseEvent => {
            register_typed_event::<InventoryCloseEvent>(resource, handler, priority, blocking);
        }
        EventType::BedrockFormResponseEvent => {
            register_typed_event::<BedrockFormResponseEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerItemConsumeEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_item_consume::PlayerItemConsumeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerItemDamageEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_item_damage::PlayerItemDamageEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerDropItemEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_drop_item::PlayerDropItemEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerBedEnterEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_bed::PlayerBedEnterEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerBedLeaveEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_bed::PlayerBedLeaveEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerBucketEmptyEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_bucket::PlayerBucketEmptyEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerBucketFillEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_bucket::PlayerBucketFillEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::AsyncPlayerChatEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::async_player_chat::AsyncPlayerChatEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::AsyncPlayerPreLoginEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::async_player_pre_login::AsyncPlayerPreLoginEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerAdvancementDoneEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_advancement_done::PlayerAdvancementDoneEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerAnimationEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_animation::PlayerAnimationEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerArmorStandManipulateEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_armor_stand_manipulate::PlayerArmorStandManipulateEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerBucketEntityEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_bucket_entity::PlayerBucketEntityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerChangedWorldEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_changed_world::PlayerChangedWorldEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerChannelEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_channel::PlayerChannelEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerCommandPreprocessEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_command_preprocess::PlayerCommandPreprocessEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerEditBookEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_edit_book::PlayerEditBookEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerElytraBoostEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_elytra_boost::PlayerElytraBoostEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerExpCooldownChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_exp_cooldown_change::PlayerExpCooldownChangeEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerHarvestBlockEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_harvest_block::PlayerHarvestBlockEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerHideEntityEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_hide_entity::PlayerHideEntityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerItemBreakEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_item_break::PlayerItemBreakEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerItemMendEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_item_mend::PlayerItemMendEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerKickEvent => {
            register_typed_event::<crate::plugin::api::events::player::player_kick::PlayerKickEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PlayerLeashEntityEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_leash_entity::PlayerLeashEntityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerLevelChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_level_change::PlayerLevelChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerLocaleChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_locale_change::PlayerLocaleChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerNameEntityEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_name_entity::PlayerNameEntityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerOpenSignEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_open_sign::PlayerOpenSignEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerPortalEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_portal::PlayerPortalEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerPreLoginEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_pre_login::PlayerPreLoginEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerRiptideEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_riptide::PlayerRiptideEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerShearEntityEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerShowEntityEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_show_entity::PlayerShowEntityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerSpawnChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_spawn_change::PlayerSpawnChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerStatisticIncrementEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerSwapHandsEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_swap_hands::PlayerSwapHandItemsEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerTakeLecternBookEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_take_lectern_book::PlayerTakeLecternBookEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerUnleashEntityEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_unleash_entity::PlayerUnleashEntityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerVelocityEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_velocity::PlayerVelocityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerInputEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_input::PlayerInputEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerInteractAtEntityEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_interact_at_entity::PlayerInteractAtEntityEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerLinksSendEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_links_send::PlayerLinksSendEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerPickupArrowEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_pickup_arrow::PlayerPickupArrowEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerRecipeBookClickEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_recipe_book_click::PlayerRecipeBookClickEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerRecipeBookSettingsChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_recipe_book_settings_change::PlayerRecipeBookSettingsChangeEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerRecipeDiscoverEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_recipe_discover::PlayerRecipeDiscoverEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerRegisterChannelEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_register_channel::PlayerRegisterChannelEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerResourcePackStatusEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_resource_pack_status::PlayerResourcePackStatusEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::PlayerSpawnLocationEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_spawn_location::PlayerSpawnLocationEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PlayerUnregisterChannelEvent => {
            register_typed_event::<
                crate::plugin::api::events::player::player_unregister_channel::PlayerUnregisterChannelEvent,
            >(resource, handler, priority, blocking)
            ;
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

#[allow(clippy::too_many_lines)]
fn register_entity_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::entity::{
        area_effect_cloud_apply::AreaEffectCloudApplyEvent,
        arrow_body_count_change::ArrowBodyCountChangeEvent,
        bat_toggle_sleep::BatToggleSleepEvent,
        creature_spawn::CreatureSpawnEvent,
        creeper_power::CreeperPowerEvent,
        ender_dragon_change_phase::EnderDragonChangePhaseEvent,
        entity_air_change::EntityAirChangeEvent,
        entity_break_door::EntityBreakDoorEvent,
        entity_breed::EntityBreedEvent,
        entity_change_block::EntityChangeBlockEvent,
        entity_combust::EntityCombustEvent,
        entity_combust_by_block::EntityCombustByBlockEvent,
        entity_combust_by_entity::EntityCombustByEntityEvent,
        entity_damage::EntityDamageEvent,
        entity_damage_by_block::EntityDamageByBlockEvent,
        entity_damage_by_entity::EntityDamageByEntityEvent,
        entity_death::{EntityDeathEvent, PlayerDeathEvent},
        entity_dismount::EntityDismountEvent,
        entity_drop_item::EntityDropItemEvent,
        entity_dye::EntityDyeEvent,
        entity_enter_block::EntityEnterBlockEvent,
        entity_enter_love_mode::EntityEnterLoveModeEvent,
        entity_exhaustion::EntityExhaustionEvent,
        entity_explode::EntityExplodeEvent,
        entity_interact::EntityInteractEvent,
        entity_knockback::EntityKnockbackEvent,
        entity_knockback_by_entity::EntityKnockbackByEntityEvent,
        entity_mount::EntityMountEvent,
        entity_pickup_item::EntityPickupItemEvent,
        entity_place::EntityPlaceEvent,
        entity_portal::EntityPortalEvent,
        entity_portal_enter::EntityPortalEnterEvent,
        entity_portal_exit::EntityPortalExitEvent,
        entity_pose_change::EntityPoseChangeEvent,
        entity_potion_effect::EntityPotionEffectEvent,
        entity_regain_health::EntityRegainHealthEvent,
        entity_remove::EntityRemoveEvent,
        entity_resurrect::EntityResurrectEvent,
        entity_shoot_bow::EntityShootBowEvent,
        entity_spawn::EntitySpawnEvent,
        entity_spell_cast::EntitySpellCastEvent,
        entity_tame::EntityTameEvent,
        entity_target::EntityTargetEvent,
        entity_target_block::EntityTargetBlockEvent,
        entity_target_living_entity::EntityTargetLivingEntityEvent,
        entity_teleport::EntityTeleportEvent,
        entity_toggle_glide::EntityToggleGlideEvent,
        entity_toggle_swim::EntityToggleSwimEvent,
        entity_transform::EntityTransformEvent,
        entity_unleash::EntityUnleashEvent,
        exp_bottle::ExpBottleEvent,
        explosion_prime::ExplosionPrimeEvent,
        firework_explode::FireworkExplodeEvent,
        food_level_change::FoodLevelChangeEvent,
        horse_jump::HorseJumpEvent,
        item_despawn::ItemDespawnEvent,
        item_merge::ItemMergeEvent,
        item_spawn::ItemSpawnEvent,
        lingering_potion_splash::LingeringPotionSplashEvent,
        pig_zap::PigZapEvent,
        pig_zombie_anger::PigZombieAngerEvent,
        piglin_barter::PiglinBarterEvent,
        potion_splash::PotionSplashEvent,
        projectile_hit::ProjectileHitEvent,
        projectile_launch::ProjectileLaunchEvent,
        sheep_dye_wool::SheepDyeWoolEvent,
        sheep_regrow_wool::SheepRegrowWoolEvent,
        slime_split::SlimeSplitEvent,
        spawner_spawn::SpawnerSpawnEvent,
        strider_temperature_change::StriderTemperatureChangeEvent,
        trial_spawner_spawn::TrialSpawnerSpawnEvent,
        villager_acquire_trade::VillagerAcquireTradeEvent,
        villager_career_change::VillagerCareerChangeEvent,
        villager_replenish_trade::VillagerReplenishTradeEvent,
        villager_reputation_change::VillagerReputationChangeEvent,
        warden_anger_change::WardenAngerChangeEvent,
    };

    match event_type {
        EventType::EntityDamageEvent => {
            register_typed_event::<EntityDamageEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityDeathEvent => {
            register_typed_event::<EntityDeathEvent>(resource, handler, priority, blocking);
        }
        EventType::PlayerDeathEvent => {
            register_typed_event::<PlayerDeathEvent>(resource, handler, priority, blocking);
        }
        EventType::EntitySpawnEvent => {
            register_typed_event::<EntitySpawnEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityCombustEvent => {
            register_typed_event::<EntityCombustEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityRegainHealthEvent => {
            register_typed_event::<EntityRegainHealthEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityAirChangeEvent => {
            register_typed_event::<EntityAirChangeEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityBreedEvent => {
            register_typed_event::<EntityBreedEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityDismountEvent => {
            register_typed_event::<EntityDismountEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityDyeEvent => {
            register_typed_event::<EntityDyeEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityEnterLoveModeEvent => {
            register_typed_event::<EntityEnterLoveModeEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityExplodeEvent => {
            register_typed_event::<EntityExplodeEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityMountEvent => {
            register_typed_event::<EntityMountEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityPickupItemEvent => {
            register_typed_event::<EntityPickupItemEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityPortalEvent => {
            register_typed_event::<EntityPortalEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityResurrectEvent => {
            register_typed_event::<EntityResurrectEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityShootBowEvent => {
            register_typed_event::<EntityShootBowEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityTameEvent => {
            register_typed_event::<EntityTameEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityTargetEvent => {
            register_typed_event::<EntityTargetEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityTeleportEvent => {
            register_typed_event::<EntityTeleportEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityToggleGlideEvent => {
            register_typed_event::<EntityToggleGlideEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityTransformEvent => {
            register_typed_event::<EntityTransformEvent>(resource, handler, priority, blocking);
        }
        EventType::CreatureSpawnEvent => {
            register_typed_event::<CreatureSpawnEvent>(resource, handler, priority, blocking);
        }
        EventType::EnderDragonChangePhaseEvent => {
            register_typed_event::<EnderDragonChangePhaseEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntityBreakDoorEvent => {
            register_typed_event::<EntityBreakDoorEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityChangeBlockEvent => {
            register_typed_event::<EntityChangeBlockEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityDamageByBlockEvent => {
            register_typed_event::<EntityDamageByBlockEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityDamageByEntityEvent => {
            register_typed_event::<EntityDamageByEntityEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntityDropItemEvent => {
            register_typed_event::<EntityDropItemEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityEnterBlockEvent => {
            register_typed_event::<EntityEnterBlockEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityExhaustionEvent => {
            register_typed_event::<EntityExhaustionEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityInteractEvent => {
            register_typed_event::<EntityInteractEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityKnockbackEvent => {
            register_typed_event::<EntityKnockbackEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityPlaceEvent => {
            register_typed_event::<EntityPlaceEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityPoseChangeEvent => {
            register_typed_event::<EntityPoseChangeEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityPotionEffectEvent => {
            register_typed_event::<EntityPotionEffectEvent>(resource, handler, priority, blocking);
        }
        EventType::EntitySpellCastEvent => {
            register_typed_event::<EntitySpellCastEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityTargetLivingEntityEvent => {
            register_typed_event::<EntityTargetLivingEntityEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntityToggleSwimEvent => {
            register_typed_event::<EntityToggleSwimEvent>(resource, handler, priority, blocking);
        }
        EventType::ExplosionPrimeEvent => {
            register_typed_event::<ExplosionPrimeEvent>(resource, handler, priority, blocking);
        }
        EventType::FireworkExplodeEvent => {
            register_typed_event::<FireworkExplodeEvent>(resource, handler, priority, blocking);
        }
        EventType::FoodLevelChangeEvent => {
            register_typed_event::<FoodLevelChangeEvent>(resource, handler, priority, blocking);
        }
        EventType::ItemDespawnEvent => {
            register_typed_event::<ItemDespawnEvent>(resource, handler, priority, blocking);
        }
        EventType::ItemMergeEvent => {
            register_typed_event::<ItemMergeEvent>(resource, handler, priority, blocking);
        }
        EventType::ItemSpawnEvent => {
            register_typed_event::<ItemSpawnEvent>(resource, handler, priority, blocking);
        }
        EventType::PiglinBarterEvent => {
            register_typed_event::<PiglinBarterEvent>(resource, handler, priority, blocking);
        }
        EventType::ProjectileHitEvent => {
            register_typed_event::<ProjectileHitEvent>(resource, handler, priority, blocking);
        }
        EventType::ProjectileLaunchEvent => {
            register_typed_event::<ProjectileLaunchEvent>(resource, handler, priority, blocking);
        }
        EventType::SheepDyeWoolEvent => {
            register_typed_event::<SheepDyeWoolEvent>(resource, handler, priority, blocking);
        }
        EventType::SheepRegrowWoolEvent => {
            register_typed_event::<SheepRegrowWoolEvent>(resource, handler, priority, blocking);
        }
        EventType::SlimeSplitEvent => {
            register_typed_event::<SlimeSplitEvent>(resource, handler, priority, blocking);
        }
        EventType::StriderTemperatureChangeEvent => {
            register_typed_event::<StriderTemperatureChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::VillagerAcquireTradeEvent => {
            register_typed_event::<VillagerAcquireTradeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::VillagerCareerChangeEvent => {
            register_typed_event::<VillagerCareerChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::VillagerReplenishTradeEvent => {
            register_typed_event::<VillagerReplenishTradeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::WardenAngerChangeEvent => {
            register_typed_event::<WardenAngerChangeEvent>(resource, handler, priority, blocking);
        }
        EventType::AreaEffectCloudApplyEvent => {
            register_typed_event::<AreaEffectCloudApplyEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::ArrowBodyCountChangeEvent => {
            register_typed_event::<ArrowBodyCountChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BatToggleSleepEvent => {
            register_typed_event::<BatToggleSleepEvent>(resource, handler, priority, blocking);
        }
        EventType::CreeperPowerEvent => {
            register_typed_event::<CreeperPowerEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityCombustByBlockEvent => {
            register_typed_event::<EntityCombustByBlockEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntityCombustByEntityEvent => {
            register_typed_event::<EntityCombustByEntityEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntityKnockbackByEntityEvent => {
            register_typed_event::<EntityKnockbackByEntityEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntityPortalEnterEvent => {
            register_typed_event::<EntityPortalEnterEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityPortalExitEvent => {
            register_typed_event::<EntityPortalExitEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityRemoveEvent => {
            register_typed_event::<EntityRemoveEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityTargetBlockEvent => {
            register_typed_event::<EntityTargetBlockEvent>(resource, handler, priority, blocking);
        }
        EventType::EntityUnleashEvent => {
            register_typed_event::<EntityUnleashEvent>(resource, handler, priority, blocking);
        }
        EventType::ExpBottleEvent => {
            register_typed_event::<ExpBottleEvent>(resource, handler, priority, blocking);
        }
        EventType::HorseJumpEvent => {
            register_typed_event::<HorseJumpEvent>(resource, handler, priority, blocking);
        }
        EventType::LingeringPotionSplashEvent => {
            register_typed_event::<LingeringPotionSplashEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PigZapEvent => {
            register_typed_event::<PigZapEvent>(resource, handler, priority, blocking);
        }
        EventType::PigZombieAngerEvent => {
            register_typed_event::<PigZombieAngerEvent>(resource, handler, priority, blocking);
        }
        EventType::PotionSplashEvent => {
            register_typed_event::<PotionSplashEvent>(resource, handler, priority, blocking);
        }
        EventType::SpawnerSpawnEvent => {
            register_typed_event::<SpawnerSpawnEvent>(resource, handler, priority, blocking);
        }
        EventType::TrialSpawnerSpawnEvent => {
            register_typed_event::<TrialSpawnerSpawnEvent>(resource, handler, priority, blocking);
        }
        EventType::VillagerReputationChangeEvent => {
            register_typed_event::<VillagerReputationChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        _ => {
            tracing::error!("non-entity event should not be routed to register_entity_event");
        }
    }
}

fn register_inventory_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::inventory::{
        brew::BrewEvent, brewing_stand_fuel::BrewingStandFuelEvent, craft_item::CraftItemEvent,
        furnace_burn::FurnaceBurnEvent, furnace_extract::FurnaceExtractEvent,
        furnace_smelt::FurnaceSmeltEvent, furnace_start_smelt::FurnaceStartSmeltEvent,
        hopper_inventory_search::HopperInventorySearchEvent,
        inventory_creative::InventoryCreativeEvent, inventory_drag::InventoryDragEvent,
        inventory_interact::InventoryInteractEvent, inventory_move_item::InventoryMoveItemEvent,
        inventory_open::InventoryOpenEvent, inventory_pickup_item::InventoryPickupItemEvent,
        prepare_anvil::PrepareAnvilEvent, prepare_grindstone::PrepareGrindstoneEvent,
        prepare_inventory_result::PrepareInventoryResultEvent,
        prepare_item_craft::PrepareItemCraftEvent, prepare_smithing::PrepareSmithingEvent,
        smith_item::SmithItemEvent, trade_select::TradeSelectEvent,
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
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::vehicle::{
        vehicle_block_collision::VehicleBlockCollisionEvent,
        vehicle_collision::VehicleCollisionEvent, vehicle_create::VehicleCreateEvent,
        vehicle_damage::VehicleDamageEvent, vehicle_destroy::VehicleDestroyEvent,
        vehicle_enter::VehicleEnterEvent, vehicle_entity_collision::VehicleEntityCollisionEvent,
        vehicle_exit::VehicleExitEvent, vehicle_move::VehicleMoveEvent,
        vehicle_update::VehicleUpdateEvent,
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
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::enchantment::{
        enchant_item::EnchantItemEvent, prepare_item_enchant::PrepareItemEnchantEvent,
    };

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
            register_typed_event::<SpawnChangeEvent>(resource, handler, priority, blocking);
        }
        EventType::ChunkLoadEvent => {
            register_typed_event::<ChunkLoad>(resource, handler, priority, blocking);
        }
        EventType::ChunkSaveEvent => {
            register_typed_event::<ChunkSave>(resource, handler, priority, blocking);
        }
        EventType::ChunkSendEvent => {
            register_typed_event::<ChunkSend>(resource, handler, priority, blocking);
        }
        EventType::WeatherChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::weather_change::WeatherChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::ThunderChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::weather_change::ThunderChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::WorldLoadEvent => {
            register_typed_event::<crate::plugin::api::events::world::world_load::WorldLoadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::WorldUnloadEvent => {
            register_typed_event::<crate::plugin::api::events::world::world_load::WorldUnloadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::AsyncStructureGenerateEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::async_structure_generate::AsyncStructureGenerateEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::AsyncStructureSpawnEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::async_structure_spawn::AsyncStructureSpawnEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::ChunkPopulateEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::chunk_populate::ChunkPopulateEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::ChunkUnloadEvent => {
            register_typed_event::<crate::plugin::api::events::world::chunk_unload::ChunkUnloadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::EntitiesLoadEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::entities_load::EntitiesLoadEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::EntitiesUnloadEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::entities_unload::EntitiesUnloadEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::GenericGameEvent => {
            register_typed_event::<crate::plugin::api::events::world::generic_game::GenericGameEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::LootGenerateEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::loot_generate::LootGenerateEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PortalCreateEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::portal_create::PortalCreateEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::StructureGrowEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::structure_grow::StructureGrowEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::TimeSkipEvent => {
            register_typed_event::<crate::plugin::api::events::world::time_skip::TimeSkipEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::WorldInitEvent => {
            register_typed_event::<crate::plugin::api::events::world::world_init::WorldInitEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::WorldSaveEvent => {
            register_typed_event::<crate::plugin::api::events::world::world_save::WorldSaveEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::LightningStrikeEvent => {
            register_typed_event::<
                crate::plugin::api::events::world::lightning_strike::LightningStrikeEvent,
            >(resource, handler, priority, blocking);
        }
        _ => {
            tracing::error!("non-world event should not be routed to register_world_event");
        }
    }
}

#[allow(clippy::too_many_lines)]
fn register_block_event(
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
            register_typed_event::<crate::plugin::api::events::block::block_damage::BlockDamageEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockIgniteEvent => {
            register_typed_event::<crate::plugin::api::events::block::block_ignite::BlockIgniteEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockFromToEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_from_to::BlockFromToEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockFormEvent => {
            register_typed_event::<crate::plugin::api::events::block::block_form::BlockFormEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockFadeEvent => {
            register_typed_event::<crate::plugin::api::events::block::block_fade::BlockFadeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockDispenseEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_dispense::BlockDispenseEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockExplodeEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_explode::BlockExplodeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockPhysicsEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_physics::BlockPhysicsEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockPistonExtendEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_piston::BlockPistonExtendEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockPistonRetractEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_piston::BlockPistonRetractEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::NotePlayEvent => {
            register_typed_event::<crate::plugin::api::events::block::note_play::NotePlayEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::SignChangeEvent => {
            register_typed_event::<crate::plugin::api::events::block::sign_change::SignChangeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::SpongeAbsorbEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::sponge_absorb::SpongeAbsorbEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::TntPrimeEvent => {
            register_typed_event::<crate::plugin::api::events::block::tnt_prime::TNTPrimeEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BellResonateEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::bell_resonate::BellResonateEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BellRingEvent => {
            register_typed_event::<crate::plugin::api::events::block::bell_ring::BellRingEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockBrushEvent => {
            register_typed_event::<crate::plugin::api::events::block::block_brush::BlockBrushEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockCookEvent => {
            register_typed_event::<crate::plugin::api::events::block::block_cook::BlockCookEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockDamageAbortEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_damage_abort::BlockDamageAbortEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockDispenseArmorEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_dispense_armor::BlockDispenseArmorEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockDispenseLootEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_dispense_loot::BlockDispenseLootEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockDropItemEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_drop_item::BlockDropItemEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockExpEvent => {
            register_typed_event::<crate::plugin::api::events::block::block_exp::BlockExpEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BlockFertilizeEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_fertilize::BlockFertilizeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockMultiPlaceEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_multi_place::BlockMultiPlaceEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockReceiveGameEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_receive_game::BlockReceiveGameEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockShearEntityEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::block_shear_entity::BlockShearEntityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockSpreadEvent => {
            register_typed_event::<crate::plugin::api::events::block::block_spread::BlockSpreadEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::BrewingStartEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::brewing_start::BrewingStartEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::CampfireStartEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::campfire_start::CampfireStartEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::CauldronLevelChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::cauldron_level_change::CauldronLevelChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::CrafterCraftEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::crafter_craft::CrafterCraftEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::EntityBlockFormEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::entity_block_form::EntityBlockFormEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::FluidLevelChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::fluid_level_change::FluidLevelChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::InventoryBlockStartEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::inventory_block_start::InventoryBlockStartEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::LeavesDecayEvent => {
            register_typed_event::<crate::plugin::api::events::block::leaves_decay::LeavesDecayEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::MoistureChangeEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::moisture_change::MoistureChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::SculkBloomEvent => {
            register_typed_event::<crate::plugin::api::events::block::sculk_bloom::SculkBloomEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::VaultDisplayItemEvent => {
            register_typed_event::<
                crate::plugin::api::events::block::vault_display_item::VaultDisplayItemEvent,
            >(resource, handler, priority, blocking);
        }

        _ => {
            tracing::error!("non-block event should not be routed to register_block_event");
        }
    }
}

fn register_raid_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::raid::{
        raid_finish::RaidFinishEvent, raid_spawn_wave::RaidSpawnWaveEvent,
        raid_stop::RaidStopEvent, raid_trigger::RaidTriggerEvent,
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
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::api::events::dialog::{
        DialogClearEvent, DialogClickActionEvent, DialogShowEvent,
    };

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
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::server::{
        list_ping::ServerListPingEvent,
        packet::{PacketReceivedEvent, PacketSentEvent},
        server_broadcast::ServerBroadcastEvent,
        server_command::ServerCommandEvent,
        server_load::ServerLoadEvent,
        server_tick_end::ServerTickEndEvent,
        server_tick_start::ServerTickStartEvent,
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
            register_typed_event::<
                crate::plugin::api::events::server::map_initialize::MapInitializeEvent,
            >(resource, handler, priority, blocking);
        }
        _ => {
            tracing::error!("non-server event should not be routed to register_server_event");
        }
    }
}

fn register_hanging_event(
    resource: &ContextResource,
    handler: &Arc<WasmPluginEventHandler>,
    priority: crate::plugin::EventPriority,
    blocking: bool,
    event_type: EventType,
) {
    use crate::plugin::hanging::{
        hanging_break::HangingBreakEvent, hanging_break_by_entity::HangingBreakByEntityEvent,
        hanging_place::HangingPlaceEvent,
    };

    match event_type {
        EventType::HangingBreakEvent => {
            register_typed_event::<HangingBreakEvent>(resource, handler, priority, blocking);
        }
        EventType::HangingBreakByEntityEvent => {
            register_typed_event::<HangingBreakByEntityEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::HangingPlaceEvent => {
            register_typed_event::<HangingPlaceEvent>(resource, handler, priority, blocking);
        }
        _ => {
            tracing::error!("non-hanging event should not be routed to register_hanging_event");
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
    #[allow(clippy::too_many_lines)]
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
            | EventType::ServerListPingEvent
            | EventType::ServerBroadcastEvent
            | EventType::ServerLoadEvent
            | EventType::ServerTickEndEvent
            | EventType::ServerTickStartEvent
            | EventType::MapInitializeEvent) => {
                register_server_event(resource, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::HangingBreakEvent
            | EventType::HangingBreakByEntityEvent
            | EventType::HangingPlaceEvent) => {
                register_hanging_event(resource, &handler, priority, blocking, event_type);
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
                register_world_event(resource, &handler, priority, blocking, event_type);
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
                register_entity_event(resource, &handler, priority, blocking, event_type);
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
                register_block_event(resource, &handler, priority, blocking, event_type);
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
                register_inventory_event(resource, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::PrepareItemEnchantEvent | EventType::EnchantItemEvent) => {
                register_enchantment_event(resource, &handler, priority, blocking, event_type);
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
                register_vehicle_event(resource, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::RaidFinishEvent
            | EventType::RaidSpawnWaveEvent
            | EventType::RaidStopEvent
            | EventType::RaidTriggerEvent) => {
                register_raid_event(resource, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::DialogClickActionEvent
            | EventType::DialogShowEvent
            | EventType::DialogClearEvent) => {
                register_dialog_event(resource, &handler, priority, blocking, event_type);
            }
            event_type => {
                register_player_event(resource, &handler, priority, blocking, event_type);
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
            .register_command(command_res.provider, permission);
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
        Ok(context_res.provider.register_permission(util_permission))
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

    async fn get_marketplace_metadata(
        &mut self,
        _context: Resource<Context>,
    ) -> wasmtime::Result<Option<MarketplaceMetadata>> {
        Ok(self.marketplace_metadata.clone())
    }

    async fn drop(&mut self, rep: Resource<Context>) -> wasmtime::Result<()> {
        let _ = self
            .resource_table
            .delete::<ContextResource>(Resource::new_own(rep.rep()));
        Ok(())
    }
}
