use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering::Relaxed},
};

use pumpkin_data::entity::EntityType;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::goal::{
        active_target::ActiveTargetGoal, avoid_entity::AvoidEntityGoal,
        look_around::RandomLookAroundGoal, look_at_entity::LookAtEntityGoal,
        melee_attack::MeleeAttackGoal, swim::SwimGoal, wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity, hoglin_gore},
};
use crate::world::World;

/// `HoglinAi.REPELLENT_DETECTION_RANGE_HORIZONTAL/VERTICAL`.
const REPELLENT_RANGE_HORIZONTAL: i32 = 8;
const REPELLENT_RANGE_VERTICAL: i32 = 4;
/// `HoglinAi.REPELLENT_PACIFY_TIME`.
const REPELLENT_PACIFY_TICKS: i32 = 200;
/// Matches vanilla's `Sensor` default scan rate (`HoglinSpecificSensor` inherits it) --
/// re-scanning every tick would be needlessly expensive for an 8x4x8-block box check.
const REPELLENT_SCAN_INTERVAL_TICKS: i32 = 20;

/// `HoglinSpecificSensor.findNearestRepellent`/`BlockPos.findClosestMatch(pos, 8, 4, ...)`:
/// true if any block within an 8-horizontal/4-vertical box of `center` carries the
/// `hoglin_repellents` tag.
fn repellent_nearby(world: &World, center: BlockPos) -> bool {
    for dy in -REPELLENT_RANGE_VERTICAL..=REPELLENT_RANGE_VERTICAL {
        for dx in -REPELLENT_RANGE_HORIZONTAL..=REPELLENT_RANGE_HORIZONTAL {
            for dz in -REPELLENT_RANGE_HORIZONTAL..=REPELLENT_RANGE_HORIZONTAL {
                let pos = BlockPos(center.0 + Vector3::new(dx, dy, dz));
                if world
                    .get_block(&pos)
                    .has_tag(&tag::Block::MINECRAFT_HOGLIN_REPELLENTS)
                {
                    return true;
                }
            }
        }
    }
    false
}

pub struct HoglinEntity {
    pub mob_entity: MobEntity,
    /// Ticks left pacified by a nearby repellent block (`HoglinAi.isPacified`).
    pacify_ticks: Arc<AtomicI32>,
    repellent_scan_countdown: AtomicI32,
}

impl HoglinEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let pacify_ticks = Arc::new(AtomicI32::new(0));
        let hoglin = Self {
            mob_entity,
            pacify_ticks: pacify_ticks.clone(),
            repellent_scan_countdown: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(hoglin);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            // Vanilla: adult hoglins flee visible piglins within 8 blocks while idle
            // (HoglinAi.java:74, DESIRED_DISTANCE_FROM_PIGLIN_WHEN_IDLING=8, speed
            // 0.4F) and flee harder once actually hit (initRetreatActivity, distance
            // 15, speed SPEED_MULTIPLIER_WHEN_RETREATING=1.3F). Pumpkin's
            // `AvoidEntityGoal` has a single close/far speed model rather than two
            // separate Brain activities, so this merges both into one goal; the
            // adult/baby and `isPacified`/repellent gating from vanilla are also not
            // reproduced here.
            goal_selector.add_goal(
                4,
                Box::new(AvoidEntityGoal::new(&EntityType::PIGLIN, 8.0, 0.4, 1.3)),
            );
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(1.0)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            let pacify_check = pacify_ticks;
            target_selector.add_goal(
                1,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    Some(move |_target, _world| {
                        let pacify_check = pacify_check.clone();
                        async move { pacify_check.load(Relaxed) <= 0 }
                    }),
                )),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn is_adult(&self) -> bool {
        self.mob_entity.living_entity.entity.age.load(Relaxed) >= 0
    }

    /// `HoglinAi.isPacified` (200 ticks after a nearby repellent was last seen).
    #[must_use]
    pub fn is_pacified(&self) -> bool {
        self.pacify_ticks.load(Relaxed) > 0
    }
}

impl NBTStorage for HoglinEntity {}

impl Mob for HoglinEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// `HoglinBase.hurtAndThrowTarget`/`throwTarget`: randomized damage roll plus
    /// resistance-adjusted knockback, replacing the generic flat-damage melee path.
    fn try_attack<'a>(&'a self, target: &'a dyn EntityBase) -> EntityBaseFuture<'a, bool> {
        Box::pin(async move { hoglin_gore::try_gore_attack(self, target).await })
    }

    /// `HoglinSpecificSensor.findNearestRepellent` + `BecomePassiveIfMemoryPresent`:
    /// re-scans for a nearby repellent block every 20 ticks, refreshing the pacify
    /// timer and clearing the current attack target when one is found.
    fn mob_tick<'a>(&'a self, _caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            let countdown = self.repellent_scan_countdown.fetch_sub(1, Relaxed);
            if countdown > 0 {
                if self.pacify_ticks.load(Relaxed) > 0 {
                    self.pacify_ticks.fetch_sub(1, Relaxed);
                }
                return;
            }
            self.repellent_scan_countdown
                .store(REPELLENT_SCAN_INTERVAL_TICKS, Relaxed);

            let pos = self.mob_entity.living_entity.entity.block_pos.load();
            let world = self.mob_entity.living_entity.entity.world.load();
            if repellent_nearby(&world, pos) {
                self.pacify_ticks.store(REPELLENT_PACIFY_TICKS, Relaxed);
                self.set_mob_target(None).await;
            } else if self.pacify_ticks.load(Relaxed) > 0 {
                self.pacify_ticks.fetch_sub(1, Relaxed);
            }
        })
    }
}
