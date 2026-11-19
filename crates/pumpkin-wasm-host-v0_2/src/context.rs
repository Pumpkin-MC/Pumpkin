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

fn register_typed_event<E: pumpkin_core::plugin::Payload + ToFromWasmEvent + 'static>(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
) {
    ctx.register_event_dyn(E::get_name_static(), Arc::clone(handler));
}

#[expect(clippy::too_many_lines)]
fn register_player_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::player::{
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
        player_toggle_sprint_event::PlayerToggleSprintEvent, player_transfer::PlayerTransferEvent,
    };

    match event_type {
        EventType::PlayerJoin => {
            register_typed_event::<PlayerJoinEvent>(ctx, handler);
        }
        EventType::PlayerLeave => {
            register_typed_event::<PlayerLeaveEvent>(ctx, handler);
        }
        EventType::PlayerLogin => {
            register_typed_event::<PlayerLoginEvent>(ctx, handler);
        }
        EventType::PlayerChat => {
            register_typed_event::<PlayerChatEvent>(ctx, handler);
        }
        EventType::PlayerCommandSend => {
            register_typed_event::<PlayerCommandSendEvent>(ctx, handler);
        }
        EventType::PlayerPermissionCheck => {
            register_typed_event::<PlayerPermissionCheckEvent>(ctx, handler);
        }
        EventType::PlayerMove => {
            register_typed_event::<PlayerMoveEvent>(ctx, handler);
        }
        EventType::PlayerTransfer => {
            register_typed_event::<PlayerTransferEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerTeleport => {
            register_typed_event::<PlayerTeleportEvent>(ctx, handler);
        }
        EventType::PlayerChangeWorld => {
            register_typed_event::<PlayerChangeWorldEvent>(ctx, handler);
        }
        EventType::PlayerRespawn => {
            register_typed_event::<PlayerRespawnEvent>(ctx, handler);
        }
        EventType::PlayerExpChange => {
            register_typed_event::<PlayerExpChangeEvent>(ctx, handler);
        }
        EventType::PlayerItemHeld => {
            register_typed_event::<PlayerItemHeldEvent>(ctx, handler);
        }
        EventType::PlayerChangedMainHand => {
            register_typed_event::<PlayerChangedMainHandEvent>(ctx, handler);
        }
        EventType::PlayerGamemodeChange => {
            register_typed_event::<PlayerGamemodeChangeEvent>(ctx, handler);
        }
        EventType::PlayerCustomPayload => {
            register_typed_event::<PlayerCustomPayloadEvent>(ctx, handler);
        }
        EventType::PlayerFish => {
            register_typed_event::<PlayerFishEvent>(ctx, handler);
        }
        EventType::PlayerEggThrow => {
            register_typed_event::<PlayerEggThrowEvent>(ctx, handler);
        }
        EventType::PlayerInteractUnknownEntity => {
            register_typed_event::<PlayerInteractUnknownEntityEvent>(ctx, handler);
        }
        EventType::PlayerInteractEntity => {
            register_typed_event::<PlayerInteractEntityEvent>(ctx, handler);
        }
        EventType::PlayerInteract => {
            register_typed_event::<PlayerInteractEvent>(ctx, handler);
        }
        EventType::PlayerToggleSneak => {
            register_typed_event::<PlayerToggleSneakEvent>(ctx, handler);
        }
        EventType::PlayerToggleFlight => {
            register_typed_event::<PlayerToggleFlightEvent>(ctx, handler);
        }
        EventType::PlayerToggleSprint => {
            register_typed_event::<PlayerToggleSprintEvent>(ctx, handler);
        }
        EventType::InventoryClick => {
            register_typed_event::<InventoryClickEvent>(ctx, handler);
        }
        EventType::InventoryClose => {
            register_typed_event::<InventoryCloseEvent>(ctx, handler);
        }
        EventType::BedrockFormResponse => {
            register_typed_event::<BedrockFormResponseEvent>(ctx, handler);
        }
        EventType::PlayerItemConsume => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_item_consume::PlayerItemConsumeEvent,
            >(ctx, handler);
        }
        EventType::PlayerItemDamage => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_item_damage::PlayerItemDamageEvent,
            >(ctx, handler);
        }
        EventType::PlayerDropItem => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_drop_item::PlayerDropItemEvent,
            >(ctx, handler);
        }
        EventType::PlayerBedEnter => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bed::PlayerBedEnterEvent,
            >(ctx, handler);
        }
        EventType::PlayerBedLeave => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bed::PlayerBedLeaveEvent,
            >(ctx, handler);
        }
        EventType::PlayerBucketEmpty => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bucket::PlayerBucketEmptyEvent,
            >(ctx, handler);
        }
        EventType::PlayerBucketFill => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bucket::PlayerBucketFillEvent,
            >(ctx, handler);
        }
        EventType::AsyncPlayerChat => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::async_player_chat::AsyncPlayerChatEvent,
            >(ctx, handler);
        }
        EventType::AsyncPlayerPreLogin => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::async_player_pre_login::AsyncPlayerPreLoginEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerAdvancementDone => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_advancement_done::PlayerAdvancementDoneEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerAnimation => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_animation::PlayerAnimationEvent,
            >(ctx, handler);
        }
        EventType::PlayerArmorStandManipulate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_armor_stand_manipulate::PlayerArmorStandManipulateEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerBucketEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bucket_entity::PlayerBucketEntityEvent,
            >(ctx, handler);
        }
        EventType::PlayerChangedWorld => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_changed_world::PlayerChangedWorldEvent,
            >(ctx, handler);
        }
        EventType::PlayerChannel => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_channel::PlayerChannelEvent,
            >(ctx, handler);
        }
        EventType::PlayerCommandPreprocess => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_command_preprocess::PlayerCommandPreprocessEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerEditBook => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_edit_book::PlayerEditBookEvent,
            >(ctx, handler);
        }
        EventType::PlayerElytraBoost => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_elytra_boost::PlayerElytraBoostEvent,
            >(ctx, handler);
        }
        EventType::PlayerExpCooldownChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_exp_cooldown_change::PlayerExpCooldownChangeEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerHarvestBlock => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_harvest_block::PlayerHarvestBlockEvent,
            >(ctx, handler);
        }
        EventType::PlayerHideEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_hide_entity::PlayerHideEntityEvent,
            >(ctx, handler);
        }
        EventType::PlayerItemBreak => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_item_break::PlayerItemBreakEvent,
            >(ctx, handler);
        }
        EventType::PlayerItemMend => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_item_mend::PlayerItemMendEvent,
            >(ctx, handler);
        }
        EventType::PlayerKick => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_kick::PlayerKickEvent,
            >(ctx, handler);
        }
        EventType::PlayerLeashEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_leash_entity::PlayerLeashEntityEvent,
            >(ctx, handler);
        }
        EventType::PlayerLevelChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_level_change::PlayerLevelChangeEvent,
            >(ctx, handler);
        }
        EventType::PlayerLocaleChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_locale_change::PlayerLocaleChangeEvent,
            >(ctx, handler);
        }
        EventType::PlayerNameEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_name_entity::PlayerNameEntityEvent,
            >(ctx, handler);
        }
        EventType::PlayerOpenSign => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_open_sign::PlayerOpenSignEvent,
            >(ctx, handler);
        }
        EventType::PlayerPortal => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_portal::PlayerPortalEvent,
            >(ctx, handler);
        }
        EventType::PlayerPreLogin => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_pre_login::PlayerPreLoginEvent,
            >(ctx, handler);
        }
        EventType::PlayerRiptide => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_riptide::PlayerRiptideEvent,
            >(ctx, handler);
        }
        EventType::PlayerShearEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent,
            >(ctx, handler);
        }
        EventType::PlayerShowEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_show_entity::PlayerShowEntityEvent,
            >(ctx, handler);
        }
        EventType::PlayerSpawnChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_spawn_change::PlayerSpawnChangeEvent,
            >(ctx, handler);
        }
        EventType::PlayerStatisticIncrement => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerSwapHands => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_swap_hands::PlayerSwapHandItemsEvent,
            >(ctx, handler);
        }
        EventType::PlayerTakeLecternBook => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_take_lectern_book::PlayerTakeLecternBookEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerUnleashEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_unleash_entity::PlayerUnleashEntityEvent,
            >(ctx, handler);
        }
        EventType::PlayerVelocity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_velocity::PlayerVelocityEvent,
            >(ctx, handler);
        }
        EventType::PlayerInput => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_input::PlayerInputEvent,
            >(ctx, handler);
        }
        EventType::PlayerInteractAtEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_interact_at_entity::PlayerInteractAtEntityEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerLinksSend => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_links_send::PlayerLinksSendEvent,
            >(ctx, handler);
        }
        EventType::PlayerPickupArrow => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_pickup_arrow::PlayerPickupArrowEvent,
            >(ctx, handler);
        }
        EventType::PlayerRecipeBookClick => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_recipe_book_click::PlayerRecipeBookClickEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerRecipeBookSettingsChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_recipe_book_settings_change::PlayerRecipeBookSettingsChangeEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerRecipeDiscover => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_recipe_discover::PlayerRecipeDiscoverEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerRegisterChannel => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_register_channel::PlayerRegisterChannelEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerResourcePackStatus => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_resource_pack_status::PlayerResourcePackStatusEvent,
            >(ctx, handler)
            ;
        }
        EventType::PlayerSpawnLocation => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_spawn_location::PlayerSpawnLocationEvent,
            >(ctx, handler);
        }
        EventType::PlayerUnregisterChannel => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_unregister_channel::PlayerUnregisterChannelEvent,
            >(ctx, handler)
            ;
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
    event_type: EventType,
) {
    use pumpkin_core::plugin::entity::{
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
        entity_toggle_sit::EntityToggleSitEvent,
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
        EventType::EntityDamage => {
            register_typed_event::<EntityDamageEvent>(ctx, handler);
        }
        EventType::EntityDeath => {
            register_typed_event::<EntityDeathEvent>(ctx, handler);
        }
        EventType::PlayerDeath => {
            register_typed_event::<PlayerDeathEvent>(ctx, handler);
        }
        EventType::EntitySpawn => {
            register_typed_event::<EntitySpawnEvent>(ctx, handler);
        }
        EventType::EntityCombust => {
            register_typed_event::<EntityCombustEvent>(ctx, handler);
        }
        EventType::EntityRegainHealth => {
            register_typed_event::<EntityRegainHealthEvent>(ctx, handler);
        }
        EventType::EntityAirChange => {
            register_typed_event::<EntityAirChangeEvent>(ctx, handler);
        }
        EventType::EntityBreed => {
            register_typed_event::<EntityBreedEvent>(ctx, handler);
        }
        EventType::EntityDismount => {
            register_typed_event::<EntityDismountEvent>(ctx, handler);
        }
        EventType::EntityDye => {
            register_typed_event::<EntityDyeEvent>(ctx, handler);
        }
        EventType::EntityEnterLoveMode => {
            register_typed_event::<EntityEnterLoveModeEvent>(ctx, handler);
        }
        EventType::EntityExplode => {
            register_typed_event::<EntityExplodeEvent>(ctx, handler);
        }
        EventType::EntityMount => {
            register_typed_event::<EntityMountEvent>(ctx, handler);
        }
        EventType::EntityPickupItem => {
            register_typed_event::<EntityPickupItemEvent>(ctx, handler);
        }
        EventType::EntityPortal => {
            register_typed_event::<EntityPortalEvent>(ctx, handler);
        }
        EventType::EntityResurrect => {
            register_typed_event::<EntityResurrectEvent>(ctx, handler);
        }
        EventType::EntityShootBow => {
            register_typed_event::<EntityShootBowEvent>(ctx, handler);
        }
        EventType::EntityTame => {
            register_typed_event::<EntityTameEvent>(ctx, handler);
        }
        EventType::EntityTarget => {
            register_typed_event::<EntityTargetEvent>(ctx, handler);
        }
        EventType::EntityTeleport => {
            register_typed_event::<EntityTeleportEvent>(ctx, handler);
        }
        EventType::EntityToggleGlide => {
            register_typed_event::<EntityToggleGlideEvent>(ctx, handler);
        }
        EventType::EntityTransform => {
            register_typed_event::<EntityTransformEvent>(ctx, handler);
        }
        EventType::CreatureSpawn => {
            register_typed_event::<CreatureSpawnEvent>(ctx, handler);
        }
        EventType::EnderDragonChangePhase => {
            register_typed_event::<EnderDragonChangePhaseEvent>(ctx, handler);
        }
        EventType::EntityBreakDoor => {
            register_typed_event::<EntityBreakDoorEvent>(ctx, handler);
        }
        EventType::EntityChangeBlock => {
            register_typed_event::<EntityChangeBlockEvent>(ctx, handler);
        }
        EventType::EntityDamageByBlock => {
            register_typed_event::<EntityDamageByBlockEvent>(ctx, handler);
        }
        EventType::EntityDamageByEntity => {
            register_typed_event::<EntityDamageByEntityEvent>(ctx, handler);
        }
        EventType::EntityDropItem => {
            register_typed_event::<EntityDropItemEvent>(ctx, handler);
        }
        EventType::EntityEnterBlock => {
            register_typed_event::<EntityEnterBlockEvent>(ctx, handler);
        }
        EventType::EntityExhaustion => {
            register_typed_event::<EntityExhaustionEvent>(ctx, handler);
        }
        EventType::EntityInteract => {
            register_typed_event::<EntityInteractEvent>(ctx, handler);
        }
        EventType::EntityKnockback => {
            register_typed_event::<EntityKnockbackEvent>(ctx, handler);
        }
        EventType::EntityPlace => {
            register_typed_event::<EntityPlaceEvent>(ctx, handler);
        }
        EventType::EntityPoseChange => {
            register_typed_event::<EntityPoseChangeEvent>(ctx, handler);
        }
        EventType::EntityPotionEffect => {
            register_typed_event::<EntityPotionEffectEvent>(ctx, handler);
        }
        EventType::EntitySpellCast => {
            register_typed_event::<EntitySpellCastEvent>(ctx, handler);
        }
        EventType::EntityTargetLivingEntity => {
            register_typed_event::<EntityTargetLivingEntityEvent>(ctx, handler);
        }
        EventType::EntityToggleSit => {
            register_typed_event::<EntityToggleSitEvent>(ctx, handler);
        }
        EventType::EntityToggleSwim => {
            register_typed_event::<EntityToggleSwimEvent>(ctx, handler);
        }
        EventType::ExplosionPrime => {
            register_typed_event::<ExplosionPrimeEvent>(ctx, handler);
        }
        EventType::FireworkExplode => {
            register_typed_event::<FireworkExplodeEvent>(ctx, handler);
        }
        EventType::FoodLevelChange => {
            register_typed_event::<FoodLevelChangeEvent>(ctx, handler);
        }
        EventType::ItemDespawn => {
            register_typed_event::<ItemDespawnEvent>(ctx, handler);
        }
        EventType::ItemMerge => {
            register_typed_event::<ItemMergeEvent>(ctx, handler);
        }
        EventType::ItemSpawn => {
            register_typed_event::<ItemSpawnEvent>(ctx, handler);
        }
        EventType::PiglinBarter => {
            register_typed_event::<PiglinBarterEvent>(ctx, handler);
        }
        EventType::ProjectileHit => {
            register_typed_event::<ProjectileHitEvent>(ctx, handler);
        }
        EventType::ProjectileLaunch => {
            register_typed_event::<ProjectileLaunchEvent>(ctx, handler);
        }
        EventType::SheepDyeWool => {
            register_typed_event::<SheepDyeWoolEvent>(ctx, handler);
        }
        EventType::SheepRegrowWool => {
            register_typed_event::<SheepRegrowWoolEvent>(ctx, handler);
        }
        EventType::SlimeSplit => {
            register_typed_event::<SlimeSplitEvent>(ctx, handler);
        }
        EventType::StriderTemperatureChange => {
            register_typed_event::<StriderTemperatureChangeEvent>(ctx, handler);
        }
        EventType::VillagerAcquireTrade => {
            register_typed_event::<VillagerAcquireTradeEvent>(ctx, handler);
        }
        EventType::VillagerCareerChange => {
            register_typed_event::<VillagerCareerChangeEvent>(ctx, handler);
        }
        EventType::VillagerReplenishTrade => {
            register_typed_event::<VillagerReplenishTradeEvent>(ctx, handler);
        }
        EventType::WardenAngerChange => {
            register_typed_event::<WardenAngerChangeEvent>(ctx, handler);
        }
        EventType::AreaEffectCloudApply => {
            register_typed_event::<AreaEffectCloudApplyEvent>(ctx, handler);
        }
        EventType::ArrowBodyCountChange => {
            register_typed_event::<ArrowBodyCountChangeEvent>(ctx, handler);
        }
        EventType::BatToggleSleep => {
            register_typed_event::<BatToggleSleepEvent>(ctx, handler);
        }
        EventType::CreeperPower => {
            register_typed_event::<CreeperPowerEvent>(ctx, handler);
        }
        EventType::EntityCombustByBlock => {
            register_typed_event::<EntityCombustByBlockEvent>(ctx, handler);
        }
        EventType::EntityCombustByEntity => {
            register_typed_event::<EntityCombustByEntityEvent>(ctx, handler);
        }
        EventType::EntityKnockbackByEntity => {
            register_typed_event::<EntityKnockbackByEntityEvent>(ctx, handler);
        }
        EventType::EntityPortalEnter => {
            register_typed_event::<EntityPortalEnterEvent>(ctx, handler);
        }
        EventType::EntityPortalExit => {
            register_typed_event::<EntityPortalExitEvent>(ctx, handler);
        }
        EventType::EntityRemove => {
            register_typed_event::<EntityRemoveEvent>(ctx, handler);
        }
        EventType::EntityTargetBlock => {
            register_typed_event::<EntityTargetBlockEvent>(ctx, handler);
        }
        EventType::EntityUnleash => {
            register_typed_event::<EntityUnleashEvent>(ctx, handler);
        }
        EventType::ExpBottle => {
            register_typed_event::<ExpBottleEvent>(ctx, handler);
        }
        EventType::HorseJump => {
            register_typed_event::<HorseJumpEvent>(ctx, handler);
        }
        EventType::LingeringPotionSplash => {
            register_typed_event::<LingeringPotionSplashEvent>(ctx, handler);
        }
        EventType::PigZap => {
            register_typed_event::<PigZapEvent>(ctx, handler);
        }
        EventType::PigZombieAnger => {
            register_typed_event::<PigZombieAngerEvent>(ctx, handler);
        }
        EventType::PotionSplash => {
            register_typed_event::<PotionSplashEvent>(ctx, handler);
        }
        EventType::SpawnerSpawn => {
            register_typed_event::<SpawnerSpawnEvent>(ctx, handler);
        }
        EventType::TrialSpawnerSpawn => {
            register_typed_event::<TrialSpawnerSpawnEvent>(ctx, handler);
        }
        EventType::VillagerReputationChange => {
            register_typed_event::<VillagerReputationChangeEvent>(ctx, handler);
        }
        _ => {
            tracing::error!("non-entity event should not be routed to register_entity_event");
        }
    }
}

fn register_inventory_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::inventory::{
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
        EventType::InventoryOpen => {
            register_typed_event::<InventoryOpenEvent>(ctx, handler);
        }
        EventType::InventoryDrag => {
            register_typed_event::<InventoryDragEvent>(ctx, handler);
        }
        EventType::CraftItem => {
            register_typed_event::<CraftItemEvent>(ctx, handler);
        }
        EventType::FurnaceSmelt => {
            register_typed_event::<FurnaceSmeltEvent>(ctx, handler);
        }
        EventType::Brew => {
            register_typed_event::<BrewEvent>(ctx, handler);
        }
        EventType::BrewingStandFuel => {
            register_typed_event::<BrewingStandFuelEvent>(ctx, handler);
        }
        EventType::FurnaceBurn => {
            register_typed_event::<FurnaceBurnEvent>(ctx, handler);
        }
        EventType::FurnaceExtract => {
            register_typed_event::<FurnaceExtractEvent>(ctx, handler);
        }
        EventType::FurnaceStartSmelt => {
            register_typed_event::<FurnaceStartSmeltEvent>(ctx, handler);
        }
        EventType::HopperInventorySearch => {
            register_typed_event::<HopperInventorySearchEvent>(ctx, handler);
        }
        EventType::InventoryCreative => {
            register_typed_event::<InventoryCreativeEvent>(ctx, handler);
        }
        EventType::InventoryInteract => {
            register_typed_event::<InventoryInteractEvent>(ctx, handler);
        }
        EventType::InventoryMoveItem => {
            register_typed_event::<InventoryMoveItemEvent>(ctx, handler);
        }
        EventType::InventoryPickupItem => {
            register_typed_event::<InventoryPickupItemEvent>(ctx, handler);
        }
        EventType::PrepareAnvil => {
            register_typed_event::<PrepareAnvilEvent>(ctx, handler);
        }
        EventType::PrepareGrindstone => {
            register_typed_event::<PrepareGrindstoneEvent>(ctx, handler);
        }
        EventType::PrepareInventoryResult => {
            register_typed_event::<PrepareInventoryResultEvent>(ctx, handler);
        }
        EventType::PrepareItemCraft => {
            register_typed_event::<PrepareItemCraftEvent>(ctx, handler);
        }
        EventType::PrepareSmithing => {
            register_typed_event::<PrepareSmithingEvent>(ctx, handler);
        }
        EventType::SmithItem => {
            register_typed_event::<SmithItemEvent>(ctx, handler);
        }
        EventType::TradeSelect => {
            register_typed_event::<TradeSelectEvent>(ctx, handler);
        }
        _ => {
            tracing::error!("non-inventory event should not be routed to register_inventory_event");
        }
    }
}

fn register_vehicle_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::vehicle::{
        vehicle_block_collision::VehicleBlockCollisionEvent,
        vehicle_collision::VehicleCollisionEvent, vehicle_create::VehicleCreateEvent,
        vehicle_damage::VehicleDamageEvent, vehicle_destroy::VehicleDestroyEvent,
        vehicle_enter::VehicleEnterEvent, vehicle_entity_collision::VehicleEntityCollisionEvent,
        vehicle_exit::VehicleExitEvent, vehicle_move::VehicleMoveEvent,
        vehicle_update::VehicleUpdateEvent,
    };

    match event_type {
        EventType::VehicleBlockCollision => {
            register_typed_event::<VehicleBlockCollisionEvent>(ctx, handler);
        }
        EventType::VehicleCollision => {
            register_typed_event::<VehicleCollisionEvent>(ctx, handler);
        }
        EventType::VehicleCreate => {
            register_typed_event::<VehicleCreateEvent>(ctx, handler);
        }
        EventType::VehicleDamage => {
            register_typed_event::<VehicleDamageEvent>(ctx, handler);
        }
        EventType::VehicleDestroy => {
            register_typed_event::<VehicleDestroyEvent>(ctx, handler);
        }
        EventType::VehicleEnter => {
            register_typed_event::<VehicleEnterEvent>(ctx, handler);
        }
        EventType::VehicleEntityCollision => {
            register_typed_event::<VehicleEntityCollisionEvent>(ctx, handler);
        }
        EventType::VehicleExit => {
            register_typed_event::<VehicleExitEvent>(ctx, handler);
        }
        EventType::VehicleMove => {
            register_typed_event::<VehicleMoveEvent>(ctx, handler);
        }
        EventType::VehicleUpdate => {
            register_typed_event::<VehicleUpdateEvent>(ctx, handler);
        }
        _ => {
            tracing::error!("non-vehicle event should not be routed to register_vehicle_event");
        }
    }
}

fn register_enchantment_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::enchantment::{
        enchant_item::EnchantItemEvent, prepare_item_enchant::PrepareItemEnchantEvent,
    };

    match event_type {
        EventType::PrepareItemEnchant => {
            register_typed_event::<PrepareItemEnchantEvent>(ctx, handler);
        }
        EventType::EnchantItem => {
            register_typed_event::<EnchantItemEvent>(ctx, handler);
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
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::world::{
        chunk_load::ChunkLoad, chunk_save::ChunkSave, chunk_send::ChunkSend,
        spawn_change::SpawnChangeEvent,
    };

    match event_type {
        EventType::SpawnChange => {
            register_typed_event::<SpawnChangeEvent>(ctx, handler);
        }
        EventType::ChunkLoad => {
            register_typed_event::<ChunkLoad>(ctx, handler);
        }
        EventType::ChunkSave => {
            register_typed_event::<ChunkSave>(ctx, handler);
        }
        EventType::ChunkSend => {
            register_typed_event::<ChunkSend>(ctx, handler);
        }
        EventType::WeatherChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::weather_change::WeatherChangeEvent,
            >(ctx, handler);
        }
        EventType::ThunderChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::weather_change::ThunderChangeEvent,
            >(ctx, handler);
        }
        EventType::WorldLoad => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::world_load::WorldLoadEvent,
            >(ctx, handler);
        }
        EventType::WorldUnload => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::world_load::WorldUnloadEvent,
            >(ctx, handler);
        }
        EventType::AsyncStructureGenerate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::async_structure_generate::AsyncStructureGenerateEvent,
            >(ctx, handler)
            ;
        }
        EventType::AsyncStructureSpawn => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::async_structure_spawn::AsyncStructureSpawnEvent,
            >(ctx, handler);
        }
        EventType::ChunkPopulate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::chunk_populate::ChunkPopulateEvent,
            >(ctx, handler);
        }
        EventType::ChunkUnload => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::chunk_unload::ChunkUnloadEvent,
            >(ctx, handler);
        }
        EventType::EntitiesLoad => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::entities_load::EntitiesLoadEvent,
            >(ctx, handler);
        }
        EventType::EntitiesUnload => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::entities_unload::EntitiesUnloadEvent,
            >(ctx, handler);
        }
        EventType::GenericGame => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::generic_game::GenericGameEvent,
            >(ctx, handler);
        }
        EventType::LootGenerate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::loot_generate::LootGenerateEvent,
            >(ctx, handler);
        }
        EventType::PortalCreate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::portal_create::PortalCreateEvent,
            >(ctx, handler);
        }
        EventType::StructureGrow => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::structure_grow::StructureGrowEvent,
            >(ctx, handler);
        }
        EventType::TimeSkip => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::time_skip::TimeSkipEvent,
            >(ctx, handler);
        }
        EventType::WorldInit => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::world_init::WorldInitEvent,
            >(ctx, handler);
        }
        EventType::WorldSave => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::world_save::WorldSaveEvent,
            >(ctx, handler);
        }
        EventType::LightningStrike => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::lightning_strike::LightningStrikeEvent,
            >(ctx, handler);
        }
        _ => {
            tracing::error!("non-world event should not be routed to register_world_event");
        }
    }
}

#[allow(clippy::too_many_lines)]
fn register_block_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::block::{
        block_break::BlockBreakEvent, block_burn::BlockBurnEvent,
        block_can_build::BlockCanBuildEvent, block_grow::BlockGrowEvent,
        block_place::BlockPlaceEvent, block_redstone::BlockRedstoneEvent,
    };

    match event_type {
        EventType::BlockRedstone => {
            register_typed_event::<BlockRedstoneEvent>(ctx, handler);
        }
        EventType::BlockBreak => {
            register_typed_event::<BlockBreakEvent>(ctx, handler);
        }
        EventType::BlockBurn => {
            register_typed_event::<BlockBurnEvent>(ctx, handler);
        }
        EventType::BlockCanBuild => {
            register_typed_event::<BlockCanBuildEvent>(ctx, handler);
        }
        EventType::BlockGrow => {
            register_typed_event::<BlockGrowEvent>(ctx, handler);
        }
        EventType::BlockPlace => {
            register_typed_event::<BlockPlaceEvent>(ctx, handler);
        }
        EventType::BlockDamage => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_damage::BlockDamageEvent,
            >(ctx, handler);
        }
        EventType::BlockIgnite => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_ignite::BlockIgniteEvent,
            >(ctx, handler);
        }
        EventType::BlockFromTo => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_from_to::BlockFromToEvent,
            >(ctx, handler);
        }
        EventType::BlockForm => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_form::BlockFormEvent,
            >(ctx, handler);
        }
        EventType::BlockFade => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_fade::BlockFadeEvent,
            >(ctx, handler);
        }
        EventType::BlockDispense => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_dispense::BlockDispenseEvent,
            >(ctx, handler);
        }
        EventType::BlockExplode => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_explode::BlockExplodeEvent,
            >(ctx, handler);
        }
        EventType::BlockPhysics => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_physics::BlockPhysicsEvent,
            >(ctx, handler);
        }
        EventType::BlockPistonExtend => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_piston::BlockPistonExtendEvent,
            >(ctx, handler);
        }
        EventType::BlockPistonRetract => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_piston::BlockPistonRetractEvent,
            >(ctx, handler);
        }
        EventType::NotePlay => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::note_play::NotePlayEvent,
            >(ctx, handler);
        }
        EventType::SignChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::sign_change::SignChangeEvent,
            >(ctx, handler);
        }
        EventType::SpongeAbsorb => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::sponge_absorb::SpongeAbsorbEvent,
            >(ctx, handler);
        }
        EventType::TntPrime => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::tnt_prime::TNTPrimeEvent,
            >(ctx, handler);
        }
        EventType::BellResonate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::bell_resonate::BellResonateEvent,
            >(ctx, handler);
        }
        EventType::BellRing => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::bell_ring::BellRingEvent,
            >(ctx, handler);
        }
        EventType::BlockBrush => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_brush::BlockBrushEvent,
            >(ctx, handler);
        }
        EventType::BlockCook => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_cook::BlockCookEvent,
            >(ctx, handler);
        }
        EventType::BlockDamageAbort => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_damage_abort::BlockDamageAbortEvent,
            >(ctx, handler);
        }
        EventType::BlockDispenseArmor => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_dispense_armor::BlockDispenseArmorEvent,
            >(ctx, handler);
        }
        EventType::BlockDispenseLoot => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_dispense_loot::BlockDispenseLootEvent,
            >(ctx, handler);
        }
        EventType::BlockDropItem => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_drop_item::BlockDropItemEvent,
            >(ctx, handler);
        }
        EventType::BlockExp => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_exp::BlockExpEvent,
            >(ctx, handler);
        }
        EventType::BlockFertilize => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_fertilize::BlockFertilizeEvent,
            >(ctx, handler);
        }
        EventType::BlockMultiPlace => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_multi_place::BlockMultiPlaceEvent,
            >(ctx, handler);
        }
        EventType::BlockReceiveGame => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_receive_game::BlockReceiveGameEvent,
            >(ctx, handler);
        }
        EventType::BlockShearEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_shear_entity::BlockShearEntityEvent,
            >(ctx, handler);
        }
        EventType::BlockSpread => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_spread::BlockSpreadEvent,
            >(ctx, handler);
        }
        EventType::BrewingStart => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::brewing_start::BrewingStartEvent,
            >(ctx, handler);
        }
        EventType::CampfireStart => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::campfire_start::CampfireStartEvent,
            >(ctx, handler);
        }
        EventType::CauldronLevelChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::cauldron_level_change::CauldronLevelChangeEvent,
            >(ctx, handler);
        }
        EventType::CrafterCraft => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::crafter_craft::CrafterCraftEvent,
            >(ctx, handler);
        }
        EventType::EntityBlockForm => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::entity_block_form::EntityBlockFormEvent,
            >(ctx, handler);
        }
        EventType::FluidLevelChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::fluid_level_change::FluidLevelChangeEvent,
            >(ctx, handler);
        }
        EventType::InventoryBlockStart => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::inventory_block_start::InventoryBlockStartEvent,
            >(ctx, handler);
        }
        EventType::LeavesDecay => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::leaves_decay::LeavesDecayEvent,
            >(ctx, handler);
        }
        EventType::MoistureChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::moisture_change::MoistureChangeEvent,
            >(ctx, handler);
        }
        EventType::SculkBloom => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::sculk_bloom::SculkBloomEvent,
            >(ctx, handler);
        }
        EventType::VaultDisplayItem => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::vault_display_item::VaultDisplayItemEvent,
            >(ctx, handler);
        }

        _ => {
            tracing::error!("non-block event should not be routed to register_block_event");
        }
    }
}

fn register_raid_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::raid::{
        raid_finish::RaidFinishEvent, raid_spawn_wave::RaidSpawnWaveEvent,
        raid_stop::RaidStopEvent, raid_trigger::RaidTriggerEvent,
    };

    match event_type {
        EventType::RaidFinish => {
            register_typed_event::<RaidFinishEvent>(ctx, handler);
        }
        EventType::RaidSpawnWave => {
            register_typed_event::<RaidSpawnWaveEvent>(ctx, handler);
        }
        EventType::RaidStop => {
            register_typed_event::<RaidStopEvent>(ctx, handler);
        }
        EventType::RaidTrigger => {
            register_typed_event::<RaidTriggerEvent>(ctx, handler);
        }
        _ => {
            tracing::error!("non-raid event should not be routed to register_raid_event");
        }
    }
}

fn register_dialog_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::api::events::dialog::{
        DialogClearEvent, DialogClickActionEvent, DialogShowEvent,
    };

    match event_type {
        EventType::DialogClickAction => {
            register_typed_event::<DialogClickActionEvent>(ctx, handler);
        }
        EventType::DialogShow => {
            register_typed_event::<DialogShowEvent>(ctx, handler);
        }
        EventType::DialogClear => {
            register_typed_event::<DialogClearEvent>(ctx, handler);
        }
        _ => {
            tracing::error!("non-dialog event should not be routed to register_dialog_event");
        }
    }
}

fn register_server_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::server::{
        list_ping::ServerListPingEvent,
        packet::{
            BedrockConnectionPacketReceivedEvent, BedrockConnectionPacketSentEvent,
            JavaConnectionPacketReceivedEvent, JavaConnectionPacketSentEvent, PacketReceivedEvent,
            PacketSentEvent,
        },
        server_broadcast::ServerBroadcastEvent,
        server_command::ServerCommandEvent,
        server_load::ServerLoadEvent,
        server_tick_end::ServerTickEndEvent,
        server_tick_start::ServerTickStartEvent,
    };

    match event_type {
        EventType::PacketReceived => {
            register_typed_event::<PacketReceivedEvent>(ctx, handler);
        }
        EventType::PacketSent => {
            register_typed_event::<PacketSentEvent>(ctx, handler);
        }
        EventType::JavaConnectionPacketReceived => {
            register_typed_event::<JavaConnectionPacketReceivedEvent>(ctx, handler);
        }
        EventType::JavaConnectionPacketSent => {
            register_typed_event::<JavaConnectionPacketSentEvent>(ctx, handler);
        }
        EventType::BedrockConnectionPacketReceived => {
            register_typed_event::<BedrockConnectionPacketReceivedEvent>(ctx, handler);
        }
        EventType::BedrockConnectionPacketSent => {
            register_typed_event::<BedrockConnectionPacketSentEvent>(ctx, handler);
        }
        EventType::ServerCommand => {
            register_typed_event::<ServerCommandEvent>(ctx, handler);
        }
        EventType::ServerListPing => {
            register_typed_event::<ServerListPingEvent>(ctx, handler);
        }
        EventType::ServerBroadcast => {
            register_typed_event::<ServerBroadcastEvent>(ctx, handler);
        }
        EventType::ServerLoad => {
            register_typed_event::<ServerLoadEvent>(ctx, handler);
        }
        EventType::ServerTickEnd => {
            register_typed_event::<ServerTickEndEvent>(ctx, handler);
        }
        EventType::ServerTickStart => {
            register_typed_event::<ServerTickStartEvent>(ctx, handler);
        }
        EventType::MapInitialize => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::server::map_initialize::MapInitializeEvent,
            >(ctx, handler);
        }
        _ => {
            tracing::error!("non-server event should not be routed to register_server_event");
        }
    }
}

fn register_hanging_event(
    ctx: &Context,
    handler: &Arc<WasmPluginEventHandler>,
    event_type: EventType,
) {
    use pumpkin_core::plugin::hanging::{
        hanging_break::HangingBreakEvent, hanging_break_by_entity::HangingBreakByEntityEvent,
        hanging_place::HangingPlaceEvent,
    };

    match event_type {
        EventType::HangingBreak => {
            register_typed_event::<HangingBreakEvent>(ctx, handler);
        }
        EventType::HangingBreakByEntity => {
            register_typed_event::<HangingBreakByEntityEvent>(ctx, handler);
        }
        EventType::HangingPlace => {
            register_typed_event::<HangingPlaceEvent>(ctx, handler);
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
        let handler = Arc::new(WasmPluginEventHandler {
            handler_id,
            priority,
            blocking,
            plugin,
        });

        match event_type {
            event_type @ (EventType::PacketReceived
            | EventType::PacketSent
            | EventType::JavaConnectionPacketReceived
            | EventType::JavaConnectionPacketSent
            | EventType::BedrockConnectionPacketReceived
            | EventType::BedrockConnectionPacketSent
            | EventType::ServerCommand
            | EventType::ServerListPing
            | EventType::ServerBroadcast
            | EventType::ServerLoad
            | EventType::ServerTickEnd
            | EventType::ServerTickStart
            | EventType::MapInitialize) => {
                register_server_event(ctx, &handler, event_type);
            }
            event_type @ (EventType::HangingBreak
            | EventType::HangingBreakByEntity
            | EventType::HangingPlace) => {
                register_hanging_event(ctx, &handler, event_type);
            }
            event_type @ (EventType::SpawnChange
            | EventType::ChunkLoad
            | EventType::ChunkSave
            | EventType::ChunkSend
            | EventType::WeatherChange
            | EventType::ThunderChange
            | EventType::WorldLoad
            | EventType::WorldUnload
            | EventType::AsyncStructureGenerate
            | EventType::AsyncStructureSpawn
            | EventType::ChunkPopulate
            | EventType::ChunkUnload
            | EventType::EntitiesLoad
            | EventType::EntitiesUnload
            | EventType::GenericGame
            | EventType::LootGenerate
            | EventType::PortalCreate
            | EventType::StructureGrow
            | EventType::TimeSkip
            | EventType::WorldInit
            | EventType::WorldSave
            | EventType::LightningStrike) => {
                register_world_event(ctx, &handler, event_type);
            }
            event_type @ (EventType::EntityDamage
            | EventType::EntityDeath
            | EventType::PlayerDeath
            | EventType::EntitySpawn
            | EventType::EntityCombust
            | EventType::EntityRegainHealth
            | EventType::EntityAirChange
            | EventType::EntityBreed
            | EventType::EntityDismount
            | EventType::EntityDye
            | EventType::EntityEnterLoveMode
            | EventType::EntityExplode
            | EventType::EntityMount
            | EventType::EntityPickupItem
            | EventType::EntityPortal
            | EventType::EntityResurrect
            | EventType::EntityShootBow
            | EventType::EntityTame
            | EventType::EntityTarget
            | EventType::EntityTeleport
            | EventType::EntityToggleGlide
            | EventType::EntityTransform
            | EventType::CreatureSpawn
            | EventType::EnderDragonChangePhase
            | EventType::EntityBreakDoor
            | EventType::EntityChangeBlock
            | EventType::EntityDamageByBlock
            | EventType::EntityDamageByEntity
            | EventType::EntityDropItem
            | EventType::EntityEnterBlock
            | EventType::EntityExhaustion
            | EventType::EntityInteract
            | EventType::EntityKnockback
            | EventType::EntityPlace
            | EventType::EntityPoseChange
            | EventType::EntityPotionEffect
            | EventType::EntitySpellCast
            | EventType::EntityTargetLivingEntity
            | EventType::EntityToggleSit
            | EventType::EntityToggleSwim
            | EventType::ExplosionPrime
            | EventType::FireworkExplode
            | EventType::FoodLevelChange
            | EventType::ItemDespawn
            | EventType::ItemMerge
            | EventType::ItemSpawn
            | EventType::PiglinBarter
            | EventType::ProjectileHit
            | EventType::ProjectileLaunch
            | EventType::SheepDyeWool
            | EventType::SheepRegrowWool
            | EventType::SlimeSplit
            | EventType::StriderTemperatureChange
            | EventType::VillagerAcquireTrade
            | EventType::VillagerCareerChange
            | EventType::VillagerReplenishTrade
            | EventType::WardenAngerChange
            | EventType::AreaEffectCloudApply
            | EventType::ArrowBodyCountChange
            | EventType::BatToggleSleep
            | EventType::CreeperPower
            | EventType::EntityCombustByBlock
            | EventType::EntityCombustByEntity
            | EventType::EntityKnockbackByEntity
            | EventType::EntityPortalEnter
            | EventType::EntityPortalExit
            | EventType::EntityRemove
            | EventType::EntityTargetBlock
            | EventType::EntityUnleash
            | EventType::ExpBottle
            | EventType::HorseJump
            | EventType::LingeringPotionSplash
            | EventType::PigZap
            | EventType::PigZombieAnger
            | EventType::PotionSplash
            | EventType::SpawnerSpawn
            | EventType::TrialSpawnerSpawn
            | EventType::VillagerReputationChange) => {
                register_entity_event(ctx, &handler, event_type);
            }
            event_type @ (EventType::BlockRedstone
            | EventType::BlockBreak
            | EventType::BlockBurn
            | EventType::BlockCanBuild
            | EventType::BlockGrow
            | EventType::BlockPlace
            | EventType::BlockDamage
            | EventType::BlockIgnite
            | EventType::BlockFromTo
            | EventType::BlockForm
            | EventType::BlockFade
            | EventType::BlockDispense
            | EventType::BlockExplode
            | EventType::BlockPhysics
            | EventType::BlockPistonExtend
            | EventType::BlockPistonRetract
            | EventType::NotePlay
            | EventType::SignChange
            | EventType::SpongeAbsorb
            | EventType::TntPrime
            | EventType::BellResonate
            | EventType::BellRing
            | EventType::BlockBrush
            | EventType::BlockCook
            | EventType::BlockDamageAbort
            | EventType::BlockDispenseArmor
            | EventType::BlockDispenseLoot
            | EventType::BlockDropItem
            | EventType::BlockExp
            | EventType::BlockFertilize
            | EventType::BlockMultiPlace
            | EventType::BlockReceiveGame
            | EventType::BlockShearEntity
            | EventType::BlockSpread
            | EventType::BrewingStart
            | EventType::CampfireStart
            | EventType::CauldronLevelChange
            | EventType::CrafterCraft
            | EventType::EntityBlockForm
            | EventType::FluidLevelChange
            | EventType::InventoryBlockStart
            | EventType::LeavesDecay
            | EventType::MoistureChange
            | EventType::SculkBloom
            | EventType::VaultDisplayItem) => {
                register_block_event(ctx, &handler, event_type);
            }
            event_type @ (EventType::InventoryOpen
            | EventType::InventoryDrag
            | EventType::CraftItem
            | EventType::FurnaceSmelt
            | EventType::Brew
            | EventType::BrewingStandFuel
            | EventType::FurnaceBurn
            | EventType::FurnaceExtract
            | EventType::FurnaceStartSmelt
            | EventType::HopperInventorySearch
            | EventType::InventoryCreative
            | EventType::InventoryInteract
            | EventType::InventoryMoveItem
            | EventType::InventoryPickupItem
            | EventType::PrepareAnvil
            | EventType::PrepareGrindstone
            | EventType::PrepareInventoryResult
            | EventType::PrepareItemCraft
            | EventType::PrepareSmithing
            | EventType::SmithItem
            | EventType::TradeSelect) => {
                register_inventory_event(ctx, &handler, event_type);
            }
            event_type @ (EventType::PrepareItemEnchant | EventType::EnchantItem) => {
                register_enchantment_event(ctx, &handler, event_type);
            }
            event_type @ (EventType::VehicleBlockCollision
            | EventType::VehicleCollision
            | EventType::VehicleCreate
            | EventType::VehicleDamage
            | EventType::VehicleDestroy
            | EventType::VehicleEnter
            | EventType::VehicleEntityCollision
            | EventType::VehicleExit
            | EventType::VehicleMove
            | EventType::VehicleUpdate) => {
                register_vehicle_event(ctx, &handler, event_type);
            }
            event_type @ (EventType::RaidFinish
            | EventType::RaidSpawnWave
            | EventType::RaidStop
            | EventType::RaidTrigger) => {
                register_raid_event(ctx, &handler, event_type);
            }
            event_type @ (EventType::DialogClickAction
            | EventType::DialogShow
            | EventType::DialogClear) => {
                register_dialog_event(ctx, &handler, event_type);
            }
            event_type => {
                register_player_event(ctx, &handler, event_type);
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

    fn register_java_versions(
        &mut self,
        context: Resource<WitContext>,
        versions: Vec<pumpkin::plugin::player::JavaMinecraftVersion>,
    ) -> wasmtime::Result<()> {
        let context_res = self.get(&context)?;
        let native_versions: Vec<pumpkin_util::version::JavaMinecraftVersion> = versions
            .into_iter()
            .map(crate::player::from_wasm_java_version)
            .collect();
        context_res
            .plugin_manager
            .java_versions
            .register(&context_res.get_metadata().name, native_versions);
        Ok(())
    }

    fn register_bedrock_versions(
        &mut self,
        context: Resource<WitContext>,
        protocols: Vec<u32>,
    ) -> wasmtime::Result<()> {
        let context_res = self.get(&context)?;
        context_res
            .plugin_manager
            .bedrock_versions
            .register(&context_res.get_metadata().name, protocols);
        Ok(())
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
