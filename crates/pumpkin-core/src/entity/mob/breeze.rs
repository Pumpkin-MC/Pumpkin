use std::sync::{Arc, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};

use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        active_target::ActiveTargetGoal, breeze_attack::BreezeAttackGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    custom_sound::CustomSound,
    mob::{Mob, MobEntity},
};

pub struct BreezeEntity {
    pub mob_entity: MobEntity,
}

impl BreezeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let breeze = Self { mob_entity };
        let mob_arc = Arc::new(breeze);
        let breeze_weak = Arc::downgrade(&mob_arc);
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
            goal_selector.add_goal(1, BreezeAttackGoal::new(breeze_weak));
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
            target_selector.add_goal(
                2,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc
    }
}

impl Mob for BreezeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn pre_damage(&self, _damage_type: DamageType, source: Option<&dyn EntityBase>) -> bool {
        if let Some(src) = source {
            let src_type = src.get_entity().entity_type;
            if src_type == &EntityType::BREEZE {
                return false;
            }
            if src_type != &EntityType::BREEZE_WIND_CHARGE && src_type != &EntityType::WIND_CHARGE {
                if crate::entity::projectile::is_projectile(src_type) {
                    let world = self.get_entity().world.load();
                    world.play_sound_fine(
                        Sound::EntityBreezeDeflect,
                        SoundCategory::Hostile,
                        &self.get_entity().pos.load(),
                        1.0,
                        1.0,
                    );
                    return false;
                }
            }
        }
        true
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = self.get_entity();
        if !entity.silent.load(std::sync::atomic::Ordering::Relaxed)
            && rand::random_range(0..80) == 0
        {
            let world = entity.world.load();
            let sound = if entity.on_ground.load(std::sync::atomic::Ordering::Relaxed) {
                Sound::EntityBreezeIdleGround
            } else {
                Sound::EntityBreezeIdleAir
            };
            world.play_sound_fine(
                sound,
                SoundCategory::Hostile,
                &entity.pos.load(),
                1.0,
                1.0,
            );
        }
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_base_experience_reward(&self) -> u32 {
        10
    }
}

impl CustomSound for BreezeEntity {
    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBreezeHurt)
    }

    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityBreezeDeath)
    }
}
