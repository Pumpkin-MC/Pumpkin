use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use rand::RngExt;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage, NbtFuture,
    ai::goal::{
        ambient_stand::AmbientStandGoal, follow_parent::FollowParentGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        run_around_like_crazy::RunAroundLikeCrazyGoal, skeleton_trap::SkeletonTrapGoal,
        swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
    passive::{
        animal::Animal,
        equine::{AbstractHorse, AbstractHorseData},
    },
    player::Player,
};

/// Vanilla: `SkeletonHorse.TRAP_MAX_LIFE` -- an un-persistence-required trap horse despawns after
/// this many ticks if it's never triggered.
const TRAP_MAX_LIFE: i32 = 18000;

pub struct SkeletonHorseEntity {
    pub mob_entity: MobEntity,
    /// Vanilla `SkeletonHorse.isTrap` -- set on horses spawned by the "lightning near a lightning
    /// rod" environmental trap mechanic. Nothing in Pumpkin currently spawns a skeleton horse
    /// with this set to `true` at construction time (the lightning-triggered spawn path in
    /// `World`'s chunk tick spawns a plain, non-trap `SkeletonHorseEntity` today), so in practice
    /// this stays `false` -- exactly like `MoveTowardsRestrictionGoal`'s dormant
    /// `position_target_range`, it's still correct to carry the field and the gated goal so that
    /// whichever spawn path is later updated to set it just works.
    is_trap: AtomicBool,
    trap_time: AtomicI32,
    pub horse_data: AbstractHorseData,
}

impl SkeletonHorseEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let horse = Self {
            mob_entity,
            is_trap: AtomicBool::new(false),
            trap_time: AtomicI32::new(0),
            horse_data: AbstractHorseData::default(),
        };
        let mob_arc = Arc::new(horse);
        AbstractHorse::randomize_attributes(mob_arc.as_ref(), &mut rand::rng());
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        let horse_weak: Weak<Self> = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            // Both at vanilla priority 1: `RunAroundLikeCrazyGoal` from the base
            // `AbstractHorse.registerGoals` (`AbstractHorse.java:134`) -- effectively inert here
            // since `mob_interact` below never lets a player mount an untamed skeleton horse,
            // but kept for architectural parity -- and `skeletonTrapGoal`, dynamically
            // added/removed via `setTrap` in vanilla; see `SkeletonTrapGoal`'s doc comment for
            // why Pumpkin registers it unconditionally.
            goal_selector.add_goal(1, RunAroundLikeCrazyGoal::new(horse_weak.clone(), 1.2));
            goal_selector.add_goal(1, SkeletonTrapGoal::new(Arc::downgrade(&mob_arc)));
            // `SkeletonHorse.java:65-66`: `addBehaviourGoals` is overridden to empty, so no
            // tempt/float/panic goal here (unlike Horse/Donkey/Mule) -- only the base
            // `AbstractHorse.registerGoals` priorities 4/6/7/8/9 apply.
            goal_selector.add_goal(4, Box::new(FollowParentGoal::new(1.0)));
            goal_selector.add_goal(6, Box::new(WanderAroundGoal::new_water_avoiding(0.7)));
            goal_selector.add_goal(
                7,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 6.0),
            );
            goal_selector.add_goal(8, Box::new(RandomLookAroundGoal::default()));
            goal_selector.add_goal(9, AmbientStandGoal::new(horse_weak));
        };

        mob_arc
    }

    #[must_use]
    pub fn is_trap(&self) -> bool {
        self.is_trap.load(Ordering::Relaxed)
    }

    pub fn set_trap(&self, trap: bool) {
        self.is_trap.store(trap, Ordering::Relaxed);
    }
}

impl NBTStorage for SkeletonHorseEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.write_nbt(nbt).await;
            self.write_animal_nbt(nbt);
            self.write_horse_nbt(nbt);
            nbt.put_bool("SkeletonTrap", self.is_trap.load(Ordering::Relaxed));
            nbt.put_int("SkeletonTrapTime", self.trap_time.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async {
            self.mob_entity.living_entity.read_nbt_non_mut(nbt).await;
            self.read_animal_nbt(nbt);
            self.read_horse_nbt(nbt);
            self.is_trap.store(
                nbt.get_bool("SkeletonTrap").unwrap_or(false),
                Ordering::Relaxed,
            );
            self.trap_time.store(
                nbt.get_int("SkeletonTrapTime").unwrap_or(0),
                Ordering::Relaxed,
            );
        })
    }
}

impl Animal for SkeletonHorseEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_HORSE_FOOD)
    }

    /// `SkeletonHorse.canAgeUp` -- babies never grow up from food.
    fn can_age_up(&self) -> bool {
        false
    }
}

impl AbstractHorse for SkeletonHorseEntity {
    fn horse_data(&self) -> &AbstractHorseData {
        &self.horse_data
    }

    /// `SkeletonHorse.randomizeAttributes`: only jump-strength is rolled (the base
    /// `AbstractHorse` formula); max-health (15) and speed (0.2) stay fixed at
    /// `createAttributes`'s values.
    fn randomize_attributes(&self, random: &mut impl RngExt)
    where
        Self: Sized,
    {
        let mut attrs = self.mob_entity.living_entity.attributes.write().unwrap();
        if let Some(a) = attrs.get_mut(&pumpkin_data::attributes::Attributes::JUMP_STRENGTH.id) {
            a.base_value = crate::entity::passive::equine::generate_jump_strength(random);
            a.dirty.store(true, Ordering::Relaxed);
        }
    }
}

impl Mob for SkeletonHorseEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// `SkeletonHorse.mobInteract` (`SkeletonHorse.java:174-176`): the trap-despawn timer above
    /// is already correctly ported; this is the one missing gate -- an untamed (non-trap-yet)
    /// skeleton horse cannot be fed/ridden/opened at all until tamed by some other mechanism
    /// (this fully replaces `AbstractHorse::abstract_horse_mob_interact`'s feed/makeMad path
    /// when untamed, it doesn't just skip the ride step).
    fn mob_interact<'a>(
        &'a self,
        player: &'a Arc<Player>,
        item_stack: &'a mut ItemStack,
    ) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move {
            if !self.is_tamed() {
                return false;
            }
            self.abstract_horse_mob_interact(player, item_stack).await
        })
    }

    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            // Vanilla: `SkeletonHorse.aiStep` -- an untriggered trap horse despawns after
            // `TRAP_MAX_LIFE` ticks. `isPersistenceRequired` gating is skipped (Pumpkin doesn't
            // expose that flag to entities generically here); this only matters once something
            // actually spawns a trap horse, which nothing does yet (see `is_trap`'s doc comment).
            if self.is_trap.load(Ordering::Relaxed) {
                let elapsed = self.trap_time.fetch_add(1, Ordering::Relaxed) + 1;
                if elapsed >= TRAP_MAX_LIFE {
                    self.mob_entity.living_entity.entity.remove().await;
                }
            }
        })
    }
}
