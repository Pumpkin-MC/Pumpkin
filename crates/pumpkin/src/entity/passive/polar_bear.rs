use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::meta_data_type::MetaDataType;
use pumpkin_data::sound::Sound;
use pumpkin_data::tracked_data::TrackedData;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        active_target::ActiveTargetGoal, escape_danger::EscapeDangerGoal,
        follow_parent::FollowParentGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, polar_bear_attack_players::PolarBearAttackPlayersGoal,
        polar_bear_hurt_by_target::PolarBearHurtByTargetGoal,
        polar_bear_melee_attack::PolarBearMeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    living::LivingEntity,
    mob::{Mob, MobEntity},
    persistent_anger::PersistentAnger,
};
use crate::world::World;

/// Represents a Polar Bear, a neutral mob found in cold biomes.
///
/// Wiki: <https://minecraft.wiki/w/Polar_Bear>
pub struct PolarBearEntity {
    pub mob_entity: MobEntity,
    pub persistent_anger: PersistentAnger,
    /// Vanilla `PolarBear.DATA_STANDING_ID`: server-driven rearing-up state that the client
    /// plays the stand animation off of.
    is_standing: AtomicBool,
    /// Vanilla `PolarBear.warningSoundTicks`.
    warning_sound_ticks: AtomicI32,
}

impl PolarBearEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let polar_bear = Self {
            mob_entity,
            persistent_anger: PersistentAnger::default(),
            is_standing: AtomicBool::new(false),
            warning_sound_ticks: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(polar_bear);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            // Priorities follow vanilla `PolarBear.registerGoals`.
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(1, PolarBearMeleeAttackGoal::new());
            goal_selector.add_goal(1, EscapeDangerGoal::new(2.0));
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.25)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            // Vanilla priority 1: a baby bear alerts nearby adults and doesn't fight back.
            target_selector.add_goal(1, PolarBearHurtByTargetGoal::new());
            // Vanilla priority 2: an adult with a cub nearby attacks players on sight.
            target_selector.add_goal(2, PolarBearAttackPlayersGoal::new(&mob_arc.mob_entity));

            // Vanilla priority 3: `NearestAttackableTargetGoal<Player>(..., this::isAngryAt)`.
            // The predicate only sees the candidate, not the mob, so it closes over a weak
            // handle back to this bear to consult its own `PersistentAnger` state.
            let angry_weak = mob_weak.clone();
            target_selector.add_goal(
                3,
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

            // Vanilla priority 4: `(target, level) -> !this.isBaby()`. The predicate ignores
            // the candidate fox and tests this bear's own age, so it closes over a weak
            // handle back to the bear rather than reading `target`.
            let baby_weak = mob_weak.clone();
            target_selector.add_goal(
                4,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::FOX,
                    10,
                    true,
                    true,
                    Some(move |_target: Arc<LivingEntity>, _world: Arc<World>| {
                        let baby_weak = baby_weak.clone();
                        async move {
                            let Some(mob) = baby_weak.upgrade() else {
                                return false;
                            };
                            mob.get_entity().age.load(Relaxed) >= 0
                        }
                    }),
                )),
            );

            // Vanilla priority 5, `ResetUniversalAngerTargetGoal(this, false)`, re-targets any
            // nearby player while "universally angry" (gated behind the `UNIVERSAL_ANGER` game
            // rule, which Pumpkin doesn't have) and separately expires the anger timer. Deferred
            // like the same goal on `WolfEntity` -- `PersistentAnger::tick` already expires the
            // timer on its own.
        };

        mob_arc
    }

    #[must_use]
    pub fn is_standing(&self) -> bool {
        self.is_standing.load(Relaxed)
    }

    /// Vanilla `PolarBear::setStanding`.
    pub fn set_standing(&self, value: bool) {
        if self.is_standing.swap(value, Relaxed) != value {
            self.mob_entity.living_entity.entity.send_meta_data(
                &[Metadata::new(
                    TrackedData::STANDING_ID,
                    MetaDataType::BOOLEAN,
                    value,
                )],
                None,
            );
        }
    }

    /// Vanilla `PolarBear::playWarningSound`.
    pub fn play_warning_sound(&self) {
        if self.warning_sound_ticks.load(Relaxed) <= 0 {
            self.mob_entity
                .living_entity
                .entity
                .play_sound(Sound::EntityPolarBearWarning);
            self.warning_sound_ticks.store(40, Relaxed);
        }
    }
}

impl NBTStorage for PolarBearEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.persistent_anger.write_nbt(nbt).await;
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.persistent_anger.read_nbt(nbt).await;
        })
    }
}

impl Mob for PolarBearEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn persistent_anger(&self) -> Option<&PersistentAnger> {
        Some(&self.persistent_anger)
    }

    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.persistent_anger.tick().await;

            // Vanilla `PolarBear::tick`: `if (warningSoundTicks > 0) warningSoundTicks--;`.
            let ticks = self.warning_sound_ticks.load(Relaxed);
            if ticks > 0 {
                self.warning_sound_ticks.store(ticks - 1, Relaxed);
            }

            // Simplified `NeutralMob::updatePersistentAnger(level, true)`: whenever this bear
            // currently has a live target (set by e.g. `PolarBearHurtByTargetGoal`), adopt it as
            // the anger target and (re)start the timer, mirroring `WolfEntity::mob_tick`.
            let current_target = self.mob_entity.target.lock().await.clone();
            if let Some(target) = current_target {
                let target_uuid = target.get_entity().entity_uuid;
                if !self.persistent_anger.is_angry_at(target_uuid).await {
                    self.persistent_anger.set_angry_at(Some(target_uuid)).await;
                    self.persistent_anger.start_timer();
                }
            }
        })
    }
}
