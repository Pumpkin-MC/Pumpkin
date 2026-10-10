use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::Block;
use pumpkin_data::block_properties::{
    CaveVinesLikeProperties, CaveVinesPlantLikeProperties, NetherWartLikeProperties,
};
use pumpkin_data::entity::EntityType;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::world::BlockFlags;
use rand::RngExt;

use crate::entity::ai::goal::escape_danger::EscapeDangerGoal;
use crate::entity::ai::goal::move_to_block::MoveToBlockGoal;
use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::mob::Mob;
use crate::entity::{Entity, EntityBase};
use crate::world::World;

use super::{FoxEntity, is_alertable, is_path_clear};

pub struct FoxPanicGoal {
    fox: Weak<FoxEntity>,
    inner: Box<EscapeDangerGoal>,
}

impl FoxPanicGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
        Self {
            fox,
            inner: EscapeDangerGoal::new(speed),
        }
    }
}

impl Goal for FoxPanicGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.fox.upgrade().is_some_and(|f| f.is_defending()) {
            return false;
        }
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        if self.fox.upgrade().is_some_and(|f| f.is_defending()) {
            return false;
        }
        self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.inner.tick(mob);
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}

pub struct FaceplantGoal {
    fox: Weak<FoxEntity>,
    countdown: i32,
}

impl FaceplantGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self { fox, countdown: 0 }
    }
}

impl Goal for FaceplantGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.fox.upgrade().is_some_and(|f| f.is_faceplanted())
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.countdown > 0
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.countdown = 40;
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_faceplanted(false);
            fox.get_entity().pitch.store(0.0);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.countdown -= 1;
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}

pub struct StalkPreyGoal {
    fox: Weak<FoxEntity>,
    update_countdown_ticks: i32,
}

impl StalkPreyGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            update_countdown_ticks: 0,
        }
    }
}

impl Goal for StalkPreyGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sleeping() || fox.is_crouching() || fox.is_interested() || fox.is_pouncing() {
            return false;
        }
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        if !target.is_alive() {
            return false;
        }
        let target_entity = target.get_entity();
        let target_type = target_entity.entity_type;
        if target_type != &EntityType::CHICKEN && target_type != &EntityType::RABBIT {
            return false;
        }
        let fox_pos = fox.get_entity().pos.load();
        let target_pos = target_entity.pos.load();
        fox_pos.squared_distance_to_vec(&target_pos) > 36.0
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.can_start(mob)
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.update_countdown_ticks = 0;
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(false);
            fox.set_faceplanted(false);
        }
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let target = mob.get_mob_entity().get_target();
        if let Some(target) = target {
            let world = fox.get_entity().world.load();
            let fox_pos = fox.get_entity().pos.load();
            let target_pos = target.get_entity().pos.load();
            if is_path_clear(fox_pos, target_pos, &world) {
                fox.set_interested(true);
                fox.set_crouching(true);
                let living = &mob.get_mob_entity().living_entity;
                living.set_speed(0.0);
                living.movement_input.store(Vector3::default());
                let mut vel = living.entity.velocity.load();
                vel.x = 0.0;
                vel.z = 0.0;
                living.entity.velocity.store(vel);
                mob.stop_navigation();
                let mut look = mob
                    .get_mob_entity()
                    .look_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                look.look_at_entity(mob, &target);
                return;
            }
        }
        fox.set_interested(false);
        fox.set_crouching(false);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };

        let mut look = mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_entity(mob, &target);

        let fox_pos = fox.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = fox_pos.squared_distance_to_vec(&target_pos);

        if dist_sq <= 36.0 {
            fox.set_interested(true);
            fox.set_crouching(true);
            let living = &mob.get_mob_entity().living_entity;
            living.set_speed(0.0);
            living.movement_input.store(Vector3::default());
            let mut vel = living.entity.velocity.load();
            vel.x = 0.0;
            vel.z = 0.0;
            living.entity.velocity.store(vel);
            mob.stop_navigation();
        } else {
            self.update_countdown_ticks = (self.update_countdown_ticks - 1).max(0);
            if self.update_countdown_ticks <= 0 {
                self.update_countdown_ticks = 4 + mob.get_random().random_range(0..7);
                mob.navigate_to(fox_pos, target_pos, 1.5);
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct FoxPounceGoal {
    fox: Weak<FoxEntity>,
    pounce_ticks: i32,
}

impl FoxPounceGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            pounce_ticks: 0,
        }
    }
}

impl Goal for FoxPounceGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if !fox.is_fully_crouched() {
            return false;
        }

        let Some(target) = mob.get_mob_entity().get_target() else {
            fox.set_crouching(false);
            fox.set_interested(false);
            return false;
        };
        if !target.is_alive() {
            fox.set_crouching(false);
            fox.set_interested(false);
            return false;
        }

        let world = fox.get_entity().world.load();
        let fox_pos = fox.get_entity().pos.load();
        let target_pos = target.get_entity().pos.load();
        let has_clear_path = is_path_clear(fox_pos, target_pos, &world);
        if !has_clear_path {
            fox.set_crouching(false);
            fox.set_interested(false);
            mob.navigate_to(fox_pos, target_pos, 1.5);
            return false;
        }
        true
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        let entity = fox.get_entity();
        if !target.is_alive() || fox.is_faceplanted() {
            return false;
        }
        let vel = entity.velocity.load();
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        let pitch = entity.pitch.load();
        (vel.y * vel.y >= 0.05 || pitch.abs() >= 15.0 || !on_ground) && !fox.is_faceplanted()
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.pounce_ticks = 0;
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let living = &fox.mob_entity.living_entity;
        living.jumping.store(true, Ordering::SeqCst);
        living.set_speed(0.0);
        living.movement_input.store(Vector3::default());
        fox.set_pouncing(true);
        fox.set_interested(false);

        let entity = fox.get_entity();

        if let Some(target) = mob.get_mob_entity().get_target() {
            let fox_pos = entity.pos.load();
            let target_pos = target.get_entity().pos.load();
            let diff = target_pos.sub(&fox_pos);
            let dist_3d = (diff.x * diff.x + diff.y * diff.y + diff.z * diff.z).sqrt();
            let (uv_x, uv_z) = if dist_3d > 1e-5 {
                (diff.x / dist_3d, diff.z / dist_3d)
            } else {
                (0.0, 0.0)
            };

            let jump_yaw = (uv_z.atan2(uv_x).to_degrees() as f32) - 90.0;
            entity.yaw.store(jump_yaw);
            entity.head_yaw.store(jump_yaw);
            entity.body_yaw.store(jump_yaw);

            entity.set_velocity(Vector3::new(uv_x * 0.8, 0.9, uv_z * 0.8));
        }

        mob.stop_navigation();
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.pounce_ticks = 0;
        if let Some(fox) = self.fox.upgrade() {
            fox.set_crouching(false);
            fox.crouch_amount.store(0.0);
            fox.set_interested(false);
            fox.set_pouncing(false);
            if !fox.is_faceplanted() {
                fox.get_entity().pitch.store(0.0);
            }
            fox.mob_entity
                .living_entity
                .jumping
                .store(false, Ordering::SeqCst);
            fox.mob_entity
                .living_entity
                .movement_input
                .store(Vector3::default());
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.pounce_ticks += 1;
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let entity = fox.get_entity();
        let living = &fox.mob_entity.living_entity;
        living.set_speed(0.0);
        living.movement_input.store(Vector3::default());

        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };

        let mut look = mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_entity(mob, &target);

        let vel = entity.velocity.load();
        let on_ground = entity.on_ground.load(Ordering::Relaxed);
        if !fox.is_faceplanted() {
            let cur_pitch = entity.pitch.load();
            if (vel.y * vel.y < 0.03 || on_ground) && cur_pitch != 0.0 {
                let new_pitch = cur_pitch + (0.0 - cur_pitch) * 0.2;
                entity.pitch.store(new_pitch);
            } else {
                let h_dist = vel.x.hypot(vel.z);
                let upwards_bias = if living.jumping.load(Ordering::Relaxed) && vel.y > 0.0 {
                    6.5
                } else {
                    1.0
                };
                let biased_y = vel.y * upwards_bias;
                let len = h_dist.hypot(biased_y);
                if len > 1e-5 {
                    let rotation =
                        (-biased_y).signum() * (h_dist / len).clamp(-1.0, 1.0).acos().to_degrees();
                    entity.pitch.store(rotation as f32);
                }
            }
        }

        let fox_pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let dx = target_pos.x - fox_pos.x;
        let dy = target_pos.y - fox_pos.y;
        let dz = target_pos.z - fox_pos.z;
        let dist_3d_sq = dx * dx + dy * dy + dz * dz;
        if dist_3d_sq <= 4.0 {
            mob.get_mob_entity()
                .try_attack(mob.get_entity(), target.as_ref());
            let mut cur_vel = entity.velocity.load();
            cur_vel.x *= 0.6;
            cur_vel.z *= 0.6;
            entity.velocity.store(cur_vel);
            let world = entity.world.load();
            world.play_sound(
                pumpkin_data::sound::Sound::EntityFoxBite,
                pumpkin_data::sound::SoundCategory::Neutral,
                &entity.pos.load(),
            );
            if on_ground {
                entity.pitch.store(0.0);
            }
        } else if entity.pitch.load() > 0.0 && entity.on_ground.load(Ordering::Relaxed) {
            let world = entity.world.load();
            let block = world
                .get_block_state(&entity.block_pos.load())
                .id
                .to_block();
            if block.id == Block::SNOW.id {
                entity.pitch.store(60.0);
                fox.set_faceplanted(true);
                mob.get_mob_entity().set_target(None);
            } else {
                entity.pitch.store(0.0);
            }
        }
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::JUMP
    }
}

pub struct SleepGoal {
    fox: Weak<FoxEntity>,
    countdown: i32,
    alertable_timer: i32,
}

impl SleepGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            countdown: rand::rng().random_range(0..140),
            alertable_timer: 0,
        }
    }

    fn can_sleep(&mut self, fox: &FoxEntity, world: &World) -> bool {
        if self.countdown > 0 {
            self.countdown -= 1;
            return false;
        }
        let pos = fox.get_entity().block_pos.load();
        if !world.is_bright_outside() || world.can_see_sky(&pos) {
            return false;
        }
        let block_here = world.get_block_state(&pos).id.to_block();
        if block_here.id == Block::POWDER_SNOW.id {
            return false;
        }
        self.alertable_timer -= 1;
        if self.alertable_timer <= 0 {
            self.alertable_timer = 20;
            if is_alertable(fox, world) {
                return false;
            }
        }
        true
    }
}

impl Goal for SleepGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        let entity = fox.get_entity();
        if entity.is_in_water() {
            return false;
        }
        let world = entity.world.load();

        let input = fox.mob_entity.living_entity.movement_input.load();
        if input.x.abs() > 1e-4 || input.z.abs() > 1e-4 {
            return false;
        }

        self.can_sleep(&fox, &world) || fox.is_sleeping()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.get_entity().is_in_water() {
            return false;
        }
        let world = fox.get_entity().world.load();
        self.can_sleep(&fox, &world)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(false);
            fox.set_crouching(false);
            fox.set_interested(false);
            fox.mob_entity
                .living_entity
                .jumping
                .store(false, Ordering::SeqCst);
            fox.set_sleeping(true);
            mob.stop_navigation();
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.countdown = rand::rng().random_range(0..140);
        self.alertable_timer = 0;
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP
    }
}

pub struct PerchAndSearchGoal {
    fox: Weak<FoxEntity>,
    rel_x: f64,
    rel_z: f64,
    look_time: i32,
    looks_remaining: i32,
}

impl PerchAndSearchGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            rel_x: 0.0,
            rel_z: 0.0,
            look_time: 0,
            looks_remaining: 0,
        }
    }

    fn reset_look(&mut self) {
        let rnd = 2.0 * std::f64::consts::PI * rand::rng().random::<f64>();
        self.rel_x = rnd.cos();
        self.rel_z = rnd.sin();
        self.look_time = 80 + rand::rng().random_range(0..20);
    }
}

impl Goal for PerchAndSearchGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sleeping()
            || fox.is_pouncing()
            || fox.is_crouching()
            || fox.get_entity().is_in_water()
            || mob.get_mob_entity().get_target().is_some()
            || mob
                .get_mob_entity()
                .living_entity
                .last_attacker_id
                .load(Ordering::Relaxed)
                != 0
            || !mob.is_navigator_idle()
            || rand::random::<f32>() >= 0.02
        {
            return false;
        }

        let world = fox.get_entity().world.load();
        !is_alertable(&fox, &world)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        self.looks_remaining > 0
            && !fox.get_entity().is_in_water()
            && !fox.is_sleeping()
            && mob.get_mob_entity().get_target().is_none()
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.reset_look();
        self.looks_remaining = 2 + rand::rng().random_range(0..3);
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(true);
        }
        mob.stop_navigation();
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_sitting(false);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.look_time -= 1;
        if self.look_time <= 0 {
            self.looks_remaining -= 1;
            self.reset_look();
        }

        let entity = mob.get_entity();
        let pos = entity.pos.load();
        let eye_y = entity.get_eye_pos().y;
        let look_target = Vector3::new(pos.x + self.rel_x * 5.0, eye_y, pos.z + self.rel_z * 5.0);

        let mut look = mob
            .get_mob_entity()
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        look.look_at_position(mob, look_target);
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

pub struct FoxSearchForItemsGoal {
    fox: Weak<FoxEntity>,
    target_item: Option<Arc<dyn EntityBase>>,
    update_countdown_ticks: i32,
}

impl FoxSearchForItemsGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            target_item: None,
            update_countdown_ticks: 0,
        }
    }
}

impl Goal for FoxSearchForItemsGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        let mouth_item = fox.get_mouth_item();
        if !mouth_item.is_empty() {
            return false;
        }
        if fox.mob_entity.get_target().is_some() || !fox.can_move() {
            return false;
        }
        if rand::rng().random_range(0..10) != 0 {
            return false;
        }

        let entity = fox.get_entity();
        let world = entity.world.load();
        let pos = entity.pos.load();
        let entities = world.entities.load();
        let mut closest: Option<(f64, Arc<dyn EntityBase>)> = None;

        for candidate in entities.iter() {
            let Some(item) = candidate.get_item_entity() else {
                continue;
            };
            if item.get_pickup_delay() > 0 || !item.get_entity().is_alive() {
                continue;
            }
            let stack = item
                .get_item_stack()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if !fox.can_hold_item(&stack) {
                continue;
            }
            let dist_sq = pos.squared_distance_to_vec(&item.get_entity().pos.load());
            if dist_sq <= 64.0 {
                match &closest {
                    Some((best, _)) if dist_sq >= *best => {}
                    _ => closest = Some((dist_sq, candidate.clone())),
                }
            }
        }

        self.target_item = closest.map(|(_, item)| item);
        self.target_item.is_some()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        let mouth_item = fox.get_mouth_item();
        if !mouth_item.is_empty() || !fox.can_move() {
            return false;
        }
        self.target_item.as_ref().is_some_and(|item| {
            if !item.get_entity().is_alive() {
                return false;
            }
            item.get_item_entity().is_some_and(|item_ent| {
                let stack = item_ent
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                fox.can_hold_item(&stack)
            })
        })
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.update_countdown_ticks = 0;
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        if let Some(target) = &self.target_item {
            let fox_pos = mob.get_entity().pos.load();
            let item_pos = target.get_entity().pos.load();
            mob.navigate_to(fox_pos, item_pos, 1.2);
            self.update_countdown_ticks = 10;
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(fox) = self.fox.upgrade() else {
            return;
        };
        let Some(target) = &self.target_item else {
            return;
        };

        let fox_pos = mob.get_entity().pos.load();
        let item_pos = target.get_entity().pos.load();
        let dist = fox_pos.squared_distance_to_vec(&item_pos).sqrt();

        if dist <= 1.5 {
            if let Some(item_ent) = target.get_item_entity() {
                let stack = item_ent
                    .get_item_stack()
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .clone();
                if !stack.is_empty() && fox.can_hold_item(&stack) {
                    let world = mob.get_entity().world.load_full();
                    let prev_item = fox.get_mouth_item();
                    if !prev_item.is_empty() {
                        fox.spit_out_item(&prev_item);
                    }
                    if stack.item_count > 1 {
                        let mut remainder = stack.clone();
                        remainder.item_count = remainder.item_count.saturating_sub(1);
                        let item_entity = crate::entity::item::ItemEntity::new(
                            Entity::new(world.clone(), fox_pos, &EntityType::ITEM),
                            remainder,
                        );
                        world.spawn_entity(Arc::new(item_entity));
                    }
                    let mut single = stack;
                    single.item_count = 1;
                    fox.set_mouth_item(single);
                    fox.ticks_since_eaten.store(0, Ordering::Relaxed);
                    target.get_entity().remove();
                }
            }
            self.target_item = None;
        } else {
            self.update_countdown_ticks = (self.update_countdown_ticks - 1).max(0);
            if mob.is_navigator_idle() || self.update_countdown_ticks <= 0 {
                self.update_countdown_ticks = 10;
                mob.navigate_to(fox_pos, item_pos, 1.2);
            }
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.target_item = None;
        self.update_countdown_ticks = 0;
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

pub struct FoxEatBerriesGoal {
    fox: Weak<FoxEntity>,
    move_to_block: MoveToBlockGoal,
    ticks_waited: i32,
}

impl FoxEatBerriesGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>, speed: f64) -> Self {
        let move_to_block = MoveToBlockGoal::new(
            speed,
            12,
            1,
            Box::new(|world, pos| {
                let state = world.get_block_state(&pos);
                let block_id = state.id.to_block().id;
                if block_id == Block::SWEET_BERRY_BUSH.id {
                    let props =
                        NetherWartLikeProperties::from_state_id(world.get_block_state_id(&pos));
                    props.age >= 2
                } else if block_id == Block::CAVE_VINES.id {
                    CaveVinesLikeProperties::from_state_id(world.get_block_state_id(&pos)).berries
                } else if block_id == Block::CAVE_VINES_PLANT.id {
                    CaveVinesPlantLikeProperties::from_state_id(world.get_block_state_id(&pos))
                        .berries
                } else {
                    false
                }
            }),
        )
        .with_accepted_distance(2.0)
        .with_recalculate_interval(100);

        Self {
            fox,
            move_to_block,
            ticks_waited: 0,
        }
    }

    fn harvest_berries(
        fox: &FoxEntity,
        world: &Arc<World>,
        pos: &BlockPos,
        target_vec: &Vector3<f64>,
    ) {
        let state_id = world.get_block_state_id(pos);
        let block = Block::from_state_id(state_id);
        if block.id == Block::SWEET_BERRY_BUSH.id {
            let mut props = NetherWartLikeProperties::from_state_id(state_id);
            if props.age >= 2 {
                let count: u8 = 1 + rand::rng().random_range(0..=1) + u8::from(props.age == 3);
                let mut remaining: u8 = count;

                if fox.get_mouth_item().is_empty() {
                    fox.set_mouth_item(ItemStack::new(1, &Item::SWEET_BERRIES));
                    fox.ticks_since_eaten.store(0, Ordering::Relaxed);
                    remaining = remaining.saturating_sub(1);
                }

                if remaining > 0 {
                    world.drop_stack(pos, ItemStack::new(remaining, &Item::SWEET_BERRIES));
                }

                props.age = 1;
                world.set_block_state(
                    pos,
                    props.to_state_id(&Block::SWEET_BERRY_BUSH),
                    BlockFlags::NOTIFY_ALL,
                );
            }

            world.play_sound(
                Sound::BlockSweetBerryBushPickBerries,
                SoundCategory::Blocks,
                target_vec,
            );
        } else if block.id == Block::CAVE_VINES.id {
            let mut props = CaveVinesLikeProperties::from_state_id(state_id);
            if props.berries {
                props.berries = false;
                world.set_block_state(
                    pos,
                    props.to_state_id(&Block::CAVE_VINES),
                    BlockFlags::NOTIFY_ALL,
                );
                world.drop_stack(pos, ItemStack::new(1, &Item::GLOW_BERRIES));
            }

            world.play_sound(
                Sound::BlockCaveVinesPickBerries,
                SoundCategory::Blocks,
                target_vec,
            );
        } else if block.id == Block::CAVE_VINES_PLANT.id {
            let mut props = CaveVinesPlantLikeProperties::from_state_id(state_id);
            if props.berries {
                props.berries = false;
                world.set_block_state(
                    pos,
                    props.to_state_id(&Block::CAVE_VINES_PLANT),
                    BlockFlags::NOTIFY_ALL,
                );
                world.drop_stack(pos, ItemStack::new(1, &Item::GLOW_BERRIES));
            }

            world.play_sound(
                Sound::BlockCaveVinesPickBerries,
                SoundCategory::Blocks,
                target_vec,
            );
        }
    }
}

impl Goal for FoxEatBerriesGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sleeping() {
            return false;
        }
        self.move_to_block.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.ticks_waited <= 120 && self.move_to_block.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
            fox.set_sitting(false);
        }
        self.ticks_waited = 0;
        self.move_to_block.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.ticks_waited = 0;
        self.move_to_block.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        self.move_to_block.tick(mob);
        let Some(fox) = self.fox.upgrade() else {
            return;
        };

        if self.move_to_block.is_reached_target() {
            self.ticks_waited += 1;
            if self.ticks_waited >= 40 {
                let target_pos = self.move_to_block.target_pos;
                let world = mob.get_entity().world.load_full();
                if world.level_info.load().game_rules.mob_griefing {
                    let target_vec = Vector3::new(
                        f64::from(target_pos.0.x) + 0.5,
                        f64::from(target_pos.0.y),
                        f64::from(target_pos.0.z) + 0.5,
                    );
                    Self::harvest_berries(&fox, &world, &target_pos, &target_vec);
                }
                self.move_to_block.target_pos = BlockPos::ZERO;
                self.move_to_block.reached_target = false;
            }
        } else if rand::random::<f32>() < 0.05 {
            let fox_pos = mob.get_entity().pos.load();
            mob.get_entity().world.load().play_sound(
                Sound::EntityFoxSniff,
                SoundCategory::Neutral,
                &fox_pos,
            );
        }
    }

    fn controls(&self) -> Controls {
        self.move_to_block.controls()
    }
}

pub struct DefendTrustedTargetGoal {
    fox: Weak<FoxEntity>,
    target_attacker: Option<Arc<dyn EntityBase>>,
    last_timestamp: i32,
    pending_timestamp: i32,
}

impl DefendTrustedTargetGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>) -> Self {
        Self {
            fox,
            target_attacker: None,
            last_timestamp: 0,
            pending_timestamp: 0,
        }
    }
}

impl Goal for DefendTrustedTargetGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        let world = fox.get_entity().world.load();
        let fox_pos = fox.get_entity().pos.load();
        let max_dist_sq = 32.0 * 32.0;

        let trusted_uuids = [fox.trusted_0.load(), fox.trusted_1.load()];
        for opt_uuid in trusted_uuids {
            let Some(uuid) = opt_uuid else {
                continue;
            };
            if let Some(player) = world.get_player_by_uuid(uuid) {
                let player_pos = player.get_entity().pos.load();
                if fox_pos.squared_distance_to_vec(&player_pos) > max_dist_sq {
                    continue;
                }
                let attacker_id = player
                    .living_entity
                    .last_attacker_id
                    .load(Ordering::Relaxed);
                let attack_time = player
                    .living_entity
                    .last_attacked_time
                    .load(Ordering::Relaxed);
                let current_age = player.living_entity.entity.age.load(Ordering::Relaxed);
                if attacker_id != 0
                    && attack_time > 0
                    && current_age - attack_time <= 600
                    && attack_time != self.last_timestamp
                    && let Some(attacker_ent) = world.get_entity_by_id(attacker_id)
                {
                    let att = attacker_ent.get_entity();
                    if attacker_ent.is_alive() && !fox.trusts(&att.entity_uuid) {
                        self.target_attacker = Some(attacker_ent);
                        self.pending_timestamp = attack_time;
                        return true;
                    }
                }
            } else {
                let entities = world.entities.load();
                for candidate in entities.iter() {
                    let c_ent = candidate.get_entity();
                    if c_ent.entity_uuid == uuid {
                        let c_pos = c_ent.pos.load();
                        if fox_pos.squared_distance_to_vec(&c_pos) > max_dist_sq {
                            break;
                        }
                        if let Some(living) = candidate.get_living_entity() {
                            let attacker_id = living.last_attacker_id.load(Ordering::Relaxed);
                            let attack_time = living.last_attacked_time.load(Ordering::Relaxed);
                            let current_age = c_ent.age.load(Ordering::Relaxed);
                            if attacker_id != 0
                                && attack_time > 0
                                && current_age - attack_time <= 600
                                && attack_time != self.last_timestamp
                                && let Some(attacker_ent) = world.get_entity_by_id(attacker_id)
                            {
                                let att = attacker_ent.get_entity();
                                if attacker_ent.is_alive() && !fox.trusts(&att.entity_uuid) {
                                    self.target_attacker = Some(attacker_ent);
                                    self.pending_timestamp = attack_time;
                                    return true;
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }
        false
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.target_attacker.as_ref().is_some_and(|t| t.is_alive())
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.last_timestamp = self.pending_timestamp;
        if let Some(target) = &self.target_attacker {
            mob.get_mob_entity().set_target(Some(target.clone()));
            if let Some(fox) = self.fox.upgrade() {
                fox.set_defending(true);
                fox.wake_up();
                let pos = mob.get_entity().pos.load();
                mob.get_entity().world.load().play_sound(
                    Sound::EntityFoxAggro,
                    SoundCategory::Neutral,
                    &pos,
                );
            }
        }
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_defending(false);
        }
        self.target_attacker = None;
    }

    fn controls(&self) -> Controls {
        Controls::TARGET
    }
}

pub struct FoxSeekShelterGoal {
    fox: Weak<FoxEntity>,
    speed_modifier: f64,
    wanted_x: f64,
    wanted_y: f64,
    wanted_z: f64,
    interval: i32,
}

impl FoxSeekShelterGoal {
    #[must_use]
    pub const fn new(fox: Weak<FoxEntity>, speed_modifier: f64) -> Self {
        Self {
            fox,
            speed_modifier,
            wanted_x: 0.0,
            wanted_y: 0.0,
            wanted_z: 0.0,
            interval: 100,
        }
    }

    fn find_hide_pos(mob: &dyn Mob) -> Option<Vector3<f64>> {
        let mob_pos = mob.get_entity().block_pos.load();
        let world = mob.get_entity().world.load();
        let mut rng = mob.get_random();

        for _ in 0..10 {
            let offset_x = rng.random_range(-10..=10);
            let offset_y = rng.random_range(-3..=3);
            let offset_z = rng.random_range(-10..=10);
            let check_pos = BlockPos::new(
                mob_pos.0.x + offset_x,
                mob_pos.0.y + offset_y,
                mob_pos.0.z + offset_z,
            );

            if !world.can_see_sky(&check_pos) {
                let block_at = world.get_block_state(&check_pos);
                let block_below = world.get_block_state(&check_pos.down());
                if !block_at.is_solid() && block_below.is_solid() {
                    return Some(Vector3::new(
                        f64::from(check_pos.0.x) + 0.5,
                        f64::from(check_pos.0.y),
                        f64::from(check_pos.0.z) + 0.5,
                    ));
                }
            }
        }
        None
    }
    fn is_near_village(mob: &dyn Mob, world: &World) -> bool {
        let mob_pos = mob.get_entity().pos.load();
        let entities = world.entities.load();
        entities.iter().any(|e| {
            e.get_entity().entity_type == &EntityType::VILLAGER
                && mob_pos.squared_distance_to_vec(&e.get_entity().pos.load()) <= 32.0 * 32.0
        })
    }
}

impl Goal for FoxSeekShelterGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sleeping() || mob.get_mob_entity().get_target().is_some() {
            return false;
        }
        let entity = fox.get_entity();
        let world = entity.world.load();
        let pos = entity.block_pos.load();
        if world.is_thundering() && world.can_see_sky(&pos) {
            if let Some(pos) = Self::find_hide_pos(mob) {
                self.wanted_x = pos.x;
                self.wanted_y = pos.y;
                self.wanted_z = pos.z;
                return true;
            }
            return false;
        }
        if self.interval > 0 {
            self.interval -= 1;
            return false;
        }
        self.interval = 100;
        if world.is_bright_outside()
            && world.can_see_sky(&pos)
            && !Self::is_near_village(mob, &world)
            && let Some(hide_pos) = Self::find_hide_pos(mob)
        {
            self.wanted_x = hide_pos.x;
            self.wanted_y = hide_pos.y;
            self.wanted_z = hide_pos.z;
            return true;
        }
        false
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        !mob.is_navigator_idle()
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.clear_states();
        }
        let current_pos = mob.get_mob_entity().living_entity.entity.pos.load();
        let target_pos = Vector3::new(self.wanted_x, self.wanted_y, self.wanted_z);
        mob.navigate_to(current_pos, target_pos, self.speed_modifier);
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

pub struct FoxMeleeAttackGoal {
    fox: Weak<FoxEntity>,
    inner: crate::entity::ai::goal::melee_attack::MeleeAttackGoal,
}

impl FoxMeleeAttackGoal {
    #[must_use]
    pub fn new(fox: Weak<FoxEntity>, speed: f64, pause_when_mob_idle: bool) -> Self {
        Self {
            fox,
            inner: crate::entity::ai::goal::melee_attack::MeleeAttackGoal::new(
                speed,
                pause_when_mob_idle,
            ),
        }
    }
}

impl Goal for FoxMeleeAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sitting() || fox.is_sleeping() || fox.is_crouching() || fox.is_faceplanted() {
            return false;
        }
        self.inner.can_start(mob)
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(fox) = self.fox.upgrade() else {
            return false;
        };
        if fox.is_sitting() || fox.is_sleeping() || fox.is_crouching() || fox.is_faceplanted() {
            return false;
        }
        self.inner.should_continue(mob)
    }

    fn start(&mut self, mob: &dyn Mob) {
        if let Some(fox) = self.fox.upgrade() {
            fox.set_interested(false);
        }
        self.inner.start(mob);
    }

    fn stop(&mut self, mob: &dyn Mob) {
        self.inner.stop(mob);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let prev_cooldown = self.inner.cooldown;
        self.inner.tick(mob);
        if prev_cooldown <= 0 && self.inner.cooldown > 0 {
            mob.get_entity().world.load().play_sound(
                Sound::EntityFoxBite,
                SoundCategory::Neutral,
                &mob.get_entity().pos.load(),
            );
        }
    }

    fn controls(&self) -> Controls {
        self.inner.controls()
    }
}
