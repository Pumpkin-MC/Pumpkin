use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};
use uuid::Uuid;

use pumpkin_data::{damage::DamageType, entity::EntityType, sound::Sound};

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    custom_sound::CustomSound,
    mob::{Mob, MobEntity},
};

pub struct VexEntity {
    pub mob_entity: MobEntity,
    pub has_limited_life: AtomicBool,
    pub limited_life_ticks: AtomicI32,
    pub owner_uuid: Mutex<Option<Uuid>>,
}

impl VexEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let vex = Self {
            mob_entity,
            has_limited_life: AtomicBool::new(false),
            limited_life_ticks: AtomicI32::new(0),
            owner_uuid: Mutex::new(None),
        };
        let mob_arc = Arc::new(vex);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub fn set_limited_life(&self, life_ticks: i32) {
        self.has_limited_life.store(true, Ordering::Relaxed);
        self.limited_life_ticks.store(life_ticks, Ordering::Relaxed);
    }

    pub fn set_owner(&self, owner_uuid: Option<Uuid>) {
        *self
            .owner_uuid
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = owner_uuid;
    }

    #[must_use]
    pub fn get_owner(&self) -> Option<Uuid> {
        *self
            .owner_uuid
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Mob for VexEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        if self.has_limited_life.load(Ordering::Relaxed) {
            let ticks = self.limited_life_ticks.fetch_sub(1, Ordering::Relaxed);
            if ticks <= 1 {
                self.limited_life_ticks.store(20, Ordering::Relaxed);
                self.mob_entity
                    .living_entity
                    .damage_with_context(self, 1.0, DamageType::STARVE, None, None, None);
            }
        }

        let entity = self.get_entity();
        if !entity.silent.load(Ordering::Relaxed) && rand::random_range(0..80) == 0 {
            let world = entity.world.load();
            world.play_sound(
                Sound::EntityVexAmbient,
                pumpkin_data::sound::SoundCategory::Hostile,
                &entity.pos.load(),
            );
        }
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_base_experience_reward(&self) -> u32 {
        3
    }
}

impl CustomSound for VexEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityVexHurt)
    }

    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityVexDeath)
    }
}
