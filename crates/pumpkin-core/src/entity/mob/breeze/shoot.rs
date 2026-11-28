use std::sync::Arc;

use pumpkin_data::{
    effect::StatusEffect,
    entity::{EntityPose, EntityType},
    item_stack::ItemStack,
    sound::Sound,
};
use pumpkin_util::math::vector3::Vector3;

use super::look_from_eyes;
use crate::entity::{
    Entity, EntityBase,
    ai::brain::{
        BrainTick,
        behavior::{Behavior, BehaviorControl, Timed},
        memory::{MemoryModuleId, MemoryStatus, types},
    },
    projectile::{
        ThrownItemEntity, apply_on_projectile_spawned,
        wind_charge::{WIND_CHARGE_GRAVITY, WindChargeEntity},
    },
};

const ATTACK_RANGE_MAX_SQRT: f64 = 256.0;
const UNCERTAINTY_BASE: i32 = 5;
const UNCERTAINTY_MULTIPLIER: i32 = 4;
const PROJECTILE_MOVEMENT_SCALE: f32 = 0.7;
const SHOOT_INITIAL_DELAY_TICKS: i32 = 15;
const SHOOT_RECOVER_DELAY_TICKS: i32 = 4;
const SHOOT_COOLDOWN_TICKS: i64 = 10;
const SHOOT_DURATION_TICKS: i32 = SHOOT_INITIAL_DELAY_TICKS + 1 + SHOOT_RECOVER_DELAY_TICKS;
pub(super) const SHOOT_VOLUME: f32 = 1.5;

pub(super) fn shoot() -> Box<dyn BehaviorControl> {
    Box::new(Timed::new(Shoot))
}

pub(super) fn shoot_when_stuck() -> Box<dyn BehaviorControl> {
    Box::new(Timed::new(ShootWhenStuck))
}

struct Shoot;

impl Behavior for Shoot {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 7] = [
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::BREEZE_SHOOT_COOLDOWN.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_SHOOT_CHARGING.id(), MemoryStatus::ValueAbsent),
            (
                types::BREEZE_SHOOT_RECOVERING.id(),
                MemoryStatus::ValueAbsent,
            ),
            (types::BREEZE_SHOOT.id(), MemoryStatus::ValuePresent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_JUMP_TARGET.id(), MemoryStatus::ValueAbsent),
        ];
        &CONDITIONS
    }

    fn min_duration(&self) -> i32 {
        SHOOT_DURATION_TICKS
    }

    fn max_duration(&self) -> i32 {
        SHOOT_DURATION_TICKS
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        let breeze = tick.mob.get_entity();
        if tick.pose() != EntityPose::Standing {
            return false;
        }

        let Some(target) = tick.state.get(types::ATTACK_TARGET) else {
            return false;
        };
        let distance = breeze
            .pos
            .load()
            .squared_distance_to_vec(&target.get_entity().pos.load());
        let in_range = distance < ATTACK_RANGE_MAX_SQRT;
        if !in_range {
            tick.state.erase(types::BREEZE_SHOOT.id());
        }
        in_range
    }

    fn can_still_use(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.state.has_memory_value(types::ATTACK_TARGET.id())
            && tick.state.has_memory_value(types::BREEZE_SHOOT.id())
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.set_pose(EntityPose::Shooting);
        tick.defer_with_update(
            |_| (),
            |tick, ()| {
                tick.state.set_with_expiry(
                    types::BREEZE_SHOOT_CHARGING,
                    (),
                    i64::from(SHOOT_INITIAL_DELAY_TICKS),
                );
                tick.defer(|breeze| breeze.play_entity_sound(Sound::EntityBreezeInhale, 1.0, 1.0));
            },
        );
    }

    fn tick(&mut self, tick: &mut BrainTick<'_>) {
        let Some(target) = tick.state.get(types::ATTACK_TARGET) else {
            return;
        };
        let target_entity = target.get_entity();
        let target_position = target_entity.pos.load();
        let breeze = tick.mob.get_entity();
        look_from_eyes(breeze, target_position);

        if tick
            .state
            .has_memory_value(types::BREEZE_SHOOT_CHARGING.id())
            || tick
                .state
                .has_memory_value(types::BREEZE_SHOOT_RECOVERING.id())
        {
            return;
        }

        let aim_fraction = if target.is_passenger() { 0.8 } else { 0.3 };
        let aim_at =
            target_position.add_raw(0.0, f64::from(target_entity.height()) * aim_fraction, 0.0);
        tick.state.set_with_expiry(
            types::BREEZE_SHOOT_RECOVERING,
            (),
            i64::from(SHOOT_RECOVER_DELAY_TICKS),
        );
        fire_wind_charge(tick, aim_at);
    }

    fn stop(&mut self, tick: &mut BrainTick<'_>, _timed_out: bool) {
        if tick.pose() == EntityPose::Shooting {
            tick.set_pose(EntityPose::Standing);
        }
        tick.state
            .set_with_expiry(types::BREEZE_SHOOT_COOLDOWN, (), SHOOT_COOLDOWN_TICKS);
        tick.state.erase(types::BREEZE_SHOOT.id());
    }

    fn debug_name(&self) -> &'static str {
        "Shoot"
    }
}

fn fire_wind_charge(tick: &mut BrainTick<'_>, aim_at: Vector3<f64>) {
    let breeze = tick.mob.get_entity();
    let position = breeze.pos.load();
    let firing_position = position.add_raw(
        0.0,
        f64::from(breeze.height() / 2.0) + f64::from(0.3f32),
        0.0,
    );

    let direction = aim_at - firing_position;
    let difficulty = i32::from(u8::from(tick.world.level_info.load().difficulty));
    let projectile = Entity::new(
        tick.world.clone(),
        firing_position,
        &EntityType::BREEZE_WIND_CHARGE,
    );
    let thrown = ThrownItemEntity::owned(projectile, breeze, WIND_CHARGE_GRAVITY);
    thrown.set_velocity(
        direction.x,
        direction.y,
        direction.z,
        f64::from(PROJECTILE_MOVEMENT_SCALE),
        f64::from(UNCERTAINTY_BASE - difficulty * UNCERTAINTY_MULTIPLIER),
    );

    tick.defer(move |shooter| {
        let charge = Arc::new(WindChargeEntity::new_breeze(thrown));
        let world = charge.get_entity().world.load_full();
        if world.spawn_entity(charge.clone()) {
            apply_on_projectile_spawned(charge.get_entity(), ItemStack::EMPTY, None, None);
        }
        shooter.play_entity_sound(Sound::EntityBreezeShoot, SHOOT_VOLUME, 1.0);
    });
}

struct ShootWhenStuck;

impl Behavior for ShootWhenStuck {
    fn entry_conditions(&self) -> &[(MemoryModuleId, MemoryStatus)] {
        const CONDITIONS: [(MemoryModuleId, MemoryStatus); 5] = [
            (types::ATTACK_TARGET.id(), MemoryStatus::ValuePresent),
            (types::BREEZE_JUMP_INHALING.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_JUMP_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::WALK_TARGET.id(), MemoryStatus::ValueAbsent),
            (types::BREEZE_SHOOT.id(), MemoryStatus::ValueAbsent),
        ];
        &CONDITIONS
    }

    fn check_extra_start_conditions(&mut self, tick: &mut BrainTick<'_>) -> bool {
        tick.mob.is_passenger()
            || tick.mob.get_entity().is_in_water()
            || tick
                .mob
                .get_mob_entity()
                .living_entity
                .has_effect(&StatusEffect::LEVITATION)
    }

    fn start(&mut self, tick: &mut BrainTick<'_>) {
        tick.state.set_with_expiry(types::BREEZE_SHOOT, (), 60);
    }

    fn debug_name(&self) -> &'static str {
        "ShootWhenStuck"
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

    use futures::future::BoxFuture;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_data::environment_attribute::Activity;
    use pumpkin_data::packet::clientbound::play::SOUND;
    use pumpkin_protocol::ser::NetworkReadExt;

    use super::super::BreezeEntity;
    use super::*;
    use crate::entity::{
        ai::{
            brain::{Brain, activity::ActivityData},
            test_support::{java_sound_recipient, server},
        },
        passive::iron_golem::IronGolemEntity,
    };
    use crate::plugin::api::events::entity::{
        entity_pose_change::EntityPoseChangeEvent, entity_spawn::EntitySpawnEvent,
    };
    use crate::plugin::{EventPriority, EventRwHandler, server::packet::PacketSentEvent};
    use crate::server::Server;

    struct ShotSounds {
        breeze: Arc<BreezeEntity>,
        pose_vetoes: AtomicUsize,
        inhales: AtomicUsize,
        shots: AtomicUsize,
    }

    impl EventRwHandler<PacketSentEvent> for ShotSounds {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut PacketSentEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if event.packet_id != SOUND.0 {
                    return;
                }
                let Ok(id) = event.payload.as_ref().get_var_int() else {
                    return;
                };
                // Registry sound holders use the sound ID plus one on the wire.
                if id.0 == i32::from(Sound::EntityBreezeInhale as u16) + 1 {
                    assert_eq!(self.pose_vetoes.load(Relaxed), 1);
                    assert!(
                        self.breeze
                            .mob_entity
                            .brain_has_memory_value(types::BREEZE_SHOOT_CHARGING.id())
                    );
                    self.inhales.fetch_add(1, Relaxed);
                } else if id.0 == i32::from(Sound::EntityBreezeShoot as u16) + 1 {
                    self.shots.fetch_add(1, Relaxed);
                }
            })
        }
    }

    struct RetargetOnChargeSpawn {
        breeze: Arc<BreezeEntity>,
        replacement: Arc<dyn EntityBase>,
        attempts: AtomicUsize,
    }

    impl EventRwHandler<EntitySpawnEvent> for RetargetOnChargeSpawn {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntitySpawnEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if event.entity_type.parse() != Ok(EntityType::BREEZE_WIND_CHARGE.id) {
                    return;
                }
                let attempt = self.attempts.fetch_add(1, Relaxed);
                self.breeze
                    .mob_entity
                    .with_brain(self.breeze.as_ref(), |brain| {
                        brain.set(types::ATTACK_TARGET, self.replacement.clone());
                    });
                event.cancelled = attempt == 1;
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_spawn_callback_can_retarget_the_breeze_or_cancel_the_charge()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let (player, _peer) = java_sound_recipient(&server, &world, "BreezeTest")?;
        let position = player.get_entity().pos.load();
        let target_position = position.add_raw(8.0, 0.0, 0.0);
        let breeze = BreezeEntity::new(Entity::new(world.clone(), position, &EntityType::BREEZE));
        let sounds = Arc::new(ShotSounds {
            breeze: breeze.clone(),
            pose_vetoes: AtomicUsize::new(0),
            inhales: AtomicUsize::new(0),
            shots: AtomicUsize::new(0),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(sounds.clone(), EventPriority::Normal, None);
        let target: Arc<dyn EntityBase> = IronGolemEntity::new(Entity::new(
            world.clone(),
            target_position,
            &EntityType::IRON_GOLEM,
        ));
        let observer = Arc::new(RetargetOnChargeSpawn {
            breeze: breeze.clone(),
            replacement: IronGolemEntity::new(Entity::new(
                world.clone(),
                target_position.add_raw(0.0, 0.0, 4.0),
                &EntityType::IRON_GOLEM,
            )),
            attempts: AtomicUsize::new(0),
        });
        server
            .plugin_manager
            .register_blocking::<EntitySpawnEvent, _>(
                observer.clone(),
                EventPriority::Normal,
                None,
            );
        let body = &breeze.mob_entity;
        for attempts in 1..=2 {
            let mut brain = Brain::default();
            let mut behavior = Timed::new(Shoot);
            behavior.start_at(world.get_world_age(), SHOOT_DURATION_TICKS);
            brain.add_activity(ActivityData::with_priority_start(
                Activity::Idle,
                0,
                vec![Box::new(behavior)],
            ));
            brain.set(types::ATTACK_TARGET, target.clone());
            brain.set(types::BREEZE_SHOOT, ());
            body.set_brain(brain);
            body.tick_brain(breeze.as_ref());

            assert_eq!(observer.attempts.load(Relaxed), attempts);
            assert_eq!(sounds.shots.load(Relaxed), attempts);
            body.with_brain(breeze.as_ref(), |brain| {
                assert_eq!(
                    brain.time_until_expiry(types::BREEZE_SHOOT_RECOVERING.id()),
                    Some(i64::from(SHOOT_RECOVER_DELAY_TICKS))
                );
                assert!(
                    brain
                        .get(types::ATTACK_TARGET)
                        .is_some_and(|target| Arc::ptr_eq(target, &observer.replacement))
                );
            });
            assert_eq!(
                world
                    .entities
                    .load()
                    .iter()
                    .filter(
                        |entity| entity.get_entity().entity_type == &EntityType::BREEZE_WIND_CHARGE
                    )
                    .count(),
                1,
                "spawn attempts: {attempts}"
            );
        }
        server.stop();
        server.shutdown().await;
        Ok(())
    }

    impl EventRwHandler<EntityPoseChangeEvent> for ShotSounds {
        fn handle<'a>(
            &'a self,
            _server: &'a Arc<Server>,
            event: &'a mut EntityPoseChangeEvent,
        ) -> BoxFuture<'a, ()> {
            Box::pin(async {
                if event.entity_id == self.breeze.get_entity().entity_id
                    && event.pose.parse() == Ok(EntityPose::Shooting as u8)
                {
                    let body = &self.breeze.mob_entity;
                    assert!(body.brain_has_memory_value(types::ATTACK_TARGET.id()));
                    assert!(!body.brain_has_memory_value(types::BREEZE_SHOOT_CHARGING.id()));
                    self.pose_vetoes.fetch_add(1, Relaxed);
                    event.cancelled = true;
                }
            })
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn a_stuck_breeze_fires_once_even_when_its_shooting_pose_is_vetoed()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let server = server(&folder).await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        let (player, _peer) = java_sound_recipient(&server, &world, "BreezeTest")?;
        let breeze = BreezeEntity::new(Entity::new(
            world.clone(),
            player.get_entity().pos.load(),
            &EntityType::BREEZE,
        ));
        let sounds = Arc::new(ShotSounds {
            breeze: breeze.clone(),
            pose_vetoes: AtomicUsize::new(0),
            inhales: AtomicUsize::new(0),
            shots: AtomicUsize::new(0),
        });
        server
            .plugin_manager
            .register_blocking::<PacketSentEvent, _>(sounds.clone(), EventPriority::Normal, None);
        let target = IronGolemEntity::new(Entity::new(
            world.clone(),
            player.get_entity().pos.load().add_raw(8.0, 0.0, 0.0),
            &EntityType::IRON_GOLEM,
        ));
        server
            .plugin_manager
            .register_blocking::<EntityPoseChangeEvent, _>(
                sounds.clone(),
                EventPriority::Normal,
                None,
            );
        breeze.get_entity().touching_water.store(true, Relaxed);
        let mut brain = Brain::default();
        brain.add_activity(ActivityData::with_pairs(
            Activity::Fight,
            vec![(1, shoot()), (3, shoot_when_stuck())],
        ));
        brain.set_default_activity(Activity::Fight);
        brain.use_default_activity();
        brain.set(types::ATTACK_TARGET, target);
        breeze.mob_entity.set_brain(brain);
        let tick = |time| {
            world
                .level_time
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .world_age = time;
            breeze.mob_entity.tick_brain(breeze.as_ref());
        };

        let count_charges = || {
            world
                .entities
                .load()
                .iter()
                .filter(|entity| entity.get_entity().entity_type == &EntityType::BREEZE_WIND_CHARGE)
                .count()
        };
        tick(0);
        tick(1);
        assert_eq!(sounds.pose_vetoes.load(Relaxed), 1);
        assert_eq!(sounds.inhales.load(Relaxed), 1);
        assert_eq!(breeze.get_entity().pose.load(), EntityPose::Standing);
        breeze.mob_entity.with_brain(breeze.as_ref(), |brain| {
            assert_eq!(
                brain.time_until_expiry(types::BREEZE_SHOOT_CHARGING.id()),
                Some(i64::from(SHOOT_INITIAL_DELAY_TICKS))
            );
        });
        for time in 2..=16 {
            tick(time);
        }
        assert_eq!(count_charges(), 0);
        tick(17);
        assert_eq!(count_charges(), 1);
        let owner = world
            .entities
            .load()
            .iter()
            .find_map(|entity| entity.get_projectile_owner().and_then(|owner| owner.uuid()));
        assert_eq!(owner, Some(breeze.get_entity().entity_uuid));
        for time in 18..=22 {
            tick(time);
        }
        assert_eq!(breeze.get_entity().pose.load(), EntityPose::Standing);
        assert_eq!(count_charges(), 1);
        assert_eq!(sounds.inhales.load(Relaxed), 1);
        assert_eq!(sounds.shots.load(Relaxed), 1);
        breeze.mob_entity.with_brain(breeze.as_ref(), |brain| {
            assert!(!brain.has_memory_value(types::BREEZE_SHOOT.id()));
            assert_eq!(
                brain.time_until_expiry(types::BREEZE_SHOOT_COOLDOWN.id()),
                Some(10)
            );
        });
        server.stop();
        server.shutdown().await;
        Ok(())
    }
}
