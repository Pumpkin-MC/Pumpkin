use std::f64::consts::{FRAC_PI_2, PI};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, AtomicI32, Ordering},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::block_properties::TurtleEggLikeProperties;
use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::game_event::GameEvent;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tag;
use pumpkin_data::tag::Taggable;
use pumpkin_data::world::WorldEvent;
use pumpkin_data::{Block, BlockState};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::block::blocks::turtle_egg::TurtleEggBlock;
use crate::entity::ai::brain::memory::codec::{block_pos_from_tag, block_pos_to_tag};
use crate::entity::ai::control::move_control::{MoveControl, Operation};
use crate::entity::ai::control::{Control, MoveControlTrait};
use crate::entity::ai::goal::move_to_target_pos::{MoveToTargetPos, MoveToTargetPosGoal};
use crate::entity::ai::goal::{Controls, Goal, GoalReach, ParentHandle, to_goal_ticks};
use crate::entity::ai::pathfinder::node::PathType;
use crate::entity::ai::pathfinder::{Navigator, NavigatorGoal};
use crate::entity::ai::util::default_random_pos;
use crate::entity::custom_sound::CustomSound;
use crate::entity::lightning::LightningBoltEntity;
use crate::entity::{
    Entity, EntityBase,
    ageable::{AgeableData, AgeableMob},
    ai::goal::{
        breed::BreedGoal, escape_danger::EscapeDangerGoal, look_at_entity::LookAtEntityGoal,
        tempt::TemptGoal, wander_around::WanderAroundGoal,
    },
    mob::{
        Mob, MobEntity, drop_from_gift_loot_table,
        spawn::{EntitySpawnReason, SpawnGroupData},
    },
    passive::animal::Animal,
    player::Player,
};
use crate::world::World;
use crate::world::game_event::GameEventContext;

const TURTLE_GROW_LOOT_TABLE: &str = "minecraft:gameplay/turtle_grow";
const PANIC_WATER_SEARCH_XZ: i32 = 7;
const AMBIENT_SOUND_INTERVAL: i32 = 200;
const LAY_EGG_TICKS: i32 = 200;
const LAY_EGG_PARTICLE_INTERVAL: i32 = 5;
const LAY_EGG_HOME_DISTANCE: f64 = 9.0;
const IN_LOVE_TICKS_AFTER_LAYING: i32 = 600;
const LAY_EGG_SOUND_VOLUME: f32 = 0.3;
const GO_HOME_CHANCE: i32 = 700;
const GO_HOME_GIVE_UP_TICKS: i32 = 600;
const GO_TO_WATER_GIVE_UP_TICKS: i32 = 1200;
const GO_TO_WATER_RECALCULATE_TICKS: i32 = 160;
const TRAVEL_XZ_DIST: i32 = 512;
const TRAVEL_Y_DIST: i32 = 4;
const TRAVEL_LOADED_RADIUS: i32 = 34;
const SWIM_SPEED: f64 = 0.1;
const SWIM_DRAG: f64 = 0.9;
const SINK_SPEED: f64 = 0.005;
const SINK_HOME_DISTANCE: f64 = 20.0;
const STEP_INTERVAL: f32 = 0.15;

/// Turtle is prey for wolves and cats.
pub fn is_baby_on_land(mob: &dyn Mob) -> bool {
    mob.as_ageable().is_some_and(AgeableMob::is_baby) && !mob.get_entity().is_in_water()
}

fn as_turtle(mob: &dyn Mob) -> Option<&TurtleEntity> {
    mob.cast_any().downcast_ref::<TurtleEntity>()
}

fn closer_to_center_than(pos: BlockPos, entity_pos: Vector3<f64>, distance: f64) -> bool {
    pos.to_centered_f64().squared_distance_to_vec(&entity_pos) < distance * distance
}

fn bottom_center(pos: BlockPos) -> Vector3<f64> {
    Vector3::new(
        f64::from(pos.0.x) + 0.5,
        f64::from(pos.0.y),
        f64::from(pos.0.z) + 0.5,
    )
}

pub struct TurtleEntity {
    pub mob_entity: MobEntity,
    pub ageable_data: AgeableData,
    has_egg: AtomicBool,
    laying_egg: AtomicBool,
    lay_egg_counter: AtomicI32,
    home_pos: AtomicCell<BlockPos>,
    travel_pos: AtomicCell<Option<BlockPos>>,
    going_home: AtomicBool,
}

impl TurtleEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        {
            let mut navigator = mob_entity
                .navigator
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *navigator = Navigator::amphibious(false);
            navigator.set_pathfinding_malus(PathType::Water, 0.0);
            navigator.set_pathfinding_malus(PathType::DoorIronClosed, -1.0);
            navigator.set_pathfinding_malus(PathType::DoorWoodClosed, -1.0);
            navigator.set_pathfinding_malus(PathType::DoorOpen, -1.0);
        };
        *mob_entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Box::new(TurtleMoveControl::default());
        let turtle = Self {
            mob_entity,
            ageable_data: AgeableData::default(),
            has_egg: AtomicBool::new(false),
            laying_egg: AtomicBool::new(false),
            lay_egg_counter: AtomicI32::new(0),
            home_pos: AtomicCell::new(BlockPos::ZERO),
            travel_pos: AtomicCell::new(None),
            going_home: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(turtle);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let goal_selector = &mob_arc.mob_entity.goals_selector;

            goal_selector.add_goal(
                0,
                EscapeDangerGoal::seeking_water(1.2, PANIC_WATER_SEARCH_XZ),
            );
            goal_selector.add_goal(
                1,
                Box::new(TurtleBreedGoal {
                    breed: BreedGoal::new(1.0),
                }),
            );
            goal_selector.add_goal(1, Box::new(TurtleLayEggGoal::new(1.0)));
            goal_selector.add_goal(
                2,
                Box::new(TemptGoal::with_tag(
                    1.1,
                    &tag::Item::MINECRAFT_TURTLE_FOOD,
                    false,
                )),
            );
            goal_selector.add_goal(3, Box::new(TurtleGoToWaterGoal::new(1.0)));
            goal_selector.add_goal(4, Box::new(TurtleGoHomeGoal::new(1.0)));
            goal_selector.add_goal(7, Box::new(TurtleTravelGoal::new(1.0)));
            goal_selector.add_goal(
                8,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(
                9,
                Box::new(
                    WanderAroundGoal::with_interval(1.0, 100)
                        .with_start_condition(Self::can_random_stroll),
                ),
            );
        };

        mob_arc
    }

    #[must_use]
    pub fn has_egg(&self) -> bool {
        self.has_egg.load(Ordering::Relaxed)
    }

    pub fn set_has_egg(&self, has_egg: bool) {
        self.has_egg.store(has_egg, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::turtle::HAS_EGG, has_egg);
    }

    #[must_use]
    pub fn is_laying_egg(&self) -> bool {
        self.laying_egg.load(Ordering::Relaxed)
    }

    pub fn set_laying_egg(&self, laying_egg: bool) {
        self.lay_egg_counter
            .store(i32::from(laying_egg), Ordering::Relaxed);
        self.laying_egg.store(laying_egg, Ordering::Relaxed);
        let entity = self.get_entity();
        entity.set_synced_data(pumpkin_data::tracked_data::turtle::LAYING_EGG, laying_egg);
    }

    #[must_use]
    pub fn get_home_pos(&self) -> BlockPos {
        self.home_pos.load()
    }

    pub fn set_home_pos(&self, pos: BlockPos) {
        self.home_pos.store(pos);
    }

    fn is_going_home(&self) -> bool {
        self.going_home.load(Ordering::Relaxed)
    }

    fn is_home_within(&self, distance: f64) -> bool {
        closer_to_center_than(self.get_home_pos(), self.get_entity().pos.load(), distance)
    }

    fn can_random_stroll(mob: &dyn Mob) -> bool {
        as_turtle(mob).is_some_and(|turtle| {
            !turtle.get_entity().is_in_water() && !turtle.is_going_home() && !turtle.has_egg()
        })
    }
}

impl AgeableMob for TurtleEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable_data
    }

    fn age_boundary_reached(&self) {
        let world = self.get_entity().world.load();
        if !self.is_baby() && world.level_info.load().game_rules.mob_drops {
            let entity = self.get_entity();
            drop_from_gift_loot_table(self, TURTLE_GROW_LOOT_TABLE, |stack| {
                entity.spawn_at_location(stack);
            });
        }
    }
}

impl Animal for TurtleEntity {
    fn is_food(&self, item_stack: &ItemStack) -> bool {
        item_stack.item.has_tag(&tag::Item::MINECRAFT_TURTLE_FOOD)
    }

    fn can_fall_in_love(&self) -> bool {
        !self.get_mob_entity().is_in_love() && !self.has_egg()
    }

    fn spawns_offspring(&self) -> bool {
        false
    }

    fn on_bred_without_offspring(&self, _brain: Option<&mut crate::entity::ai::brain::BrainState>) {
        self.set_has_egg(true);
    }

    fn shows_breeding_hearts(&self) -> bool {
        false
    }
}

impl CustomSound for TurtleEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityTurtleDeathBaby
        } else {
            Sound::EntityTurtleDeath
        })
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(if self.is_baby() {
            Sound::EntityTurtleHurtBaby
        } else {
            Sound::EntityTurtleHurt
        })
    }
}

impl Mob for TurtleEntity {
    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        let sound = if self.is_baby() {
            Sound::EntityTurtleShambleBaby
        } else {
            Sound::EntityTurtleShamble
        };
        self.play_entity_sound(sound, 0.15, 1.0);
    }

    fn mob_next_step(&self) -> f32 {
        self.get_entity().move_dist.load() + STEP_INTERVAL
    }

    fn mob_play_swim_sound(&self, volume: f32) {
        self.play_default_swim_sound(volume * 1.5);
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        let entity = self.get_entity();
        (!entity.is_in_water() && entity.on_ground.load(Ordering::Relaxed) && !self.is_baby())
            .then_some(Sound::EntityTurtleAmbientLand)
    }

    fn get_ambient_sound_interval(&self) -> i32 {
        AMBIENT_SOUND_INTERVAL
    }

    fn mob_is_pushed_by_fluids(&self) -> bool {
        false
    }

    fn can_be_leashed(&self) -> bool {
        false
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn as_animal(&self) -> Option<&dyn Animal> {
        Some(self)
    }

    fn get_walk_target_value(&self, pos: &BlockPos) -> f32 {
        let world = self.get_entity().world.load();
        if !self.is_going_home() && world.get_fluid(pos).has_tag(&tag::Fluid::MINECRAFT_WATER) {
            return 10.0;
        }
        if TurtleEggBlock::on_sand(world.as_ref(), pos) {
            10.0
        } else {
            world.get_pathfinding_cost_from_light_levels(pos)
        }
    }

    fn is_stable_destination(&self, world: &World, pos: &BlockPos) -> bool {
        if self.travel_pos.load().is_some() {
            world.get_block(pos) == &Block::WATER
        } else {
            !world.get_block_state(&pos.down()).is_air()
        }
    }

    fn travel_in_water(&self, caller: &dyn EntityBase, _gravity: f64, _falling: bool) {
        let living = &self.mob_entity.living_entity;
        let entity = &living.entity;
        entity.update_velocity_from_input(living.movement_input.load(), SWIM_SPEED);
        living.make_move(caller);
        entity.velocity.store(entity.velocity.load() * SWIM_DRAG);
        if self.mob_entity.get_target().is_none()
            && (!self.is_going_home() || !self.is_home_within(SINK_HOME_DISTANCE))
        {
            entity
                .velocity
                .store(entity.velocity.load().add_raw(0.0, -SINK_SPEED, 0.0));
        }
    }

    fn mob_ai_step(&self) {
        let entity = self.get_entity();
        let counter = self.lay_egg_counter.load(Ordering::Relaxed);
        if entity.is_alive()
            && self.is_laying_egg()
            && counter >= 1
            && counter % LAY_EGG_PARTICLE_INTERVAL == 0
        {
            let world = entity.world.load();
            let pos = entity.block_pos.load();
            if TurtleEggBlock::on_sand(world.as_ref(), &pos) {
                let below = world.get_block_state_id(&pos.down());
                world.sync_world_event(
                    WorldEvent::ParticlesAndSoundDestroyBlock,
                    pos,
                    i32::from(below.as_u16()),
                );
                entity.emit_own_game_event(self, GameEvent::EntityAction);
            }
        }
    }

    fn mob_on_lightning_strike(&self, caller: &dyn EntityBase, lightning: &LightningBoltEntity) {
        let cause = lightning.get_cause();
        self.mob_entity.living_entity.damage_with_context(
            caller,
            f32::MAX,
            DamageType::LIGHTNING_BOLT,
            None,
            Some(lightning),
            cause.as_deref().map(|player| player as &dyn EntityBase),
        );
    }

    fn finalize_spawn(
        &self,
        world: &Arc<World>,
        _reason: EntitySpawnReason,
        group_data: Option<SpawnGroupData>,
    ) -> Option<SpawnGroupData> {
        self.set_home_pos(self.get_entity().block_pos.load());
        self.default_finalize_spawn(world, group_data)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put("home_pos", block_pos_to_tag(self.get_home_pos()));
        nbt.put_bool("has_egg", self.has_egg());
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_home_pos(
            nbt.get("home_pos")
                .and_then(block_pos_from_tag)
                .unwrap_or_else(|| self.get_entity().block_pos.load()),
        );
        self.set_has_egg(nbt.get_bool("has_egg").unwrap_or(false));
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        if self.is_baby() {
            entity.set_synced_data(pumpkin_data::tracked_data::turtle::DATA_BABY_ID, true);
        }
        entity.set_synced_data(pumpkin_data::tracked_data::turtle::HAS_EGG, self.has_egg());
        entity.set_synced_data(
            pumpkin_data::tracked_data::turtle::LAYING_EGG,
            self.is_laying_egg(),
        );
    }

    fn mob_interact(&self, player: &Arc<Player>, item_stack: &mut ItemStack) -> bool {
        self.animal_interact(player, item_stack)
    }
}

/// Swims towards its path while pushing itself up out of the water.
#[derive(Default)]
struct TurtleMoveControl {
    inner: MoveControl,
}

impl TurtleMoveControl {
    const MIN_DISTANCE: f64 = 1.0E-5;

    fn update_speed(turtle: &TurtleEntity) {
        let mob_entity = &turtle.mob_entity;
        let living = &mob_entity.living_entity;
        let entity = &living.entity;
        if entity.is_in_water() {
            entity
                .velocity
                .store(entity.velocity.load().add_raw(0.0, 0.005, 0.0));
            if !turtle.is_home_within(16.0) {
                mob_entity.set_speed((living.get_speed() / 2.0).max(0.08));
            }
            if turtle.is_baby() {
                mob_entity.set_speed((living.get_speed() / 3.0).max(0.06));
            }
        } else if entity.on_ground.load(Ordering::Relaxed) {
            mob_entity.set_speed((living.get_speed() / 2.0).max(0.06));
        }
    }
}

impl Control for TurtleMoveControl {}

impl MoveControlTrait for TurtleMoveControl {
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(turtle) = as_turtle(mob) else {
            self.inner.tick(mob);
            return;
        };
        Self::update_speed(turtle);
        let mob_entity = &turtle.mob_entity;
        if self.inner.operation != Operation::MoveTo || mob.is_navigator_idle() {
            mob_entity.set_speed(0.0);
            return;
        }
        let living = &mob_entity.living_entity;
        let entity = &living.entity;
        let pos = entity.pos.load();
        let xd = self.inner.wanted_x - pos.x;
        let yd = self.inner.wanted_y - pos.y;
        let zd = self.inner.wanted_z - pos.z;
        let dd = (xd * xd + yd * yd + zd * zd).sqrt();
        if dd < Self::MIN_DISTANCE {
            mob_entity.set_speed(0.0);
            return;
        }
        let yd = yd / dd;
        let y_rot_d = (zd.atan2(xd).to_degrees() as f32) - 90.0;
        let yaw = MoveControl::rotlerp(entity.yaw.load(), y_rot_d, MoveControl::MAX_TURN);
        entity.yaw.store(yaw);
        entity.body_yaw.store(yaw);
        let target_speed = (self.inner.speed_modifier
            * living.get_attribute_value(&Attributes::MOVEMENT_SPEED))
            as f32;
        let speed = (target_speed - living.get_speed()).mul_add(0.125, living.get_speed());
        mob_entity.set_speed(speed);
        entity.velocity.store(entity.velocity.load().add_raw(
            0.0,
            f64::from(speed) * yd * 0.1,
            0.0,
        ));
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

/// Breeds only without an egg already on the way.
struct TurtleBreedGoal {
    breed: Box<BreedGoal>,
}

impl Goal for TurtleBreedGoal {
    fn tick_reach(&self, mob: &dyn Mob, running: bool) -> Option<GoalReach> {
        self.breed.tick_reach(mob, running)
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.breed.can_start(mob) && !as_turtle(mob).is_some_and(TurtleEntity::has_egg)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.breed.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.breed.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.breed.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.breed.tick(mob);
    }

    fn controls(&self) -> Controls {
        self.breed.controls()
    }
}

struct WaterTarget;

static WATER_TARGET: WaterTarget = WaterTarget;

impl MoveToTargetPos for WaterTarget {
    fn is_target_pos(&self, world: &World, block_pos: BlockPos) -> bool {
        world.get_block(&block_pos) == &Block::WATER
    }

    fn should_recalculate_path(&self, trying_time: i32) -> bool {
        trying_time % GO_TO_WATER_RECALCULATE_TICKS == 0
    }
}

/// Walks back to water, babies always, adults unless heading home or carrying an egg.
struct TurtleGoToWaterGoal {
    move_to_target_pos_goal: MoveToTargetPosGoal<WaterTarget>,
}

impl TurtleGoToWaterGoal {
    fn new(speed: f64) -> Self {
        let mut move_to_target_pos_goal =
            MoveToTargetPosGoal::new(ParentHandle::from_static(&WATER_TARGET), speed, 24, 1);
        move_to_target_pos_goal.lowest_y = -1;
        Self {
            move_to_target_pos_goal,
        }
    }
}

impl Goal for TurtleGoToWaterGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(turtle) = as_turtle(mob) else {
            return false;
        };
        let in_water = turtle.get_entity().is_in_water();
        if turtle.is_baby() && !in_water {
            return self.move_to_target_pos_goal.can_start(mob);
        }
        !turtle.is_going_home()
            && !in_water
            && !turtle.has_egg()
            && self.move_to_target_pos_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let world = mob.get_entity().world.load();
        !mob.get_entity().is_in_water()
            && self.move_to_target_pos_goal.trying_time <= GO_TO_WATER_GIVE_UP_TICKS
            && WATER_TARGET.is_target_pos(&world, self.move_to_target_pos_goal.target_pos)
    }

    fn start(&mut self, mob: &dyn Mob) {
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

struct SandTarget;

static SAND_TARGET: SandTarget = SandTarget;

impl MoveToTargetPos for SandTarget {
    fn is_target_pos(&self, world: &World, block_pos: BlockPos) -> bool {
        world.get_block_state(&block_pos.up()).is_air()
            && TurtleEggBlock::is_sand(world, &block_pos)
    }
}

/// Near home with an egg, digs into sand and lays one to four eggs on top of it.
struct TurtleLayEggGoal {
    move_to_target_pos_goal: MoveToTargetPosGoal<SandTarget>,
}

impl TurtleLayEggGoal {
    fn new(speed: f64) -> Self {
        Self {
            move_to_target_pos_goal: MoveToTargetPosGoal::with_default(
                ParentHandle::from_static(&SAND_TARGET),
                speed,
                16,
            ),
        }
    }

    fn lay_eggs(&self, turtle: &TurtleEntity) {
        let entity = turtle.get_entity();
        let world = entity.world.load();
        let turtle_pos = entity.block_pos.load();
        let pitch = rand::random::<f32>().mul_add(0.2, 0.9);
        world.play_sound_raw(
            Sound::EntityTurtleLayEgg as u16,
            SoundCategory::Blocks,
            &turtle_pos.to_centered_f64(),
            LAY_EGG_SOUND_VOLUME,
            pitch,
        );
        let egg_pos = self.move_to_target_pos_goal.target_pos.up();
        let mut properties = TurtleEggLikeProperties::default(&Block::TURTLE_EGG);
        properties.eggs = turtle
            .get_random()
            .random_range(TurtleEggBlock::MIN_EGGS..=TurtleEggBlock::MAX_EGGS);
        let egg_state = properties.to_state_id(&Block::TURTLE_EGG);
        world.set_block_state(&egg_pos, egg_state, BlockFlags::NOTIFY_ALL);
        let source: &dyn EntityBase = turtle;
        world.emit_game_event_with_context(
            GameEvent::BlockPlace.name(),
            egg_pos.to_centered_f64(),
            &GameEventContext {
                source_entity: Some(source),
            },
        );
        turtle.set_has_egg(false);
        turtle.set_laying_egg(false);
        turtle
            .mob_entity
            .set_love_ticks(IN_LOVE_TICKS_AFTER_LAYING, turtle.mob_entity.breeder.load());
    }
}

impl Goal for TurtleLayEggGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        as_turtle(mob)
            .is_some_and(|turtle| turtle.has_egg() && turtle.is_home_within(LAY_EGG_HOME_DISTANCE))
            && self.move_to_target_pos_goal.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.move_to_target_pos_goal.should_continue(mob)
            && as_turtle(mob).is_some_and(|turtle| {
                turtle.has_egg() && turtle.is_home_within(LAY_EGG_HOME_DISTANCE)
            })
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.move_to_target_pos_goal.tick(mob);
        let Some(turtle) = as_turtle(mob) else {
            return;
        };
        if turtle.get_entity().is_in_water() || !self.move_to_target_pos_goal.is_reached_target() {
            return;
        }
        let counter = turtle.lay_egg_counter.load(Ordering::Relaxed);
        if counter < 1 {
            turtle.set_laying_egg(true);
        } else if counter > self.get_tick_count(LAY_EGG_TICKS) {
            self.lay_eggs(turtle);
        }
        if turtle.is_laying_egg() {
            turtle.lay_egg_counter.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        self.move_to_target_pos_goal.should_run_every_tick()
    }

    fn controls(&self) -> Controls {
        self.move_to_target_pos_goal.controls()
    }
}

/// Swims back to the beach it hatched on, always with an egg, now and then when far away.
struct TurtleGoHomeGoal {
    speed_modifier: f64,
    stuck: bool,
    close_to_home_try_ticks: i32,
}

impl TurtleGoHomeGoal {
    const fn new(speed_modifier: f64) -> Self {
        Self {
            speed_modifier,
            stuck: false,
            close_to_home_try_ticks: 0,
        }
    }
}

impl Goal for TurtleGoHomeGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(turtle) = as_turtle(mob) else {
            return false;
        };
        if turtle.is_baby() {
            return false;
        }
        if turtle.has_egg() {
            return true;
        }
        mob.get_random()
            .random_range(0..to_goal_ticks(GO_HOME_CHANCE))
            == 0
            && !turtle.is_home_within(64.0)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        as_turtle(mob).is_some_and(|turtle| !turtle.is_home_within(7.0))
            && !self.stuck
            && self.close_to_home_try_ticks <= self.get_tick_count(GO_HOME_GIVE_UP_TICKS)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(turtle) = as_turtle(mob) {
            turtle.going_home.store(true, Ordering::Relaxed);
        }
        self.stuck = false;
        self.close_to_home_try_ticks = 0;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(turtle) = as_turtle(mob) {
            turtle.going_home.store(false, Ordering::Relaxed);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(turtle) = as_turtle(mob) else {
            return;
        };
        let home_pos = turtle.get_home_pos();
        let close_to_home = turtle.is_home_within(16.0);
        if close_to_home {
            self.close_to_home_try_ticks += 1;
        }
        if !mob.is_navigator_idle() {
            return;
        }
        let home = bottom_center(home_pos);
        let mut next_pos = default_random_pos::get_pos_towards(mob, 16, 3, home, PI / 10.0)
            .or_else(|| default_random_pos::get_pos_towards(mob, 8, 7, home, FRAC_PI_2));
        if let Some(pos) = next_pos
            && !close_to_home
            && turtle
                .get_entity()
                .world
                .load()
                .get_block(&BlockPos::floored_v(pos))
                != &Block::WATER
        {
            next_pos = default_random_pos::get_pos_towards(mob, 16, 5, home, FRAC_PI_2);
        }
        let Some(next_pos) = next_pos else {
            self.stuck = true;
            return;
        };
        mob.get_mob_entity()
            .steered()
            .navigator()
            .set_progress(NavigatorGoal {
                current_progress: mob.get_entity().pos.load(),
                destination: next_pos,
                speed: self.speed_modifier,
            });
    }
}

/// Wanders far across the ocean between random water destinations.
struct TurtleTravelGoal {
    speed_modifier: f64,
    stuck: bool,
}

impl TurtleTravelGoal {
    const fn new(speed_modifier: f64) -> Self {
        Self {
            speed_modifier,
            stuck: false,
        }
    }
}

impl Goal for TurtleTravelGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        as_turtle(mob).is_some_and(|turtle| {
            !turtle.is_going_home() && !turtle.has_egg() && turtle.get_entity().is_in_water()
        })
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.is_navigator_idle()
            && !self.stuck
            && as_turtle(mob).is_some_and(|turtle| {
                !turtle.is_going_home() && !turtle.mob_entity.is_in_love() && !turtle.has_egg()
            })
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(turtle) = as_turtle(mob) else {
            return;
        };
        let entity = turtle.get_entity();
        let pos = entity.pos.load();
        let mut rng = mob.get_random();
        let xt = rng.random_range(0..=2 * TRAVEL_XZ_DIST) - TRAVEL_XZ_DIST;
        let mut yt = rng.random_range(0..=2 * TRAVEL_Y_DIST) - TRAVEL_Y_DIST;
        let zt = rng.random_range(0..=2 * TRAVEL_XZ_DIST) - TRAVEL_XZ_DIST;
        if f64::from(yt) + pos.y > f64::from(entity.world.load().sea_level - 1) {
            yt = 0;
        }
        turtle.travel_pos.store(Some(BlockPos::floored(
            f64::from(xt) + pos.x,
            f64::from(yt) + pos.y,
            f64::from(zt) + pos.z,
        )));
        self.stuck = false;
    }

    fn stop(&mut self, mob: &dyn Mob) {
        if let Some(turtle) = as_turtle(mob) {
            turtle.travel_pos.store(None);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(turtle) = as_turtle(mob) else {
            return;
        };
        let Some(travel_pos) = turtle.travel_pos.load() else {
            self.stuck = true;
            return;
        };
        if !mob.is_navigator_idle() {
            return;
        }
        let target = bottom_center(travel_pos);
        let world = turtle.get_entity().world.load();
        let next_pos = default_random_pos::get_pos_towards(mob, 16, 3, target, PI / 10.0)
            .or_else(|| default_random_pos::get_pos_towards(mob, 8, 7, target, FRAC_PI_2))
            .filter(|pos| {
                let x = pos.x.floor() as i32;
                let z = pos.z.floor() as i32;
                world.has_chunks_at(
                    x - TRAVEL_LOADED_RADIUS,
                    z - TRAVEL_LOADED_RADIUS,
                    x + TRAVEL_LOADED_RADIUS,
                    z + TRAVEL_LOADED_RADIUS,
                )
            });
        let Some(next_pos) = next_pos else {
            self.stuck = true;
            return;
        };
        mob.get_mob_entity()
            .steered()
            .navigator()
            .set_progress(NavigatorGoal {
                current_progress: mob.get_entity().pos.load(),
                destination: next_pos,
                speed: self.speed_modifier,
            });
    }
}
