use std::f64::consts::FRAC_PI_2;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use pumpkin_data::Block;
use pumpkin_data::BlockState;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::data_component_impl::EquipmentSlot;
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::Sound;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Hand;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::control::move_control::{MoveControl, Operation};
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::ai::goal::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use crate::entity::ai::goal::ranged_attack::RangedAttackGoal;
use crate::entity::ai::goal::wander_around::WanderAroundGoal;
use crate::entity::ai::goal::zombie_attack::ZombieAttackGoal;
use crate::entity::ai::goal::{Controls, Goal, ParentHandle};
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::pathfinder::{Navigator, NavigatorGoal};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::ai::util::default_random_pos;
use crate::entity::custom_sound::CustomSound;
use crate::entity::living::LivingEntity;
use crate::entity::mob::equipment::{self, RegionalDifficulty};
use crate::entity::mob::spawn::{EntitySpawnReason, SpawnGroupData};
use crate::entity::mob::zombie::ZombieEntityBase;
use crate::entity::mob::{Mob, RangedAttackMob};
use crate::entity::projectile::arrow::ArrowPickup;
use crate::entity::projectile::trident::TridentEntity;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

const DROWNED_HUNTS: &[Hunt] = &[
    Hunt::new(2, TargetMatch::Type(&EntityType::PLAYER), true).matching(ok_target_living),
    Hunt::new(3, TargetMatch::VILLAGER_LIKE, false),
    Hunt::new(3, TargetMatch::Type(&EntityType::IRON_GOLEM), true),
    Hunt::new(3, TargetMatch::Type(&EntityType::AXOLOTL), true),
    hunting::baby_turtles(5, true),
];

pub const NAUTILUS_SHELL_CHANCE: f32 = 0.03;
const TRIDENT_ROLL: f32 = 0.9;
const TRIDENT_WEIGHT: i32 = 10;
const WEAPON_WEIGHTS: i32 = 16;
const TRIDENT_POWER: f64 = 1.6;
const TRIDENT_USE_DURATION: i32 = 72_000;
const SWIM_SPEED: f64 = 0.01;
const SWIM_DRAG: f64 = 0.9;

/// Players only make good targets at night or in water.
fn ok_target_living(_hunter: &dyn Mob, target: &LivingEntity, world: &World) -> bool {
    !world.is_bright_outside() || target.entity.is_in_water()
}

fn ok_target(mob: &dyn Mob) -> bool {
    mob.get_mob_entity().get_target().is_some_and(|target| {
        !mob.get_entity().world.load().is_bright_outside() || target.get_entity().is_in_water()
    })
}

fn as_drowned(mob: &dyn Mob) -> Option<&DrownedEntity> {
    mob.cast_any().downcast_ref::<DrownedEntity>()
}

pub struct DrownedEntity {
    entity: Arc<ZombieEntityBase>,
    searching_for_land: AtomicBool,
}

impl DrownedEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let entity = ZombieEntityBase::with_behaviour_goals(entity, |_| {});
        let mut navigator = entity
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *navigator = Navigator::amphibious(false);
        navigator.set_pathfinding_malus(PathType::Water, 0.0);
        drop(navigator);
        *entity
            .mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(DrownedMoveControl::default());
        let drowned = Arc::new(Self {
            entity,
            searching_for_land: AtomicBool::new(false),
        });
        let ranged: Weak<dyn RangedAttackMob> = {
            let ranged: Arc<dyn RangedAttackMob> = drowned.clone();
            Arc::downgrade(&ranged)
        };

        let mob_entity = &drowned.entity.mob_entity;
        let goal_selector = &mob_entity.goals_selector;
        goal_selector.add_goal(1, Box::new(DrownedGoToWaterGoal::new(1.0)));
        goal_selector.add_goal(
            2,
            Box::new(DrownedTridentAttackGoal {
                ranged: RangedAttackGoal::new(ranged, 1.0, 40, 10.0),
            }),
        );
        goal_selector.add_goal(
            2,
            Box::new(DrownedAttackGoal {
                attack: ZombieAttackGoal::new(1.0, false),
            }),
        );
        goal_selector.add_goal(5, Box::new(DrownedGoToBeachGoal::new(1.0)));
        let sea_level = mob_entity.living_entity.entity.world.load().sea_level;
        goal_selector.add_goal(6, Box::new(DrownedSwimUpGoal::new(1.0, sea_level)));
        goal_selector.add_goal(7, Box::new(WanderAroundGoal::new(1.0)));

        let target_selector = &mob_entity.target_selector;
        target_selector.add_goal(
            1,
            Box::new(
                ZombieEntityBase::revenge_goal()
                    .ignoring(|entity_type| entity_type == &EntityType::DROWNED),
            ),
        );
        hunting::apply(target_selector, mob_entity, DROWNED_HUNTS);

        drowned
    }

    fn wants_to_swim(&self) -> bool {
        self.searching_for_land.load(Ordering::Relaxed)
            || self
                .get_mob_entity()
                .get_target()
                .is_some_and(|target| target.get_entity().is_in_water())
    }

    fn set_searching_for_land(&self, searching: bool) {
        self.searching_for_land.store(searching, Ordering::Relaxed);
    }

    fn is_searching_for_land(&self) -> bool {
        self.searching_for_land.load(Ordering::Relaxed)
    }

    fn close_to_next_pos(&self) -> bool {
        let entity = self.get_entity();
        self.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_target_pos()
            .is_some_and(|pos| entity.pos.load().squared_distance_to_vec(&pos.0.to_f64()) < 4.0)
    }
}

impl CustomSound for DrownedEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(if self.get_entity().is_in_water() {
            Sound::EntityDrownedDeathWater
        } else {
            Sound::EntityDrownedDeath
        })
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(if self.get_entity().is_in_water() {
            Sound::EntityDrownedHurtWater
        } else {
            Sound::EntityDrownedHurt
        })
    }
}

impl RangedAttackMob for DrownedEntity {
    fn perform_ranged_attack(&self, target: &Arc<dyn EntityBase>, _power: f32) {
        let entity = self.get_entity();
        let world = entity.world.load_full();
        let main_hand = self
            .get_mob_entity()
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND);
        let trident_stack = if main_hand.item == &Item::TRIDENT {
            main_hand
        } else {
            ItemStack::new(1, &Item::TRIDENT)
        };
        let trident = TridentEntity::new_shot(
            Entity::new(world.clone(), entity.pos.load(), &EntityType::TRIDENT),
            entity,
            trident_stack.clone(),
            ArrowPickup::Disallowed,
        );
        trident.apply_on_projectile_spawned(&trident_stack);

        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let dx = target_pos.x - entity.pos.load().x;
        let dy = (target_pos.y + f64::from(target_entity.entity_dimension.load().height) / 3.0)
            - trident.get_entity().pos.load().y;
        let dz = target_pos.z - entity.pos.load().z;
        let horizontal_distance = dx.hypot(dz);
        let difficulty = world.level_info.load().difficulty as i32;
        trident.set_velocity(
            dx,
            horizontal_distance.mul_add(f64::from(0.2f32), dy),
            dz,
            TRIDENT_POWER,
            f64::from(14 - difficulty * 4),
        );
        world.spawn_entity(Arc::new(trident));

        let mut rng = self.get_random();
        let pitch = 1.0 / rng.random::<f32>().mul_add(0.4, 0.8);
        world.play_sound_raw(
            Sound::EntityDrownedShoot as u16,
            self.mob_sound_source(),
            &entity.pos.load(),
            1.0,
            pitch,
        );
    }
}

impl Mob for DrownedEntity {
    crate::zombie_mob_hooks!(entity);

    fn wants_to_pick_up(&self, stack: &ItemStack) -> bool {
        !stack.item.has_tag(&tag::Item::MINECRAFT_SPEARS) && super::wants_to_pick_up(self, stack)
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityDrownedStep, 0.15, 1.0);
    }

    fn mob_get_swim_sound(&self) -> Option<Sound> {
        Some(Sound::EntityDrownedSwim)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        Some(if self.get_entity().is_in_water() {
            Sound::EntityDrownedAmbientWater
        } else {
            Sound::EntityDrownedAmbient
        })
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        let group_data = self
            .entity
            .finalize_zombie_spawn(self, world, reason, group_data);
        let mob_entity = self.get_mob_entity();
        let offhand_empty = mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::OFF_HAND)
            .is_empty();
        if offhand_empty && rand::random::<f32>() < NAUTILUS_SHELL_CHANCE {
            mob_entity.set_item_slot_and_drop_when_killed(
                &EquipmentSlot::OFF_HAND,
                ItemStack::new(1, &Item::NAUTILUS_SHELL),
            );
        }
        group_data
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        self.entity.do_hurt_target(self, target)
    }

    fn populate_default_equipment_slots(
        &self,
        _world: &Arc<World>,
        _difficulty: &RegionalDifficulty,
    ) {
        let mut rng = self.get_random();
        if rng.random::<f32>() > TRIDENT_ROLL {
            let weapon = if rng.random_range(0..WEAPON_WEIGHTS) < TRIDENT_WEIGHT {
                &Item::TRIDENT
            } else {
                &Item::FISHING_ROD
            };
            self.get_mob_entity()
                .living_entity
                .entity_equipment
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .put(&EquipmentSlot::MAIN_HAND, ItemStack::new(1, weapon));
        }
    }

    fn can_replace_current_item(
        &self,
        new_item: &ItemStack,
        current_item: &ItemStack,
        slot: &EquipmentSlot,
    ) -> bool {
        current_item.item != &Item::NAUTILUS_SHELL
            && equipment::can_replace_current_item(
                self.get_mob_entity(),
                self.get_preferred_weapon_type(),
                new_item,
                current_item,
                slot,
            )
    }

    fn get_preferred_weapon_type(&self) -> Option<&'static tag::Tag> {
        Some(&tag::Item::MINECRAFT_DROWNED_PREFERRED_WEAPONS)
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        world.is_unobstructed(self.get_entity())
    }

    fn mob_is_pushed_by_fluids(&self) -> bool {
        !self.get_entity().is_swimming()
    }

    fn base_tick(&self) {
        let entity = self.get_entity();
        entity.set_swimming(
            !self.get_mob_entity().is_no_ai() && entity.is_under_water() && self.wants_to_swim(),
        );
    }

    fn travel_in_water(&self, caller: &dyn EntityBase, gravity: f64, falling: bool) {
        let living = &self.get_mob_entity().living_entity;
        let entity = &living.entity;
        if entity.is_under_water() && self.wants_to_swim() {
            entity.update_velocity_from_input(living.movement_input.load(), SWIM_SPEED);
            living.make_move(caller);
            entity.velocity.store(entity.velocity.load() * SWIM_DRAG);
        } else {
            living.travel_in_water(caller, gravity, falling);
        }
    }

    fn mob_ride_tick(&self) {
        let entity = self.get_entity();
        if let Some(vehicle) = entity.get_vehicle()
            && vehicle.get_mob().is_some()
            && vehicle
                .get_controlling_passenger()
                .is_some_and(|controller| controller.get_entity().entity_id == entity.entity_id)
        {
            entity.body_yaw.store(vehicle.get_entity().body_yaw.load());
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.entity.write_nbt(nbt);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.entity.read_nbt(self, nbt);
    }
}

/// Swims towards its target or the surface, sinking otherwise.
#[derive(Default)]
struct DrownedMoveControl {
    inner: MoveControl,
}

impl Control for DrownedMoveControl {}

impl MoveControlTrait for DrownedMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(drowned) = as_drowned(mob) else {
            self.inner.tick(mob);
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let living = &mob_entity.living_entity;
        let entity = &living.entity;
        if drowned.wants_to_swim() && entity.is_in_water() {
            let rising = mob_entity
                .get_target()
                .is_some_and(|target| target.get_entity().pos.load().y > entity.pos.load().y)
                || drowned.is_searching_for_land();
            if rising {
                entity
                    .velocity
                    .store(entity.velocity.load().add_raw(0.0, 0.002, 0.0));
            }
            if self.inner.operation != Operation::MoveTo || mob.is_navigator_idle() {
                mob_entity.set_speed(0.0);
                return;
            }
            let pos = entity.pos.load();
            let xd = self.inner.wanted_x - pos.x;
            let mut yd = self.inner.wanted_y - pos.y;
            let zd = self.inner.wanted_z - pos.z;
            let dd = (xd * xd + yd * yd + zd * zd).sqrt();
            // Already at the wanted position: dividing by zero would poison the velocity with NaN.
            if dd == 0.0 {
                mob_entity.set_speed(0.0);
                return;
            }
            yd /= dd;
            let y_rot_d = (zd.atan2(xd).to_degrees() as f32) - 90.0;
            let yaw = MoveControl::rotlerp(entity.yaw.load(), y_rot_d, MoveControl::MAX_TURN);
            entity.yaw.store(yaw);
            entity.body_yaw.store(yaw);
            let target_speed = (self.inner.speed_modifier
                * living.get_attribute_value(&Attributes::MOVEMENT_SPEED))
                as f32;
            let new_speed = (target_speed - living.get_speed()).mul_add(0.125, living.get_speed());
            mob_entity.set_speed(new_speed);
            let new_speed = f64::from(new_speed);
            entity.velocity.store(entity.velocity.load().add_raw(
                new_speed * xd * 0.005,
                new_speed * yd * 0.1,
                new_speed * zd * 0.005,
            ));
        } else {
            if !entity.on_ground.load(Ordering::Relaxed) {
                entity
                    .velocity
                    .store(entity.velocity.load().add_raw(0.0, -0.008, 0.0));
            }
            self.inner.tick(mob);
        }
    }

    fn set_wanted_position(&mut self, x: f64, y: f64, z: f64, speed_modifier: f64) {
        self.inner.set_wanted_position(x, y, z, speed_modifier);
    }

    fn strafe(&mut self, forwards: f32, right: f32) {
        self.inner.strafe(forwards, right);
    }

    fn has_wanted(&self) -> bool {
        self.inner.has_wanted()
    }

    fn get_speed_modifier(&self) -> f64 {
        self.inner.get_speed_modifier()
    }

    fn get_wanted_position(&self) -> Vector3<f64> {
        self.inner.get_wanted_position()
    }
}

/// Melee only while the target is fair game.
struct DrownedAttackGoal {
    attack: Box<ZombieAttackGoal>,
}

impl Goal for DrownedAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.attack.can_start(mob) && ok_target(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.attack.should_continue(mob) && ok_target(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.attack.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.attack.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.attack.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.attack.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.attack.controls()
    }
}

/// Throws its trident from range while holding one.
struct DrownedTridentAttackGoal {
    ranged: RangedAttackGoal,
}

fn holds_trident(mob: &dyn Mob) -> bool {
    mob.get_mob_entity()
        .living_entity
        .entity_equipment
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&EquipmentSlot::MAIN_HAND)
        .item
        == &Item::TRIDENT
}

impl Goal for DrownedTridentAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.ranged.can_start(mob) && holds_trident(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob) || self.ranged.target_alive() && !mob.is_navigator_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.ranged.start(mob);
        let mob_entity = mob.get_mob_entity();
        mob_entity.set_attacking(true);
        let stack = mob_entity
            .living_entity
            .entity_equipment
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&EquipmentSlot::MAIN_HAND);
        mob_entity
            .living_entity
            .set_active_hand(Hand::Right, stack, TRIDENT_USE_DURATION);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.ranged.stop(mob);
        let mob_entity = mob.get_mob_entity();
        mob_entity.living_entity.clear_active_hand();
        mob_entity.set_attacking(false);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.ranged.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.ranged.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.ranged.controls()
    }
}

/// By day, walks back into water.
struct DrownedGoToWaterGoal {
    speed_modifier: f64,
    wanted: Vector3<f64>,
}

impl DrownedGoToWaterGoal {
    const fn new(speed_modifier: f64) -> Self {
        Self {
            speed_modifier,
            wanted: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    fn water_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let mut rng = mob.get_random();
        let entity = mob.get_entity();
        let world = entity.world.load();
        let pos = entity.block_pos.load();
        for _ in 0..10 {
            let candidate = pos.offset(Vector3::new(
                rng.random_range(0..20) - 10,
                2 - rng.random_range(0..8),
                rng.random_range(0..20) - 10,
            ));
            if world.get_block(&candidate) == &Block::WATER {
                return Some(Vector3::new(
                    f64::from(candidate.0.x) + 0.5,
                    f64::from(candidate.0.y),
                    f64::from(candidate.0.z) + 0.5,
                ));
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
        let Some(wanted) = Self::water_pos(mob) else {
            return false;
        };
        self.wanted = wanted;
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.is_navigator_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        let steered = mob.get_mob_entity().steered();
        steered.navigator().set_progress(NavigatorGoal {
            current_progress: mob.get_entity().pos.load(),
            destination: self.wanted,
            speed: self.speed_modifier,
        });
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

struct BeachTarget;

static BEACH_TARGET: BeachTarget = BeachTarget;

impl MoveToTargetPos for BeachTarget {
    fn is_target_pos(&self, world: &World, block_pos: BlockPos) -> bool {
        let above = block_pos.up();
        world.get_block_state(&above).is_air()
            && world.get_block_state(&above.up()).is_air()
            && world
                .get_block_state(&block_pos)
                .is_side_solid(pumpkin_data::BlockDirection::Up)
    }
}

/// At night, climbs out onto a standable shore block.
struct DrownedGoToBeachGoal {
    move_to_target_pos_goal: MoveToTargetPosGoal<BeachTarget>,
}

impl DrownedGoToBeachGoal {
    fn new(speed: f64) -> Self {
        Self {
            move_to_target_pos_goal: MoveToTargetPosGoal::new(
                ParentHandle::from_static(&BEACH_TARGET),
                speed,
                8,
                2,
            ),
        }
    }
}

impl Goal for DrownedGoToBeachGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        let world = entity.world.load();
        self.move_to_target_pos_goal.can_start(mob)
            && !world.is_bright_outside()
            && entity.is_in_water()
            && entity.pos.load().y >= f64::from(world.sea_level - 3)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.move_to_target_pos_goal.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(drowned) = as_drowned(mob) {
            drowned.set_searching_for_land(false);
        }
        self.move_to_target_pos_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.tick(mob);
    }

    fn should_run_every_tick(&self) -> bool {
        self.move_to_target_pos_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}

/// At night, swims up towards the surface to look for land.
struct DrownedSwimUpGoal {
    speed_modifier: f64,
    sea_level: i32,
    stuck: bool,
}

impl DrownedSwimUpGoal {
    const fn new(speed_modifier: f64, sea_level: i32) -> Self {
        Self {
            speed_modifier,
            sea_level,
            stuck: false,
        }
    }

    fn can_use(&self, mob: &dyn Mob) -> bool {
        let entity = mob.get_entity();
        !entity.world.load().is_bright_outside()
            && entity.is_in_water()
            && entity.pos.load().y < f64::from(self.sea_level - 2)
    }
}

impl Goal for DrownedSwimUpGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.can_use(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_use(mob) && !self.stuck
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(drowned) = as_drowned(mob) {
            drowned.set_searching_for_land(true);
        }
        self.stuck = false;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(drowned) = as_drowned(mob) {
            drowned.set_searching_for_land(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let close_to_next = as_drowned(mob).is_some_and(DrownedEntity::close_to_next_pos);
        if pos.y < f64::from(self.sea_level - 1) && (mob.is_navigator_idle() || close_to_next) {
            let Some(next_pos) = default_random_pos::get_pos_towards(
                mob,
                4,
                8,
                Vector3::new(pos.x, f64::from(self.sea_level - 1), pos.z),
                FRAC_PI_2,
            ) else {
                self.stuck = true;
                return;
            };
            let steered = mob.get_mob_entity().steered();
            steered.navigator().set_progress(NavigatorGoal {
                current_progress: pos,
                destination: next_pos,
                speed: self.speed_modifier,
            });
        }
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}
