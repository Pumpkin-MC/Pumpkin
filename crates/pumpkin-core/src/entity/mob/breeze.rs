mod ai;
mod movement;
mod shoot;

use std::sync::{Arc, Mutex, PoisonError};

use pumpkin_data::{
    damage::DamageType,
    entity::{EntityPose, EntityType},
    sound::Sound,
};
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase,
    ai::{
        brain::{
            Brain, BrainTick,
            memory::{PackedMemories, types},
        },
        pathfinder::node::PathType,
    },
    mob::{Mob, MobEntity},
    projectile_deflection::{ProjectileDeflection, default_deflection},
};
use crate::plugin::api::events::entity::{
    entity_pose_change::EntityPoseChangeEvent, entity_spawn::EntitySpawnEvent,
    entity_target::TargetReason,
};
use crate::plugin::server::packet::PacketSentEvent;

const MAX_HEAD_Y_ROT: f32 = 30.0;
const HEAD_ROT_SPEED: f32 = 25.0;

type BrainEffect = Box<dyn FnOnce(&BreezeEntity) + Send>;

#[derive(Default)]
struct BrainEffects {
    pose: Option<EntityPose>,
    actions: Vec<BrainEffect>,
}

fn breeze_from_tick<'a>(tick: &BrainTick<'a>) -> Option<&'a BreezeEntity> {
    tick.mob.cast_any().downcast_ref()
}

fn look_from_eyes(entity: &Entity, target: Vector3<f64>) {
    entity.look_at(target.add_raw(0.0, -entity.get_eye_height(), 0.0));
    let yaw = entity.yaw.load();
    entity.head_yaw.store(yaw);
    entity.body_yaw.store(yaw);
}

pub struct BreezeEntity {
    pub mob_entity: MobEntity,
    brain_effects: Mutex<BrainEffects>,
}

impl BreezeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        let navigator = mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        navigator.set_pathfinding_malus(PathType::DangerTrapdoor, -1.0);
        navigator.set_pathfinding_malus(PathType::DamageFire, -1.0);
        Arc::new(Self {
            mob_entity,
            brain_effects: Mutex::default(),
        })
    }

    fn planned_pose(&self) -> EntityPose {
        self.brain_effects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .pose
            .unwrap_or_else(|| self.get_entity().pose.load())
    }

    fn change_pose(&self, pose: EntityPose, after: impl FnOnce(&Self) + Send + 'static) {
        let mut effects = self
            .brain_effects
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        effects.pose = Some(pose);
        effects.actions.push(Box::new(move |breeze| {
            let entity = breeze.get_entity();
            entity.set_pose(pose);
            if !entity.is_removed() && entity.pose.load() == pose {
                after(breeze);
            }
        }));
    }

    fn after_brain_tick(&self, after: impl FnOnce(&Self) + Send + 'static) {
        self.brain_effects
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .actions
            .push(Box::new(after));
    }

    fn flush_brain_effects(&self) {
        let effects = std::mem::take(
            &mut *self
                .brain_effects
                .lock()
                .unwrap_or_else(PoisonError::into_inner),
        );
        // Pose, sound and spawn callbacks can reenter the mob through plugins.
        for apply in effects.actions {
            apply(self);
        }
    }
}

impl Mob for BreezeEntity {
    fn is_allowed_in_peaceful(&self) -> bool {
        false
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        // Plugins can set the mob's target between Brain ticks.
        let current = self.mob_entity.stored_target();
        let target_id = |target: &Arc<dyn EntityBase>| target.get_entity().entity_id;
        self.mob_entity.with_brain(self, |brain| {
            if brain.get(types::ATTACK_TARGET).map(target_id) != current.as_ref().map(target_id) {
                brain.set_optional(types::ATTACK_TARGET, current.clone());
            }
        });
        self.mob_entity.tick_brain(self);
        let (proposed, nearest_id) = self.mob_entity.with_brain(self, |brain| {
            (
                brain.get(types::ATTACK_TARGET).cloned(),
                brain.get(types::NEAREST_ATTACKABLE).map(target_id),
            )
        });

        // A plugin callback can inspect the mob's Brain. Dispatch outside its lock.
        let proposed_id = proposed.as_ref().map(target_id);
        if proposed_id != current.as_ref().map(target_id) {
            let reason = proposed
                .as_ref()
                .map_or(TargetReason::ForgotTarget, |target| {
                    if proposed_id != nearest_id {
                        TargetReason::TargetAttackedEntity
                    } else if target.get_player().is_some() {
                        TargetReason::ClosestPlayer
                    } else {
                        TargetReason::ClosestEntity
                    }
                });
            self.set_mob_target(proposed, reason);
        }
        let target = self.mob_entity.stored_target();
        self.mob_entity.with_brain(self, |brain| {
            if target.as_ref().map(target_id) != proposed_id {
                brain.set_optional(types::ATTACK_TARGET, target);
            }
            ai::update_activity(brain);
        });
        self.flush_brain_effects();
    }

    fn additional_tick_movement(&self) -> f64 {
        movement::max_jump_speed(&self.mob_entity.living_entity)
    }

    fn conservative_block_read_radius(&self) -> f64 {
        movement::block_read_radius(&self.mob_entity.living_entity)
    }

    fn has_global_tick_reach(&self) -> bool {
        self.get_entity()
            .world
            .load()
            .server
            .upgrade()
            .is_some_and(|server| {
                server.plugin_manager.has_handlers::<EntitySpawnEvent>()
                    || server
                        .plugin_manager
                        .has_handlers::<EntityPoseChangeEvent>()
                    || server.plugin_manager.has_handlers::<PacketSentEvent>()
            })
    }

    fn can_attack(&self, target: &dyn EntityBase) -> bool {
        let entity_type = target.get_entity().entity_type;
        (entity_type == &EntityType::PLAYER || entity_type == &EntityType::IRON_GOLEM)
            && self.can_attack_base(target)
    }

    fn cause_fall_damage(
        &self,
        caller: &dyn EntityBase,
        fall_distance: f32,
        damage_modifier: f32,
    ) -> bool {
        if fall_distance > 3.0 {
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

    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_util::math::vector3::Vector3;

    use super::*;
    use crate::entity::ai::brain::{
        activity::ActivityData, behavior::OneShot, memory::MemoryStatus,
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
                    let Some(breeze) = breeze_from_tick(tick) else {
                        return false;
                    };
                    tick.state.set(types::BREEZE_SHOOT, ());
                    let probe = probe.clone();
                    breeze.change_pose(EntityPose::Inhaling, move |breeze| {
                        probe.store(
                            breeze
                                .mob_entity
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
