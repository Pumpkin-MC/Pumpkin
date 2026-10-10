#![allow(
    clippy::missing_const_for_fn,
    clippy::semicolon_outside_block,
    clippy::semicolon_if_nothing_returned
)]
use std::sync::atomic::{AtomicI32, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, Weak};

use pumpkin_data::entity::EntityType;
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::goal::{Controls, Goal};
use crate::entity::mob::Mob;
use crate::entity::{Entity, EntityBase, mob::MobEntity};

#[derive(Clone, Copy, PartialEq, Eq)]
enum PhantomMovementType {
    Circle = 0,
    Swoop = 1,
}

pub struct PhantomEntity {
    pub mob_entity: MobEntity,
    target_position: Mutex<Vector3<f64>>,
    circling_center: Mutex<BlockPos>,
    movement_type: AtomicU8,
    phantom_size: AtomicI32,
}

impl PhantomEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let phantom = Self {
            mob_entity,
            target_position: Mutex::new(Vector3::new(0.0, 0.0, 0.0)),
            circling_center: Mutex::new(BlockPos::new(0, 0, 0)),
            movement_type: AtomicU8::new(PhantomMovementType::Circle as u8),
            phantom_size: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(phantom);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };
        let phantom_weak = Arc::downgrade(&mob_arc);

        // Set no gravity and flying
        mob_arc
            .mob_entity
            .living_entity
            .entity
            .set_has_no_gravity(true);

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(1, Box::new(StartAttackGoal::new(phantom_weak.clone())));
            goal_selector.add_goal(2, Box::new(SwoopMovementGoal::new(phantom_weak.clone())));
            goal_selector.add_goal(3, Box::new(CircleMovementGoal::new(phantom_weak.clone())));

            // Target selector
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            target_selector.add_goal(1, Box::new(FindTargetGoal::new(phantom_weak)));
            // Keep look goals at low priority
            goal_selector.add_goal(
                6,
                crate::entity::ai::goal::look_at_entity::LookAtEntityGoal::with_default(
                    mob_weak,
                    &EntityType::PLAYER,
                    8.0,
                ),
            );
            goal_selector.add_goal(
                6,
                Box::new(crate::entity::ai::goal::look_around::RandomLookAroundGoal::default()),
            );
        }

        // Initialize circling center to current pos up 5
        {
            let pos = mob_arc.mob_entity.living_entity.entity.pos.load();
            let block_pos = BlockPos::new(
                pos.x.floor() as i32,
                pos.y.floor() as i32 + 5,
                pos.z.floor() as i32,
            );
            *mob_arc
                .circling_center
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = block_pos;
            *mob_arc
                .target_position
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                Vector3::new(pos.x, pos.y, pos.z);
        }

        mob_arc
    }

    fn get_movement_type(&self) -> PhantomMovementType {
        match self.movement_type.load(Ordering::Relaxed) {
            1 => PhantomMovementType::Swoop,
            _ => PhantomMovementType::Circle,
        }
    }

    fn set_movement_type(&self, t: PhantomMovementType) {
        self.movement_type.store(t as u8, Ordering::Relaxed);
    }

    fn move_towards_target(&self, speed: f64) {
        let entity = &self.mob_entity.living_entity.entity;
        let target = *self
            .target_position
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pos = entity.pos.load();
        let dx = target.x - pos.x;
        let dy = target.y - pos.y;
        let dz = target.z - pos.z;
        let dist_sq = dx * dx + dy * dy + dz * dz;
        if dist_sq < 0.001 {
            return;
        }
        let dist = dist_sq.sqrt();
        let dir_x = dx / dist;
        let dir_y = dy / dist;
        let dir_z = dz / dist;

        // Yaw towards target
        let yaw = (-dir_x.atan2(dir_z).to_degrees()) as f32;
        // Simple yaw lerp
        let current_yaw = entity.yaw.load();
        let diff = (yaw - current_yaw + 540.0).rem_euclid(360.0) - 180.0;
        let new_yaw = (current_yaw + (diff * 0.2).clamp(-20.0, 20.0)).rem_euclid(360.0);
        entity.yaw.store(new_yaw);
        entity.body_yaw.store(new_yaw);
        entity.head_yaw.store(new_yaw);

        // Pitch towards target
        let horiz = dir_x.hypot(dir_z);
        let pitch = (-dir_y.atan2(horiz).to_degrees()) as f32;
        let current_pitch = entity.pitch.load();
        let pitch_diff = pitch - current_pitch;
        let new_pitch = current_pitch + (pitch_diff * 0.2).clamp(-20.0, 20.0);
        entity.pitch.store(new_pitch);

        // Velocity lerp
        let vel = entity.velocity.load();
        let target_vel = Vector3::new(dir_x * speed, dir_y * speed, dir_z * speed);
        let new_vel = Vector3::new(
            vel.x + (target_vel.x - vel.x) * 0.2,
            vel.y + (target_vel.y - vel.y) * 0.2,
            vel.z + (target_vel.z - vel.z) * 0.2,
        );
        entity.velocity.store(new_vel);
    }

    fn is_near_target(&self) -> bool {
        let target = *self
            .target_position
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pos = self.mob_entity.living_entity.entity.pos.load();
        let dx = target.x - pos.x;
        let dy = target.y - pos.y;
        let dz = target.z - pos.z;
        dx * dx + dy * dy + dz * dz < 4.0
    }
}

impl Mob for PhantomEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        entity.set_synced_data(
            tracked_data::phantom::ID_SIZE,
            self.phantom_size.load(Ordering::Relaxed),
        );
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        let center = *self
            .circling_center
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        nbt.put_int("AX", center.0.x);
        nbt.put_int("AY", center.0.y);
        nbt.put_int("AZ", center.0.z);
        nbt.put_int("Size", self.phantom_size.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if nbt.get_int("AX").is_some() {
            let x = nbt.get_int("AX").unwrap_or(0);
            let y = nbt.get_int("AY").unwrap_or(0);
            let z = nbt.get_int("AZ").unwrap_or(0);
            *self
                .circling_center
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = BlockPos::new(x, y, z);
        }
        if let Some(size) = nbt.get_int("Size") {
            self.phantom_size.store(size, Ordering::Relaxed);
            let entity = self.get_entity();
            entity.set_synced_data(tracked_data::phantom::ID_SIZE, size);
        }
    }
}

// ---- CircleMovementGoal ----

struct CircleMovementGoal {
    phantom: Weak<PhantomEntity>,
    angle: f32,
    radius: f32,
    y_offset: f32,
    direction: f32,
}

impl CircleMovementGoal {
    const fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            angle: 0.0,
            radius: 10.0,
            y_offset: 0.0,
            direction: 1.0,
        }
    }

    fn adjust_direction(&mut self) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        let center = {
            let mut c = phantom
                .circling_center
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if c.0.x == 0 && c.0.y == 0 && c.0.z == 0 {
                let pos = phantom.mob_entity.living_entity.entity.pos.load();
                let new_center = BlockPos::new(
                    pos.x.floor() as i32,
                    pos.y.floor() as i32,
                    pos.z.floor() as i32,
                );
                *c = new_center;
                new_center
            } else {
                *c
            }
        };
        self.angle += self.direction * 15.0 * std::f32::consts::PI / 180.0;
        let target = Vector3::new(
            center.0.x as f64 + (self.radius * self.angle.cos()) as f64,
            center.0.y as f64 - 4.0 + self.y_offset as f64,
            center.0.z as f64 + (self.radius * self.angle.sin()) as f64,
        );
        *phantom
            .target_position
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = target;
    }
}

impl Goal for CircleMovementGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        phantom.get_movement_type() == PhantomMovementType::Circle
            || phantom.mob_entity.get_target().is_none()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        phantom.get_movement_type() == PhantomMovementType::Circle
    }

    fn start(&mut self, _mob: &dyn Mob) {
        if self.phantom.upgrade().is_none() {
            return;
        }
        self.radius = 5.0 + rand::rng().random::<f32>() * 10.0;
        self.y_offset = -4.0 + rand::rng().random::<f32>() * 9.0;
        self.direction = if rand::rng().random_bool(0.5) {
            1.0
        } else {
            -1.0
        };
        self.adjust_direction();
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        if rand::rng().random_range(0..350) == 0 {
            self.y_offset = -4.0 + rand::rng().random::<f32>() * 9.0;
        }
        if rand::rng().random_range(0..250) == 0 {
            self.radius += 1.0;
            if self.radius > 15.0 {
                self.radius = 5.0;
                self.direction = -self.direction;
            }
        }
        if rand::rng().random_range(0..450) == 0 {
            self.angle = rand::rng().random::<f32>() * 2.0 * std::f32::consts::PI;
            self.adjust_direction();
        }
        if phantom.is_near_target() {
            self.adjust_direction();
        }
        // Adjust for collision above/below
        let entity = &phantom.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let block_pos = BlockPos::new(
            pos.x.floor() as i32,
            pos.y.floor() as i32,
            pos.z.floor() as i32,
        );
        let target_y = phantom
            .target_position
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .y;
        if target_y < pos.y && !world.get_block_state(&block_pos.down()).is_air() {
            self.y_offset = self.y_offset.max(1.0);
            self.adjust_direction();
        }
        if target_y > pos.y && !world.get_block_state(&block_pos.up()).is_air() {
            self.y_offset = self.y_offset.min(-1.0);
            self.adjust_direction();
        }
        phantom.move_towards_target(0.5);
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

// ---- SwoopMovementGoal ----

struct SwoopMovementGoal {
    phantom: Weak<PhantomEntity>,
    next_cat_check: i32,
    #[allow(dead_code)]
    cats_nearby: bool,
}

impl SwoopMovementGoal {
    const fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            next_cat_check: 0,
            cats_nearby: false,
        }
    }
}

impl Goal for SwoopMovementGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        phantom.get_movement_type() == PhantomMovementType::Swoop
            && phantom.mob_entity.get_target().is_some()
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        let target = phantom.mob_entity.get_target();
        if target.is_none() {
            return false;
        }
        let Some(t) = target else {
            return false;
        };
        if !t.get_entity().is_alive() {
            return false;
        }
        // Simplified: ignore spectator/creative and cats for now (no cat hiss)
        if phantom.get_movement_type() != PhantomMovementType::Swoop {
            return false;
        }
        // Cat check every 20 ticks - simplified to just skip if cats found (we don't have cats check fully)
        // For now, ignore cats
        let _ = self.next_cat_check;
        true
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        phantom.mob_entity.set_target(None);
        phantom.set_movement_type(PhantomMovementType::Circle);
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        let Some(target) = phantom.mob_entity.get_target() else {
            return;
        };
        let target_entity = target.get_entity();
        let target_pos = target_entity.pos.load();
        let target_y = target_pos.y + target_entity.get_eye_height() * 0.5;
        *phantom
            .target_position
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            Vector3::new(target_pos.x, target_y, target_pos.z);

        let phantom_pos = phantom.mob_entity.living_entity.entity.pos.load();
        let phantom_bb = phantom.mob_entity.living_entity.entity.bounding_box.load();
        let target_bb = target_entity.bounding_box.load();
        // Simple intersect check: distance < 1.5 and expanded check
        let dx = phantom_pos.x - target_pos.x;
        let dy = phantom_pos.y - target_y;
        let dz = phantom_pos.z - target_pos.z;
        let dist_sq = dx * dx + dy * dy + dz * dz;
        let intersects = phantom_bb.expand(0.2, 0.2, 0.2).intersects(&target_bb) || dist_sq < 2.0;

        if intersects {
            // Attack
            phantom.mob_entity.try_attack(
                phantom.as_ref() as &dyn crate::entity::EntityBase,
                target.as_ref(),
            );
            phantom.set_movement_type(PhantomMovementType::Circle);
        } else if phantom
            .mob_entity
            .living_entity
            .entity
            .horizontal_collision
            .load(Ordering::Relaxed)
        {
            phantom.set_movement_type(PhantomMovementType::Circle);
        }
        phantom.move_towards_target(1.2);
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}

// ---- StartAttackGoal ----

struct StartAttackGoal {
    phantom: Weak<PhantomEntity>,
    cooldown: i32,
}

impl StartAttackGoal {
    fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            cooldown: 0,
        }
    }

    fn start_swoop(&self) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        let Some(target) = phantom.mob_entity.get_target() else {
            return;
        };
        let target_pos = target.get_entity().pos.load();
        let world = phantom.mob_entity.living_entity.entity.world.load();
        let mut center = BlockPos::new(
            target_pos.x.floor() as i32,
            (target_pos.y + 20.0 + rand::rng().random::<f32>() as f64 * 20.0).floor() as i32,
            target_pos.z.floor() as i32,
        );
        // Clamp to sea level + 1
        let sea_level = world.sea_level;
        if center.0.y < sea_level {
            center = BlockPos::new(center.0.x, sea_level + 1, center.0.z);
        }
        *phantom
            .circling_center
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = center;
    }
}

impl Goal for StartAttackGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        let target = phantom.mob_entity.get_target();
        target.is_some_and(|t| t.get_entity().is_alive())
    }

    fn start(&mut self, _mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        self.cooldown = 10;
        phantom.set_movement_type(PhantomMovementType::Circle);
        self.start_swoop();
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        // Lift circling center to top motion blocking + 10-30
        let center = *phantom
            .circling_center
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let world = phantom.mob_entity.living_entity.entity.world.load();
        // Simplified: just up 10 + rand 20
        let new_y = center.0.y + 10 + rand::rng().random_range(0..20);
        *phantom
            .circling_center
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) =
            BlockPos::new(center.0.x, new_y, center.0.z);
        let _ = world;
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        if phantom.get_movement_type() == PhantomMovementType::Circle {
            self.cooldown -= 1;
            if self.cooldown <= 0 {
                phantom.set_movement_type(PhantomMovementType::Swoop);
                self.start_swoop();
                self.cooldown = (8 + rand::rng().random_range(0..4)) * 20;
                // Play swoop sound
                let pos = phantom.mob_entity.living_entity.entity.pos.load();
                let world = phantom.mob_entity.living_entity.entity.world.load();
                world.play_sound_fine(
                    pumpkin_data::sound::Sound::EntityPhantomSwoop,
                    pumpkin_data::sound::SoundCategory::Hostile,
                    &BlockPos::new(
                        pos.x.floor() as i32,
                        pos.y.floor() as i32,
                        pos.z.floor() as i32,
                    )
                    .to_f64(),
                    10.0,
                    0.95 + rand::rng().random::<f32>() * 0.1,
                );
            }
        }
    }
}

// ---- FindTargetGoal ----

struct FindTargetGoal {
    phantom: Weak<PhantomEntity>,
    delay: i32,
}

impl FindTargetGoal {
    const fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self { phantom, delay: 20 }
    }
}

impl Goal for FindTargetGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        if self.delay > 0 {
            self.delay -= 1;
            return false;
        }
        self.delay = 60;
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        let entity = &phantom.mob_entity.living_entity.entity;
        let world = entity.world.load();
        let pos = entity.pos.load();
        let bb = entity.bounding_box.load().expand(16.0, 64.0, 16.0);
        // Use get_nearby_players or get_players_at_box
        let players = world.get_players_at_box(&bb);
        if players.is_empty() {
            return false;
        }
        // Sort by Y descending
        let mut sorted = players;
        sorted.sort_by(|a, b| {
            let ay = a.get_entity().pos.load().y;
            let by = b.get_entity().pos.load().y;
            by.partial_cmp(&ay).unwrap_or(std::cmp::Ordering::Equal)
        });
        for player in sorted {
            let living = &player.living_entity;
            // Check isTarget predicate simplified: not spectator/creative and alive
            if !living.entity.is_alive() {
                continue;
            }
            if living.entity.is_spectator() {
                continue;
            }
            if player.is_creative() {
                continue;
            }
            // Check distance and line of sight simplified
            let player_pos = living.entity.pos.load();
            let dist_sq = pos.squared_distance_to_vec(&player_pos);
            if dist_sq > 4096.0 {
                continue;
            }
            phantom
                .mob_entity
                .set_target(Some(player.clone() as Arc<dyn crate::entity::EntityBase>));
            // Actually need to set target as LivingEntity? Check API
            // MobEntity stores target as Option<Arc<dyn Mob>> or Living?
            // Use set_target with player
            return true;
        }
        false
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        let Some(target) = phantom.mob_entity.get_target() else {
            return false;
        };
        let entity = target.get_entity();
        if !entity.is_alive() || entity.is_spectator() {
            return false;
        }
        if let Some(player) = entity.world.load().get_player_by_id(entity.entity_id)
            && player.is_creative()
        {
            return false;
        }
        true
    }
}
