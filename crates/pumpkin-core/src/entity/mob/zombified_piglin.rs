use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::BlockState;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_util::math::position::BlockPos;

use crate::entity::ai::util::goal_utils;
use crate::entity::{
    Entity, EntityBase,
    ai::behavior::neutral::apply_targets,
    ai::goal::{
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, revenge::RevengeGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity,
        equipment::RegionalDifficulty,
        neutral::{NeutralData, NeutralMob},
    },
};
use crate::plugin::api::events::entity::entity_target::TargetReason;
use crate::plugin::api::events::entity::pig_zombie_anger::PigZombieAngerEvent;
use crate::world::World;

/// Vanilla `ALERT_INTERVAL`: 4 to 6 seconds.
const ALERT_INTERVAL: std::ops::RangeInclusive<i32> = 80..=120;

pub struct ZombifiedPiglinEntity {
    pub mob_entity: MobEntity,
    neutral_data: NeutralData,
    ticks_until_next_alert: AtomicI32,
    /// Detects the target transition that restarts the alert interval.
    had_target: AtomicBool,
}

impl ZombifiedPiglinEntity {
    pub const XP_REWARD: u32 = 5;

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let piglin = Self {
            mob_entity,
            neutral_data: NeutralData::default(),
            ticks_until_next_alert: AtomicI32::new(0),
            had_target: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(piglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let target_selector = &mob_arc.mob_entity.target_selector;
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true).alerting_others()));
            apply_targets(target_selector, &mob_arc.mob_entity, 2, 3, true);
        };

        mob_arc
    }

    /// Hands the current target to nearby piglins that have none of their own.
    fn maybe_alert_others(&self, target: &Arc<dyn EntityBase>) {
        let remaining = self.ticks_until_next_alert.load(Ordering::Relaxed);
        if remaining > 0 {
            self.ticks_until_next_alert
                .store(remaining - 1, Ordering::Relaxed);
            return;
        }

        if self.has_line_of_sight(target.get_entity()) {
            for other in goal_utils::nearby_same_type(self) {
                let Some(other_mob) = other.get_mob() else {
                    continue;
                };
                if other_mob.get_mob_entity().get_target().is_some()
                    || other.is_allied_to(target.as_ref())
                {
                    continue;
                }
                other_mob.set_mob_target(Some(target.clone()), TargetReason::PigZombieTarget);
            }
        }

        self.ticks_until_next_alert
            .store(rand::random_range(ALERT_INTERVAL), Ordering::Relaxed);
    }
}

impl NeutralMob for ZombifiedPiglinEntity {
    fn get_neutral_data(&self) -> &NeutralData {
        &self.neutral_data
    }

    fn approve_grudge(&self, target: &dyn EntityBase, ticks: i32) -> Option<i32> {
        let entity = &self.mob_entity.living_entity.entity;
        let Some(server) = entity.world.load().server.upgrade() else {
            return Some(ticks);
        };
        let mut event =
            PigZombieAngerEvent::new(entity.entity_id, Some(target.get_entity().entity_id), ticks);
        server.plugin_manager.fire_blocking(&server, &mut event);
        (!event.cancelled).then_some(event.new_anger)
    }

    fn updates_anger_in_ai_step(&self) -> bool {
        true
    }
}

impl Mob for ZombifiedPiglinEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityZombieStep, 0.15, 1.0);
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn spawn_as_baby(&self) -> bool {
        self.mob_entity.set_baby_by_age();
        true
    }

    fn as_neutral(&self) -> Option<&dyn NeutralMob> {
        Some(self)
    }

    fn populate_default_equipment_slots(
        &self,
        _world: &Arc<World>,
        _difficulty: &RegionalDifficulty,
    ) {
        let living = &self.mob_entity.living_entity;
        let mut equipment = living
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let weapon = if rand::random_range(0..20) == 0 {
            &Item::GOLDEN_SPEAR
        } else {
            &Item::GOLDEN_SWORD
        };
        equipment.put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, weapon));
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.update_persistent_anger();
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let Some(target) = self.mob_entity.get_target() else {
            self.had_target.store(false, Ordering::Relaxed);
            return;
        };

        // Fresh target: wait out a full interval before spreading the word.
        if !self.had_target.swap(true, Ordering::Relaxed) {
            self.ticks_until_next_alert
                .store(rand::random_range(ALERT_INTERVAL), Ordering::Relaxed);
        }

        self.maybe_alert_others(&target);
    }
}
