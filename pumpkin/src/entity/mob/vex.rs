use std::sync::atomic::{AtomicBool, AtomicI32, Ordering::Relaxed};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::damage::DamageType;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_util::math::position::BlockPos;
use tokio::sync::Mutex;

use crate::entity::{
    Entity, EntityBase, EntityBaseFuture, NBTStorage,
    ai::control::vex_move_control::VexMoveControl,
    ai::goal::{
        active_target::ActiveTargetGoal, look_at_entity::LookAtEntityGoal, revenge::RevengeGoal,
        swim::SwimGoal, vex_charge_attack::VexChargeAttackGoal,
        vex_copy_owner_target::VexCopyOwnerTargetGoal, vex_random_move::VexRandomMoveGoal,
    },
    mob::{Mob, MobEntity},
};

pub struct VexEntity {
    pub mob_entity: MobEntity,
    /// Vanilla: `Vex.owner` (`OwnableEntity`). Set by `EvokerSummonSpellGoal`.
    owner_id: AtomicCell<Option<i32>>,
    /// Vanilla: `Vex.boundOrigin`, the point `VexRandomMoveGoal` wanders around.
    bound_origin: AtomicCell<Option<BlockPos>>,
    /// Vanilla: `Vex.hasLimitedLife` / `limitedLifeTicks`.
    has_limited_life: AtomicBool,
    limited_life_ticks: AtomicI32,
    /// Vanilla: `Vex.DATA_FLAGS_ID` bit 0 (`FLAG_IS_CHARGING`). Driven by `VexChargeAttackGoal`.
    is_charging: AtomicBool,
}

impl VexEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        // Vanilla: `Vex`'s constructor replaces the default `MoveControl` with `VexMoveControl`.
        *mob_entity.move_control.lock().unwrap() = Box::new(VexMoveControl::default());
        let vex = Self {
            mob_entity,
            owner_id: AtomicCell::new(None),
            bound_origin: AtomicCell::new(None),
            has_limited_life: AtomicBool::new(false),
            limited_life_ticks: AtomicI32::new(0),
            is_charging: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(vex);
        mob_arc
            .mob_entity
            .living_entity
            .entity_equipment
            .try_lock()
            .expect("new vex equipment is uncontended")
            .equipment
            .insert(
                EquipmentSlot::MAIN_HAND,
                Arc::new(Mutex::new(ItemStack::new(1, &Item::IRON_SWORD))),
            );
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let vex_weak = Arc::downgrade(&mob_arc);

        {
            let mut goal_selector = mob_arc.mob_entity.goals_selector.lock().unwrap();

            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(4, Box::new(VexChargeAttackGoal::new(vex_weak.clone())));
            goal_selector.add_goal(8, Box::new(VexRandomMoveGoal::new(vex_weak.clone())));
            // Vex.java:91: `LookAtPlayerGoal(this, Player.class, 3.0F, 1.0F)` -- range 3.0,
            // probability 1.0 (always looks), not the crate default range/chance.
            goal_selector.add_goal(
                9,
                Box::new(LookAtEntityGoal::new(
                    mob_weak.clone(),
                    &EntityType::PLAYER,
                    3.0,
                    1.0,
                    false,
                )),
            );
            // Vex.java:92: `LookAtPlayerGoal(this, Mob.class, 8.0F)`.
            goal_selector.add_goal(
                10,
                LookAtEntityGoal::with_default_any_mob(mob_weak.clone(), 8.0),
            );

            let mut target_selector = mob_arc.mob_entity.target_selector.lock().unwrap();
            // Vex.java:93: `HurtByTargetGoal(this, Raider.class).setAlertOthers()`.
            target_selector.add_goal(1, Box::new(RevengeGoal::new(true).exclude_raiders()));
            target_selector.add_goal(2, Box::new(VexCopyOwnerTargetGoal::new(vex_weak)));
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    /// Vanilla: `Vex#setOwner`.
    pub fn set_owner(&self, owner: &Entity) {
        self.owner_id.store(Some(owner.entity_id));
    }

    #[must_use]
    pub fn owner_id(&self) -> Option<i32> {
        self.owner_id.load()
    }

    /// Vanilla: `Vex#setBoundOrigin`.
    pub fn set_bound_origin(&self, origin: BlockPos) {
        self.bound_origin.store(Some(origin));
    }

    #[must_use]
    pub fn bound_origin(&self) -> Option<BlockPos> {
        self.bound_origin.load()
    }

    /// Vanilla: `Vex#setLimitedLife`.
    pub fn set_limited_life(&self, life_ticks: i32) {
        self.has_limited_life.store(true, Relaxed);
        self.limited_life_ticks.store(life_ticks, Relaxed);
    }

    /// Vanilla: `Vex#isCharging`.
    #[must_use]
    pub fn is_charging(&self) -> bool {
        self.is_charging.load(Relaxed)
    }

    /// Vanilla: `Vex#setIsCharging`.
    pub fn set_is_charging(&self, value: bool) {
        self.is_charging.store(value, Relaxed);
    }
}

impl NBTStorage for VexEntity {}

impl Mob for VexEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    /// Vanilla: `Vex#tick` -- while `hasLimitedLife`, deals 1 starvation damage every 20 ticks
    /// once the counter runs out, resetting it to keep ticking down.
    ///
    /// Scope reduction: vanilla's `Vex.tick` also forces `noPhysics = true` around
    /// `super.tick()` every tick so the vex can fly through blocks mid-charge/wander; Pumpkin's
    /// entity/physics model has no such toggle anywhere (`no_physics` does not exist on `Entity`),
    /// so this vex will still collide with blocks while flying. `setNoGravity(true)` (persistent,
    /// unlike `noPhysics`) is ported below via `get_mob_gravity`.
    fn mob_tick<'a>(&'a self, caller: &'a Arc<dyn EntityBase>) -> EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if self.has_limited_life.load(Relaxed) {
                let remaining = self.limited_life_ticks.fetch_sub(1, Relaxed) - 1;
                if remaining <= 0 {
                    self.limited_life_ticks.store(20, Relaxed);
                    caller
                        .damage(caller.as_ref(), 1.0, DamageType::STARVE)
                        .await;
                }
            }
        })
    }

    /// Vanilla: `Vex#tick`'s persistent `setNoGravity(true)`.
    fn get_mob_gravity(&self) -> f64 {
        0.0
    }
}
