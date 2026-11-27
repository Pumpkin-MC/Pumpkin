use std::{collections::HashMap, sync::Arc};
use wasmtime::component::Resource;

use crate::{
    events::WasmPluginEventHandler,
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
use pumpkin_core::plugin::{Context, DynEventHandler, Payload};
use pumpkin_wasm_host_common::state::PluginHostState;

impl pumpkin::plugin::context::Host for PluginHostState {}

#[expect(clippy::too_many_lines)]
fn event_type_to_name(event_type: EventType) -> &'static str {
    use pumpkin_core::plugin::api::events::{
        block::{
            bell_resonate::BellResonateEvent,
            bell_ring::BellRingEvent,
            block_break::BlockBreakEvent,
            block_brush::BlockBrushEvent,
            block_burn::BlockBurnEvent,
            block_can_build::BlockCanBuildEvent,
            block_cook::BlockCookEvent,
            block_damage::BlockDamageEvent,
            block_damage_abort::BlockDamageAbortEvent,
            block_dispense::BlockDispenseEvent,
            block_dispense_armor::BlockDispenseArmorEvent,
            block_dispense_loot::BlockDispenseLootEvent,
            block_drop_item::BlockDropItemEvent,
            block_exp::BlockExpEvent,
            block_explode::BlockExplodeEvent,
            block_fade::BlockFadeEvent,
            block_fertilize::BlockFertilizeEvent,
            block_form::BlockFormEvent,
            block_from_to::BlockFromToEvent,
            block_grow::BlockGrowEvent,
            block_ignite::BlockIgniteEvent,
            block_multi_place::BlockMultiPlaceEvent,
            block_physics::BlockPhysicsEvent,
            block_piston::{BlockPistonExtendEvent, BlockPistonRetractEvent},
            block_place::BlockPlaceEvent,
            block_receive_game::BlockReceiveGameEvent,
            block_redstone::BlockRedstoneEvent,
            block_shear_entity::BlockShearEntityEvent,
            block_spread::BlockSpreadEvent,
            brewing_start::BrewingStartEvent,
            campfire_start::CampfireStartEvent,
            cauldron_level_change::CauldronLevelChangeEvent,
            crafter_craft::CrafterCraftEvent,
            entity_block_form::EntityBlockFormEvent,
            fluid_level_change::FluidLevelChangeEvent,
            inventory_block_start::InventoryBlockStartEvent,
            leaves_decay::LeavesDecayEvent,
            moisture_change::MoistureChangeEvent,
            note_play::NotePlayEvent,
            sculk_bloom::SculkBloomEvent,
            sign_change::SignChangeEvent,
            sponge_absorb::SpongeAbsorbEvent,
            tnt_prime::TNTPrimeEvent,
            vault_display_item::VaultDisplayItemEvent,
        },
        dialog::{DialogClearEvent, DialogClickActionEvent, DialogShowEvent},
        enchantment::{
            enchant_item::EnchantItemEvent, prepare_item_enchant::PrepareItemEnchantEvent,
        },
        entity::{
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
        },
        hanging::{
            hanging_break::HangingBreakEvent, hanging_break_by_entity::HangingBreakByEntityEvent,
            hanging_place::HangingPlaceEvent,
        },
        inventory::{
            brew::BrewEvent, brewing_stand_fuel::BrewingStandFuelEvent, craft_item::CraftItemEvent,
            furnace_burn::FurnaceBurnEvent, furnace_extract::FurnaceExtractEvent,
            furnace_smelt::FurnaceSmeltEvent, furnace_start_smelt::FurnaceStartSmeltEvent,
            hopper_inventory_search::HopperInventorySearchEvent,
            inventory_creative::InventoryCreativeEvent, inventory_drag::InventoryDragEvent,
            inventory_interact::InventoryInteractEvent,
            inventory_move_item::InventoryMoveItemEvent, inventory_open::InventoryOpenEvent,
            inventory_pickup_item::InventoryPickupItemEvent, prepare_anvil::PrepareAnvilEvent,
            prepare_grindstone::PrepareGrindstoneEvent,
            prepare_inventory_result::PrepareInventoryResultEvent,
            prepare_item_craft::PrepareItemCraftEvent, prepare_smithing::PrepareSmithingEvent,
            smith_item::SmithItemEvent, trade_select::TradeSelectEvent,
        },
        player::{
            async_player_chat::AsyncPlayerChatEvent,
            async_player_pre_login::AsyncPlayerPreLoginEvent,
            bedrock_form_response::BedrockFormResponseEvent,
            changed_main_hand::PlayerChangedMainHandEvent,
            egg_throw::PlayerEggThrowEvent,
            exp_change::PlayerExpChangeEvent,
            fish::PlayerFishEvent,
            inventory_close::InventoryCloseEvent,
            inventory_interact::InventoryClickEvent,
            item_held::PlayerItemHeldEvent,
            player_advancement_done::PlayerAdvancementDoneEvent,
            player_animation::PlayerAnimationEvent,
            player_armor_stand_manipulate::PlayerArmorStandManipulateEvent,
            player_bed::{PlayerBedEnterEvent, PlayerBedLeaveEvent},
            player_bucket::{PlayerBucketEmptyEvent, PlayerBucketFillEvent},
            player_bucket_entity::PlayerBucketEntityEvent,
            player_change_world::PlayerChangeWorldEvent,
            player_changed_world::PlayerChangedWorldEvent,
            player_channel::PlayerChannelEvent,
            player_chat::PlayerChatEvent,
            player_command_preprocess::PlayerCommandPreprocessEvent,
            player_command_send::PlayerCommandSendEvent,
            player_custom_payload::PlayerCustomPayloadEvent,
            player_drop_item::PlayerDropItemEvent,
            player_edit_book::PlayerEditBookEvent,
            player_elytra_boost::PlayerElytraBoostEvent,
            player_exp_cooldown_change::PlayerExpCooldownChangeEvent,
            player_gamemode_change::PlayerGamemodeChangeEvent,
            player_harvest_block::PlayerHarvestBlockEvent,
            player_hide_entity::PlayerHideEntityEvent,
            player_input::PlayerInputEvent,
            player_interact_at_entity::PlayerInteractAtEntityEvent,
            player_interact_entity_event::PlayerInteractEntityEvent,
            player_interact_event::PlayerInteractEvent,
            player_interact_unknown_entity_event::PlayerInteractUnknownEntityEvent,
            player_item_break::PlayerItemBreakEvent,
            player_item_consume::PlayerItemConsumeEvent,
            player_item_damage::PlayerItemDamageEvent,
            player_item_mend::PlayerItemMendEvent,
            player_join::PlayerJoinEvent,
            player_kick::PlayerKickEvent,
            player_leash_entity::PlayerLeashEntityEvent,
            player_leave::PlayerLeaveEvent,
            player_level_change::PlayerLevelChangeEvent,
            player_links_send::PlayerLinksSendEvent,
            player_locale_change::PlayerLocaleChangeEvent,
            player_login::PlayerLoginEvent,
            player_move::PlayerMoveEvent,
            player_name_entity::PlayerNameEntityEvent,
            player_open_sign::PlayerOpenSignEvent,
            player_permission_check::PlayerPermissionCheckEvent,
            player_pickup_arrow::PlayerPickupArrowEvent,
            player_portal::PlayerPortalEvent,
            player_pre_login::PlayerPreLoginEvent,
            player_recipe_book_click::PlayerRecipeBookClickEvent,
            player_recipe_book_settings_change::PlayerRecipeBookSettingsChangeEvent,
            player_recipe_discover::PlayerRecipeDiscoverEvent,
            player_register_channel::PlayerRegisterChannelEvent,
            player_resource_pack_status::PlayerResourcePackStatusEvent,
            player_respawn::PlayerRespawnEvent,
            player_riptide::PlayerRiptideEvent,
            player_shear_entity::PlayerShearEntityEvent,
            player_show_entity::PlayerShowEntityEvent,
            player_spawn_change::PlayerSpawnChangeEvent,
            player_spawn_location::PlayerSpawnLocationEvent,
            player_statistic_increment::PlayerStatisticIncrementEvent,
            player_swap_hands::PlayerSwapHandItemsEvent,
            player_take_lectern_book::PlayerTakeLecternBookEvent,
            player_teleport::PlayerTeleportEvent,
            player_toggle_flight_event::PlayerToggleFlightEvent,
            player_toggle_sneak_event::PlayerToggleSneakEvent,
            player_toggle_sprint_event::PlayerToggleSprintEvent,
            player_transfer::PlayerTransferEvent,
            player_unleash_entity::PlayerUnleashEntityEvent,
            player_unregister_channel::PlayerUnregisterChannelEvent,
            player_velocity::PlayerVelocityEvent,
        },
        raid::{
            raid_finish::RaidFinishEvent, raid_spawn_wave::RaidSpawnWaveEvent,
            raid_stop::RaidStopEvent, raid_trigger::RaidTriggerEvent,
        },
        server::{
            list_ping::ServerListPingEvent,
            map_initialize::MapInitializeEvent,
            packet::{
                BedrockConnectionPacketReceivedEvent, BedrockConnectionPacketSentEvent,
                JavaConnectionPacketReceivedEvent, JavaConnectionPacketSentEvent,
                PacketReceivedEvent, PacketSentEvent,
            },
            server_broadcast::ServerBroadcastEvent,
            server_command::ServerCommandEvent,
            server_load::ServerLoadEvent,
            server_tick_end::ServerTickEndEvent,
            server_tick_start::ServerTickStartEvent,
        },
        vehicle::{
            vehicle_block_collision::VehicleBlockCollisionEvent,
            vehicle_collision::VehicleCollisionEvent, vehicle_create::VehicleCreateEvent,
            vehicle_damage::VehicleDamageEvent, vehicle_destroy::VehicleDestroyEvent,
            vehicle_enter::VehicleEnterEvent,
            vehicle_entity_collision::VehicleEntityCollisionEvent, vehicle_exit::VehicleExitEvent,
            vehicle_move::VehicleMoveEvent, vehicle_update::VehicleUpdateEvent,
        },
        world::{
            async_structure_generate::AsyncStructureGenerateEvent,
            async_structure_spawn::AsyncStructureSpawnEvent,
            chunk_load::ChunkLoad,
            chunk_populate::ChunkPopulateEvent,
            chunk_save::ChunkSave,
            chunk_send::ChunkSend,
            chunk_unload::ChunkUnloadEvent,
            entities_load::EntitiesLoadEvent,
            entities_unload::EntitiesUnloadEvent,
            generic_game::GenericGameEvent,
            lightning_strike::LightningStrikeEvent,
            loot_generate::LootGenerateEvent,
            portal_create::PortalCreateEvent,
            spawn_change::SpawnChangeEvent,
            structure_grow::StructureGrowEvent,
            time_skip::TimeSkipEvent,
            weather_change::{ThunderChangeEvent, WeatherChangeEvent},
            world_init::WorldInitEvent,
            world_load::{WorldLoadEvent, WorldUnloadEvent},
            world_save::WorldSaveEvent,
        },
    };

    match event_type {
        // Block
        EventType::BellResonate => BellResonateEvent::get_name_static(),
        EventType::BellRing => BellRingEvent::get_name_static(),
        EventType::BlockBreak => BlockBreakEvent::get_name_static(),
        EventType::BlockBrush => BlockBrushEvent::get_name_static(),
        EventType::BlockBurn => BlockBurnEvent::get_name_static(),
        EventType::BlockCanBuild => BlockCanBuildEvent::get_name_static(),
        EventType::BlockCook => BlockCookEvent::get_name_static(),
        EventType::BlockDamage => BlockDamageEvent::get_name_static(),
        EventType::BlockDamageAbort => BlockDamageAbortEvent::get_name_static(),
        EventType::BlockDispense => BlockDispenseEvent::get_name_static(),
        EventType::BlockDispenseArmor => BlockDispenseArmorEvent::get_name_static(),
        EventType::BlockDispenseLoot => BlockDispenseLootEvent::get_name_static(),
        EventType::BlockDropItem => BlockDropItemEvent::get_name_static(),
        EventType::BlockExp => BlockExpEvent::get_name_static(),
        EventType::BlockExplode => BlockExplodeEvent::get_name_static(),
        EventType::BlockFade => BlockFadeEvent::get_name_static(),
        EventType::BlockFertilize => BlockFertilizeEvent::get_name_static(),
        EventType::BlockForm => BlockFormEvent::get_name_static(),
        EventType::BlockFromTo => BlockFromToEvent::get_name_static(),
        EventType::BlockGrow => BlockGrowEvent::get_name_static(),
        EventType::BlockIgnite => BlockIgniteEvent::get_name_static(),
        EventType::BlockMultiPlace => BlockMultiPlaceEvent::get_name_static(),
        EventType::BlockPhysics => BlockPhysicsEvent::get_name_static(),
        EventType::BlockPistonExtend => BlockPistonExtendEvent::get_name_static(),
        EventType::BlockPistonRetract => BlockPistonRetractEvent::get_name_static(),
        EventType::BlockPlace => BlockPlaceEvent::get_name_static(),
        EventType::BlockReceiveGame => BlockReceiveGameEvent::get_name_static(),
        EventType::BlockRedstone => BlockRedstoneEvent::get_name_static(),
        EventType::BlockShearEntity => BlockShearEntityEvent::get_name_static(),
        EventType::BlockSpread => BlockSpreadEvent::get_name_static(),
        EventType::BrewingStart => BrewingStartEvent::get_name_static(),
        EventType::CampfireStart => CampfireStartEvent::get_name_static(),
        EventType::CauldronLevelChange => CauldronLevelChangeEvent::get_name_static(),
        EventType::CrafterCraft => CrafterCraftEvent::get_name_static(),
        EventType::EntityBlockForm => EntityBlockFormEvent::get_name_static(),
        EventType::FluidLevelChange => FluidLevelChangeEvent::get_name_static(),
        EventType::InventoryBlockStart => InventoryBlockStartEvent::get_name_static(),
        EventType::LeavesDecay => LeavesDecayEvent::get_name_static(),
        EventType::MoistureChange => MoistureChangeEvent::get_name_static(),
        EventType::NotePlay => NotePlayEvent::get_name_static(),
        EventType::SculkBloom => SculkBloomEvent::get_name_static(),
        EventType::SignChange => SignChangeEvent::get_name_static(),
        EventType::SpongeAbsorb => SpongeAbsorbEvent::get_name_static(),
        EventType::TntPrime => TNTPrimeEvent::get_name_static(),
        EventType::VaultDisplayItem => VaultDisplayItemEvent::get_name_static(),

        // Dialog
        EventType::DialogClear => DialogClearEvent::get_name_static(),
        EventType::DialogClickAction => DialogClickActionEvent::get_name_static(),
        EventType::DialogShow => DialogShowEvent::get_name_static(),

        // Enchantment
        EventType::EnchantItem => EnchantItemEvent::get_name_static(),
        EventType::PrepareItemEnchant => PrepareItemEnchantEvent::get_name_static(),

        // Entity
        EventType::AreaEffectCloudApply => AreaEffectCloudApplyEvent::get_name_static(),
        EventType::ArrowBodyCountChange => ArrowBodyCountChangeEvent::get_name_static(),
        EventType::BatToggleSleep => BatToggleSleepEvent::get_name_static(),
        EventType::CreatureSpawn => CreatureSpawnEvent::get_name_static(),
        EventType::CreeperPower => CreeperPowerEvent::get_name_static(),
        EventType::EnderDragonChangePhase => EnderDragonChangePhaseEvent::get_name_static(),
        EventType::EntityAirChange => EntityAirChangeEvent::get_name_static(),
        EventType::EntityBreakDoor => EntityBreakDoorEvent::get_name_static(),
        EventType::EntityBreed => EntityBreedEvent::get_name_static(),
        EventType::EntityChangeBlock => EntityChangeBlockEvent::get_name_static(),
        EventType::EntityCombust => EntityCombustEvent::get_name_static(),
        EventType::EntityCombustByBlock => EntityCombustByBlockEvent::get_name_static(),
        EventType::EntityCombustByEntity => EntityCombustByEntityEvent::get_name_static(),
        EventType::EntityDamage => EntityDamageEvent::get_name_static(),
        EventType::EntityDamageByBlock => EntityDamageByBlockEvent::get_name_static(),
        EventType::EntityDamageByEntity => EntityDamageByEntityEvent::get_name_static(),
        EventType::EntityDeath => EntityDeathEvent::get_name_static(),
        EventType::PlayerDeath => PlayerDeathEvent::get_name_static(),
        EventType::EntityDismount => EntityDismountEvent::get_name_static(),
        EventType::EntityDropItem => EntityDropItemEvent::get_name_static(),
        EventType::EntityDye => EntityDyeEvent::get_name_static(),
        EventType::EntityEnterBlock => EntityEnterBlockEvent::get_name_static(),
        EventType::EntityEnterLoveMode => EntityEnterLoveModeEvent::get_name_static(),
        EventType::EntityExhaustion => EntityExhaustionEvent::get_name_static(),
        EventType::EntityExplode => EntityExplodeEvent::get_name_static(),
        EventType::EntityInteract => EntityInteractEvent::get_name_static(),
        EventType::EntityKnockback => EntityKnockbackEvent::get_name_static(),
        EventType::EntityKnockbackByEntity => EntityKnockbackByEntityEvent::get_name_static(),
        EventType::EntityMount => EntityMountEvent::get_name_static(),
        EventType::EntityPickupItem => EntityPickupItemEvent::get_name_static(),
        EventType::EntityPlace => EntityPlaceEvent::get_name_static(),
        EventType::EntityPortal => EntityPortalEvent::get_name_static(),
        EventType::EntityPortalEnter => EntityPortalEnterEvent::get_name_static(),
        EventType::EntityPortalExit => EntityPortalExitEvent::get_name_static(),
        EventType::EntityPoseChange => EntityPoseChangeEvent::get_name_static(),
        EventType::EntityPotionEffect => EntityPotionEffectEvent::get_name_static(),
        EventType::EntityRegainHealth => EntityRegainHealthEvent::get_name_static(),
        EventType::EntityRemove => EntityRemoveEvent::get_name_static(),
        EventType::EntityResurrect => EntityResurrectEvent::get_name_static(),
        EventType::EntityShootBow => EntityShootBowEvent::get_name_static(),
        EventType::EntitySpawn => EntitySpawnEvent::get_name_static(),
        EventType::EntitySpellCast => EntitySpellCastEvent::get_name_static(),
        EventType::EntityTame => EntityTameEvent::get_name_static(),
        EventType::EntityTarget => EntityTargetEvent::get_name_static(),
        EventType::EntityTargetBlock => EntityTargetBlockEvent::get_name_static(),
        EventType::EntityTargetLivingEntity => EntityTargetLivingEntityEvent::get_name_static(),
        EventType::EntityTeleport => EntityTeleportEvent::get_name_static(),
        EventType::EntityToggleGlide => EntityToggleGlideEvent::get_name_static(),
        EventType::EntityToggleSit => EntityToggleSitEvent::get_name_static(),
        EventType::EntityToggleSwim => EntityToggleSwimEvent::get_name_static(),
        EventType::EntityTransform => EntityTransformEvent::get_name_static(),
        EventType::EntityUnleash => EntityUnleashEvent::get_name_static(),
        EventType::ExpBottle => ExpBottleEvent::get_name_static(),
        EventType::ExplosionPrime => ExplosionPrimeEvent::get_name_static(),
        EventType::FireworkExplode => FireworkExplodeEvent::get_name_static(),
        EventType::FoodLevelChange => FoodLevelChangeEvent::get_name_static(),
        EventType::HorseJump => HorseJumpEvent::get_name_static(),
        EventType::ItemDespawn => ItemDespawnEvent::get_name_static(),
        EventType::ItemMerge => ItemMergeEvent::get_name_static(),
        EventType::ItemSpawn => ItemSpawnEvent::get_name_static(),
        EventType::LingeringPotionSplash => LingeringPotionSplashEvent::get_name_static(),
        EventType::PigZap => PigZapEvent::get_name_static(),
        EventType::PigZombieAnger => PigZombieAngerEvent::get_name_static(),
        EventType::PiglinBarter => PiglinBarterEvent::get_name_static(),
        EventType::PotionSplash => PotionSplashEvent::get_name_static(),
        EventType::ProjectileHit => ProjectileHitEvent::get_name_static(),
        EventType::ProjectileLaunch => ProjectileLaunchEvent::get_name_static(),
        EventType::SheepDyeWool => SheepDyeWoolEvent::get_name_static(),
        EventType::SheepRegrowWool => SheepRegrowWoolEvent::get_name_static(),
        EventType::SlimeSplit => SlimeSplitEvent::get_name_static(),
        EventType::SpawnerSpawn => SpawnerSpawnEvent::get_name_static(),
        EventType::StriderTemperatureChange => StriderTemperatureChangeEvent::get_name_static(),
        EventType::TrialSpawnerSpawn => TrialSpawnerSpawnEvent::get_name_static(),
        EventType::VillagerAcquireTrade => VillagerAcquireTradeEvent::get_name_static(),
        EventType::VillagerCareerChange => VillagerCareerChangeEvent::get_name_static(),
        EventType::VillagerReplenishTrade => VillagerReplenishTradeEvent::get_name_static(),
        EventType::VillagerReputationChange => VillagerReputationChangeEvent::get_name_static(),
        EventType::WardenAngerChange => WardenAngerChangeEvent::get_name_static(),

        // Hanging
        EventType::HangingBreak => HangingBreakEvent::get_name_static(),
        EventType::HangingBreakByEntity => HangingBreakByEntityEvent::get_name_static(),
        EventType::HangingPlace => HangingPlaceEvent::get_name_static(),

        // Inventory
        EventType::Brew => BrewEvent::get_name_static(),
        EventType::BrewingStandFuel => BrewingStandFuelEvent::get_name_static(),
        EventType::CraftItem => CraftItemEvent::get_name_static(),
        EventType::FurnaceBurn => FurnaceBurnEvent::get_name_static(),
        EventType::FurnaceExtract => FurnaceExtractEvent::get_name_static(),
        EventType::FurnaceSmelt => FurnaceSmeltEvent::get_name_static(),
        EventType::FurnaceStartSmelt => FurnaceStartSmeltEvent::get_name_static(),
        EventType::HopperInventorySearch => HopperInventorySearchEvent::get_name_static(),
        EventType::InventoryCreative => InventoryCreativeEvent::get_name_static(),
        EventType::InventoryDrag => InventoryDragEvent::get_name_static(),
        EventType::InventoryInteract => InventoryInteractEvent::get_name_static(),
        EventType::InventoryMoveItem => InventoryMoveItemEvent::get_name_static(),
        EventType::InventoryOpen => InventoryOpenEvent::get_name_static(),
        EventType::InventoryPickupItem => InventoryPickupItemEvent::get_name_static(),
        EventType::PrepareAnvil => PrepareAnvilEvent::get_name_static(),
        EventType::PrepareGrindstone => PrepareGrindstoneEvent::get_name_static(),
        EventType::PrepareInventoryResult => PrepareInventoryResultEvent::get_name_static(),
        EventType::PrepareItemCraft => PrepareItemCraftEvent::get_name_static(),
        EventType::PrepareSmithing => PrepareSmithingEvent::get_name_static(),
        EventType::SmithItem => SmithItemEvent::get_name_static(),
        EventType::TradeSelect => TradeSelectEvent::get_name_static(),

        // Player
        EventType::AsyncPlayerChat => AsyncPlayerChatEvent::get_name_static(),
        EventType::AsyncPlayerPreLogin => AsyncPlayerPreLoginEvent::get_name_static(),
        EventType::BedrockFormResponse => BedrockFormResponseEvent::get_name_static(),
        EventType::PlayerChangedMainHand => PlayerChangedMainHandEvent::get_name_static(),
        EventType::PlayerEggThrow => PlayerEggThrowEvent::get_name_static(),
        EventType::PlayerExpChange => PlayerExpChangeEvent::get_name_static(),
        EventType::PlayerFish => PlayerFishEvent::get_name_static(),
        EventType::InventoryClose => InventoryCloseEvent::get_name_static(),
        EventType::InventoryClick => InventoryClickEvent::get_name_static(),
        EventType::PlayerItemHeld => PlayerItemHeldEvent::get_name_static(),
        EventType::PlayerAdvancementDone => PlayerAdvancementDoneEvent::get_name_static(),
        EventType::PlayerAnimation => PlayerAnimationEvent::get_name_static(),
        EventType::PlayerArmorStandManipulate => PlayerArmorStandManipulateEvent::get_name_static(),
        EventType::PlayerBedEnter => PlayerBedEnterEvent::get_name_static(),
        EventType::PlayerBedLeave => PlayerBedLeaveEvent::get_name_static(),
        EventType::PlayerBucketEmpty => PlayerBucketEmptyEvent::get_name_static(),
        EventType::PlayerBucketFill => PlayerBucketFillEvent::get_name_static(),
        EventType::PlayerBucketEntity => PlayerBucketEntityEvent::get_name_static(),
        EventType::PlayerChangeWorld => PlayerChangeWorldEvent::get_name_static(),
        EventType::PlayerChangedWorld => PlayerChangedWorldEvent::get_name_static(),
        EventType::PlayerChannel => PlayerChannelEvent::get_name_static(),
        EventType::PlayerChat => PlayerChatEvent::get_name_static(),
        EventType::PlayerCommandPreprocess => PlayerCommandPreprocessEvent::get_name_static(),
        EventType::PlayerCommandSend => PlayerCommandSendEvent::get_name_static(),
        EventType::PlayerCustomPayload => PlayerCustomPayloadEvent::get_name_static(),
        EventType::PlayerDropItem => PlayerDropItemEvent::get_name_static(),
        EventType::PlayerEditBook => PlayerEditBookEvent::get_name_static(),
        EventType::PlayerElytraBoost => PlayerElytraBoostEvent::get_name_static(),
        EventType::PlayerExpCooldownChange => PlayerExpCooldownChangeEvent::get_name_static(),
        EventType::PlayerGamemodeChange => PlayerGamemodeChangeEvent::get_name_static(),
        EventType::PlayerHarvestBlock => PlayerHarvestBlockEvent::get_name_static(),
        EventType::PlayerHideEntity => PlayerHideEntityEvent::get_name_static(),
        EventType::PlayerInput => PlayerInputEvent::get_name_static(),
        EventType::PlayerInteractAtEntity => PlayerInteractAtEntityEvent::get_name_static(),
        EventType::PlayerInteractEntity => PlayerInteractEntityEvent::get_name_static(),
        EventType::PlayerInteract => PlayerInteractEvent::get_name_static(),
        EventType::PlayerInteractUnknownEntity => {
            PlayerInteractUnknownEntityEvent::get_name_static()
        }
        EventType::PlayerItemBreak => PlayerItemBreakEvent::get_name_static(),
        EventType::PlayerItemConsume => PlayerItemConsumeEvent::get_name_static(),
        EventType::PlayerItemDamage => PlayerItemDamageEvent::get_name_static(),
        EventType::PlayerItemMend => PlayerItemMendEvent::get_name_static(),
        EventType::PlayerJoin => PlayerJoinEvent::get_name_static(),
        EventType::PlayerKick => PlayerKickEvent::get_name_static(),
        EventType::PlayerLeashEntity => PlayerLeashEntityEvent::get_name_static(),
        EventType::PlayerLeave => PlayerLeaveEvent::get_name_static(),
        EventType::PlayerLevelChange => PlayerLevelChangeEvent::get_name_static(),
        EventType::PlayerLinksSend => PlayerLinksSendEvent::get_name_static(),
        EventType::PlayerLocaleChange => PlayerLocaleChangeEvent::get_name_static(),
        EventType::PlayerLogin => PlayerLoginEvent::get_name_static(),
        EventType::PlayerMove => PlayerMoveEvent::get_name_static(),
        EventType::PlayerNameEntity => PlayerNameEntityEvent::get_name_static(),
        EventType::PlayerOpenSign => PlayerOpenSignEvent::get_name_static(),
        EventType::PlayerPermissionCheck => PlayerPermissionCheckEvent::get_name_static(),
        EventType::PlayerPickupArrow => PlayerPickupArrowEvent::get_name_static(),
        EventType::PlayerPortal => PlayerPortalEvent::get_name_static(),
        EventType::PlayerPreLogin => PlayerPreLoginEvent::get_name_static(),
        EventType::PlayerRecipeBookClick => PlayerRecipeBookClickEvent::get_name_static(),
        EventType::PlayerRecipeBookSettingsChange => {
            PlayerRecipeBookSettingsChangeEvent::get_name_static()
        }
        EventType::PlayerRecipeDiscover => PlayerRecipeDiscoverEvent::get_name_static(),
        EventType::PlayerRegisterChannel => PlayerRegisterChannelEvent::get_name_static(),
        EventType::PlayerResourcePackStatus => PlayerResourcePackStatusEvent::get_name_static(),
        EventType::PlayerRespawn => PlayerRespawnEvent::get_name_static(),
        EventType::PlayerRiptide => PlayerRiptideEvent::get_name_static(),
        EventType::PlayerShearEntity => PlayerShearEntityEvent::get_name_static(),
        EventType::PlayerShowEntity => PlayerShowEntityEvent::get_name_static(),
        EventType::PlayerSpawnChange => PlayerSpawnChangeEvent::get_name_static(),
        EventType::PlayerSpawnLocation => PlayerSpawnLocationEvent::get_name_static(),
        EventType::PlayerStatisticIncrement => PlayerStatisticIncrementEvent::get_name_static(),
        EventType::PlayerSwapHands => PlayerSwapHandItemsEvent::get_name_static(),
        EventType::PlayerTakeLecternBook => PlayerTakeLecternBookEvent::get_name_static(),
        EventType::PlayerTeleport => PlayerTeleportEvent::get_name_static(),
        EventType::PlayerToggleFlight => PlayerToggleFlightEvent::get_name_static(),
        EventType::PlayerToggleSneak => PlayerToggleSneakEvent::get_name_static(),
        EventType::PlayerToggleSprint => PlayerToggleSprintEvent::get_name_static(),
        EventType::PlayerTransfer => PlayerTransferEvent::get_name_static(),
        EventType::PlayerUnleashEntity => PlayerUnleashEntityEvent::get_name_static(),
        EventType::PlayerUnregisterChannel => PlayerUnregisterChannelEvent::get_name_static(),
        EventType::PlayerVelocity => PlayerVelocityEvent::get_name_static(),

        // Raid
        EventType::RaidFinish => RaidFinishEvent::get_name_static(),
        EventType::RaidSpawnWave => RaidSpawnWaveEvent::get_name_static(),
        EventType::RaidStop => RaidStopEvent::get_name_static(),
        EventType::RaidTrigger => RaidTriggerEvent::get_name_static(),

        // Server
        EventType::ServerListPing => ServerListPingEvent::get_name_static(),
        EventType::MapInitialize => MapInitializeEvent::get_name_static(),
        EventType::BedrockConnectionPacketReceived => {
            BedrockConnectionPacketReceivedEvent::get_name_static()
        }
        EventType::BedrockConnectionPacketSent => {
            BedrockConnectionPacketSentEvent::get_name_static()
        }
        EventType::JavaConnectionPacketReceived => {
            JavaConnectionPacketReceivedEvent::get_name_static()
        }
        EventType::JavaConnectionPacketSent => JavaConnectionPacketSentEvent::get_name_static(),
        EventType::PacketReceived => PacketReceivedEvent::get_name_static(),
        EventType::PacketSent => PacketSentEvent::get_name_static(),
        EventType::ServerBroadcast => ServerBroadcastEvent::get_name_static(),
        EventType::ServerCommand => ServerCommandEvent::get_name_static(),
        EventType::ServerLoad => ServerLoadEvent::get_name_static(),
        EventType::ServerTickEnd => ServerTickEndEvent::get_name_static(),
        EventType::ServerTickStart => ServerTickStartEvent::get_name_static(),

        // Vehicle
        EventType::VehicleBlockCollision => VehicleBlockCollisionEvent::get_name_static(),
        EventType::VehicleCollision => VehicleCollisionEvent::get_name_static(),
        EventType::VehicleCreate => VehicleCreateEvent::get_name_static(),
        EventType::VehicleDamage => VehicleDamageEvent::get_name_static(),
        EventType::VehicleDestroy => VehicleDestroyEvent::get_name_static(),
        EventType::VehicleEnter => VehicleEnterEvent::get_name_static(),
        EventType::VehicleEntityCollision => VehicleEntityCollisionEvent::get_name_static(),
        EventType::VehicleExit => VehicleExitEvent::get_name_static(),
        EventType::VehicleMove => VehicleMoveEvent::get_name_static(),
        EventType::VehicleUpdate => VehicleUpdateEvent::get_name_static(),

        // World
        EventType::AsyncStructureGenerate => AsyncStructureGenerateEvent::get_name_static(),
        EventType::AsyncStructureSpawn => AsyncStructureSpawnEvent::get_name_static(),
        EventType::ChunkLoad => ChunkLoad::get_name_static(),
        EventType::ChunkPopulate => ChunkPopulateEvent::get_name_static(),
        EventType::ChunkSave => ChunkSave::get_name_static(),
        EventType::ChunkSend => ChunkSend::get_name_static(),
        EventType::ChunkUnload => ChunkUnloadEvent::get_name_static(),
        EventType::EntitiesLoad => EntitiesLoadEvent::get_name_static(),
        EventType::EntitiesUnload => EntitiesUnloadEvent::get_name_static(),
        EventType::GenericGame => GenericGameEvent::get_name_static(),
        EventType::LightningStrike => LightningStrikeEvent::get_name_static(),
        EventType::LootGenerate => LootGenerateEvent::get_name_static(),
        EventType::PortalCreate => PortalCreateEvent::get_name_static(),
        EventType::SpawnChange => SpawnChangeEvent::get_name_static(),
        EventType::StructureGrow => StructureGrowEvent::get_name_static(),
        EventType::TimeSkip => TimeSkipEvent::get_name_static(),
        EventType::ThunderChange => ThunderChangeEvent::get_name_static(),
        EventType::WeatherChange => WeatherChangeEvent::get_name_static(),
        EventType::WorldInit => WorldInitEvent::get_name_static(),
        EventType::WorldLoad => WorldLoadEvent::get_name_static(),
        EventType::WorldUnload => WorldUnloadEvent::get_name_static(),
        EventType::WorldSave => WorldSaveEvent::get_name_static(),
    }
}

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

        let ctx: &Context = self.get(&context)?.as_ref();
        let handler: Arc<dyn DynEventHandler> = Arc::new(WasmPluginEventHandler {
            handler_id,
            priority,
            blocking,
            plugin,
            source: ctx.get_metadata().name.clone(),
        });

        ctx.register_event_dyn(event_type_to_name(event_type), &handler);

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
