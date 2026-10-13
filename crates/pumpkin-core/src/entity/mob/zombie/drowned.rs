use std::f64::consts::FRAC_PI_2;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::lerp;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::control::move_control::MoveControl;
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::ai::goal::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use crate::entity::ai::goal::revenge::RevengeGoal;
use crate::entity::ai::goal::wander_around::WanderAroundGoal;
use crate::entity::ai::goal::zombie_attack::ZombieAttackGoal;
use crate::entity::ai::goal::{Controls, Goal, ParentHandle, active_target::ActiveTargetGoal};
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::pathfinder::{Navigator, NavigatorGoal};
use crate::entity::ai::util::default_random_pos;
use crate::entity::living::LivingEntity;
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::passive::turtle::TurtleEntity;
use crate::entity::{
    Entity, EntityBase,
    mob::{Mob, MobEntity},
};
use crate::world::World;

pub struct DrownedEntity {
    entity: Arc<ZombieEntityBase>,
    searching_for_land: AtomicBool,
}

impl DrownedEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        Self::with_can_break_doors(entity, false)
    }

    #[must_use]
    pub fn with_can_break_doors(entity: Entity, can_break_doors: bool) -> Arc<Self> {
        let entity = ZombieEntityBase::without_behaviour_goals(entity, can_break_doors);
        let drowned = Arc::new(Self {
            entity,
            searching_for_land: AtomicBool::new(false),
        });

        let mob_entity = &drowned.entity.mob_entity;
        let mut navigator = Navigator::amphibious(false);
        navigator.set_pathfinding_malus(PathType::Water, 0.0);
        *mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = navigator;
        *mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(DrownedMoveControl::new(Arc::downgrade(&drowned)));

        drowned.add_behaviour_goals();
        drowned
    }

    /// Vanilla `Drowned.addBehaviourGoals`.
    fn add_behaviour_goals(self: &Arc<Self>) {
        let mob_entity = &self.entity.mob_entity;
        let sea_level = mob_entity.living_entity.entity.world.load().sea_level;
        {
            let mut goal_selector = mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            goal_selector.add_goal(1, Box::new(DrownedGoToWaterGoal::new(1.0)));
            goal_selector.add_goal(2, Box::new(DrownedAttackGoal::new(1.0, false)));
            goal_selector.add_goal(5, DrownedGoToBeachGoal::new(Arc::downgrade(self), 1.0));
            goal_selector.add_goal(
                6,
                Box::new(DrownedSwimUpGoal::new(Arc::downgrade(self), 1.0, sea_level)),
            );
            goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));
        };

        let mut target_selector = mob_entity
            .target_selector
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        target_selector.add_goal(
            1,
            Box::new(
                RevengeGoal::new(true)
                    .ignoring(|entity_type| entity_type == &EntityType::DROWNED)
                    .alerting_others_except(|entity_type| {
                        entity_type == &EntityType::ZOMBIFIED_PIGLIN
                    }),
            ),
        );
        target_selector.add_goal(
            2,
            Box::new(ActiveTargetGoal::new(
                mob_entity,
                &EntityType::PLAYER,
                10,
                true,
                false,
                Some(|target: &LivingEntity, world: &World| Self::ok_target(&target.entity, world)),
            )),
        );
        target_selector.add_goal(
            3,
            ActiveTargetGoal::with_default(mob_entity, &EntityType::VILLAGER, false),
        );
        target_selector.add_goal(
            3,
            ActiveTargetGoal::with_default(mob_entity, &EntityType::IRON_GOLEM, true),
        );
        target_selector.add_goal(
            3,
            ActiveTargetGoal::with_default(mob_entity, &EntityType::AXOLOTL, true),
        );
        target_selector.add_goal(
            5,
            Box::new(ActiveTargetGoal::new(
                mob_entity,
                &EntityType::TURTLE,
                10,
                true,
                false,
                Some(TurtleEntity::baby_on_land_selector),
            )),
        );
    }

    /// Vanilla `Drowned.okTarget`: by day only targets in water are worth chasing.
    #[must_use]
    pub fn ok_target(target: &Entity, world: &World) -> bool {
        !world.is_bright_outside() || target.is_in_water()
    }

    fn wants_to_swim(&self) -> bool {
        self.searching_for_land.load(Ordering::Relaxed)
            || self
                .entity
                .mob_entity
                .get_target()
                .is_some_and(|target| target.get_entity().is_in_water())
    }

    pub fn set_searching_for_land(&self, searching_for_land: bool) {
        self.searching_for_land
            .store(searching_for_land, Ordering::Relaxed);
    }

    /// Vanilla `Drowned.updateSwimming`, which runs from `baseTick`.
    fn update_swimming(&self) {
        let mob_entity = &self.entity.mob_entity;
        let entity = &mob_entity.living_entity.entity;
        entity.set_swimming(
            !mob_entity.is_no_ai() && entity.is_under_water() && self.wants_to_swim(),
        );
    }

    /// Vanilla `Drowned.closeToNextPos`: within two blocks of where the current path ends.
    fn close_to_next_pos(&self) -> bool {
        let pos = self.entity.mob_entity.living_entity.entity.pos.load();
        self.entity
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_path()
            .is_some_and(|path| {
                let target = path.get_target();
                pos.squared_distance_to(
                    f64::from(target.0.x),
                    f64::from(target.0.y),
                    f64::from(target.0.z),
                ) < 4.0
            })
    }
}

impl Mob for DrownedEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity.mob_entity
    }

    fn spawn_as_baby(&self) -> bool {
        self.entity.spawn_as_baby()
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.update_swimming();
    }

    /// Vanilla `Drowned.travelInWater`: swims freely while it wants to swim, otherwise moves like
    /// any other mob in water.
    fn mob_travel_in_water(&self, caller: &dyn EntityBase) -> bool {
        let living = &self.entity.mob_entity.living_entity;
        let entity = &living.entity;
        if !entity.is_under_water() || !self.wants_to_swim() {
            return false;
        }
        // Vanilla has no vertical input here; the move control does the climbing.
        let input = living.movement_input.load();
        entity.update_velocity_from_input(Vector3::new(input.x, 0.0, input.z), 0.01);
        entity.move_entity(caller, entity.velocity.load());
        entity.velocity.store(entity.velocity.load() * 0.9);
        true
    }

    fn mob_is_pushed_by_fluids(&self) -> bool {
        !self.get_entity().is_swimming()
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.mob_write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.entity.mob_read_nbt(nbt);
    }
}

/// Vanilla `Drowned.DrownedMoveControl`: swims towards the next node while the drowned wants to
/// swim, otherwise moves like a zombie and sinks faster.
struct DrownedMoveControl {
    drowned: Weak<DrownedEntity>,
    inner: MoveControl,
    /// Vanilla `Mob.speed`, which the swim stroke eases towards the navigator's speed.
    speed: f64,
}

impl DrownedMoveControl {
    fn new(drowned: Weak<DrownedEntity>) -> Self {
        Self {
            drowned,
            inner: MoveControl::default(),
            speed: 0.0,
        }
    }
}

impl Control for DrownedMoveControl {}

impl MoveControlTrait for DrownedMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(drowned) = self.drowned.upgrade() else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let living = &mob_entity.living_entity;
        let entity = &living.entity;
        let mut velocity = entity.velocity.load();

        if !drowned.wants_to_swim() || !entity.is_in_water() {
            if !entity.on_ground.load(Ordering::Relaxed) {
                velocity.y -= 0.008;
                entity.velocity.store(velocity);
            }
            self.inner.tick(mob);
            // The navigator writes the walking speed straight into the input.
            self.speed = living.movement_input.load().z;
            return;
        }

        let pos = entity.pos.load();
        let target_above = mob_entity
            .get_target()
            .is_some_and(|target| target.get_entity().pos.load().y > pos.y);
        if target_above || drowned.searching_for_land.load(Ordering::Relaxed) {
            velocity.y += 0.002;
        }

        // Pumpkin's navigator steers by itself, so take the spot vanilla's `followThePath` would
        // have handed to `setWantedPosition`.
        let wanted = mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_path()
            .and_then(|path| path.get_next_entity_pos(entity.width()));
        let Some(wanted) = wanted else {
            self.speed = 0.0;
            living.movement_input.store(Vector3::default());
            entity.velocity.store(velocity);
            return;
        };

        let delta = wanted - pos;
        let distance = delta.length();
        // Pumpkin's navigator can leave the mob right on that spot; don't divide by zero.
        let dy = if distance > 0.0 {
            delta.y / distance
        } else {
            0.0
        };
        // This tick the navigator put `speedModifier * MOVEMENT_SPEED` in the forward input.
        let speed = lerp(0.125, self.speed, living.movement_input.load().z);
        self.speed = speed;
        living.movement_input.store(Vector3::new(0.0, 0.0, speed));
        entity.velocity.store(
            velocity
                + Vector3::new(
                    speed * delta.x * 0.005,
                    speed * dy * 0.1,
                    speed * delta.z * 0.005,
                ),
        );
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        MoveControlTrait::set_wanted_position(&mut self.inner, x, y, z, speed_modifier);
    }

    fn strafe(&mut self, forward: f32, right: f32) {
        self.inner.strafe(forward, right);
    }

    fn has_wanted(&self) -> bool {
        MoveControlTrait::has_wanted(&self.inner)
    }
}

/// Vanilla `Drowned.DrownedGoToWaterGoal`: by day, heads for water nearby.
struct DrownedGoToWaterGoal {
    speed: f64,
    wanted: Vector3<f64>,
}

impl DrownedGoToWaterGoal {
    const fn new(speed: f64) -> Self {
        Self {
            speed,
            wanted: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    fn get_water_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let entity = mob.get_entity();
        let world = entity.world.load();
        let block_pos = entity.block_pos.load();
        let mut rng = mob.get_random();
        for _ in 0..10 {
            let pos = block_pos.add(
                rng.random_range(0..20) - 10,
                2 - rng.random_range(0..8),
                rng.random_range(0..20) - 10,
            );
            if world.get_block(&pos) == &Block::WATER {
                return Some(pos.to_f64());
            }
        }
        None
    }
}

impl Goal for DrownedGoToWaterGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        if !entity.world.load().is_bright_outside() || entity.is_in_water() {
            return false;
        }
        let Some(wanted) = Self::get_water_pos(mob) else {
            return false;
        };
        self.wanted = wanted;
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.is_navigator_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        let pos = mob.get_entity().pos.load();
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_progress(NavigatorGoal::new(pos, self.wanted, self.speed));
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

/// Vanilla `Drowned.DrownedAttackGoal`: the zombie attack, only against targets `okTarget` allows.
struct DrownedAttackGoal {
    zombie_attack: Box<ZombieAttackGoal>,
}

impl DrownedAttackGoal {
    fn new(speed: f64, pause_when_mob_idle: bool) -> Self {
        Self {
            zombie_attack: ZombieAttackGoal::new(speed, pause_when_mob_idle),
        }
    }

    fn has_ok_target(mob: &dyn Mob) -> bool {
        let world = mob.get_entity().world.load();
        mob.get_mob_entity()
            .get_target()
            .is_some_and(|target| DrownedEntity::ok_target(target.get_entity(), &world))
    }
}

impl Goal for DrownedAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.zombie_attack.can_start(mob) && Self::has_ok_target(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.zombie_attack.should_continue(mob) && Self::has_ok_target(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.zombie_attack.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.zombie_attack.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.zombie_attack.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.zombie_attack.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.zombie_attack.controls()
    }
}

/// Vanilla `Drowned.DrownedGoToBeachGoal`: at night, near the surface, heads for dry land.
struct DrownedGoToBeachGoal {
    drowned: Weak<DrownedEntity>,
    move_to_block: MoveToTargetPosGoal<Self>,
}

impl DrownedGoToBeachGoal {
    #[expect(
        clippy::unnecessary_box_returns,
        reason = "the inner goal points back into this heap allocation"
    )]
    fn new(drowned: Weak<DrownedEntity>, speed: f64) -> Box<Self> {
        let mut this = Box::new(Self {
            drowned,
            move_to_block: MoveToTargetPosGoal::new(ParentHandle::none(), speed, 8, 2),
        });
        // SAFETY: `this` heap allocation address is pinned in Box and outlives `ParentHandle` references.
        this.move_to_block.move_to_target_pos = unsafe { ParentHandle::new(&this) };
        this
    }
}

impl Goal for DrownedGoToBeachGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if !self.move_to_block.can_start(mob) {
            return false;
        }
        let entity = mob.get_entity();
        let world = entity.world.load();
        !world.is_bright_outside()
            && entity.is_in_water()
            && entity.pos.load().y >= f64::from(world.sea_level - 3)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.move_to_block.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(drowned) = self.drowned.upgrade() {
            drowned.set_searching_for_land(false);
        }
        self.move_to_block.start(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.move_to_block.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.move_to_block.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.move_to_block.controls()
    }
}

impl MoveToTargetPos for DrownedGoToBeachGoal {
    fn is_target_pos(&self, world: Arc<World>, block_pos: BlockPos) -> bool {
        let above = block_pos.up();
        world.get_block_state(&above).is_air()
            && world.get_block_state(&above.up()).is_air()
            && world
                .get_block_state(&block_pos)
                .is_side_solid(BlockDirection::Up)
    }
}

/// Vanilla `Drowned.DrownedSwimUpGoal`: at night, swims up from the depths towards the surface.
struct DrownedSwimUpGoal {
    drowned: Weak<DrownedEntity>,
    speed: f64,
    sea_level: i32,
    stuck: bool,
}

impl DrownedSwimUpGoal {
    const fn new(drowned: Weak<DrownedEntity>, speed: f64, sea_level: i32) -> Self {
        Self {
            drowned,
            speed,
            sea_level,
            stuck: false,
        }
    }
}

impl Goal for DrownedSwimUpGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        !entity.world.load().is_bright_outside()
            && entity.is_in_water()
            && entity.pos.load().y < f64::from(self.sea_level - 2)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob) && !self.stuck
    }

    fn start(&mut self, _mob: &dyn Mob) {
        if let Some(drowned) = self.drowned.upgrade() {
            drowned.set_searching_for_land(true);
        }
        self.stuck = false;
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(drowned) = self.drowned.upgrade() {
            drowned.set_searching_for_land(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(drowned) = self.drowned.upgrade() else {
            return;
        };
        let pos = mob.get_entity().pos.load();
        let surface_y = f64::from(self.sea_level - 1);
        if pos.y < surface_y && (mob.is_navigator_idle() || drowned.close_to_next_pos()) {
            let towards = Vector3::new(pos.x, surface_y, pos.z);
            let Some(target) = default_random_pos::get_pos_towards(mob, 4, 8, towards, FRAC_PI_2)
            else {
                self.stuck = true;
                return;
            };
            mob.get_mob_entity()
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .set_progress(NavigatorGoal::new(pos, target, self.speed));
        }
    }
}
