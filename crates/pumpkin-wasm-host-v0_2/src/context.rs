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
        player_toggle_sprint_event::PlayerToggleSprintEvent,
    };

    match event_type {
        EventType::PlayerJoin => {
            register_typed_event::<PlayerJoinEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerLeave => {
            register_typed_event::<PlayerLeaveEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerLogin => {
            register_typed_event::<PlayerLoginEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerChat => {
            register_typed_event::<PlayerChatEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerCommandSend => {
            register_typed_event::<PlayerCommandSendEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerPermissionCheck => {
            register_typed_event::<PlayerPermissionCheckEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerMove => {
            register_typed_event::<PlayerMoveEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerTeleport => {
            register_typed_event::<PlayerTeleportEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerChangeWorld => {
            register_typed_event::<PlayerChangeWorldEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerRespawn => {
            register_typed_event::<PlayerRespawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerExpChange => {
            register_typed_event::<PlayerExpChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerItemHeld => {
            register_typed_event::<PlayerItemHeldEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerChangedMainHand => {
            register_typed_event::<PlayerChangedMainHandEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerGamemodeChange => {
            register_typed_event::<PlayerGamemodeChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerCustomPayload => {
            register_typed_event::<PlayerCustomPayloadEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerFish => {
            register_typed_event::<PlayerFishEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerEggThrow => {
            register_typed_event::<PlayerEggThrowEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerInteractUnknownEntity => {
            register_typed_event::<PlayerInteractUnknownEntityEvent>(
                ctx, handler, priority, blocking,
            );
        }
        EventType::PlayerInteractEntity => {
            register_typed_event::<PlayerInteractEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerInteract => {
            register_typed_event::<PlayerInteractEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerToggleSneak => {
            register_typed_event::<PlayerToggleSneakEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerToggleFlight => {
            register_typed_event::<PlayerToggleFlightEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerToggleSprint => {
            register_typed_event::<PlayerToggleSprintEvent>(ctx, handler, priority, blocking);
        }
        EventType::InventoryClick => {
            register_typed_event::<InventoryClickEvent>(ctx, handler, priority, blocking);
        }
        EventType::InventoryClose => {
            register_typed_event::<InventoryCloseEvent>(ctx, handler, priority, blocking);
        }
        EventType::BedrockFormResponse => {
            register_typed_event::<BedrockFormResponseEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerItemConsume => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_item_consume::PlayerItemConsumeEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerItemDamage => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_item_damage::PlayerItemDamageEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerDropItem => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_drop_item::PlayerDropItemEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerBedEnter => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bed::PlayerBedEnterEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerBedLeave => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bed::PlayerBedLeaveEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerBucketEmpty => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bucket::PlayerBucketEmptyEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerBucketFill => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bucket::PlayerBucketFillEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::AsyncPlayerChat => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::async_player_chat::AsyncPlayerChatEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::AsyncPlayerPreLogin => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::async_player_pre_login::AsyncPlayerPreLoginEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerAdvancementDone => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_advancement_done::PlayerAdvancementDoneEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerAnimation => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_animation::PlayerAnimationEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerArmorStandManipulate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_armor_stand_manipulate::PlayerArmorStandManipulateEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerBucketEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_bucket_entity::PlayerBucketEntityEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerChangedWorld => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_changed_world::PlayerChangedWorldEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerChannel => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_channel::PlayerChannelEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerCommandPreprocess => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_command_preprocess::PlayerCommandPreprocessEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerEditBook => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_edit_book::PlayerEditBookEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerElytraBoost => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_elytra_boost::PlayerElytraBoostEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerExpCooldownChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_exp_cooldown_change::PlayerExpCooldownChangeEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerHarvestBlock => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_harvest_block::PlayerHarvestBlockEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerHideEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_hide_entity::PlayerHideEntityEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerItemBreak => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_item_break::PlayerItemBreakEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerItemMend => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_item_mend::PlayerItemMendEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerKick => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_kick::PlayerKickEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerLeashEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_leash_entity::PlayerLeashEntityEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerLevelChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_level_change::PlayerLevelChangeEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerLocaleChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_locale_change::PlayerLocaleChangeEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerNameEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_name_entity::PlayerNameEntityEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerOpenSign => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_open_sign::PlayerOpenSignEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerPortal => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_portal::PlayerPortalEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerPreLogin => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_pre_login::PlayerPreLoginEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerRiptide => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_riptide::PlayerRiptideEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerShearEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_shear_entity::PlayerShearEntityEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerShowEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_show_entity::PlayerShowEntityEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerSpawnChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_spawn_change::PlayerSpawnChangeEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerStatisticIncrement => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_statistic_increment::PlayerStatisticIncrementEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerSwapHands => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_swap_hands::PlayerSwapHandItemsEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerTakeLecternBook => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_take_lectern_book::PlayerTakeLecternBookEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerUnleashEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_unleash_entity::PlayerUnleashEntityEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerVelocity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_velocity::PlayerVelocityEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerInput => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_input::PlayerInputEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerInteractAtEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_interact_at_entity::PlayerInteractAtEntityEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerLinksSend => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_links_send::PlayerLinksSendEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerPickupArrow => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_pickup_arrow::PlayerPickupArrowEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerRecipeBookClick => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_recipe_book_click::PlayerRecipeBookClickEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerRecipeBookSettingsChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_recipe_book_settings_change::PlayerRecipeBookSettingsChangeEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerRecipeDiscover => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_recipe_discover::PlayerRecipeDiscoverEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerRegisterChannel => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_register_channel::PlayerRegisterChannelEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerResourcePackStatus => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_resource_pack_status::PlayerResourcePackStatusEvent,
            >(ctx, handler, priority, blocking)
            ;
        }
        EventType::PlayerSpawnLocation => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_spawn_location::PlayerSpawnLocationEvent,
            >(ctx, handler, priority, blocking);
        }
        EventType::PlayerUnregisterChannel => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::player::player_unregister_channel::PlayerUnregisterChannelEvent,
            >(ctx, handler, priority, blocking)
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
    priority: pumpkin_core::plugin::EventPriority,
    blocking: bool,
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
            register_typed_event::<EntityDamageEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDeath => {
            register_typed_event::<EntityDeathEvent>(ctx, handler, priority, blocking);
        }
        EventType::PlayerDeath => {
            register_typed_event::<PlayerDeathEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntitySpawn => {
            register_typed_event::<EntitySpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityCombust => {
            register_typed_event::<EntityCombustEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityRegainHealth => {
            register_typed_event::<EntityRegainHealthEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityAirChange => {
            register_typed_event::<EntityAirChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityBreed => {
            register_typed_event::<EntityBreedEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDismount => {
            register_typed_event::<EntityDismountEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDye => {
            register_typed_event::<EntityDyeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityEnterLoveMode => {
            register_typed_event::<EntityEnterLoveModeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityExplode => {
            register_typed_event::<EntityExplodeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityMount => {
            register_typed_event::<EntityMountEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPickupItem => {
            register_typed_event::<EntityPickupItemEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPortal => {
            register_typed_event::<EntityPortalEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityResurrect => {
            register_typed_event::<EntityResurrectEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityShootBow => {
            register_typed_event::<EntityShootBowEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTame => {
            register_typed_event::<EntityTameEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTarget => {
            register_typed_event::<EntityTargetEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTeleport => {
            register_typed_event::<EntityTeleportEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityToggleGlide => {
            register_typed_event::<EntityToggleGlideEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTransform => {
            register_typed_event::<EntityTransformEvent>(ctx, handler, priority, blocking);
        }
        EventType::CreatureSpawn => {
            register_typed_event::<CreatureSpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::EnderDragonChangePhase => {
            register_typed_event::<EnderDragonChangePhaseEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityBreakDoor => {
            register_typed_event::<EntityBreakDoorEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityChangeBlock => {
            register_typed_event::<EntityChangeBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDamageByBlock => {
            register_typed_event::<EntityDamageByBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDamageByEntity => {
            register_typed_event::<EntityDamageByEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityDropItem => {
            register_typed_event::<EntityDropItemEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityEnterBlock => {
            register_typed_event::<EntityEnterBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityExhaustion => {
            register_typed_event::<EntityExhaustionEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityInteract => {
            register_typed_event::<EntityInteractEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityKnockback => {
            register_typed_event::<EntityKnockbackEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPlace => {
            register_typed_event::<EntityPlaceEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPoseChange => {
            register_typed_event::<EntityPoseChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPotionEffect => {
            register_typed_event::<EntityPotionEffectEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntitySpellCast => {
            register_typed_event::<EntitySpellCastEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTargetLivingEntity => {
            register_typed_event::<EntityTargetLivingEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityToggleSwim => {
            register_typed_event::<EntityToggleSwimEvent>(ctx, handler, priority, blocking);
        }
        EventType::ExplosionPrime => {
            register_typed_event::<ExplosionPrimeEvent>(ctx, handler, priority, blocking);
        }
        EventType::FireworkExplode => {
            register_typed_event::<FireworkExplodeEvent>(ctx, handler, priority, blocking);
        }
        EventType::FoodLevelChange => {
            register_typed_event::<FoodLevelChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::ItemDespawn => {
            register_typed_event::<ItemDespawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::ItemMerge => {
            register_typed_event::<ItemMergeEvent>(ctx, handler, priority, blocking);
        }
        EventType::ItemSpawn => {
            register_typed_event::<ItemSpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::PiglinBarter => {
            register_typed_event::<PiglinBarterEvent>(ctx, handler, priority, blocking);
        }
        EventType::ProjectileHit => {
            register_typed_event::<ProjectileHitEvent>(ctx, handler, priority, blocking);
        }
        EventType::ProjectileLaunch => {
            register_typed_event::<ProjectileLaunchEvent>(ctx, handler, priority, blocking);
        }
        EventType::SheepDyeWool => {
            register_typed_event::<SheepDyeWoolEvent>(ctx, handler, priority, blocking);
        }
        EventType::SheepRegrowWool => {
            register_typed_event::<SheepRegrowWoolEvent>(ctx, handler, priority, blocking);
        }
        EventType::SlimeSplit => {
            register_typed_event::<SlimeSplitEvent>(ctx, handler, priority, blocking);
        }
        EventType::StriderTemperatureChange => {
            register_typed_event::<StriderTemperatureChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::VillagerAcquireTrade => {
            register_typed_event::<VillagerAcquireTradeEvent>(ctx, handler, priority, blocking);
        }
        EventType::VillagerCareerChange => {
            register_typed_event::<VillagerCareerChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::VillagerReplenishTrade => {
            register_typed_event::<VillagerReplenishTradeEvent>(ctx, handler, priority, blocking);
        }
        EventType::WardenAngerChange => {
            register_typed_event::<WardenAngerChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::AreaEffectCloudApply => {
            register_typed_event::<AreaEffectCloudApplyEvent>(ctx, handler, priority, blocking);
        }
        EventType::ArrowBodyCountChange => {
            register_typed_event::<ArrowBodyCountChangeEvent>(ctx, handler, priority, blocking);
        }
        EventType::BatToggleSleep => {
            register_typed_event::<BatToggleSleepEvent>(ctx, handler, priority, blocking);
        }
        EventType::CreeperPower => {
            register_typed_event::<CreeperPowerEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityCombustByBlock => {
            register_typed_event::<EntityCombustByBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityCombustByEntity => {
            register_typed_event::<EntityCombustByEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityKnockbackByEntity => {
            register_typed_event::<EntityKnockbackByEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPortalEnter => {
            register_typed_event::<EntityPortalEnterEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityPortalExit => {
            register_typed_event::<EntityPortalExitEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityRemove => {
            register_typed_event::<EntityRemoveEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityTargetBlock => {
            register_typed_event::<EntityTargetBlockEvent>(ctx, handler, priority, blocking);
        }
        EventType::EntityUnleash => {
            register_typed_event::<EntityUnleashEvent>(ctx, handler, priority, blocking);
        }
        EventType::ExpBottle => {
            register_typed_event::<ExpBottleEvent>(ctx, handler, priority, blocking);
        }
        EventType::HorseJump => {
            register_typed_event::<HorseJumpEvent>(ctx, handler, priority, blocking);
        }
        EventType::LingeringPotionSplash => {
            register_typed_event::<LingeringPotionSplashEvent>(ctx, handler, priority, blocking);
        }
        EventType::PigZap => {
            register_typed_event::<PigZapEvent>(ctx, handler, priority, blocking);
        }
        EventType::PigZombieAnger => {
            register_typed_event::<PigZombieAngerEvent>(ctx, handler, priority, blocking);
        }
        EventType::PotionSplash => {
            register_typed_event::<PotionSplashEvent>(ctx, handler, priority, blocking);
        }
        EventType::SpawnerSpawn => {
            register_typed_event::<SpawnerSpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::TrialSpawnerSpawn => {
            register_typed_event::<TrialSpawnerSpawnEvent>(ctx, handler, priority, blocking);
        }
        EventType::VillagerReputationChange => {
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
            register_typed_event::<InventoryOpenEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryDrag => {
            register_typed_event::<InventoryDragEvent>(resource, handler, priority, blocking);
        }
        EventType::CraftItem => {
            register_typed_event::<CraftItemEvent>(resource, handler, priority, blocking);
        }
        EventType::FurnaceSmelt => {
            register_typed_event::<FurnaceSmeltEvent>(resource, handler, priority, blocking);
        }
        EventType::Brew => {
            register_typed_event::<BrewEvent>(resource, handler, priority, blocking);
        }
        EventType::BrewingStandFuel => {
            register_typed_event::<BrewingStandFuelEvent>(resource, handler, priority, blocking);
        }
        EventType::FurnaceBurn => {
            register_typed_event::<FurnaceBurnEvent>(resource, handler, priority, blocking);
        }
        EventType::FurnaceExtract => {
            register_typed_event::<FurnaceExtractEvent>(resource, handler, priority, blocking);
        }
        EventType::FurnaceStartSmelt => {
            register_typed_event::<FurnaceStartSmeltEvent>(resource, handler, priority, blocking);
        }
        EventType::HopperInventorySearch => {
            register_typed_event::<HopperInventorySearchEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::InventoryCreative => {
            register_typed_event::<InventoryCreativeEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryInteract => {
            register_typed_event::<InventoryInteractEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryMoveItem => {
            register_typed_event::<InventoryMoveItemEvent>(resource, handler, priority, blocking);
        }
        EventType::InventoryPickupItem => {
            register_typed_event::<InventoryPickupItemEvent>(resource, handler, priority, blocking);
        }
        EventType::PrepareAnvil => {
            register_typed_event::<PrepareAnvilEvent>(resource, handler, priority, blocking);
        }
        EventType::PrepareGrindstone => {
            register_typed_event::<PrepareGrindstoneEvent>(resource, handler, priority, blocking);
        }
        EventType::PrepareInventoryResult => {
            register_typed_event::<PrepareInventoryResultEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::PrepareItemCraft => {
            register_typed_event::<PrepareItemCraftEvent>(resource, handler, priority, blocking);
        }
        EventType::PrepareSmithing => {
            register_typed_event::<PrepareSmithingEvent>(resource, handler, priority, blocking);
        }
        EventType::SmithItem => {
            register_typed_event::<SmithItemEvent>(resource, handler, priority, blocking);
        }
        EventType::TradeSelect => {
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
            register_typed_event::<VehicleBlockCollisionEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::VehicleCollision => {
            register_typed_event::<VehicleCollisionEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleCreate => {
            register_typed_event::<VehicleCreateEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleDamage => {
            register_typed_event::<VehicleDamageEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleDestroy => {
            register_typed_event::<VehicleDestroyEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleEnter => {
            register_typed_event::<VehicleEnterEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleEntityCollision => {
            register_typed_event::<VehicleEntityCollisionEvent>(
                resource, handler, priority, blocking,
            );
        }
        EventType::VehicleExit => {
            register_typed_event::<VehicleExitEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleMove => {
            register_typed_event::<VehicleMoveEvent>(resource, handler, priority, blocking);
        }
        EventType::VehicleUpdate => {
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
    use pumpkin_core::plugin::enchantment::{
        enchant_item::EnchantItemEvent, prepare_item_enchant::PrepareItemEnchantEvent,
    };

    match event_type {
        EventType::PrepareItemEnchant => {
            register_typed_event::<PrepareItemEnchantEvent>(resource, handler, priority, blocking);
        }
        EventType::EnchantItem => {
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
    use pumpkin_core::plugin::world::{
        chunk_load::ChunkLoad, chunk_save::ChunkSave, chunk_send::ChunkSend,
        spawn_change::SpawnChangeEvent,
    };

    match event_type {
        EventType::SpawnChange => {
            register_typed_event::<SpawnChangeEvent>(resource, handler, priority, blocking);
        }
        EventType::ChunkLoad => {
            register_typed_event::<ChunkLoad>(resource, handler, priority, blocking);
        }
        EventType::ChunkSave => {
            register_typed_event::<ChunkSave>(resource, handler, priority, blocking);
        }
        EventType::ChunkSend => {
            register_typed_event::<ChunkSend>(resource, handler, priority, blocking);
        }
        EventType::WeatherChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::weather_change::WeatherChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::ThunderChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::weather_change::ThunderChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::WorldLoad => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::world_load::WorldLoadEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::WorldUnload => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::world_load::WorldUnloadEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::AsyncStructureGenerate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::async_structure_generate::AsyncStructureGenerateEvent,
            >(resource, handler, priority, blocking)
            ;
        }
        EventType::AsyncStructureSpawn => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::async_structure_spawn::AsyncStructureSpawnEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::ChunkPopulate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::chunk_populate::ChunkPopulateEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::ChunkUnload => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::chunk_unload::ChunkUnloadEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::EntitiesLoad => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::entities_load::EntitiesLoadEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::EntitiesUnload => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::entities_unload::EntitiesUnloadEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::GenericGame => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::generic_game::GenericGameEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::LootGenerate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::loot_generate::LootGenerateEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::PortalCreate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::portal_create::PortalCreateEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::StructureGrow => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::structure_grow::StructureGrowEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::TimeSkip => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::time_skip::TimeSkipEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::WorldInit => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::world_init::WorldInitEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::WorldSave => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::world_save::WorldSaveEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::LightningStrike => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::world::lightning_strike::LightningStrikeEvent,
            >(resource, handler, priority, blocking);
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
    use pumpkin_core::plugin::block::{
        block_break::BlockBreakEvent, block_burn::BlockBurnEvent,
        block_can_build::BlockCanBuildEvent, block_grow::BlockGrowEvent,
        block_place::BlockPlaceEvent, block_redstone::BlockRedstoneEvent,
    };

    match event_type {
        EventType::BlockRedstone => {
            register_typed_event::<BlockRedstoneEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockBreak => {
            register_typed_event::<BlockBreakEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockBurn => {
            register_typed_event::<BlockBurnEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockCanBuild => {
            register_typed_event::<BlockCanBuildEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockGrow => {
            register_typed_event::<BlockGrowEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockPlace => {
            register_typed_event::<BlockPlaceEvent>(resource, handler, priority, blocking);
        }
        EventType::BlockDamage => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_damage::BlockDamageEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockIgnite => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_ignite::BlockIgniteEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockFromTo => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_from_to::BlockFromToEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockForm => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_form::BlockFormEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockFade => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_fade::BlockFadeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockDispense => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_dispense::BlockDispenseEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockExplode => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_explode::BlockExplodeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockPhysics => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_physics::BlockPhysicsEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockPistonExtend => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_piston::BlockPistonExtendEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockPistonRetract => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_piston::BlockPistonRetractEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::NotePlay => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::note_play::NotePlayEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::SignChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::sign_change::SignChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::SpongeAbsorb => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::sponge_absorb::SpongeAbsorbEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::TntPrime => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::tnt_prime::TNTPrimeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BellResonate => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::bell_resonate::BellResonateEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BellRing => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::bell_ring::BellRingEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockBrush => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_brush::BlockBrushEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockCook => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_cook::BlockCookEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockDamageAbort => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_damage_abort::BlockDamageAbortEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockDispenseArmor => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_dispense_armor::BlockDispenseArmorEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockDispenseLoot => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_dispense_loot::BlockDispenseLootEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockDropItem => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_drop_item::BlockDropItemEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockExp => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_exp::BlockExpEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockFertilize => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_fertilize::BlockFertilizeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockMultiPlace => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_multi_place::BlockMultiPlaceEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockReceiveGame => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_receive_game::BlockReceiveGameEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockShearEntity => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_shear_entity::BlockShearEntityEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BlockSpread => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::block_spread::BlockSpreadEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::BrewingStart => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::brewing_start::BrewingStartEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::CampfireStart => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::campfire_start::CampfireStartEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::CauldronLevelChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::cauldron_level_change::CauldronLevelChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::CrafterCraft => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::crafter_craft::CrafterCraftEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::EntityBlockForm => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::entity_block_form::EntityBlockFormEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::FluidLevelChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::fluid_level_change::FluidLevelChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::InventoryBlockStart => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::inventory_block_start::InventoryBlockStartEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::LeavesDecay => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::leaves_decay::LeavesDecayEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::MoistureChange => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::moisture_change::MoistureChangeEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::SculkBloom => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::sculk_bloom::SculkBloomEvent,
            >(resource, handler, priority, blocking);
        }
        EventType::VaultDisplayItem => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::block::vault_display_item::VaultDisplayItemEvent,
            >(resource, handler, priority, blocking);
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
    use pumpkin_core::plugin::raid::{
        raid_finish::RaidFinishEvent, raid_spawn_wave::RaidSpawnWaveEvent,
        raid_stop::RaidStopEvent, raid_trigger::RaidTriggerEvent,
    };

    match event_type {
        EventType::RaidFinish => {
            register_typed_event::<RaidFinishEvent>(resource, handler, priority, blocking);
        }
        EventType::RaidSpawnWave => {
            register_typed_event::<RaidSpawnWaveEvent>(resource, handler, priority, blocking);
        }
        EventType::RaidStop => {
            register_typed_event::<RaidStopEvent>(resource, handler, priority, blocking);
        }
        EventType::RaidTrigger => {
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
    use pumpkin_core::plugin::api::events::dialog::{
        DialogClearEvent, DialogClickActionEvent, DialogShowEvent,
    };

    match event_type {
        EventType::DialogClickAction => {
            register_typed_event::<DialogClickActionEvent>(resource, handler, priority, blocking);
        }
        EventType::DialogShow => {
            register_typed_event::<DialogShowEvent>(resource, handler, priority, blocking);
        }
        EventType::DialogClear => {
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
    use pumpkin_core::plugin::server::{
        list_ping::ServerListPingEvent,
        packet::{PacketReceivedEvent, PacketSentEvent},
        server_broadcast::ServerBroadcastEvent,
        server_command::ServerCommandEvent,
        server_load::ServerLoadEvent,
        server_tick_end::ServerTickEndEvent,
        server_tick_start::ServerTickStartEvent,
    };

    match event_type {
        EventType::PacketReceived => {
            register_typed_event::<PacketReceivedEvent>(resource, handler, priority, blocking);
        }
        EventType::PacketSent => {
            register_typed_event::<PacketSentEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerCommand => {
            register_typed_event::<ServerCommandEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerListPing => {
            register_typed_event::<ServerListPingEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerBroadcast => {
            register_typed_event::<ServerBroadcastEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerLoad => {
            register_typed_event::<ServerLoadEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerTickEnd => {
            register_typed_event::<ServerTickEndEvent>(resource, handler, priority, blocking);
        }
        EventType::ServerTickStart => {
            register_typed_event::<ServerTickStartEvent>(resource, handler, priority, blocking);
        }
        EventType::MapInitialize => {
            register_typed_event::<
                pumpkin_core::plugin::api::events::server::map_initialize::MapInitializeEvent,
            >(resource, handler, priority, blocking);
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
    use pumpkin_core::plugin::hanging::{
        hanging_break::HangingBreakEvent, hanging_break_by_entity::HangingBreakByEntityEvent,
        hanging_place::HangingPlaceEvent,
    };

    match event_type {
        EventType::HangingBreak => {
            register_typed_event::<HangingBreakEvent>(ctx, handler, priority, blocking);
        }
        EventType::HangingBreakByEntity => {
            register_typed_event::<HangingBreakByEntityEvent>(ctx, handler, priority, blocking);
        }
        EventType::HangingPlace => {
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
            event_type @ (EventType::PacketReceived
            | EventType::PacketSent
            | EventType::ServerCommand
            | EventType::ServerListPing
            | EventType::ServerBroadcast
            | EventType::ServerLoad
            | EventType::ServerTickEnd
            | EventType::ServerTickStart
            | EventType::MapInitialize) => {
                register_server_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::HangingBreak
            | EventType::HangingBreakByEntity
            | EventType::HangingPlace) => {
                register_hanging_event(ctx, &handler, priority, blocking, event_type);
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
                register_world_event(ctx, &handler, priority, blocking, event_type);
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
                register_entity_event(ctx, &handler, priority, blocking, event_type);
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
                register_block_event(ctx, &handler, priority, blocking, event_type);
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
                register_inventory_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::PrepareItemEnchant | EventType::EnchantItem) => {
                register_enchantment_event(ctx, &handler, priority, blocking, event_type);
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
                register_vehicle_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::RaidFinish
            | EventType::RaidSpawnWave
            | EventType::RaidStop
            | EventType::RaidTrigger) => {
                register_raid_event(ctx, &handler, priority, blocking, event_type);
            }
            event_type @ (EventType::DialogClickAction
            | EventType::DialogShow
            | EventType::DialogClear) => {
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
