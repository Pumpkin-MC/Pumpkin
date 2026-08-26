use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex, Weak};

use pumpkin_data::damage::DamageType;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_data::tracked_data;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::java::client::play::Metadata;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use rand::RngExt;

use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::living::LivingEntity;
use crate::entity::projectile::fireball::FireballEntity;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{Controls, Goal},
    mob::{Mob, MobEntity},
};
use crate::world::World;

pub struct GhastEntity {
    pub mob_entity: MobEntity,
    pub is_charging: AtomicBool,
    pub explosion_power: AtomicU8,
    pub wanted_fly_target: Mutex<Option<Vector3<f64>>>,
}

impl GhastEntity {
    pub const DEFAULT_EXPLOSION_POWER: u8 = 1;
    pub const XP_REWARD: u32 = 5;
    pub const FLYING_SPEED: f64 = 0.06;

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let ghast = Self {
            mob_entity,
            is_charging: AtomicBool::new(false),
            explosion_power: AtomicU8::new(Self::DEFAULT_EXPLOSION_POWER),
            wanted_fly_target: Mutex::new(None),
        };

        let mob_arc = Arc::new(ghast);

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Priority 5: Random floating around
            goal_selector.add_goal(
                5,
                Box::new(RandomFloatAroundGoal::new(Arc::downgrade(&mob_arc))),
            );

            // Priority 7: Face target / movement direction
            goal_selector.add_goal(7, Box::new(GhastLookGoal::new()));

            // Priority 7: Shoot large fireballs at target
            goal_selector.add_goal(
                7,
                Box::new(GhastShootFireballGoal::new(Arc::downgrade(&mob_arc))),
            );

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            // Priority 1: Target nearest player within vertical proximity
            target_selector.add_goal(
                1,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.mob_entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    Some(|_target: &LivingEntity, _world: &World| true),
                )),
            );
        };

        mob_arc
    }

    pub fn set_charging(&self, charging: bool) {
        self.is_charging.store(charging, Ordering::Relaxed);
        let entity = &self.mob_entity.living_entity.entity;
        entity.send_meta_data(
            &[Metadata::new(
                tracked_data::ghast::DATA_IS_CHARGING,
                charging,
            )],
            None,
        );
    }

    #[must_use]
    pub fn is_charging(&self) -> bool {
        self.is_charging.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn get_explosion_power(&self) -> u8 {
        self.explosion_power.load(Ordering::Relaxed)
    }

    pub fn set_explosion_power(&self, power: u8) {
        self.explosion_power.store(power, Ordering::Relaxed);
    }

    #[must_use]
    pub fn check_ghast_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        if world.level_info.load().difficulty == pumpkin_util::Difficulty::Peaceful {
            return false;
        }
        if rand::random_range(0..20) != 0 {
            return false;
        }
        let state = world.get_block_state(pos);
        state.is_air()
    }
}

impl Mob for GhastEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn get_mob_gravity(&self) -> f64 {
        0.0 // Ghasts fly, no gravity applied in standard travel
    }

    fn get_mob_y_velocity_drag(&self) -> Option<f64> {
        Some(0.95)
    }

    fn mob_init_data_tracker(&self) {
        let entity = self.get_entity();
        if self.is_charging() {
            entity.send_meta_data(
                &[Metadata::new(tracked_data::ghast::DATA_IS_CHARGING, true)],
                None,
            );
        }
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        let power = self.get_explosion_power();
        nbt.put_byte("ExplosionPower", i8::try_from(power).unwrap_or(1));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        if let Some(power) = nbt.get_byte("ExplosionPower") {
            self.set_explosion_power(u8::try_from(power).unwrap_or(Self::DEFAULT_EXPLOSION_POWER));
        }
    }

    fn modify_incoming_damage(&self, amount: f32, damage_type: DamageType) -> f32 {
        if damage_type.id == DamageType::FIREBALL.id {
            1000.0
        } else {
            amount
        }
    }

    fn get_base_experience_reward(&self) -> u32 {
        Self::XP_REWARD
    }
}

pub struct GhastLookGoal {
    goal_control: Controls,
}

impl Default for GhastLookGoal {
    fn default() -> Self {
        Self {
            goal_control: Controls::LOOK,
        }
    }
}

impl GhastLookGoal {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Goal for GhastLookGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        true
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let mob_entity = mob.get_mob_entity();
        let target_opt = mob_entity.get_target();

        if let Some(target) = target_opt {
            let mob_pos = mob_entity.living_entity.entity.pos.load();
            let target_pos = target.get_entity().pos.load();

            if mob_pos.squared_distance_to_vec(&target_pos) < 4096.0 {
                let dx = target_pos.x - mob_pos.x;
                let dz = target_pos.z - mob_pos.z;
                let yaw = (-f64::atan2(dx, dz).to_degrees()) as f32;
                mob_entity.living_entity.entity.yaw.store(yaw);
                mob_entity.living_entity.entity.head_yaw.store(yaw);
            }
        } else {
            let velocity = mob_entity.living_entity.entity.velocity.load();
            if velocity.x != 0.0 || velocity.z != 0.0 {
                let yaw = (-f64::atan2(velocity.x, velocity.z).to_degrees()) as f32;
                mob_entity.living_entity.entity.yaw.store(yaw);
                mob_entity.living_entity.entity.head_yaw.store(yaw);
            }
        }
    }

    fn controls(&self) -> Controls {
        self.goal_control
    }
}

pub struct GhastShootFireballGoal {
    ghast: Weak<GhastEntity>,
    charge_time: i32,
}

impl GhastShootFireballGoal {
    #[must_use]
    pub const fn new(ghast: Weak<GhastEntity>) -> Self {
        Self {
            ghast,
            charge_time: 0,
        }
    }
}

impl Goal for GhastShootFireballGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        let target = ghast.mob_entity.get_target();
        target.is_some_and(|t| t.get_entity().is_alive())
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        let target = ghast.mob_entity.get_target();
        target.is_some_and(|t| t.get_entity().is_alive())
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.charge_time = 0;
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        if let Some(ghast) = self.ghast.upgrade() {
            ghast.set_charging(false);
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(ghast) = self.ghast.upgrade() else {
            return;
        };

        let target_opt = ghast.mob_entity.get_target();
        let Some(target) = target_opt else {
            return;
        };

        let entity = &ghast.mob_entity.living_entity.entity;
        let ghast_pos = entity.pos.load();
        let target_pos = target.get_entity().pos.load();
        let dist_sq = ghast_pos.squared_distance_to_vec(&target_pos);

        if dist_sq < 4096.0 {
            let world = entity.world.load();
            self.charge_time += 1;

            if self.charge_time == 10 {
                world.play_sound_fine(
                    Sound::EntityGhastWarn,
                    SoundCategory::Hostile,
                    &ghast_pos,
                    5.0,
                    1.0,
                );
            }

            if self.charge_time == 20 {
                world.play_sound_fine(
                    Sound::EntityGhastShoot,
                    SoundCategory::Hostile,
                    &ghast_pos,
                    5.0,
                    1.0,
                );

                let yaw_rad = f64::from(entity.yaw.load()).to_radians();
                let pitch_rad = f64::from(entity.pitch.load()).to_radians();
                let view_x = -pitch_rad.cos() * yaw_rad.sin();
                let view_z = pitch_rad.cos() * yaw_rad.cos();

                let spawn_pos = Vector3::new(
                    ghast_pos.x + view_x * 4.0,
                    ghast_pos.y + 2.5,
                    ghast_pos.z + view_z * 4.0,
                );

                let target_y = target_pos.y + target.get_entity().get_eye_height() * 0.5;
                let dir_x = target_pos.x - spawn_pos.x;
                let dir_y = target_y - spawn_pos.y;
                let dir_z = target_pos.z - spawn_pos.z;
                let direction = Vector3::new(dir_x, dir_y, dir_z);

                let fireball_base = Entity::from_uuid(
                    uuid::Uuid::new_v4(),
                    world.clone(),
                    spawn_pos,
                    &EntityType::FIREBALL,
                );

                let fireball = FireballEntity::new_shot(fireball_base, entity, direction);
                fireball
                    .explosion_power
                    .store(f32::from(ghast.get_explosion_power()), Ordering::Relaxed);

                world.spawn_entity_non_save(Arc::new(fireball));
                self.charge_time = -40;
            }
        } else if self.charge_time > 0 {
            self.charge_time -= 1;
        }

        ghast.set_charging(self.charge_time > 10);
    }

    fn controls(&self) -> Controls {
        Controls::empty()
    }
}

pub struct RandomFloatAroundGoal {
    ghast: Weak<GhastEntity>,
    float_duration: i32,
}

impl RandomFloatAroundGoal {
    #[must_use]
    pub const fn new(ghast: Weak<GhastEntity>) -> Self {
        Self {
            ghast,
            float_duration: 0,
        }
    }
}

impl Goal for RandomFloatAroundGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        let wanted = *ghast
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        wanted.is_none_or(|target| {
            let pos = ghast.mob_entity.living_entity.entity.pos.load();
            let dist_sq = pos.squared_distance_to_vec(&target);
            dist_sq < 1.0 || dist_sq > 3600.0
        })
    }

    fn start(&mut self, _mob: &dyn Mob) {
        let Some(ghast) = self.ghast.upgrade() else {
            return;
        };
        let pos = ghast.mob_entity.living_entity.entity.pos.load();
        let new_target = {
            let mut rng = rand::rng();
            let target_x = pos.x + (rng.random::<f64>() * 2.0 - 1.0) * 16.0;
            let target_y = pos.y + (rng.random::<f64>() * 2.0 - 1.0) * 16.0;
            let target_z = pos.z + (rng.random::<f64>() * 2.0 - 1.0) * 16.0;
            Vector3::new(target_x, target_y, target_z)
        };
        *ghast
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(new_target);
    }

    fn should_continue(&self, _mob: &dyn Mob) -> bool {
        let Some(ghast) = self.ghast.upgrade() else {
            return false;
        };
        let wanted = *ghast
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        wanted.is_some_and(|target| {
            let pos = ghast.mob_entity.living_entity.entity.pos.load();
            let dist_sq = pos.squared_distance_to_vec(&target);
            (1.0..=3600.0).contains(&dist_sq)
        })
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(ghast) = self.ghast.upgrade() else {
            return;
        };

        let wanted = *ghast
            .wanted_fly_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(target) = wanted else {
            return;
        };

        let entity = &ghast.mob_entity.living_entity.entity;
        let pos = entity.pos.load();
        self.float_duration -= 1;

        if self.float_duration <= 0 {
            self.float_duration = rand::random_range(2..=6);
            let travel = Vector3::new(target.x - pos.x, target.y - pos.y, target.z - pos.z);
            let dist = travel.length();
            if dist > 0.001 {
                let move_scale = GhastEntity::FLYING_SPEED * 5.0 / 3.0; // 0.1
                let norm = travel.normalize();
                let delta = Vector3::new(
                    norm.x * move_scale,
                    norm.y * move_scale,
                    norm.z * move_scale,
                );
                let current_vel = entity.velocity.load();
                entity.velocity.store(Vector3::new(
                    current_vel.x + delta.x,
                    current_vel.y + delta.y,
                    current_vel.z + delta.z,
                ));
            }
        }
    }

    fn controls(&self) -> Controls {
        Controls::MOVE
    }
}
