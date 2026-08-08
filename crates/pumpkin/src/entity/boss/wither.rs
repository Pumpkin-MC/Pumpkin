use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        Controls, Goal, GoalFuture, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, revenge::RevengeGoal, track_target::TrackTargetGoal,
    },
    ai::target_predicate::TargetPredicate,
    mob::{Mob, MobEntity},
    projectile::wither_skull::WitherSkullEntity,
};

pub struct WitherEntity {
    pub mob_entity: MobEntity,
    invulnerable_ticks: AtomicI32,
}

impl WitherEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let wither = Self {
            mob_entity,
            invulnerable_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(wither);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // `WitherBoss.registerGoals` (`boss/wither/WitherBoss.java:98-107`).
            goal_selector.add_goal(0, Box::new(WitherDoNothingGoal));
            goal_selector.add_goal(2, Box::new(WitherRangedAttackGoal::new()));
            goal_selector.add_goal(5, Box::new(WitherRandomFlightGoal::new()));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            target_selector.add_goal(1, Box::new(RevengeGoal::new(false)));
            target_selector.add_goal(2, Box::new(WitherNearestTargetGoal::new()));
        };

        mob_arc
    }

    #[must_use]
    pub fn invulnerable_ticks(&self) -> i32 {
        self.invulnerable_ticks.load(Ordering::Relaxed)
    }

    pub fn make_invulnerable(&self) {
        self.invulnerable_ticks.store(220, Ordering::Relaxed);
        self.mob_entity
            .living_entity
            .set_health(self.mob_entity.living_entity.get_max_health() / 3.0);
    }

    fn set_invulnerable_ticks(&self, ticks: i32) {
        self.invulnerable_ticks
            .store(ticks.max(0), Ordering::Relaxed);
    }
}

impl NBTStorage for WitherEntity {}

impl Mob for WitherEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn can_attack(&self, target: &Entity) -> bool {
        !target
            .entity_type
            .has_tag(&tag::EntityType::MINECRAFT_WITHER_FRIENDS)
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    /// `WitherBoss.hurtServer` ignores damage while the summon countdown is active. The
    /// wither-friends tag is also used for the boss's blanket immunity to friendly undead.
    fn pre_damage<'a>(
        &'a self,
        _damage_type: DamageType,
        source: Option<&'a dyn EntityBase>,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            self.invulnerable_ticks() == 0
                && !source.is_some_and(|source| {
                    source
                        .get_entity()
                        .entity_type
                        .has_tag(&tag::EntityType::MINECRAFT_WITHER_FRIENDS)
                })
        })
    }

    /// `WitherBoss.aiStep` (`WitherBoss.java:155-203`) follows its main target directly rather
    /// than relying on ground navigation. The generic living movement tick supplies collision
    /// handling and drag after this updates the velocity.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let entity = &self.mob_entity.living_entity.entity;
            let age = entity.age.load(Ordering::Relaxed);
            let invulnerable = self.invulnerable_ticks.load(Ordering::Relaxed);
            if invulnerable > 0 {
                let next = invulnerable - 1;
                self.set_invulnerable_ticks(next);
                if age % 10 == 0 {
                    self.mob_entity.living_entity.heal(10.0);
                }
                if next == 0 {
                    let world = entity.world.load_full();
                    world.explode(entity.pos.load(), 7.0).await;
                }
                return;
            }

            let Some(target) = self.mob_entity.target.lock().await.clone() else {
                return;
            };
            if !target.get_entity().is_alive() {
                return;
            }

            let target_pos = target.get_entity().pos.load();
            let pos = entity.pos.load();
            let mut velocity = entity.velocity.load().multiply(1.0, 0.6, 1.0);
            if pos.y < target_pos.y || pos.y < target_pos.y + 5.0 {
                velocity.y = velocity.y.max(0.0);
                velocity.y += 0.3 - velocity.y * 0.6;
            }

            let horizontal = Vector3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z);
            if horizontal.length_squared() > 9.0 {
                let direction = horizontal.normalize();
                velocity.x += direction.x * 0.3 - velocity.x * 0.6;
                velocity.z += direction.z * 0.3 - velocity.z * 0.6;
            }
            entity.set_velocity(velocity);
            if velocity.horizontal_length() > 0.05 {
                entity
                    .yaw
                    .store((velocity.z.atan2(velocity.x).to_degrees() - 90.0) as f32);
            }
        })
    }
}

/// `WitherBoss.WitherDoNothingGoal` (`WitherBoss.java:586-599`).
struct WitherDoNothingGoal;

impl Goal for WitherDoNothingGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            mob.cast_any()
                .downcast_ref::<WitherEntity>()
                .is_some_and(|wither| wither.invulnerable_ticks() > 0)
        })
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}

/// `RangedAttackGoal(this, 1.0, 40, 20.0F)` from `WitherBoss.registerGoals`.
struct WitherRangedAttackGoal {
    cooldown: i32,
}

impl WitherRangedAttackGoal {
    const fn new() -> Self {
        Self { cooldown: 0 }
    }

    async fn shoot(mob: &dyn Mob, target: &dyn EntityBase) {
        let shooter = mob.get_entity();
        let world = shooter.world.load_full();
        let shooter_pos = shooter.pos.load();
        let head = Vector3::new(shooter_pos.x, shooter_pos.y + 3.0, shooter_pos.z);
        let target_pos = target.get_entity().pos.load();
        let direction = Vector3::new(
            target_pos.x - head.x,
            target_pos.y + target.get_entity().get_eye_height() * 0.5 - head.y,
            target_pos.z - head.z,
        );
        let projectile_entity = Entity::new(world.clone(), head, &EntityType::WITHER_SKULL);
        let skull = WitherSkullEntity::new_shot(projectile_entity, shooter, false);
        skull
            .thrown
            .set_velocity(direction.x, direction.y, direction.z, 1.0, 0.0);
        world.spawn_entity(Arc::new(skull)).await;
        world.play_sound(Sound::EntityWitherShoot, SoundCategory::Hostile, &head);
    }
}

impl Goal for WitherRangedAttackGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            mob.cast_any()
                .downcast_ref::<WitherEntity>()
                .is_some_and(|wither| wither.invulnerable_ticks() == 0)
                && mob
                    .get_mob_entity()
                    .target
                    .lock()
                    .await
                    .as_ref()
                    .is_some_and(|target| target.get_entity().is_alive())
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.can_start(mob)
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.cooldown = 0;
            mob.get_mob_entity().set_attacking(true);
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move { mob.get_mob_entity().set_attacking(false) })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(target) = mob.get_mob_entity().target.lock().await.clone() else {
                return;
            };
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap()
                .look_at_entity_with_range(&target, 30.0, 30.0);
            self.cooldown = (self.cooldown - 1).max(0);
            let distance = mob
                .get_entity()
                .pos
                .load()
                .squared_distance_to_vec(&target.get_entity().pos.load());
            if distance <= 400.0 && self.cooldown == 0 {
                Self::shoot(mob, target.as_ref()).await;
                self.cooldown = 40;
            }
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::LOOK
    }
}

/// The `WaterAvoidingRandomFlyingGoal` slot from `WitherBoss.registerGoals`.
/// Wither flight is driven by `WitherBoss.aiStep`; this goal supplies the idle drift when there
/// is no combat target instead of handing the flying mob to the ground navigator.
struct WitherRandomFlightGoal {
    cooldown: i32,
}

impl WitherRandomFlightGoal {
    const fn new() -> Self {
        Self { cooldown: 0 }
    }
}

impl Goal for WitherRandomFlightGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            mob.get_mob_entity().target.lock().await.is_none()
                && mob
                    .cast_any()
                    .downcast_ref::<WitherEntity>()
                    .is_some_and(|wither| wither.invulnerable_ticks() == 0)
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.can_start(mob)
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.cooldown -= 1;
            if self.cooldown > 0 {
                return;
            }
            let mut rng = mob.get_random();
            let direction = Vector3::new(
                rng.random_range(-1.0..=1.0),
                rng.random_range(-0.5..=0.5),
                rng.random_range(-1.0..=1.0),
            )
            .normalize()
                * 0.1;
            let velocity = mob.get_entity().velocity.load().multiply(0.8, 0.8, 0.8) + direction;
            mob.get_entity().set_velocity(velocity);
            self.cooldown = rng.random_range(20..60);
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }
}

/// `NearestAttackableTargetGoal<LivingEntity>` with the `WITHER_FRIENDS` exclusion from
/// `WitherBoss.LIVING_ENTITY_SELECTOR` (`WitherBoss.java:74-79`).
struct WitherNearestTargetGoal {
    tracker: TrackTargetGoal,
    target: Option<Arc<dyn EntityBase>>,
    target_predicate: TargetPredicate,
}

impl WitherNearestTargetGoal {
    fn new() -> Self {
        Self {
            tracker: TrackTargetGoal::with_default(false),
            target: None,
            target_predicate: TargetPredicate::create_attackable()
                .ignore_visibility()
                .set_base_max_distance(40.0),
        }
    }

    async fn find_target(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let origin = entity.pos.load();
        let mut candidates: Vec<Arc<dyn EntityBase>> = world
            .get_nearby_entities(origin, 40.0)
            .into_values()
            .filter(|candidate| {
                candidate.get_entity().entity_id != entity.entity_id
                    && !candidate
                        .get_entity()
                        .entity_type
                        .has_tag(&tag::EntityType::MINECRAFT_WITHER_FRIENDS)
                    && candidate.get_living_entity().is_some()
            })
            .collect();
        candidates.sort_by(|a, b| {
            let distance = |candidate: &Arc<dyn EntityBase>| {
                candidate
                    .get_entity()
                    .pos
                    .load()
                    .squared_distance_to_vec(&origin)
            };
            distance(a).partial_cmp(&distance(b)).unwrap()
        });

        self.target = None;
        for candidate in candidates {
            let Some(living) = candidate.get_living_entity() else {
                continue;
            };
            if mob.can_attack(candidate.get_entity())
                && self
                    .target_predicate
                    .test(&world, Some(&mob.get_mob_entity().living_entity), living)
                    .await
            {
                self.target = Some(candidate);
                break;
            }
        }
    }
}

impl Goal for WitherNearestTargetGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            self.find_target(mob).await;
            self.target.is_some()
        })
    }

    fn should_continue<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        self.tracker.should_continue(mob)
    }

    fn start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            mob.set_mob_target(self.target.clone()).await;
            self.tracker.start(mob).await;
        })
    }

    fn stop<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.target = None;
            self.tracker.stop(mob).await;
        })
    }

    fn controls(&self) -> Controls {
        Controls::TARGET
    }
}
