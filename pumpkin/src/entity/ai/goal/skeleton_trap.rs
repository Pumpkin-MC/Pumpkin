use std::sync::{Arc, Weak};

use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use super::{Controls, Goal, GoalFuture};
use crate::entity::{EntityBase, mob::Mob, passive::skeleton_horse::SkeletonHorseEntity};

/// Nearby-player range that arms the trap (vanilla: `hasNearbyAlivePlayer(x, y, z, 10.0)`).
const TRIGGER_RANGE: f64 = 10.0;
/// Vanilla spawns 3 extra skeleton-mounted horses in addition to the trap horse itself.
const EXTRA_HORSE_COUNT: usize = 3;

/// The "lightning strikes a skeleton horse and it becomes a mounted skeleton ambush" behavior.
///
/// Vanilla source: `net/minecraft/world/entity/animal/equine/SkeletonTrapGoal.java`. Unlike most
/// `Goal`s this one lives next to its owning entity class in vanilla rather than under
/// `ai/goal/`, but it's kept in `ai/goal/` here to match Pumpkin's convention for per-mob goals
/// that still need typed access to their owner (see `blaze_attack.rs`'s
/// `BlazeShootFireballGoal(Weak<BlazeEntity>)` for the same pattern).
///
/// Wiring note: vanilla dynamically adds/removes this goal from the horse's `goalSelector`
/// whenever `SkeletonHorse::setTrap` toggles, rather than registering it unconditionally. Pumpkin
/// registers it unconditionally on every `SkeletonHorseEntity` and instead gates on
/// `SkeletonHorseEntity::is_trap()` inside `can_start`, which is behaviorally equivalent (a
/// non-trap horse's `can_start` always returns `false`) without needing a dynamic
/// add/remove-goal API. Once triggered, `tick()` immediately clears `is_trap`, so (matching
/// Pumpkin's `Goal::should_continue` defaulting to `false` when unset) the goal naturally runs
/// for exactly one active tick before the goal selector stops it again.
///
/// Scope reduction: vanilla enchants the spawned skeletons' weapon/helmet via
/// `EnchantmentHelper.enchantItemFromProvider(..., VanillaEnchantmentProviders.MOB_SPAWN_EQUIPMENT,
/// difficulty, ...)`. Pumpkin has no equivalent difficulty-scaled loot-table-driven enchantment
/// provider reachable from here, so the spawned skeletons get a plain iron helmet (matching
/// vanilla's "if no helmet, give an iron helmet" fallback) and whatever bow/equipment
/// `SkeletonEntity::new`'s own spawn defaults already provide, with no additional enchantments.
/// `horse.invulnerableTime = 60` / `setPersistenceRequired()` are likewise skipped -- Pumpkin
/// doesn't expose either generically from a goal file.
pub struct SkeletonTrapGoal {
    horse: Weak<SkeletonHorseEntity>,
}

impl SkeletonTrapGoal {
    #[must_use]
    pub fn new(horse: Weak<SkeletonHorseEntity>) -> Box<Self> {
        Box::new(Self { horse })
    }

    /// Spawns one skeleton-horse + mounted skeleton pair near `pos` (vanilla:
    /// `SkeletonTrapGoal.createHorse`/`createSkeleton`).
    async fn spawn_mounted_skeleton(mob: &dyn Mob, pos: Vector3<f64>) {
        let world = mob.get_entity().world.load();

        let horse = crate::entity::r#type::from_type(
            &EntityType::SKELETON_HORSE,
            pos,
            &world,
            uuid::Uuid::new_v4(),
        );
        let skeleton = crate::entity::r#type::from_type(
            &EntityType::SKELETON,
            pos,
            &world,
            uuid::Uuid::new_v4(),
        );

        Self::ensure_helmet(&skeleton).await;

        world.spawn_entity(horse.clone()).await;
        world.spawn_entity(skeleton.clone()).await;
        horse
            .get_entity()
            .add_passenger(horse.clone(), skeleton)
            .await;
    }

    async fn ensure_helmet(skeleton: &Arc<dyn EntityBase>) {
        if let Some(living) = skeleton.get_living_entity() {
            let head_empty = living
                .entity_equipment
                .lock()
                .await
                .get(&EquipmentSlot::HEAD)
                .lock()
                .await
                .is_empty();
            if head_empty {
                living
                    .entity_equipment
                    .lock()
                    .await
                    .put(&EquipmentSlot::HEAD, ItemStack::new(1, &Item::IRON_HELMET))
                    .await;
            }
        }
    }
}

impl Goal for SkeletonTrapGoal {
    fn can_start<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let Some(horse) = self.horse.upgrade() else {
                return false;
            };
            if !horse.is_trap() {
                return false;
            }

            let pos = mob.get_entity().pos.load();
            let world = mob.get_entity().world.load();
            world.get_closest_player(pos, TRIGGER_RANGE).is_some()
        })
    }

    fn tick<'a>(&'a mut self, mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            let Some(horse) = self.horse.upgrade() else {
                return;
            };
            // Vanilla: `this.horse.setTrap(false); this.horse.setTamed(true); this.horse.setAge(0);`
            // Pumpkin's generic "tamed" concept (`MobEntity::is_tamed`) is owner-uuid-based and
            // has no "tamed but ownerless" state to set here, so only the trap flag and age are
            // reset.
            horse.set_trap(false);
            mob.get_entity()
                .age
                .store(0, std::sync::atomic::Ordering::Relaxed);

            let pos = mob.get_entity().pos.load();
            let world = mob.get_entity().world.load();

            // Vanilla: `LightningBolt` marked `setVisualOnly(true)` -- a cosmetic strike, no
            // actual lightning damage/fire wiring is attempted here.
            let bolt = crate::entity::r#type::from_type(
                &EntityType::LIGHTNING_BOLT,
                pos,
                &world,
                uuid::Uuid::new_v4(),
            );
            world.spawn_entity(bolt).await;

            // Mount a skeleton on the triggering horse itself.
            let skeleton = crate::entity::r#type::from_type(
                &EntityType::SKELETON,
                pos,
                &world,
                uuid::Uuid::new_v4(),
            );
            Self::ensure_helmet(&skeleton).await;
            world.spawn_entity(skeleton.clone()).await;

            if let Some(self_arc) = world.get_entity_by_id(mob.get_entity().entity_id) {
                self_arc
                    .get_entity()
                    .add_passenger(self_arc.clone(), skeleton)
                    .await;
            }

            for _ in 0..EXTRA_HORSE_COUNT {
                let (dx, dz) = {
                    let mut rng = mob.get_random();
                    (
                        rng.random_range(-1.1485..=1.1485),
                        rng.random_range(-1.1485..=1.1485),
                    )
                };
                let spawn_pos = Vector3::new(pos.x + dx, pos.y, pos.z + dz);
                Self::spawn_mounted_skeleton(mob, spawn_pos).await;
            }
        })
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
