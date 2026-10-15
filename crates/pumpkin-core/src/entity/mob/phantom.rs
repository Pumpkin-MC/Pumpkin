use std::f32::consts::PI;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::math::wrap_degrees;
use rand::RngExt;

use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        Controls, Goal,
        look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal,
    },
    mob::{Mob, MobEntity},
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PhantomAttackPhase {
    Circle,
    Swoop,
}

pub struct PhantomEntity {
    pub mob_entity: MobEntity,
    pub attack_phase: AtomicCell<PhantomAttackPhase>,
    pub move_target_point: AtomicCell<Vector3<f64>>,
    pub anchor_point: AtomicCell<Vector3<f64>>,
    pub flight_speed: AtomicCell<f32>,
}

fn approach_degrees(from: f32, to: f32, max_step: f32) -> f32 {
    let diff = wrap_degrees(to - from);
    if diff > max_step {
        from + max_step
    } else if diff < -max_step {
        from - max_step
    } else {
        to
    }
}

fn approach(val: f32, target: f32, step: f32) -> f32 {
    if val < target {
        (val + step).min(target)
    } else {
        (val - step).max(target)
    }
}

impl PhantomEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let spawn_pos = entity.pos.load();
        let anchor = Vector3::new(spawn_pos.x, spawn_pos.y + 5.0, spawn_pos.z);
        let mob_entity = MobEntity::new(entity);
        let phantom = Self {
            mob_entity,
            attack_phase: AtomicCell::new(PhantomAttackPhase::Circle),
            move_target_point: AtomicCell::new(anchor),
            anchor_point: AtomicCell::new(anchor),
            flight_speed: AtomicCell::new(0.2),
        };
        let mob_arc = Arc::new(phantom);
        let phantom_weak = Arc::downgrade(&mob_arc);
        let mob_weak: Weak<dyn Mob> = {
            let mob_dyn: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_dyn)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(1, Box::new(PhantomAttackStrategyGoal::new(phantom_weak.clone())));
            goal_selector.add_goal(2, Box::new(PhantomSweepAttackGoal::new(phantom_weak.clone())));
            goal_selector.add_goal(3, Box::new(PhantomCircleAroundAnchorGoal::new(phantom_weak)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak, &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(6, Box::new(RandomLookAroundGoal::default()));
        };

        {
            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            target_selector.add_goal(1, Box::new(PhantomAttackPlayerTargetGoal::new()));
        };

        mob_arc
    }

    fn tick_flight(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        let target_point = self.move_target_point.load();
        let current_pos = entity.pos.load();
        let mut tdx = target_point.x - current_pos.x;
        let tdy = target_point.y - current_pos.y;
        let mut tdz = target_point.z - current_pos.z;
        let mut sd = (tdx * tdx + tdz * tdz).sqrt();

        if sd > 1.0e-5 {
            let y_relative_scale = (1.0 - (tdy * 0.7).abs() / sd).clamp(0.0, 1.0);
            tdx *= y_relative_scale;
            tdz *= y_relative_scale;
            sd = (tdx * tdx + tdz * tdz).sqrt();
            let sd2 = (tdx * tdx + tdz * tdz + tdy * tdy).sqrt().max(1.0e-5);

            let cur_yaw = entity.yaw.load();
            let b = (tdz.atan2(tdx) * 180.0 / std::f64::consts::PI) as f32;
            let target_yaw = wrap_degrees(b - 90.0);
            let new_yaw = approach_degrees(cur_yaw, target_yaw, 4.0);
            entity.yaw.store(new_yaw);
            entity.head_yaw.store(new_yaw);
            entity.body_yaw.store(new_yaw);

            let x_rot = (-tdy.atan2(sd) * 180.0 / std::f64::consts::PI) as f32;
            entity.pitch.store(x_rot);

            let mut speed = self.flight_speed.load();
            if wrap_degrees(new_yaw - target_yaw).abs() < 3.0 {
                speed = approach(speed, 1.8, 0.005 * (1.8 / speed.max(0.1)));
            } else {
                speed = approach(speed, 0.2, 0.025);
            }
            self.flight_speed.store(speed);

            let move_angle = (new_yaw + 90.0).to_radians();
            let txd = speed as f64 * (move_angle.cos() as f64) * (tdx / sd2).abs();
            let tzd = speed as f64 * (move_angle.sin() as f64) * (tdz / sd2).abs();
            let tyd = speed as f64 * (x_rot.to_radians().sin() as f64) * (tdy / sd2).abs();

            let cur_vel = entity.velocity.load();
            let target_vel = Vector3::new(txd, tyd, tzd);
            let new_vel = cur_vel + (target_vel - cur_vel) * 0.2;
            entity.set_velocity(new_vel);
        }
    }
}

pub struct PhantomAttackStrategyGoal {
    phantom: Weak<PhantomEntity>,
    next_sweep_tick: i32,
}

impl PhantomAttackStrategyGoal {
    #[must_use]
    pub fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            next_sweep_tick: 200,
        }
    }
}

impl Goal for PhantomAttackStrategyGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        target.get_entity().is_alive()
    }

    fn start(&mut self, mob: &dyn Mob) {
        self.next_sweep_tick = 200;
        if let Some(phantom) = self.phantom.upgrade() {
            phantom.attack_phase.store(PhantomAttackPhase::Circle);
            if let Some(target) = mob.get_mob_entity().get_target() {
                let pos = target.get_entity().pos.load();
                phantom.anchor_point.store(Vector3::new(
                    pos.x,
                    pos.y + 20.0 + (mob.get_random().random_range(0..20) as f64),
                    pos.z,
                ));
            }
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        if phantom.attack_phase.load() == PhantomAttackPhase::Circle {
            self.next_sweep_tick -= 1;
            if self.next_sweep_tick <= 0 {
                phantom.attack_phase.store(PhantomAttackPhase::Swoop);
                if let Some(target) = mob.get_mob_entity().get_target() {
                    let pos = target.get_entity().pos.load();
                    phantom.anchor_point.store(Vector3::new(
                        pos.x,
                        pos.y + 20.0 + (mob.get_random().random_range(0..20) as f64),
                        pos.z,
                    ));
                }
                self.next_sweep_tick = (8 + mob.get_random().random_range(0..4)) * 20;

                let entity = &phantom.mob_entity.living_entity.entity;
                let world = entity.world.load();
                world.play_sound(
                    Sound::EntityPhantomSwoop,
                    SoundCategory::Hostile,
                    &entity.pos.load(),
                );
            }
        }
    }
}

pub struct PhantomSweepAttackGoal {
    phantom: Weak<PhantomEntity>,
    cat_search_tick: i32,
    is_scared_of_cat: bool,
}

impl PhantomSweepAttackGoal {
    #[must_use]
    pub fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            cat_search_tick: 0,
            is_scared_of_cat: false,
        }
    }
}

impl Goal for PhantomSweepAttackGoal {
    fn controls(&self) -> Controls {
        Controls::MOVE
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        if phantom.attack_phase.load() != PhantomAttackPhase::Swoop {
            return false;
        }
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        target.get_entity().is_alive()
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        if phantom.attack_phase.load() != PhantomAttackPhase::Swoop {
            return false;
        }
        let Some(target) = mob.get_mob_entity().get_target() else {
            return false;
        };
        if !target.get_entity().is_alive() {
            return false;
        }

        let entity = &phantom.mob_entity.living_entity.entity;
        let age = entity.age.load(std::sync::atomic::Ordering::Relaxed);
        if age > self.cat_search_tick {
            self.cat_search_tick = age + 20;
            let world = entity.world.load();
            let bb = entity.bounding_box.load().expand(16.0, 16.0, 16.0);
            let nearby = world.get_entities_at_box(&bb);
            let has_cat = nearby.iter().any(|e| e.get_entity().entity_type == &EntityType::CAT);
            if has_cat {
                world.play_sound(
                    Sound::EntityCatHiss,
                    SoundCategory::Neutral,
                    &entity.pos.load(),
                );
                self.is_scared_of_cat = true;
            } else {
                self.is_scared_of_cat = false;
            }
        }

        !self.is_scared_of_cat
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(phantom) = self.phantom.upgrade() {
            phantom.attack_phase.store(PhantomAttackPhase::Circle);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        let Some(target) = mob.get_mob_entity().get_target() else {
            return;
        };
        let target_pos = target.get_entity().pos.load();
        let target_point = Vector3::new(target_pos.x, target_pos.y + 0.5, target_pos.z);
        phantom.move_target_point.store(target_point);

        let entity = &phantom.mob_entity.living_entity.entity;
        let phantom_bb = entity.bounding_box.load().expand_all(0.2);
        let target_bb = target.get_entity().bounding_box.load();

        if phantom_bb.intersects(&target_bb) {
            let damaged = mob.get_mob_entity().try_attack(entity, target.as_ref());
            if damaged {
                let world = entity.world.load();
                world.play_sound(
                    Sound::EntityPhantomBite,
                    SoundCategory::Hostile,
                    &entity.pos.load(),
                );
            }
            phantom.attack_phase.store(PhantomAttackPhase::Circle);
        }
    }
}

pub struct PhantomCircleAroundAnchorGoal {
    phantom: Weak<PhantomEntity>,
    angle: f32,
    distance: f32,
    height: f32,
    clockwise: f32,
}

impl PhantomCircleAroundAnchorGoal {
    #[must_use]
    pub fn new(phantom: Weak<PhantomEntity>) -> Self {
        Self {
            phantom,
            angle: 0.0,
            distance: 10.0,
            height: 0.0,
            clockwise: 1.0,
        }
    }

    fn select_next(&mut self, phantom: &PhantomEntity) {
        self.angle += self.clockwise * 15.0 * (PI / 180.0);
        let anchor = phantom.anchor_point.load();
        let target = Vector3::new(
            anchor.x + (self.distance * self.angle.cos()) as f64,
            anchor.y - 4.0 + self.height as f64,
            anchor.z + (self.distance * self.angle.sin()) as f64,
        );
        phantom.move_target_point.store(target);
    }
}

impl Goal for PhantomCircleAroundAnchorGoal {
    fn controls(&self) -> Controls {
        Controls::MOVE
    }

    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        let Some(phantom) = self.phantom.upgrade() else {
            return false;
        };
        mob.get_mob_entity().get_target().is_none()
            || phantom.attack_phase.load() == PhantomAttackPhase::Circle
    }

    fn start(&mut self, mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        self.distance = 5.0 + mob.get_random().random::<f32>() * 10.0;
        self.height = -4.0 + mob.get_random().random::<f32>() * 9.0;
        self.clockwise = if mob.get_random().random::<bool>() { 1.0 } else { -1.0 };
        self.select_next(&phantom);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(phantom) = self.phantom.upgrade() else {
            return;
        };
        if mob.get_random().random_range(0..350) == 0 {
            self.height = -4.0 + mob.get_random().random::<f32>() * 9.0;
        }
        if mob.get_random().random_range(0..250) == 0 {
            self.distance += 1.0;
            if self.distance > 15.0 {
                self.distance = 5.0;
                self.clockwise = -self.clockwise;
            }
        }
        if mob.get_random().random_range(0..450) == 0 {
            self.angle = mob.get_random().random::<f32>() * 2.0 * PI;
            self.select_next(&phantom);
        }

        let entity = &phantom.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        let target = phantom.move_target_point.load();
        if pos.squared_distance_to_vec(&target) < 4.0 {
            self.select_next(&phantom);
        }
    }
}

pub struct PhantomAttackPlayerTargetGoal {
    next_scan_tick: i32,
}

impl PhantomAttackPlayerTargetGoal {
    #[must_use]
    pub const fn new() -> Self {
        Self { next_scan_tick: 20 }
    }
}

impl Goal for PhantomAttackPlayerTargetGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.next_scan_tick > 0 {
            self.next_scan_tick -= 1;
            return false;
        }
        self.next_scan_tick = 60;

        let mob_entity = mob.get_mob_entity();
        let entity = &mob_entity.living_entity.entity;
        let world = entity.world.load();
        let bb = entity.bounding_box.load().expand(16.0, 64.0, 16.0);
        let players = world.get_players_at_box(&bb);

        let mut valid_player = None;
        let mut highest_y = f64::MIN;

        for player in players {
            if player.is_spectator() || player.is_creative() || !player.living_entity.entity.is_alive() {
                continue;
            }
            let player_y = player.living_entity.entity.pos.load().y;
            if player_y > highest_y {
                highest_y = player_y;
                valid_player = Some(player);
            }
        }

        if let Some(player) = valid_player {
            let player_base: Arc<dyn EntityBase> = player;
            mob.set_mob_target(Some(player_base));
            return true;
        }
        false
    }
}

impl CustomSound for PhantomEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityPhantomDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityPhantomHurt)
    }
}

impl Mob for PhantomEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0
    }

    fn burns_in_daylight(&self) -> bool {
        true
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        crate::entity::mob::sun_burn::tick(self);
        self.tick_flight();

        let entity = &self.mob_entity.living_entity.entity;
        if entity.is_alive()
            && !entity.silent.load(Ordering::Relaxed)
            && rand::random_range(0..100) == 0
        {
            entity.world.load().play_sound_fine(
                Sound::EntityPhantomAmbient,
                SoundCategory::Hostile,
                &entity.pos.load(),
                1.0,
                (rand::random::<f32>() - rand::random::<f32>()).mul_add(0.2, 1.0),
            );
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        let anchor = self.anchor_point.load();
        nbt.put_int("AX", anchor.x as i32);
        nbt.put_int("AY", anchor.y as i32);
        nbt.put_int("AZ", anchor.z as i32);
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let (Some(x), Some(y), Some(z)) = (nbt.get_int("AX"), nbt.get_int("AY"), nbt.get_int("AZ")) {
            self.anchor_point.store(Vector3::new(f64::from(x), f64::from(y), f64::from(z)));
        }
    }
}
