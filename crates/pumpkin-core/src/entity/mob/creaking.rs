mod ai;
mod combat;
mod gaze;
mod home;

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicI32, Ordering::Relaxed},
};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::{
    BlockState, damage::DamageType, sound::Sound, tracked_data::creaking as tracked_data,
};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::{position::BlockPos, vector3::Vector3};

use crate::{
    entity::{
        Entity, EntityBase,
        ai::{
            brain::{
                Brain,
                memory::{
                    PackedMemories,
                    codec::{block_pos_from_tag, block_pos_to_tag},
                },
            },
            pathfinder::node::PathType,
        },
        custom_sound::CustomSound,
        mob::{Mob, MobEntity},
    },
    world::World,
};

const ATTACK_ANIMATION_DURATION: i32 = 15;
const INVULNERABILITY_ANIMATION_DURATION: i32 = 8;
const TWITCH_DEATH_DURATION: u8 = 45;
const MAX_DISTANCE_TO_HOME_SQ: f64 = 1024.0;
const BRIGHTNESS_FOR_FASTER_DESPAWN: f32 = 0.5;

pub struct CreakingEntity {
    pub mob_entity: MobEntity,
    can_move: AtomicBool,
    is_active: AtomicBool,
    is_tearing_down: AtomicBool,
    home_pos: AtomicCell<Option<BlockPos>>,
    home_chunk_ticket: Mutex<Option<home::HeartChunkTicket>>,
    pending_home_feedback: AtomicCell<Option<(BlockPos, bool)>>,
    attack_animation_remaining_ticks: AtomicI32,
    invulnerability_animation_remaining_ticks: AtomicI32,
    player_stuck_counter: AtomicI32,
}

impl CreakingEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mut mob_entity = MobEntity::new(entity);
        mob_entity
            .navigator
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_can_float(true);
        Arc::new(Self {
            mob_entity,
            can_move: AtomicBool::new(true),
            is_active: AtomicBool::new(false),
            is_tearing_down: AtomicBool::new(false),
            home_pos: AtomicCell::new(None),
            home_chunk_ticket: Mutex::new(None),
            pending_home_feedback: AtomicCell::new(None),
            attack_animation_remaining_ticks: AtomicI32::new(0),
            invulnerability_animation_remaining_ticks: AtomicI32::new(0),
            player_stuck_counter: AtomicI32::new(0),
        })
    }

    pub fn set_transient(&self, pos: BlockPos) {
        let mut ticket = self
            .home_chunk_ticket
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.home_pos.swap(Some(pos)) != Some(pos) {
            *ticket = None;
            self.pending_home_feedback.store(None);
        }
        drop(ticket);
        let entity = self.get_entity();
        entity.fire_immune.store(true, Relaxed);
        entity.set_synced_data(tracked_data::HOME_POS, Some(pos));

        let steered = self.mob_entity.steered();
        let mut navigator = steered.navigator();
        navigator.set_home_distance_limit(pos, MAX_DISTANCE_TO_HOME_SQ);
        navigator.set_pathfinding_malus(PathType::DamageOther, 8.0);
        navigator.set_pathfinding_malus(PathType::PowderSnow, 8.0);
        navigator.set_pathfinding_malus(PathType::Lava, 8.0);
        navigator.set_pathfinding_malus(PathType::DamageFire, 0.0);
        navigator.set_pathfinding_malus(PathType::DangerFire, 0.0);
    }

    pub fn is_heart_bound(&self) -> bool {
        self.get_home_pos().is_some()
    }

    pub fn can_move(&self) -> bool {
        self.can_move.load(Relaxed)
    }

    fn set_can_move(&self, can_move: bool) {
        self.can_move.store(can_move, Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::CAN_MOVE, can_move);
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Relaxed)
    }

    fn set_is_active(&self, active: bool) {
        self.is_active.store(active, Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::IS_ACTIVE, active);
    }

    pub fn is_tearing_down(&self) -> bool {
        self.is_tearing_down.load(Relaxed)
    }

    fn set_tearing_down(&self) {
        self.is_tearing_down.store(true, Relaxed);
        self.get_entity()
            .set_synced_data(tracked_data::IS_TEARING_DOWN, true);
    }

    pub fn get_home_pos(&self) -> Option<BlockPos> {
        self.home_pos.load()
    }

    fn stop_in_place(&self) {
        self.mob_entity.steered().navigator().stop();
        let living = &self.mob_entity.living_entity;
        living.movement_input.store(Vector3::default());
        living.set_speed(0.0);
        living.entity.velocity.store(Vector3::default());
        living.entity.leash_angular_momentum.store(0.0);
    }
}

impl Mob for CreakingEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn make_brain(&self, packed: &PackedMemories) -> Brain {
        ai::make_brain(self, packed)
    }

    fn custom_server_ai_step(&self, _caller: &dyn EntityBase) {
        ai::tick(self);
    }

    fn apply_mob_target(&self, target: Option<Arc<dyn EntityBase>>) {
        if !self.get_entity().is_removed() {
            // A native damage handler may call the setter during a Brain tick.
            self.mob_entity.set_target(target);
        }
    }

    fn tick_links(&self) -> Vec<i32> {
        ai::tick_links(self)
    }

    fn before_mob_ai_step(&self) {
        for timer in [
            &self.attack_animation_remaining_ticks,
            &self.invulnerability_animation_remaining_ticks,
        ] {
            if timer.load(Relaxed) > 0 {
                timer.fetch_sub(1, Relaxed);
            }
        }
        self.update_gaze();
        let entity = self.get_entity();
        if entity
            .world
            .load()
            .get_light_level_dependent_magic_value(&entity.get_eye_pos().to_block_pos())
            > BRIGHTNESS_FOR_FASTER_DESPAWN
        {
            self.mob_entity
                .living_entity
                .no_action_time
                .fetch_add(2, Relaxed);
        }
    }

    fn ticks_navigation(&self) -> bool {
        self.can_move()
    }

    fn ticks_move_control(&self) -> bool {
        self.can_move()
    }

    fn ticks_look_control(&self) -> bool {
        self.can_move()
    }

    fn ticks_jump_control(&self) -> bool {
        self.can_move()
    }

    fn ticks_body_rotation(&self) -> bool {
        self.can_move()
    }

    fn mob_can_use_portal(&self) -> bool {
        !self.is_heart_bound()
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        let entity = self.get_entity();
        !world.contains_any_liquid(entity.bounding_box.load()) && world.is_unobstructed(entity)
    }

    fn conservative_block_read_radius(&self) -> f64 {
        gaze::MAX_GAZE_RANGE
    }

    fn is_allowed_in_peaceful(&self) -> bool {
        false
    }

    fn can_be_leashed(&self) -> bool {
        false
    }

    fn requires_custom_persistence(&self) -> bool {
        let entity = self.get_entity();
        entity.has_vehicle() || entity.is_leashed()
    }

    fn mob_is_pushable(&self) -> bool {
        self.can_move() && self.mob_entity.living_entity.is_pushable()
    }

    fn mob_accepts_push_impulses(&self) -> bool {
        self.can_move()
    }

    fn mob_knockback(
        &self,
        _power: f64,
        _x: f64,
        _z: f64,
        _source: Option<&Entity>,
        _damage: f32,
        _comes_from_effect: bool,
    ) -> bool {
        !self.can_move()
    }

    fn mob_play_step_sound(&self, _pos: &BlockPos, _state: &BlockState) {
        self.play_entity_sound(Sound::EntityCreakingStep, 0.15, 1.0);
    }

    fn get_ambient_sound(&self) -> Option<Sound> {
        (!self.is_active()).then_some(Sound::EntityCreakingAmbient)
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        if let Some(home) = self.get_home_pos() {
            nbt.put("home_pos", block_pos_to_tag(home));
        }
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(home) = nbt.get("home_pos").and_then(block_pos_from_tag) {
            self.set_transient(home);
        } else if let Some(legacy) = nbt.get_compound("home_pos")
            && let (Some(x), Some(y), Some(z)) = (
                legacy.get_int("x"),
                legacy.get_int("y"),
                legacy.get_int("z"),
            )
        {
            self.mob_entity.clear_home();
            self.set_transient(BlockPos::new(x, y, z));
        }
    }

    fn prepare_mob_tick(&self) -> bool {
        self.check_home()
    }

    fn on_removed(&self) {
        self.pending_home_feedback.store(None);
        self.release_home_ticket();
    }

    fn mob_hurt_server(
        &self,
        amount: f32,
        damage_type: DamageType,
        position: Option<Vector3<f64>>,
        source: Option<&dyn EntityBase>,
        cause: Option<&dyn EntityBase>,
    ) -> Option<bool> {
        self.hurt_transient(amount, damage_type, position, source, cause)
    }

    fn do_hurt_target(&self, target: &dyn EntityBase) -> bool {
        self.attack_target(target)
    }

    fn mob_tick_death(&self, death_time: u8) -> bool {
        self.tick_teardown(death_time)
    }

    fn emits_entity_die_game_event(&self) -> bool {
        true
    }

    fn mob_death_animation_ticks(&self) -> u8 {
        if self.is_heart_bound() && self.is_tearing_down() {
            TWITCH_DEATH_DURATION + 1
        } else {
            20
        }
    }
}
