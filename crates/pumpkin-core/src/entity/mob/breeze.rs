mod ai;
mod movement;
mod shoot;

use std::sync::{Arc, Weak};

use arc_swap::ArcSwapOption;
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
use crate::net::ClientPlatform;
use crate::plugin::api::events::entity::{
    entity_pose_change::EntityPoseChangeEvent, entity_spawn::EntitySpawnEvent,
    entity_target::TargetReason,
};
use crate::plugin::api::events::world::GenericGameEvent;
use crate::plugin::server::packet::PacketSentEvent;
use crate::world::chunker::{get_view_distance, is_within_chebyshev_distance};

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
    pose_cleanup: ArcSwapOption<PoseCleanup>,
}

struct PoseCleanup {
    previous: EntityPose,
    target: Option<Weak<dyn EntityBase>>,
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
            pose_cleanup: ArcSwapOption::empty(),
        })
    }

    fn finish_pose(
        tick: &mut BrainTick<'_>,
        previous: EntityPose,
        target: Option<Weak<dyn EntityBase>>,
    ) {
        tick.defer_pose_change(EntityPose::Standing, move |mob| {
            let Some(breeze) = mob.cast_any().downcast_ref::<Self>() else {
                return;
            };
            let pose = breeze.get_entity().pose.load();
            if pose != previous && pose != EntityPose::Standing {
                return;
            }
            let cleanup = Arc::new(PoseCleanup { previous, target });
            breeze.pose_cleanup.store(Some(cleanup.clone()));
            breeze.restore_standing(&cleanup, true);
        });
    }

    fn clear_pose_cleanup(&self, cleanup: &Arc<PoseCleanup>) -> bool {
        self.pose_cleanup
            .compare_and_swap(&Some(cleanup.clone()), None)
            .as_ref()
            .is_some_and(|active| Arc::ptr_eq(active, cleanup))
    }

    fn restore_standing(&self, cleanup: &Arc<PoseCleanup>, allow_standing: bool) {
        let entity = self.get_entity();
        let living = &self.mob_entity.living_entity;
        let previous = entity.pose.load();
        if !living.is_alive()
            || !(previous == cleanup.previous || allow_standing && previous == EntityPose::Standing)
        {
            self.clear_pose_cleanup(cleanup);
            return;
        }

        let restored = entity.set_pose_if(
            EntityPose::Standing,
            || {
                living.is_alive()
                    && entity.pose.load() == previous
                    && self
                        .pose_cleanup
                        .load()
                        .as_ref()
                        .is_some_and(|active| Arc::ptr_eq(active, cleanup))
            },
            || {},
        );
        if restored
            && self.clear_pose_cleanup(cleanup)
            && living.is_alive()
            && entity.pose.load() == EntityPose::Standing
            && let Some(target) = cleanup.target.as_ref().and_then(Weak::upgrade)
            && self
                .mob_entity
                .stored_target()
                .is_some_and(|current| Arc::ptr_eq(&current, &target))
        {
            self.mob_entity.with_brain(self, |brain| {
                if brain
                    .get(types::ATTACK_TARGET)
                    .is_some_and(|current| Arc::ptr_eq(current, &target))
                {
                    brain.set_with_expiry(types::BREEZE_SHOOT, (), 60);
                }
            });
        }
    }
}

impl Mob for BreezeEntity {
    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        // A veto leaves the old pose in place. Retry before the next Brain tick.
        if let Some(cleanup) = self.pose_cleanup.load_full() {
            self.restore_standing(&cleanup, false);
            if !self.mob_entity.living_entity.is_alive() {
                return;
            }
        }

        // Plugins can set the mob's target between Brain ticks.
        let current = self.mob_entity.stored_target();
        let target_id = |target: &Arc<dyn EntityBase>| target.get_entity().entity_id;
        self.mob_entity.with_brain(self, |brain| {
            if brain.get(types::ATTACK_TARGET).map(target_id) != current.as_ref().map(target_id) {
                brain.set_optional(types::ATTACK_TARGET, current.clone());
            }
        });
        let effects = self.mob_entity.tick_brain_effects(self);
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
        for effect in effects {
            effect(self);
        }
    }

    fn additional_tick_movement(&self) -> f64 {
        movement::max_jump_speed(&self.mob_entity.living_entity)
    }

    fn conservative_block_read_radius(&self) -> f64 {
        movement::block_read_radius(&self.mob_entity.living_entity)
    }

    fn has_global_tick_reach(&self) -> bool {
        let entity = self.get_entity();
        let world = entity.world.load();
        let Some(server) = world.server.upgrade() else {
            return false;
        };
        let plugins = &server.plugin_manager;
        if plugins.has_handlers::<EntitySpawnEvent>()
            || plugins.has_handlers::<EntityPoseChangeEvent>()
            || plugins.has_handlers::<GenericGameEvent>()
        {
            return true;
        }
        if !plugins.has_handlers::<PacketSentEvent>() {
            return false;
        }

        let tracked = world.entity_tracker.get_tracked_entity(entity.entity_id);
        if tracked.as_ref().is_some_and(|body| body.has_viewers()) {
            return true;
        }
        let chunk = entity.chunk_pos.load();
        // A charge can pair with clients farther away than those hearing the Brain's sounds.
        let range = (EntityType::BREEZE_WIND_CHARGE.client_tracking_range as i32)
            .max(shoot::SHOOT_VOLUME.ceil() as i32);
        let can_land =
            self.mob_entity.living_entity.fall_distance.load() > LAND_SOUND_FALL_DISTANCE;
        let is_passenger = self.is_passenger();
        let tracking_range = tracked
            .as_ref()
            .map_or(EntityType::BREEZE.client_tracking_range, |body| {
                body.get_effective_range()
            });
        world.players.load().iter().any(|player| {
            // A vehicle may move the Breeze before its turn. Fall sounds follow travel.
            let player_chunk = player.get_entity().chunk_pos.load();
            is_passenger
                || is_within_chebyshev_distance(chunk, player_chunk, range)
                || match player.client.as_ref() {
                    ClientPlatform::Java(_) => {
                        // Chunk delivery can pair this player before the Breeze ticks.
                        let view_range = u32::from(get_view_distance(player).get());
                        can_land
                            || is_within_chebyshev_distance(
                                chunk,
                                player_chunk,
                                tracking_range.min(view_range) as i32,
                            )
                    }
                    ClientPlatform::Bedrock(_) => {
                        tracked.is_none()
                            && player
                                .watched_section
                                .load()
                                .is_within_distance(chunk.x, chunk.y)
                    }
                }
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
                    tick.state.set(types::BREEZE_SHOOT, ());
                    let probe = probe.clone();
                    tick.set_pose_then(EntityPose::Inhaling, move |breeze| {
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
