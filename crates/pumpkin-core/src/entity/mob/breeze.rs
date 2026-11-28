mod ai;
mod movement;
mod shoot;

use std::sync::{Arc, Mutex};

use pumpkin_data::{damage::DamageType, entity::EntityType, sound::Sound};
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{get_seed, legacy_rand::LegacyRand};

use crate::entity::{
    Entity, EntityBase,
    ai::{
        brain::{Brain, memory::PackedMemories},
        pathfinder::node::PathType,
    },
    mob::{Mob, MobEntity},
    projectile_deflection::{ProjectileDeflection, default_deflection},
};

const MAX_HEAD_Y_ROT: f32 = 30.0;
const HEAD_ROT_SPEED: f32 = 25.0;
const LAND_SOUND_FALL_DISTANCE: f32 = 3.0;

fn look_from_eyes(entity: &Entity, target: Vector3<f64>) {
    entity.look_at(target.add_raw(0.0, -entity.get_eye_height(), 0.0));
    let yaw = entity.yaw.load();
    entity.head_yaw.store(yaw);
    entity.body_yaw.store(yaw);
}

pub struct BreezeEntity {
    pub mob_entity: MobEntity,
    movement_random: Mutex<LegacyRand>,
}

impl BreezeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        mob_entity.brain_attack_target_filter = Some(is_breeze_target_type);
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_pathfinding_malus(PathType::DangerTrapdoor, -1.0);
        navigator.set_pathfinding_malus(PathType::DamageFire, -1.0);
        let breeze = Arc::new(Self {
            mob_entity,
            movement_random: Mutex::new(LegacyRand::from_seed(get_seed())),
        });
        breeze.mob_entity.with_brain(breeze.as_ref(), |_| {});
        breeze
    }

    fn with_movement_random<T>(&self, sample: impl FnOnce(&mut LegacyRand) -> T) -> T {
        let mut random = self
            .movement_random
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        sample(&mut random)
    }
}

fn is_breeze_target_type(entity_type: &EntityType) -> bool {
    entity_type == &EntityType::PLAYER || entity_type == &EntityType::IRON_GOLEM
}

impl Mob for BreezeEntity {
    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        self.mob_entity.tick_brain(self);
        self.mob_entity
            .with_brain(self, |brain| ai::update_activity(brain));
    }

    fn additional_tick_movement(&self) -> f64 {
        movement::max_jump_speed(&self.mob_entity.living_entity)
    }

    fn conservative_block_read_radius(&self) -> f64 {
        movement::block_read_radius(&self.mob_entity.living_entity)
    }

    fn can_attack(&self, target: &dyn EntityBase) -> bool {
        is_breeze_target_type(target.get_entity().entity_type) && self.can_attack_base(target)
    }

    fn cause_fall_damage(
        &self,
        caller: &dyn EntityBase,
        fall_distance: f32,
        damage_modifier: f32,
    ) -> bool {
        if fall_distance > LAND_SOUND_FALL_DISTANCE {
            caller.play_entity_sound(Sound::EntityBreezeLand, 1.0, 1.0);
        }
        self.mob_entity
            .living_entity
            .cause_fall_damage(caller, fall_distance, damage_modifier)
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_max_head_rotation(&self) -> f32 {
        MAX_HEAD_Y_ROT
    }

    fn get_max_look_yaw_change(&self) -> f32 {
        HEAD_ROT_SPEED
    }

    fn pre_damage_with_context(
        &self,
        _damage_type: DamageType,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> bool {
        !cause
            .or(source)
            .is_some_and(|attacker| attacker.get_entity().entity_type == &EntityType::BREEZE)
    }

    fn deflection(&self, projectile: &dyn EntityBase) -> ProjectileDeflection {
        let projectile_type = projectile.get_entity().entity_type;
        if projectile_type == &EntityType::WIND_CHARGE
            || projectile_type == &EntityType::BREEZE_WIND_CHARGE
        {
            ProjectileDeflection::None
        } else if default_deflection(&self.mob_entity.living_entity.entity)
            == ProjectileDeflection::Reverse
        {
            ProjectileDeflection::Breeze
        } else {
            ProjectileDeflection::None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

    use pumpkin_data::entity::EntityPose;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::brain::{
        activity::ActivityData,
        behavior::OneShot,
        memory::{MemoryStatus, types},
    };
    use crate::entity::ai::test_support;

    #[tokio::test]
    async fn breeze_rejects_damage_from_a_breeze_but_accepts_an_ownerless_charge()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let pos = Vector3::new(0.0, 64.0, 0.0);
        let target = BreezeEntity::new(Entity::new(world.clone(), pos, &EntityType::BREEZE));
        let attacker = BreezeEntity::new(Entity::new(world.clone(), pos, &EntityType::BREEZE));
        let charge = Entity::new(world.clone(), pos, &EntityType::BREEZE_WIND_CHARGE);

        assert!(!target.pre_damage_with_context(
            DamageType::MOB_ATTACK,
            Some(attacker.as_ref()),
            None,
        ));
        assert!(!target.pre_damage_with_context(
            DamageType::WIND_CHARGE,
            Some(&charge),
            Some(attacker.as_ref()),
        ));
        assert!(target.pre_damage_with_context(DamageType::WIND_CHARGE, Some(&charge), None));

        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn mob_target_access_reads_and_writes_breeze_brain_memory()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let position = Vector3::new(0.0, 64.0, 0.0);
        let breeze = BreezeEntity::new(Entity::new(world.clone(), position, &EntityType::BREEZE));
        let target = crate::entity::r#type::from_type(
            &EntityType::IRON_GOLEM,
            position,
            &world,
            uuid::Uuid::new_v4(),
        );
        let mob = breeze.get_mob_entity();

        assert!(mob.get_target().is_none());
        mob.set_target(Some(target.clone()));
        assert!(mob.with_brain(breeze.as_ref(), |brain| {
            brain
                .get(types::ATTACK_TARGET)
                .is_some_and(|current| Arc::ptr_eq(current, &target))
        }));

        mob.with_brain(breeze.as_ref(), |brain| {
            brain.erase(types::ATTACK_TARGET.id());
        });
        assert!(mob.get_target().is_none());

        mob.with_brain(breeze.as_ref(), |brain| {
            brain.set(types::ATTACK_TARGET, target.clone());
        });
        assert!(
            mob.get_target()
                .is_some_and(|current| Arc::ptr_eq(&current, &target))
        );

        let cow = crate::entity::r#type::from_type(
            &EntityType::COW,
            position,
            &world,
            uuid::Uuid::new_v4(),
        );
        mob.set_target(Some(cow));
        assert!(mob.brain_has_memory_value(types::ATTACK_TARGET.id()));
        assert!(mob.get_target().is_none());

        mob.set_target(None);
        assert!(!mob.brain_has_memory_value(types::ATTACK_TARGET.id()));
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn pose_effect_can_read_the_brain_after_the_ai_step()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = test_support::world(&folder);
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::BREEZE,
        ));
        let observed_memory = Arc::new(AtomicBool::new(false));
        let probe = observed_memory.clone();
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_priority_start(
            Activity::Idle,
            0,
            vec![Box::new(OneShot::new(
                "CheckBreezePoseEffect",
                vec![(types::BREEZE_SHOOT.id(), MemoryStatus::Registered)],
                move |tick| {
                    tick.state.set(types::BREEZE_SHOOT, ());
                    let probe = probe.clone();
                    tick.set_pose(EntityPose::Inhaling);
                    tick.defer(move |breeze| {
                        probe.store(
                            breeze
                                .get_mob_entity()
                                .brain_has_memory_value(types::BREEZE_SHOOT.id()),
                            Relaxed,
                        );
                    });
                    true
                },
            ))],
        ));
        brain.set_default_activity(Activity::Idle);
        brain.use_default_activity();
        breeze.mob_entity.set_brain(brain);

        assert_eq!(breeze.get_entity().pose.load(), EntityPose::Standing);
        breeze.custom_server_ai_step(breeze.as_ref());
        world.level.shutdown().await;

        assert!(observed_memory.load(Relaxed));
        assert_eq!(breeze.get_entity().pose.load(), EntityPose::Inhaling);
        Ok(())
    }
}
