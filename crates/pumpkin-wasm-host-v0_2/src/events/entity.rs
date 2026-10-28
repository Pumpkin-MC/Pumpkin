use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    events::{
        ToFromWasmEvent, cleanup_event, consume_player, consume_text_component, consume_world,
        from_wasm_block_position, from_wasm_position, to_wasm_block_position, to_wasm_position,
    },
    living_entity::{from_wit_damage_type, to_wit_damage_type},
    pumpkin::plugin::event::{
        AreaEffectCloudApplyEvent as AreaEffectCloudApplyEventData,
        ArrowBodyCountChangeEvent as ArrowBodyCountChangeEventData,
        BatToggleSleepEvent as BatToggleSleepEventData,
        CreatureSpawnEvent as CreatureSpawnEventData, CreeperPowerEvent as CreeperPowerEventData,
        EnderDragonChangePhaseEvent as EnderDragonChangePhaseEventData,
        EntityAirChangeEvent as EntityAirChangeEventData,
        EntityBreakDoorEvent as EntityBreakDoorEventData, EntityBreedEvent as EntityBreedEventData,
        EntityChangeBlockEvent as EntityChangeBlockEventData,
        EntityCombustByBlockEvent as EntityCombustByBlockEventData,
        EntityCombustByEntityEvent as EntityCombustByEntityEventData,
        EntityCombustEvent as EntityCombustEventData,
        EntityDamageByBlockEvent as EntityDamageByBlockEventData,
        EntityDamageByEntityEvent as EntityDamageByEntityEventData,
        EntityDamageEvent as EntityDamageEventData, EntityDeathEvent as EntityDeathEventData,
        EntityDismountEvent as EntityDismountEventData,
        EntityDropItemEvent as EntityDropItemEventData, EntityDyeEvent as EntityDyeEventData,
        EntityEnterBlockEvent as EntityEnterBlockEventData,
        EntityEnterLoveModeEvent as EntityEnterLoveModeEventData,
        EntityExhaustionEvent as EntityExhaustionEventData,
        EntityExplodeEvent as EntityExplodeEventData,
        EntityInteractEvent as EntityInteractEventData,
        EntityKnockbackByEntityEvent as EntityKnockbackByEntityEventData,
        EntityKnockbackEvent as EntityKnockbackEventData, EntityMountEvent as EntityMountEventData,
        EntityPickupItemEvent as EntityPickupItemEventData,
        EntityPlaceEvent as EntityPlaceEventData,
        EntityPortalEnterEvent as EntityPortalEnterEventData,
        EntityPortalEvent as EntityPortalEventData,
        EntityPortalExitEvent as EntityPortalExitEventData,
        EntityPoseChangeEvent as EntityPoseChangeEventData,
        EntityPotionEffectEvent as EntityPotionEffectEventData,
        EntityRegainHealthEvent as EntityRegainHealthEventData,
        EntityRemoveEvent as EntityRemoveEventData,
        EntityResurrectEvent as EntityResurrectEventData,
        EntityShootBowEvent as EntityShootBowEventData, EntitySpawnEvent as EntitySpawnEventData,
        EntitySpellCastEvent as EntitySpellCastEventData, EntityTameEvent as EntityTameEventData,
        EntityTargetBlockEvent as EntityTargetBlockEventData,
        EntityTargetEvent as EntityTargetEventData,
        EntityTargetLivingEntityEvent as EntityTargetLivingEntityEventData,
        EntityTeleportEvent as EntityTeleportEventData,
        EntityToggleGlideEvent as EntityToggleGlideEventData,
        EntityToggleSwimEvent as EntityToggleSwimEventData,
        EntityTransformEvent as EntityTransformEventData,
        EntityUnleashEvent as EntityUnleashEventData, Event, ExpBottleEvent as ExpBottleEventData,
        ExplosionPrimeEvent as ExplosionPrimeEventData,
        FireworkExplodeEvent as FireworkExplodeEventData,
        FoodLevelChangeEvent as FoodLevelChangeEventData, HorseJumpEvent as HorseJumpEventData,
        ItemDespawnEvent as ItemDespawnEventData, ItemMergeEvent as ItemMergeEventData,
        ItemSpawnEvent as ItemSpawnEventData,
        LingeringPotionSplashEvent as LingeringPotionSplashEventData,
        PigZapEvent as PigZapEventData, PigZombieAngerEvent as PigZombieAngerEventData,
        PiglinBarterEvent as PiglinBarterEventData, PlayerDeathEvent as PlayerDeathEventData,
        PotionSplashEvent as PotionSplashEventData, ProjectileHitEvent as ProjectileHitEventData,
        ProjectileLaunchEvent as ProjectileLaunchEventData,
        SheepDyeWoolEvent as SheepDyeWoolEventData,
        SheepRegrowWoolEvent as SheepRegrowWoolEventData, SlimeSplitEvent as SlimeSplitEventData,
        SpawnerSpawnEvent as SpawnerSpawnEventData,
        StriderTemperatureChangeEvent as StriderTemperatureChangeEventData,
        TrialSpawnerSpawnEvent as TrialSpawnerSpawnEventData,
        VillagerAcquireTradeEvent as VillagerAcquireTradeEventData,
        VillagerCareerChangeEvent as VillagerCareerChangeEventData,
        VillagerReplenishTradeEvent as VillagerReplenishTradeEventData,
        VillagerReputationChangeEvent as VillagerReputationChangeEventData,
        WardenAngerChangeEvent as WardenAngerChangeEventData,
    },
};
use pumpkin_core::plugin::entity::{
    area_effect_cloud_apply::AreaEffectCloudApplyEvent,
    arrow_body_count_change::ArrowBodyCountChangeEvent,
    bat_toggle_sleep::BatToggleSleepEvent,
    creature_spawn::CreatureSpawnEvent,
    creeper_power::CreeperPowerEvent,
    ender_dragon_change_phase::EnderDragonChangePhaseEvent,
    entity_break_door::EntityBreakDoorEvent,
    entity_change_block::EntityChangeBlockEvent,
    entity_combust::EntityCombustEvent,
    entity_combust_by_block::EntityCombustByBlockEvent,
    entity_combust_by_entity::EntityCombustByEntityEvent,
    entity_damage::EntityDamageEvent,
    entity_damage_by_block::EntityDamageByBlockEvent,
    entity_damage_by_entity::EntityDamageByEntityEvent,
    entity_death::{EntityDeathEvent, PlayerDeathEvent},
    entity_drop_item::EntityDropItemEvent,
    entity_enter_block::EntityEnterBlockEvent,
    entity_exhaustion::EntityExhaustionEvent,
    entity_interact::EntityInteractEvent,
    entity_knockback::EntityKnockbackEvent,
    entity_knockback_by_entity::EntityKnockbackByEntityEvent,
    entity_place::EntityPlaceEvent,
    entity_portal_enter::EntityPortalEnterEvent,
    entity_portal_exit::EntityPortalExitEvent,
    entity_pose_change::EntityPoseChangeEvent,
    entity_potion_effect::EntityPotionEffectEvent,
    entity_regain_health::EntityRegainHealthEvent,
    entity_remove::EntityRemoveEvent,
    entity_spawn::EntitySpawnEvent,
    entity_spell_cast::EntitySpellCastEvent,
    entity_target_block::EntityTargetBlockEvent,
    entity_target_living_entity::EntityTargetLivingEntityEvent,
    entity_toggle_swim::EntityToggleSwimEvent,
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
use pumpkin_wasm_host_common::state::PluginHostState;

impl ToFromWasmEvent for EntityDamageEvent {
    type Wit = EntityDamageEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            damage: self.damage,
            damage_type: to_wit_damage_type(&self.damage_type),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            damage: data.damage,
            damage_type: from_wit_damage_type(data.damage_type),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityDeathEvent {
    type Wit = EntityDeathEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            dropped_exp: self.dropped_exp,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            dropped_exp: data.dropped_exp,
        }
    }
}

impl ToFromWasmEvent for PlayerDeathEvent {
    type Wit = PlayerDeathEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = state
            .add(self.player.clone())
            .expect("failed to add player resource");
        let death_message = state
            .add(self.death_message.clone())
            .expect("failed to add text component resource");

        Self::Wit {
            player,
            death_message,
            dropped_exp: self.dropped_exp,
            keep_inventory: self.keep_inventory,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            player: consume_player(state, &data.player),
            death_message: consume_text_component(state, &data.death_message),
            dropped_exp: data.dropped_exp,
            keep_inventory: data.keep_inventory,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntitySpawnEvent {
    type Wit = EntitySpawnEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");

        Self::Wit {
            entity_id: self.entity_id,
            entity_type: self.entity_type.clone(),
            position: to_wasm_position(self.position),
            target_world,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        let world = consume_world(state, &data.target_world);
        Self {
            entity_id: data.entity_id,
            entity_type: data.entity_type,
            position: pumpkin_util::math::vector3::Vector3::new(
                data.position.0,
                data.position.1,
                data.position.2,
            ),
            world,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityCombustEvent {
    type Wit = EntityCombustEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            duration_secs: self.duration_secs,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            duration_secs: data.duration_secs,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityRegainHealthEvent {
    type Wit = EntityRegainHealthEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            amount: self.amount,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            amount: data.amount,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_air_change::EntityAirChangeEvent
{
    type Wit = EntityAirChangeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            amount: self.amount,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            amount: data.amount,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::entity::entity_breed::EntityBreedEvent {
    type Wit = EntityBreedEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            father_id: self.father_id,
            mother_id: self.mother_id,
            child_id: self.child_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            father_id: data.father_id,
            mother_id: data.mother_id,
            child_id: data.child_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_dismount::EntityDismountEvent
{
    type Wit = EntityDismountEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            dismounted_id: self.dismounted_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            dismounted_id: data.dismounted_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::entity::entity_dye::EntityDyeEvent {
    type Wit = EntityDyeEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let player = self.player.as_ref().and_then(|p| state.add(p.clone()).ok());
        Self::Wit {
            entity_id: self.entity_id,
            color: format!("{:?}", self.color),
            player,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            color: pumpkin_core::plugin::api::events::entity::entity_dye::DyeColor::White,
            player: data.player.map(|p| consume_player(state, &p)),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_enter_love_mode::EntityEnterLoveModeEvent
{
    type Wit = EntityEnterLoveModeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            human_entity_id: self.human_entity_id,
            ticks_in_love: self.ticks_in_love,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            human_entity_id: data.human_entity_id,
            ticks_in_love: data.ticks_in_love,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_explode::EntityExplodeEvent
{
    type Wit = EntityExplodeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            position: to_wasm_position(self.position),
            yield_rate: self.yield_rate,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            position: pumpkin_util::math::vector3::Vector3::new(
                data.position.0,
                data.position.1,
                data.position.2,
            ),
            yield_rate: data.yield_rate,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::entity::entity_mount::EntityMountEvent {
    type Wit = EntityMountEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            mounted_id: self.mount_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            mount_id: data.mounted_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_pickup_item::EntityPickupItemEvent
{
    type Wit = EntityPickupItemEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            item_name: self.item_name.clone(),
            count: self.count,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            item_name: data.item_name,
            count: data.count,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_portal::EntityPortalEvent
{
    type Wit = EntityPortalEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            portal_pos: to_wasm_block_position(self.portal_pos),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            portal_pos: from_wasm_block_position(data.portal_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_resurrect::EntityResurrectEvent
{
    type Wit = EntityResurrectEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_shoot_bow::EntityShootBowEvent
{
    type Wit = EntityShootBowEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            weapon_name: self.weapon_name.clone(),
            force: self.force,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            weapon_name: data.weapon_name,
            force: data.force,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for pumpkin_core::plugin::api::events::entity::entity_tame::EntityTameEvent {
    type Wit = EntityTameEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let owner = state
            .add(self.owner.clone())
            .expect("failed to add player resource");
        Self::Wit {
            entity_id: self.entity_id,
            owner,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            owner: consume_player(state, &data.owner),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_target::EntityTargetEvent
{
    type Wit = EntityTargetEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            target_id: self.target_id,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            target_id: data.target_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_teleport::EntityTeleportEvent
{
    type Wit = EntityTeleportEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            from_position: to_wasm_position(self.from_position),
            to_position: to_wasm_position(self.to_position),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            from_position: pumpkin_util::math::vector3::Vector3::new(
                data.from_position.0,
                data.from_position.1,
                data.from_position.2,
            ),
            to_position: pumpkin_util::math::vector3::Vector3::new(
                data.to_position.0,
                data.to_position.1,
                data.to_position.2,
            ),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_toggle_glide::EntityToggleGlideEvent
{
    type Wit = EntityToggleGlideEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            is_gliding: self.is_gliding,
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            is_gliding: data.is_gliding,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent
    for pumpkin_core::plugin::api::events::entity::entity_transform::EntityTransformEvent
{
    type Wit = EntityTransformEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            new_entity_id: self.new_entity_id,
            transform_reason: self.transform_reason.clone(),
            cancelled: self.cancelled,
        }
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            new_entity_id: data.new_entity_id,
            transform_reason: data.transform_reason,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for CreatureSpawnEvent {
    type Wit = CreatureSpawnEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let target_world = state
            .add(self.world.clone())
            .expect("failed to add world resource");
        let player = self.player.as_ref().map(|player| {
            state
                .add(player.clone())
                .expect("failed to add player resource")
        });
        Self::Wit {
            entity_id: self.entity_id,
            entity_type: self.entity_type.clone(),
            position: to_wasm_position(self.position),
            target_world,
            spawn_reason: self.spawn_reason.clone(),
            player,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            entity_type: data.entity_type,
            position: from_wasm_position(data.position),
            world: consume_world(state, &data.target_world),
            spawn_reason: data.spawn_reason,
            player: data.player.map(|player| consume_player(state, &player)),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EnderDragonChangePhaseEvent {
    type Wit = EnderDragonChangePhaseEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            current_phase: self.current_phase.clone(),
            new_phase: self.new_phase.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.new_phase = data.new_phase;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            current_phase: data.current_phase,
            new_phase: data.new_phase,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityBreakDoorEvent {
    type Wit = EntityBreakDoorEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            block_pos: from_wasm_block_position(data.block_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityChangeBlockEvent {
    type Wit = EntityChangeBlockEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            block_pos: to_wasm_block_position(self.block_pos),
            new_block: self.new_block.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.new_block = data.new_block;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            block_pos: from_wasm_block_position(data.block_pos),
            new_block: data.new_block,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityDamageByBlockEvent {
    type Wit = EntityDamageByBlockEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            damager_pos: self.damager_pos.map(to_wasm_block_position),
            damage: self.damage,
            cause: self.cause.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.damage = data.damage;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            damager_pos: data.damager_pos.map(from_wasm_block_position),
            damage: data.damage,
            cause: data.cause,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityDamageByEntityEvent {
    type Wit = EntityDamageByEntityEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            damager_id: self.damager_id,
            damage: self.damage,
            cause: self.cause.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.damage = data.damage;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            damager_id: data.damager_id,
            damage: data.damage,
            cause: data.cause,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityDropItemEvent {
    type Wit = EntityDropItemEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            item_name: self.item_name.clone(),
            count: self.count,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            item_name: data.item_name,
            count: data.count,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityEnterBlockEvent {
    type Wit = EntityEnterBlockEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            block_pos: from_wasm_block_position(data.block_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityExhaustionEvent {
    type Wit = EntityExhaustionEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            exhaustion: self.exhaustion,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.exhaustion = data.exhaustion;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            exhaustion: data.exhaustion,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityInteractEvent {
    type Wit = EntityInteractEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            block_pos: from_wasm_block_position(data.block_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityKnockbackEvent {
    type Wit = EntityKnockbackEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            hit_by_id: self.hit_by_id,
            knockback: to_wasm_position(self.knockback),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.knockback = from_wasm_position(data.knockback);
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            hit_by_id: data.hit_by_id,
            knockback: from_wasm_position(data.knockback),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityPlaceEvent {
    type Wit = EntityPlaceEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            block_pos: to_wasm_block_position(self.block_pos),
            block_name: self.block_name.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            block_pos: from_wasm_block_position(data.block_pos),
            block_name: data.block_name,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityPoseChangeEvent {
    type Wit = EntityPoseChangeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            pose: self.pose.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.pose = data.pose;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            pose: data.pose,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityPotionEffectEvent {
    type Wit = EntityPotionEffectEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            effect_name: self.effect_name.clone(),
            duration: self.duration,
            amplifier: self.amplifier,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.duration = data.duration;
        self.amplifier = data.amplifier;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            effect_name: data.effect_name,
            duration: data.duration,
            amplifier: data.amplifier,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntitySpellCastEvent {
    type Wit = EntitySpellCastEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            spell: self.spell.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.spell = data.spell;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            spell: data.spell,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityTargetLivingEntityEvent {
    type Wit = EntityTargetLivingEntityEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            target_id: self.target_id,
            reason: self.reason.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.target_id = data.target_id;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            target_id: data.target_id,
            reason: data.reason,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityToggleSwimEvent {
    type Wit = EntityToggleSwimEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            is_swimming: self.is_swimming,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.is_swimming = data.is_swimming;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            is_swimming: data.is_swimming,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ExplosionPrimeEvent {
    type Wit = ExplosionPrimeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            radius: self.radius,
            fire: self.fire,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.radius = data.radius;
        self.fire = data.fire;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            radius: data.radius,
            fire: data.fire,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for FireworkExplodeEvent {
    type Wit = FireworkExplodeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for FoodLevelChangeEvent {
    type Wit = FoodLevelChangeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            food_level: self.food_level,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.food_level = data.food_level;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            food_level: data.food_level,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ItemDespawnEvent {
    type Wit = ItemDespawnEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ItemMergeEvent {
    type Wit = ItemMergeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            target_id: self.target_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            target_id: data.target_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ItemSpawnEvent {
    type Wit = ItemSpawnEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            position: to_wasm_position(self.position),
            item_name: self.item_name.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            position: from_wasm_position(data.position),
            item_name: data.item_name,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PiglinBarterEvent {
    type Wit = PiglinBarterEventData;

    fn to_wasm_event(&self, state: &mut PluginHostState) -> Self::Wit {
        let input_item = state
            .add(Arc::new(Mutex::new(self.input_item.clone())))
            .expect("failed to add item stack resource");
        let outcome = self
            .outcome
            .iter()
            .map(|i| {
                state
                    .add(Arc::new(Mutex::new(i.clone())))
                    .expect("failed to add item stack resource")
            })
            .collect();
        Self::Wit {
            entity_id: self.entity_id,
            input_item,
            outcome,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(_data: Self::Wit, _state: &mut PluginHostState) -> Self {
        panic!("Cannot construct PiglinBarterEvent from WASM");
    }
}

impl ToFromWasmEvent for ProjectileHitEvent {
    type Wit = ProjectileHitEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            hit_position: to_wasm_position(self.hit_position),
            hit_entity_id: self.hit_entity_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            hit_position: from_wasm_position(data.hit_position),
            hit_entity_id: data.hit_entity_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ProjectileLaunchEvent {
    type Wit = ProjectileLaunchEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            shooter_id: self.shooter_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            shooter_id: data.shooter_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for SheepDyeWoolEvent {
    type Wit = SheepDyeWoolEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            dye_color: self.dye_color,
            player_id: self.player_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.dye_color = data.dye_color;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            dye_color: data.dye_color,
            player_id: data.player_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for SheepRegrowWoolEvent {
    type Wit = SheepRegrowWoolEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for SlimeSplitEvent {
    type Wit = SlimeSplitEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            count: self.count,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.count = data.count;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            count: data.count,
            cancelled: data.cancelled,
        }
    }
}
impl ToFromWasmEvent for StriderTemperatureChangeEvent {
    type Wit = StriderTemperatureChangeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            is_shivering: self.is_shivering,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.is_shivering = data.is_shivering;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            is_shivering: data.is_shivering,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for VillagerAcquireTradeEvent {
    type Wit = VillagerAcquireTradeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            recipe_index: self.recipe_index,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.recipe_index = data.recipe_index;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            recipe_index: data.recipe_index,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for VillagerCareerChangeEvent {
    type Wit = VillagerCareerChangeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            profession: self.profession.clone(),
            reason: self.reason.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.profession = data.profession;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            profession: data.profession,
            reason: data.reason,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for VillagerReplenishTradeEvent {
    type Wit = VillagerReplenishTradeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            restock_quantity: self.restock_quantity,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.restock_quantity = data.restock_quantity;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            restock_quantity: data.restock_quantity,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for WardenAngerChangeEvent {
    type Wit = WardenAngerChangeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            target_id: self.target_id,
            old_anger: self.old_anger,
            new_anger: self.new_anger,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.new_anger = data.new_anger;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            target_id: data.target_id,
            old_anger: data.old_anger,
            new_anger: data.new_anger,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for AreaEffectCloudApplyEvent {
    type Wit = AreaEffectCloudApplyEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            affected_entities: self.affected_entities.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            affected_entities: data.affected_entities,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ArrowBodyCountChangeEvent {
    type Wit = ArrowBodyCountChangeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            old_amount: self.old_amount,
            new_amount: self.new_amount,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.new_amount = data.new_amount;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            old_amount: data.old_amount,
            new_amount: data.new_amount,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for BatToggleSleepEvent {
    type Wit = BatToggleSleepEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            is_awake: self.is_awake,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            is_awake: data.is_awake,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for CreeperPowerEvent {
    type Wit = CreeperPowerEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            lightning_id: self.lightning_id,
            cause: self.cause.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            lightning_id: data.lightning_id,
            cause: data.cause,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityCombustByBlockEvent {
    type Wit = EntityCombustByBlockEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            combuster: to_wasm_block_position(self.combuster),
            duration: self.duration,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.duration = data.duration;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            combuster: from_wasm_block_position(data.combuster),
            duration: data.duration,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityCombustByEntityEvent {
    type Wit = EntityCombustByEntityEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            combuster_id: self.combuster_id,
            duration: self.duration,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.duration = data.duration;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            combuster_id: data.combuster_id,
            duration: data.duration,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityKnockbackByEntityEvent {
    type Wit = EntityKnockbackByEntityEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            hit_by_id: self.hit_by_id,
            force: self.force,
            x: self.x,
            z: self.z,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.force = data.force;
        self.x = data.x;
        self.z = data.z;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            hit_by_id: data.hit_by_id,
            force: data.force,
            x: data.x,
            z: data.z,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityPortalEnterEvent {
    type Wit = EntityPortalEnterEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            location: to_wasm_block_position(self.location),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            location: from_wasm_block_position(data.location),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityPortalExitEvent {
    type Wit = EntityPortalExitEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            from_pos: to_wasm_block_position(self.from_pos),
            to_pos: self.to_pos.map(to_wasm_block_position),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.to_pos = data.to_pos.map(from_wasm_block_position);
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            from_pos: from_wasm_block_position(data.from_pos),
            to_pos: data.to_pos.map(from_wasm_block_position),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityRemoveEvent {
    type Wit = EntityRemoveEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            cause: self.cause.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.cause = data.cause;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            cause: data.cause,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityTargetBlockEvent {
    type Wit = EntityTargetBlockEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            block_pos: to_wasm_block_position(self.block_pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            block_pos: from_wasm_block_position(data.block_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for EntityUnleashEvent {
    type Wit = EntityUnleashEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            reason: self.reason.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.reason = data.reason;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            reason: data.reason,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for ExpBottleEvent {
    type Wit = ExpBottleEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            experience: self.experience,
            location: to_wasm_block_position(self.location),
            show_effect: self.show_effect,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.experience = data.experience;
        self.show_effect = data.show_effect;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            experience: data.experience,
            location: from_wasm_block_position(data.location),
            show_effect: data.show_effect,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for HorseJumpEvent {
    type Wit = HorseJumpEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            power: self.power,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.power = data.power;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            power: data.power,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for LingeringPotionSplashEvent {
    type Wit = LingeringPotionSplashEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            location: to_wasm_block_position(self.location),
            potion_item: self.potion_item.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            location: from_wasm_block_position(data.location),
            potion_item: data.potion_item,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PigZapEvent {
    type Wit = PigZapEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            lightning_id: self.lightning_id,
            pig_zombie_id: self.pig_zombie_id,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            lightning_id: data.lightning_id,
            pig_zombie_id: data.pig_zombie_id,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PigZombieAngerEvent {
    type Wit = PigZombieAngerEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            target_id: self.target_id,
            new_anger: self.new_anger,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.new_anger = data.new_anger;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            target_id: data.target_id,
            new_anger: data.new_anger,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for PotionSplashEvent {
    type Wit = PotionSplashEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            location: to_wasm_block_position(self.location),
            potion_item: self.potion_item.clone(),
            affected_entities: self.affected_entities.clone(),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            location: from_wasm_block_position(data.location),
            potion_item: data.potion_item,
            affected_entities: data.affected_entities,
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for SpawnerSpawnEvent {
    type Wit = SpawnerSpawnEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            spawner_pos: to_wasm_block_position(self.spawner_pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            spawner_pos: from_wasm_block_position(data.spawner_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for TrialSpawnerSpawnEvent {
    type Wit = TrialSpawnerSpawnEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            spawner_pos: to_wasm_block_position(self.spawner_pos),
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            spawner_pos: from_wasm_block_position(data.spawner_pos),
            cancelled: data.cancelled,
        }
    }
}

impl ToFromWasmEvent for VillagerReputationChangeEvent {
    type Wit = VillagerReputationChangeEventData;

    fn to_wasm_event(&self, _state: &mut PluginHostState) -> Self::Wit {
        Self::Wit {
            entity_id: self.entity_id,
            target_id: self.target_id,
            reputation_change: self.reputation_change,
            cancelled: self.cancelled,
        }
    }

    fn apply_wasm_event(&mut self, data: Self::Wit, _state: &mut PluginHostState) {
        self.cancelled = data.cancelled;
        self.reputation_change = data.reputation_change;
    }

    fn from_wasm_event(data: Self::Wit, _state: &mut PluginHostState) -> Self {
        Self {
            entity_id: data.entity_id,
            target_id: data.target_id,
            reputation_change: data.reputation_change,
            cancelled: data.cancelled,
        }
    }
}
