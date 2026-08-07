use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::{
    attributes::Attributes,
    entity::EntityType,
    sound::{Sound, SoundCategory},
};
use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, revenge::RevengeGoal,
        spear_use::SpearUseGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    attributes::{Modifier, ModifierOperation, send_attribute_updates_for_living},
    living::LivingEntity,
    mob::{Mob, MobEntity},
    persistent_anger::PersistentAnger,
};
use crate::world::World;

/// Vanilla `ZombifiedPiglin.java:52-55`: `SPEED_MODIFIER_ATTACKING`, +0.05 `ADD_VALUE`
/// movement speed while angry.
const SPEED_MODIFIER_ATTACKING_ID: &str = "minecraft:attacking";
const SPEED_MODIFIER_ATTACKING: f64 = 0.05;

/// Vanilla `ZombifiedPiglin.java:56`: `FIRST_ANGER_SOUND_DELAY = rangeOfSeconds(0, 1)`.
const FIRST_ANGER_SOUND_DELAY_MAX_TICKS: i32 = 20;

/// Vanilla `ZombifiedPiglin.java:61-62`: `ALERT_RANGE_Y = 10`, `ALERT_INTERVAL = rangeOfSeconds(4, 6)`.
const ALERT_RANGE_Y: f64 = 10.0;
const ALERT_INTERVAL_MIN_TICKS: i32 = 4 * 20;
const ALERT_INTERVAL_MAX_TICKS: i32 = 6 * 20;

pub struct ZombifiedPiglinEntity {
    pub mob_entity: MobEntity,
    pub persistent_anger: PersistentAnger,
    play_first_anger_sound_in: AtomicI32,
    ticks_until_next_alert: AtomicI32,
    speed_boosted: AtomicBool,
}

impl ZombifiedPiglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let piglin = Self {
            mob_entity,
            persistent_anger: PersistentAnger::default(),
            play_first_anger_sound_in: AtomicI32::new(0),
            ticks_until_next_alert: AtomicI32::new(0),
            speed_boosted: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(piglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, SpearUseGoal::new(1.0, 1.0, 10.0, 2.0));
            goal_selector.add_goal(2, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            // Zombified piglins are neutral: vanilla `ZombifiedPiglin.java:75-77` registers no
            // unconditional player target, only `HurtByTargetGoal(this).setAlertOthers()` plus
            // the anger-gated player target below.
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true)));

            // Vanilla `ZombifiedPiglin.java:76`:
            // `NearestAttackableTargetGoal<>(this, Player.class, 10, true, false, this::isAngryAt)`.
            // The predicate only sees the candidate, so it closes over a weak handle back to this
            // piglin to consult its own `PersistentAnger` state. This is what re-acquires the
            // player after the melee target is lost, for as long as the grudge lasts.
            let angry_weak = mob_weak.clone();
            target_selector.add_goal(
                2,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    Some(move |target: Arc<LivingEntity>, _world: Arc<World>| {
                        let angry_weak = angry_weak.clone();
                        async move {
                            let Some(mob) = angry_weak.upgrade() else {
                                return false;
                            };
                            let Some(anger) = mob.persistent_anger() else {
                                return false;
                            };
                            anger.is_angry_at(target.entity.entity_uuid).await
                        }
                    }),
                )),
            );
        };

        mob_arc
    }

    /// Vanilla `ZombifiedPiglin.java:98-108`: a transient +0.05 movement speed modifier held
    /// exactly while angry. The `!isBaby()` guard is dropped because Pumpkin models no baby
    /// zombie variants at all, so every zombified piglin is an adult here.
    async fn update_attacking_speed(&self) {
        let living = &self.mob_entity.living_entity;

        if self.persistent_anger.is_angry() {
            if !self.speed_boosted.swap(true, Relaxed) {
                living.update_attribute(&Attributes::MOVEMENT_SPEED, |inst| {
                    inst.add_or_replace_modifier(Modifier {
                        id: SPEED_MODIFIER_ATTACKING_ID.to_string(),
                        amount: SPEED_MODIFIER_ATTACKING,
                        operation: ModifierOperation::Add,
                    });
                });
                send_attribute_updates_for_living(living, vec![Attributes::MOVEMENT_SPEED]).await;
            }
            self.maybe_play_first_anger_sound();
        } else if self.speed_boosted.swap(false, Relaxed) {
            living.update_attribute(&Attributes::MOVEMENT_SPEED, |inst| {
                inst.remove_modifier(SPEED_MODIFIER_ATTACKING_ID);
            });
            send_attribute_updates_for_living(living, vec![Attributes::MOVEMENT_SPEED]).await;
        }
    }

    /// Vanilla `ZombifiedPiglin.java:118-125` plus `playAngerSound` at `:151-153`. Vanilla's
    /// pitch is `getVoicePitch() * 1.8`, where `getVoicePitch` is randomized per call; Pumpkin
    /// has no voice-pitch model, so the unrandomized `1.0` base is used.
    fn maybe_play_first_anger_sound(&self) {
        if self
            .play_first_anger_sound_in
            .fetch_update(Relaxed, Relaxed, |ticks| (ticks > 0).then_some(ticks - 1))
            == Ok(1)
        {
            let entity = &self.mob_entity.living_entity.entity;
            entity.world.load().play_sound_fine(
                Sound::EntityZombifiedPiglinAngry,
                SoundCategory::Hostile,
                &entity.pos.load(),
                2.0,
                1.8,
            );
        }
    }

    /// Vanilla `NeutralMob.updatePersistentAnger` (`NeutralMob.java:58-89`) as called with
    /// `stayAngryIfTargetPresent = true` (`ZombifiedPiglin.java:110`): while a target is present
    /// the grudge target is (re)adopted and the timer is resampled *every* tick, so the countdown
    /// only begins once the target is lost. Vanilla's extra early-clear branches (grudge target
    /// gone creative/spectator/peaceful, or a dead `Mob` target) are not modelled; timer expiry is
    /// handled by `PersistentAnger::tick`.
    async fn update_persistent_anger(&self) {
        let target = self.mob_entity.target.lock().await.clone();
        if let Some(target) = target {
            self.persistent_anger
                .set_angry_at(Some(target.get_entity().entity_uuid))
                .await;
            self.persistent_anger.start_timer();
        }
    }

    /// Vanilla `ZombifiedPiglin.java:127-137`: every 4-6 seconds, if the current target is in
    /// line of sight, spread it to the pack.
    async fn maybe_alert_others(&self, target: &Arc<dyn EntityBase>) {
        if self
            .ticks_until_next_alert
            .fetch_update(Relaxed, Relaxed, |ticks| (ticks > 0).then_some(ticks - 1))
            .is_ok()
        {
            return;
        }

        if self.has_line_of_sight(target).await {
            self.alert_others(target).await;
        }

        self.ticks_until_next_alert.store(
            rand::rng().random_range(ALERT_INTERVAL_MIN_TICKS..=ALERT_INTERVAL_MAX_TICKS),
            Relaxed,
        );
    }

    async fn has_line_of_sight(&self, target: &Arc<dyn EntityBase>) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        world
            .raycast(
                entity.get_eye_pos(),
                target.get_entity().get_eye_pos(),
                async |block_pos, world| world.get_block_state(block_pos).is_solid(),
            )
            .await
            .is_none()
    }

    /// Vanilla `ZombifiedPiglin.java:139-149`: hand this piglin's target to every other
    /// zombified piglin that has no target of its own, inside a box inflated by follow range
    /// horizontally but only `ALERT_RANGE_Y` vertically.
    ///
    /// Scope reductions: `get_nearby_entities` is a sphere, so the circumscribing sphere is
    /// queried and the box bounds are re-applied per candidate. Vanilla's `!isAlliedTo(target)`
    /// filter is dropped because Pumpkin models no entity teams.
    async fn alert_others(&self, target: &Arc<dyn EntityBase>) {
        let entity = &self.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let within = self
            .mob_entity
            .living_entity
            .get_attribute_value(&Attributes::FOLLOW_RANGE);
        let search_radius = within.mul_add(within, ALERT_RANGE_Y * ALERT_RANGE_Y).sqrt();

        for nearby in world.get_nearby_entities(pos, search_radius).into_values() {
            let other = nearby.get_entity();
            if other.entity_id == entity.entity_id || other.entity_type != entity.entity_type {
                continue;
            }

            let other_pos = other.pos.load();
            if (other_pos.y - pos.y).abs() > ALERT_RANGE_Y {
                continue;
            }
            if (other_pos.x - pos.x).abs() > within || (other_pos.z - pos.z).abs() > within {
                continue;
            }

            let Some(other_mob) = nearby.get_mob() else {
                continue;
            };
            if other_mob.get_mob_entity().target.lock().await.is_some() {
                continue;
            }
            other_mob.set_mob_target(Some(target.clone())).await;
        }
    }
}

impl NBTStorage for ZombifiedPiglinEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async { self.persistent_anger.write_nbt(nbt).await })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async { self.persistent_anger.read_nbt(nbt).await })
    }
}

impl Mob for ZombifiedPiglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn persistent_anger(&self) -> Option<&PersistentAnger> {
        Some(&self.persistent_anger)
    }

    /// Vanilla `ZombifiedPiglin.java:156-163`: a `null -> non-null` target transition arms the
    /// one-shot anger sound and the first pack-alert interval.
    fn set_mob_target(&self, target: Option<Arc<dyn EntityBase>>) -> EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            let mut mob_target = self.mob_entity.target.lock().await;
            if mob_target.is_none() && target.is_some() {
                let mut rng = rand::rng();
                self.play_first_anger_sound_in.store(
                    rng.random_range(0..=FIRST_ANGER_SOUND_DELAY_MAX_TICKS),
                    Relaxed,
                );
                self.ticks_until_next_alert.store(
                    rng.random_range(ALERT_INTERVAL_MIN_TICKS..=ALERT_INTERVAL_MAX_TICKS),
                    Relaxed,
                );
            }
            *mob_target = target;
        })
    }

    /// Mirrors `ZombifiedPiglin.customServerAiStep` (`ZombifiedPiglin.java:98-116`), in order.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.persistent_anger.tick().await;
            self.update_attacking_speed().await;
            self.update_persistent_anger().await;

            let target = self.mob_entity.target.lock().await.clone();
            if let Some(target) = target {
                self.maybe_alert_others(&target).await;
            }
        })
    }
}
