use std::sync::{
    Arc, Weak,
    atomic::{AtomicI32, Ordering},
};

use pumpkin_data::{
    attributes::Attributes,
    damage::DamageType,
    effect::StatusEffect,
    entity::{EntityPose, EntityStatus, EntityType},
    particle::Particle,
    potion::Effect,
    sound::{Sound, SoundCategory},
    tag::Taggable,
};
use pumpkin_util::{GameMode, math::vector3::Vector3};

use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::{
        Controls, Goal, active_target::ActiveTargetGoal, look_around::RandomLookAroundGoal,
        look_at_entity::LookAtEntityGoal, melee_attack::MeleeAttackGoal, swim::SwimGoal,
        wander_around::WanderAroundGoal,
    },
    mob::{Mob, MobEntity},
};
use crate::world::World;

const EMERGE_DURATION: i32 = 134;
const SNIFFING_DURATION: i32 = 84;
const ROAR_DURATION: i32 = 84;
const DIGGING_DURATION: i32 = 100;
const DARKNESS_DISPLAY_LIMIT: i32 = 200;
const DARKNESS_DURATION: i32 = 260;

pub struct WardenEntity {
    pub mob_entity: MobEntity,
    emerge_ticks: Arc<AtomicI32>,
    sniff_ticks: Arc<AtomicI32>,
    roar_ticks: Arc<AtomicI32>,
    dig_ticks: Arc<AtomicI32>,
    darkness_tick: AtomicI32,
    heartbeat_tick: AtomicI32,
    tendril_tick: AtomicI32,
}

struct EmergeGoal {
    emerge_ticks: Arc<AtomicI32>,
}

impl Goal for EmergeGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.emerge_ticks.load(Ordering::Relaxed) > 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let entity = &mob.get_mob_entity().living_entity.entity;
        if entity.pose.load() == EntityPose::Emerging {
            entity.set_pose(EntityPose::Standing);
        }
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.emerge_ticks.fetch_sub(1, Ordering::Relaxed);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP | Controls::TARGET
    }
}

struct SniffGoal {
    sniff_ticks: Arc<AtomicI32>,
    cooldown: AtomicI32,
}

impl Goal for SniffGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        if self.sniff_ticks.load(Ordering::Relaxed) > 0 {
            return true;
        }
        let cd = self.cooldown.load(Ordering::Relaxed);
        if cd > 0 {
            self.cooldown.store(cd - 1, Ordering::Relaxed);
            return false;
        }
        if mob.get_mob_entity().get_target().is_none() {
            let entity = &mob.get_mob_entity().living_entity.entity;
            if entity.pose.load() == EntityPose::Standing {
                self.sniff_ticks.store(SNIFFING_DURATION, Ordering::Relaxed);
                return true;
            }
        }
        false
    }

    fn start(&mut self, mob: &dyn Mob) {
        let entity = &mob.get_mob_entity().living_entity.entity;
        entity.set_pose(EntityPose::Sniffing);
        entity.world.load().play_sound_fine(
            Sound::EntityWardenSniff,
            SoundCategory::Hostile,
            &entity.pos.load(),
            5.0,
            1.0,
        );
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let entity = &mob.get_mob_entity().living_entity.entity;
        if entity.pose.load() == EntityPose::Sniffing {
            entity.set_pose(EntityPose::Standing);
        }
        self.cooldown.store(300, Ordering::Relaxed);
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let ticks = self.sniff_ticks.fetch_sub(1, Ordering::Relaxed);
        if ticks <= 1 {
            let entity = &mob.get_mob_entity().living_entity.entity;
            let world = entity.world.load();
            let my_pos = entity.pos.load();
            let players = world.get_nearby_players(my_pos, 16.0);
            for player in players {
                if matches!(
                    player.gamemode.load(),
                    GameMode::Survival | GameMode::Adventure
                ) && player.get_entity().is_alive()
                {
                    mob.get_mob_entity().set_target(Some(player));
                    break;
                }
            }
        }
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.sniff_ticks.load(Ordering::Relaxed) > 0
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

struct RoarGoal {
    roar_ticks: Arc<AtomicI32>,
}

impl Goal for RoarGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.roar_ticks.load(Ordering::Relaxed) > 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        let entity = &mob.get_mob_entity().living_entity.entity;
        entity.set_pose(EntityPose::Roaring);
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let entity = &mob.get_mob_entity().living_entity.entity;
        if entity.pose.load() == EntityPose::Roaring {
            entity.set_pose(EntityPose::Standing);
        }
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let ticks_left = self.roar_ticks.fetch_sub(1, Ordering::Relaxed);
        if ticks_left == ROAR_DURATION - 25 {
            let entity = &mob.get_mob_entity().living_entity.entity;
            entity.world.load().play_sound_fine(
                Sound::EntityWardenRoar,
                SoundCategory::Hostile,
                &entity.pos.load(),
                3.0,
                1.0,
            );
        }
        if let Some(target) = mob.get_mob_entity().get_target() {
            mob.get_mob_entity()
                .look_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .look_at_entity_with_range(&target, 90.0, 90.0);
        }
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.roar_ticks.load(Ordering::Relaxed) > 0
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::TARGET
    }
}

struct DigGoal {
    dig_ticks: Arc<AtomicI32>,
}

impl Goal for DigGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.dig_ticks.load(Ordering::Relaxed) > 0
    }

    fn start(&mut self, mob: &dyn Mob) {
        let entity = &mob.get_mob_entity().living_entity.entity;
        entity.set_pose(EntityPose::Digging);
        entity.world.load().play_sound_fine(
            Sound::EntityWardenDig,
            SoundCategory::Hostile,
            &entity.pos.load(),
            5.0,
            1.0,
        );
        mob.get_mob_entity()
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn stop(&mut self, mob: &dyn Mob) {
        let entity = &mob.get_mob_entity().living_entity.entity;
        entity.remove();
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.dig_ticks.fetch_sub(1, Ordering::Relaxed);
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.dig_ticks.load(Ordering::Relaxed) > 0
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn can_stop(&self) -> bool {
        false
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK | Controls::JUMP | Controls::TARGET
    }
}

pub struct WardenSonicBoomGoal {
    warden: Weak<WardenEntity>,
    ticks: i32,
    cooldown: i32,
}

impl WardenSonicBoomGoal {
    pub const COOLDOWN: i32 = 40;
    pub const CHARGE_DURATION: i32 = 34;
    pub const TOTAL_DURATION: i32 = 60;

    #[must_use]
    pub const fn new(warden: Weak<WardenEntity>) -> Self {
        Self {
            warden,
            ticks: 0,
            cooldown: 0,
        }
    }
}

impl Goal for WardenSonicBoomGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        if self.cooldown > 0 {
            self.cooldown -= 1;
            return false;
        }
        let Some(warden) = self.warden.upgrade() else {
            return false;
        };
        let Some(target) = warden.mob_entity.get_target() else {
            return false;
        };
        let target_entity = target.get_entity();
        if !target_entity.is_alive() {
            return false;
        }

        let my_pos = warden.mob_entity.living_entity.entity.pos.load();
        let target_pos = target_entity.pos.load();
        let dx = target_pos.x - my_pos.x;
        let dy = target_pos.y - my_pos.y;
        let dz = target_pos.z - my_pos.z;
        let hor_dist_sq = dx * dx + dz * dz;

        hor_dist_sq <= 15.0 * 15.0 && dy.abs() <= 20.0 && hor_dist_sq > 9.0
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.ticks < Self::TOTAL_DURATION
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.ticks = 0;
        let Some(warden) = self.warden.upgrade() else {
            return;
        };
        let entity = &warden.mob_entity.living_entity.entity;
        let world = entity.world.load();
        world.send_entity_status(entity, EntityStatus::SonicCharge, None);
        world.play_sound_fine(
            Sound::EntityWardenSonicCharge,
            SoundCategory::Hostile,
            &entity.pos.load(),
            3.0,
            1.0,
        );
        warden
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();
    }

    fn stop(&mut self, _mob: &dyn Mob) {
        self.cooldown = Self::COOLDOWN;
        self.ticks = 0;
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        let Some(warden) = self.warden.upgrade() else {
            return;
        };
        let Some(target) = warden.mob_entity.get_target() else {
            return;
        };
        let target_entity = target.get_entity();

        warden
            .mob_entity
            .look_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .look_at_entity_with_range(&target, 90.0, 90.0);
        warden
            .mob_entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .stop();

        self.ticks += 1;
        if self.ticks == Self::CHARGE_DURATION {
            let entity = &warden.mob_entity.living_entity.entity;
            let world = entity.world.load();
            let my_pos = entity.pos.load();
            let source_pos = Vector3::new(my_pos.x, my_pos.y + 1.6, my_pos.z);
            let target_pos = target_entity.pos.load();
            let target_eye = Vector3::new(
                target_pos.x,
                target_pos.y + target_entity.get_eye_height(),
                target_pos.z,
            );
            let delta = Vector3::new(
                target_eye.x - source_pos.x,
                target_eye.y - source_pos.y,
                target_eye.z - source_pos.z,
            );
            let dist = delta.length();
            if dist > 0.001 {
                let norm = delta.normalize();
                let steps = (dist.floor() as i32) + 7;
                for i in 1..steps {
                    let p_pos = Vector3::new(
                        norm.x.mul_add(f64::from(i), source_pos.x),
                        norm.y.mul_add(f64::from(i), source_pos.y),
                        norm.z.mul_add(f64::from(i), source_pos.z),
                    );
                    world.spawn_particle(
                        p_pos,
                        Vector3::new(0.0, 0.0, 0.0),
                        0.0,
                        1,
                        Particle::SonicBoom,
                    );
                }

                world.play_sound_fine(
                    Sound::EntityWardenSonicBoom,
                    SoundCategory::Hostile,
                    &my_pos,
                    3.0,
                    1.0,
                );

                target.damage(target.as_ref(), 10.0, DamageType::SONIC_BOOM);

                let knockback_res = target
                    .get_living_entity()
                    .map_or(0.0, |l| l.get_attribute_value(&Attributes::KNOCKBACK_RESISTANCE));
                let factor = (1.0 - knockback_res).max(0.0);
                let kb_h = 2.5 * factor;
                let kb_v = 0.5 * factor;
                let mut target_vel = target_entity.velocity.load();
                target_vel.x += norm.x * kb_h;
                target_vel.y += norm.y * kb_v;
                target_vel.z += norm.z * kb_h;
                target_entity.velocity.store(target_vel);
            }
        }
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> Controls {
        Controls::MOVE | Controls::LOOK
    }
}

impl WardenEntity {
    pub const XP_REWARD: u32 = 5;

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let emerge_ticks = Arc::new(AtomicI32::new(0));
        let sniff_ticks = Arc::new(AtomicI32::new(0));
        let roar_ticks = Arc::new(AtomicI32::new(0));
        let dig_ticks = Arc::new(AtomicI32::new(0));
        let warden = Self {
            mob_entity,
            emerge_ticks: emerge_ticks.clone(),
            sniff_ticks: sniff_ticks.clone(),
            roar_ticks: roar_ticks.clone(),
            dig_ticks: dig_ticks.clone(),
            darkness_tick: AtomicI32::new(0),
            heartbeat_tick: AtomicI32::new(0),
            tendril_tick: AtomicI32::new(0),
        };
        let mob_arc = Arc::new(warden);
        let mob_weak: Weak<dyn Mob> = {
            let mob_arc: Arc<dyn Mob> = mob_arc.clone();
            Arc::downgrade(&mob_arc)
        };

        {
            let mut goal_selector = mob_arc
                .mob_entity
                .goals_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);

            goal_selector.add_goal(0, Box::new(EmergeGoal { emerge_ticks }));
            goal_selector.add_goal(0, Box::new(DigGoal { dig_ticks }));
            goal_selector.add_goal(0, Box::new(RoarGoal { roar_ticks }));
            goal_selector.add_goal(0, Box::new(SwimGoal::default()));
            goal_selector.add_goal(
                2,
                Box::new(SniffGoal {
                    sniff_ticks,
                    cooldown: AtomicI32::new(100),
                }),
            );
            goal_selector.add_goal(
                3,
                Box::new(WardenSonicBoomGoal::new(Arc::downgrade(&mob_arc))),
            );
            goal_selector.add_goal(4, Box::new(MeleeAttackGoal::new(1.0, true)));
            goal_selector.add_goal(5, Box::new(WanderAroundGoal::new(0.5)));
            goal_selector.add_goal(
                6,
                LookAtEntityGoal::with_default(mob_weak.clone(), &EntityType::PLAYER, 8.0),
            );
            goal_selector.add_goal(7, Box::new(RandomLookAroundGoal::default()));

            let mut target_selector = mob_arc
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.mob_entity, &EntityType::PLAYER, true),
            );
        };

        mob_arc
    }

    pub fn emerge(&self) {
        let entity = &self.mob_entity.living_entity.entity;
        entity.set_pose(EntityPose::Emerging);
        self.emerge_ticks.store(EMERGE_DURATION, Ordering::Relaxed);
        entity.world.load().play_sound_fine(
            Sound::EntityWardenEmerge,
            SoundCategory::Hostile,
            &entity.pos.load(),
            5.0,
            1.0,
        );
    }

    pub fn sniff(&self) {
        self.sniff_ticks.store(SNIFFING_DURATION, Ordering::Relaxed);
    }

    pub fn roar(&self) {
        self.roar_ticks.store(ROAR_DURATION, Ordering::Relaxed);
    }

    pub fn dig(&self) {
        self.dig_ticks.store(DIGGING_DURATION, Ordering::Relaxed);
    }

    pub fn apply_darkness_around(world: &World, position: Vector3<f64>, radius: f64) {
        for player in world.get_nearby_players(position, radius) {
            if !matches!(
                player.gamemode.load(),
                GameMode::Survival | GameMode::Adventure
            ) {
                continue;
            }
            let current = player.living_entity.get_effect(&StatusEffect::DARKNESS);
            if current.is_some_and(|effect| {
                effect.duration < 0 || effect.duration >= DARKNESS_DISPLAY_LIMIT
            }) {
                continue;
            }
            let darkness = Effect {
                effect_type: &StatusEffect::DARKNESS,
                duration: DARKNESS_DURATION,
                amplifier: 0,
                ambient: false,
                show_particles: false,
                show_icon: false,
                blend: true,
            };
            player.send_effect(&darkness);
            player.living_entity.add_effect(darkness);
        }
    }
}

impl CustomSound for WardenEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(Sound::EntityWardenDeath)
    }

    fn hurt_sound(&self) -> Option<Sound> {
        Some(Sound::EntityWardenHurt)
    }
}

impl Mob for WardenEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.mob_entity
    }

    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }

    fn check_spawn_obstruction(&self, world: &World) -> bool {
        let entity = &self.mob_entity.living_entity.entity;
        let bounding_box = entity.bounding_box.load();
        !world.contains_any_liquid(bounding_box)
            && world.get_entities_at_box(&bounding_box).is_empty()
            && world.is_space_empty(bounding_box)
    }

    fn pre_damage(&self, damage_type: DamageType, _source: Option<&dyn EntityBase>) -> bool {
        if (self.emerge_ticks.load(Ordering::Relaxed) > 0
            || self.dig_ticks.load(Ordering::Relaxed) > 0)
            && !damage_type
                .has_tag(&pumpkin_data::tag::DamageType::MINECRAFT_BYPASSES_INVULNERABILITY)
        {
            return false;
        }
        true
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        let entity = &self.mob_entity.living_entity.entity;
        if !entity.is_alive() {
            return;
        }

        let world = entity.world.load();
        let pos = entity.pos.load();

        // Darkness pulsing every 120 ticks
        let dark_t = self.darkness_tick.fetch_add(1, Ordering::Relaxed) + 1;
        if dark_t >= 120 {
            self.darkness_tick.store(0, Ordering::Relaxed);
            Self::apply_darkness_around(&world, pos, 20.0);
        }

        // Heartbeat pulsing sound
        let hb_t = self.heartbeat_tick.fetch_add(1, Ordering::Relaxed) + 1;
        let hb_interval = if self.mob_entity.get_target().is_some() {
            30
        } else {
            60
        };
        if hb_t >= hb_interval {
            self.heartbeat_tick.store(0, Ordering::Relaxed);
            if !entity.silent.load(Ordering::Relaxed) {
                world.play_sound_fine(
                    Sound::EntityWardenHeartbeat,
                    SoundCategory::Hostile,
                    &pos,
                    5.0,
                    1.0,
                );
            }
        }

        // Tendril clicks & shivering every 80 ticks when idle
        if self.mob_entity.get_target().is_none() {
            let tc_t = self.tendril_tick.fetch_add(1, Ordering::Relaxed) + 1;
            if tc_t >= 80 {
                self.tendril_tick.store(0, Ordering::Relaxed);
                if !entity.silent.load(Ordering::Relaxed) {
                    world.send_entity_status(entity, EntityStatus::TendrilsShiver, None);
                    world.play_sound_fine(
                        Sound::EntityWardenTendrilClicks,
                        SoundCategory::Hostile,
                        &pos,
                        5.0,
                        1.0,
                    );
                }
            }
        }
    }

    fn get_base_experience_reward(&self) -> u32 {
        Self::XP_REWARD
    }
}
