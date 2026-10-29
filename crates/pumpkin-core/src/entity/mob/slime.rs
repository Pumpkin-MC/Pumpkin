use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::damage::DamageType;
use pumpkin_data::effect::StatusEffect;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::boundingbox::EntityDimensions;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ageable::{AgeableData, AgeableMob};
use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ai::control::{Control, MoveControlTrait, move_control::MoveControl},
    ai::goal::{Goal, active_target::ActiveTargetGoal, to_goal_ticks},
    mob::{Mob, MobEntity},
};
use crate::world::World;
use pumpkin_util::random::RandomImpl;
use rand::RngExt;

pub struct SlimeEntity {
    entity: Arc<MobEntity>,
    ageable: AgeableData,
    jump_delay: AtomicI32,
    target_yaw: AtomicCell<f32>,
    is_aggressive: AtomicBool,
    was_on_ground: AtomicBool,
    pub squish: AtomicCell<f32>,
    pub target_squish: AtomicCell<f32>,
    pub o_squish: AtomicCell<f32>,
    speed_modifier: AtomicCell<Option<f64>>,
}

impl SlimeEntity {
    pub const MIN_SIZE: i32 = 1;
    pub const MAX_SIZE: i32 = 127;

    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let slime = Self {
            entity: Arc::new(mob_entity),
            ageable: AgeableData::default(),
            jump_delay: AtomicI32::new(0),
            target_yaw: AtomicCell::new(0.0),
            is_aggressive: AtomicBool::new(false),
            was_on_ground: AtomicBool::new(false),
            squish: AtomicCell::new(0.0),
            target_squish: AtomicCell::new(0.0),
            o_squish: AtomicCell::new(0.0),
            speed_modifier: AtomicCell::new(None),
        };
        let mob_arc = Arc::new(slime);
        mob_arc
            .entity
            .navigator
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_can_float(true);

        {
            let mut move_control = mob_arc
                .entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *move_control = Box::new(SlimeMoveControl::new(Arc::downgrade(&mob_arc)));

            let goal_selector = &mob_arc.entity.goals_selector;
            let target_selector = &mob_arc.entity.target_selector;

            goal_selector.add_goal(1, Box::new(SlimeFloatGoal::new(mob_arc.clone())));
            goal_selector.add_goal(2, Box::new(SlimeAttackGoal::new(mob_arc.clone())));
            goal_selector.add_goal(4, Box::new(SlimeRandomDirectionGoal::new(mob_arc.clone())));
            goal_selector.add_goal(5, Box::new(SlimeKeepOnJumpingGoal::new(mob_arc.clone())));

            let weak_cube = Arc::downgrade(&mob_arc);
            target_selector.add_goal(
                1,
                Box::new(ActiveTargetGoal::new(
                    &mob_arc.entity,
                    &EntityType::PLAYER,
                    10,
                    true,
                    false,
                    Some(
                        move |target: &crate::entity::living::LivingEntity, _world: &World| {
                            weak_cube.upgrade().is_some_and(|cube| {
                                (target.entity.pos.load().y - cube.get_entity().pos.load().y).abs()
                                    <= 4.0
                            })
                        },
                    ),
                )),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc.set_size(Self::MIN_SIZE, true);

        mob_arc
    }

    pub fn randomize_size(&self) {
        let world = self.get_entity().world.load();
        let difficulty = crate::entity::mob::equipment::RegionalDifficulty::at(
            &world,
            self.get_entity().pos.load(),
        );
        let mut size_scale = rand::random_range(0..3);
        if size_scale < 2 && rand::random::<f32>() < 0.5 * difficulty.special_multiplier {
            size_scale += 1;
        }
        let size = 1 << size_scale;
        self.set_size(size, true);
    }

    pub fn set_size(&self, size: i32, update_health: bool) {
        let actual_size = size.clamp(Self::MIN_SIZE, Self::MAX_SIZE);
        let entity = &self.entity.living_entity.entity;
        let old_size = entity.data.swap(actual_size, Ordering::Relaxed);

        let living = &self.entity.living_entity;
        living.set_attribute_base(
            &Attributes::MAX_HEALTH,
            f64::from(actual_size * actual_size),
        );
        living.set_attribute_base(
            &Attributes::MOVEMENT_SPEED,
            f64::from(0.2 + 0.1 * actual_size as f32),
        );
        living.set_attribute_base(&Attributes::ATTACK_DAMAGE, f64::from(actual_size));
        if self.is_magma_cube() {
            living.set_attribute_base(&Attributes::ARMOR, f64::from(size.wrapping_mul(3)));
        }

        if update_health {
            let max_health = self
                .entity
                .living_entity
                .get_attribute_value(&Attributes::MAX_HEALTH) as f32;
            self.entity.living_entity.health.store(max_health);
        }

        // Refresh dimensions
        let scaled_dimensions = EntityDimensions {
            width: entity.entity_type.dimension[0] * actual_size as f32,
            height: entity.entity_type.dimension[1] * actual_size as f32,
            eye_height: entity.entity_type.eye_height * actual_size as f32,
        };
        let pos = entity.pos.load();
        entity.set_default_dimensions(scaled_dimensions);
        entity.set_pos(pos);

        if old_size != actual_size {
            entity.yaw.store(entity.head_yaw.load());
            entity.body_yaw.store(entity.head_yaw.load());
        }

        entity.set_synced_data(pumpkin_data::tracked_data::slime::ID_SIZE, actual_size);
        if let Some(meta) = self.mob_metadata() {
            entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
        }
    }

    pub fn get_size(&self) -> i32 {
        self.entity
            .living_entity
            .entity
            .data
            .load(Ordering::Relaxed)
    }

    pub fn is_tiny(&self) -> bool {
        self.get_size() <= 1
    }

    pub fn is_magma_cube(&self) -> bool {
        self.get_entity().entity_type == &EntityType::MAGMA_CUBE
    }

    pub fn read_size(&self, nbt: &NbtCompound) {
        self.set_size(nbt.get_int("Size").unwrap_or(0).wrapping_add(1), false);
    }

    fn set_up_split_cube(&self, child: &Self, half_size: i32, team: Option<&str>) {
        // ConversionType.SPLIT_ON_DEATH copies common state, but not equipment or motion.
        let from = self.get_entity();
        let to = child.get_entity();
        let living = &self.entity.living_entity;
        let child_living = &child.entity.living_entity;
        let effects: Vec<_> = living
            .active_effects
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .collect();
        child_living.set_absorption(living.get_absorption());
        for effect in effects {
            child_living.add_effect(effect);
        }
        child.set_age(self.get_age());
        child.ageable.forced_age.store(
            self.ageable.forced_age.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        child.ageable.forced_age_timer.store(
            self.ageable.forced_age_timer.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        child.entity.set_left_handed(self.entity.is_left_handed());
        child.entity.set_no_ai(self.entity.is_no_ai());
        child.entity.persistence_required.store(
            self.entity.persistence_required.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        if let Some(name) = &**from.custom_name.load() {
            to.set_custom_name(name.clone());
        }
        to.set_custom_name_visible(from.custom_name_visible.load(Ordering::Relaxed));
        to.set_on_fire(from.is_on_fire());
        to.invulnerable
            .store(from.invulnerable.load(Ordering::Relaxed), Ordering::Relaxed);
        child_living.hurt_cooldown.store(
            living.hurt_cooldown.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        to.set_has_no_gravity(from.has_no_gravity());
        to.portal_cooldown.store(
            from.portal_cooldown.load(Ordering::Relaxed),
            Ordering::Relaxed,
        );
        to.set_silent(from.is_silent());
        to.custom_data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone_from(
                &from
                    .custom_data
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
        to.scoreboard_tags
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone_from(
                &from
                    .scoreboard_tags
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner),
            );
        let passenger = from
            .passengers
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .first()
            .cloned();
        if let Some(passenger) = passenger {
            from.remove_passenger(passenger.get_entity().entity_id);
        }
        from.unleash();
        if let Some(team) = team {
            let world = from.world.load();
            let mut scoreboard = world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            scoreboard.add_player_to_team(&*world, team, to.entity_uuid.to_string());
            scoreboard.remove_player_from_team(&*world, team, &from.entity_uuid.to_string());
        }
        child.set_size(half_size, true);
    }

    pub fn jump_from_ground(&self) {
        let living = &self.entity.living_entity;
        let mut velocity = living.entity.velocity.load();
        let size_boost = if self.is_magma_cube() {
            self.get_size() as f32 * 0.1
        } else {
            0.0
        };
        velocity.y = f64::from(living.get_jump_velocity(1.0) as f32 + size_boost);
        living.entity.velocity.store(velocity);
        living.entity.velocity_dirty.store(true, Ordering::Relaxed);
    }

    pub fn jump_in_lava(&self) {
        let entity = self.get_entity();
        let mut velocity = entity.velocity.load();
        velocity.y = f64::from(0.22f32 + self.get_size() as f32 * 0.05);
        entity.velocity.store(velocity);
        entity.velocity_dirty.store(true, Ordering::Relaxed);
    }

    fn can_deal_damage(&self) -> bool {
        (self.is_magma_cube() || !self.is_tiny()) && !self.entity.is_no_ai()
    }

    pub fn deal_damage(&self, caller: &dyn Mob, target: &dyn EntityBase) {
        let living = &self.entity.living_entity;
        if !self.can_deal_damage()
            || !living.is_alive()
            || !(caller as &dyn EntityBase).do_teams_allow_damage(target)
            || !self.entity.is_in_attack_range(target)
            || !caller.has_line_of_sight(target.get_entity())
        {
            return;
        }
        let damage = living.get_attribute_value(&Attributes::ATTACK_DAMAGE) as f32
            + if self.is_magma_cube() { 2.0 } else { 0.0 };
        if target.damage_with_context(
            target,
            damage,
            DamageType::MOB_ATTACK,
            None,
            Some(caller),
            Some(caller),
        ) {
            self.play_sound(
                Sound::EntitySlimeAttack,
                1.0,
                (rand::random::<f32>() - rand::random::<f32>()) * 0.2 + 1.0,
            );
            crate::enchantment::EnchantmentHelper::on_post_attack(
                self.get_entity(),
                target.get_entity(),
                &living.held_item(caller),
            );
        }
    }

    pub fn check_slime_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }

        // TODO: check spawn reason. if it's spawner, we should return true if block below is valid
        // For now, we assume natural spawning as that's what we are implementing.

        // Swamp/Surface Spawning
        // TODO: fix
        // let biome = world.get_biome(pos);
        // if biome.has_tag(&pumpkin_data::tag::WorldgenBiome::MINECRAFT_ALLOWS_SURFACE_SLIME_SPAWNS)
        //     && pos.0.y > 50
        //     && pos.0.y < 70
        // {
        //     let time = world.level_time.lock().await.time_of_day;
        //     let moon_phase = (time / 24000) % 8;
        //     let surface_slime_spawn_chance = Self::get_spawn_chance(moon_phase);
        //     let mut rng = rand::rng();
        //     if rng.random::<f32>() < surface_slime_spawn_chance
        //         && world.get_max_local_raw_brightness(pos) <= rng.random_range(0..8)
        //     {
        //         return true;
        //     }
        // }

        // Slime Chunk Spawning
        let chunk_pos = pos.chunk_position();
        let world_seed = world.level.seed.0;
        let slime_seed = pumpkin_util::random::seed_slime_chunk(
            chunk_pos.x,
            chunk_pos.y,
            world_seed,
            987_234_911,
        );
        let mut slime_rand = pumpkin_util::random::legacy_rand::LegacyRand::from_seed(slime_seed);

        let mut rng = rand::rng();
        if rng.random_range(0..10) == 0 && slime_rand.next_bounded_i32(10) == 0 && pos.0.y < 40 {
            return true;
        }

        false
    }

    // const fn get_spawn_chance(moon_phase: i64) -> f32 {
    //     match moon_phase {
    //         0 => 1.0,
    //         1 | 7 => 0.75,
    //         2 | 6 => 0.5,
    //         3 | 5 => 0.25,
    //         _ => 0.0,
    //     }
    // }

    fn get_jump_delay(&self) -> i32 {
        rand::random_range(10..30) * if self.is_magma_cube() { 4 } else { 1 }
    }

    fn do_play_jump_sound(&self) -> bool {
        self.get_size() > 0
    }

    fn get_jump_sound(&self) -> Sound {
        if self.is_magma_cube() {
            Sound::EntityMagmaCubeJump
        } else if self.is_tiny() {
            Sound::EntitySlimeJumpSmall
        } else {
            Sound::EntitySlimeJump
        }
    }

    fn get_squish_sound(&self) -> Sound {
        if self.is_magma_cube() {
            if self.is_tiny() {
                Sound::EntityMagmaCubeSquishSmall
            } else {
                Sound::EntityMagmaCubeSquish
            }
        } else if self.is_tiny() {
            Sound::EntitySlimeSquishSmall
        } else {
            Sound::EntitySlimeSquish
        }
    }

    pub(crate) fn play_sound(&self, sound: Sound, volume: f32, pitch: f32) {
        let entity = self.get_entity();
        if !entity.is_silent() {
            entity.world.load().play_sound_fine(
                sound,
                SoundCategory::Hostile,
                &entity.pos.load(),
                volume,
                pitch,
            );
        }
    }

    pub(crate) fn get_sound_volume(&self) -> f32 {
        0.4 * self.get_size() as f32
    }

    pub(crate) fn get_sound_pitch(&self) -> f32 {
        let pitch_adjuster = if self.is_tiny() { 1.4 } else { 0.8 };
        ((rand::random::<f32>() - rand::random::<f32>()) * 0.2 + 1.0) * pitch_adjuster
    }
}

impl CustomSound for SlimeEntity {
    fn sound_volume(&self) -> f32 {
        self.get_sound_volume()
    }

    fn sound_category(&self) -> SoundCategory {
        SoundCategory::Hostile
    }

    fn death_sound(&self) -> Option<Sound> {
        let size = self.get_size();
        Some(if size == 1 {
            Sound::EntitySlimeDeathSmall
        } else {
            Sound::EntitySlimeDeath
        })
    }
    fn hurt_sound(&self) -> Option<Sound> {
        let size = self.get_size();
        Some(if size == 1 {
            Sound::EntitySlimeHurtSmall
        } else {
            Sound::EntitySlimeHurt
        })
    }
}

impl AgeableMob for SlimeEntity {
    fn get_ageable_data(&self) -> &AgeableData {
        &self.ageable
    }

    fn can_be_a_baby(&self) -> bool {
        false
    }

    fn default_should_spawn_baby(&self) -> bool {
        false
    }
}

impl Mob for SlimeEntity {
    fn as_cube_mob(&self) -> Option<&Self> {
        Some(self)
    }

    fn as_ageable(&self) -> Option<&dyn AgeableMob> {
        Some(self)
    }

    fn spawn_as_baby(&self) -> bool {
        false
    }

    fn get_max_look_pitch_change(&self) -> f32 {
        0.0
    }

    fn get_base_experience_reward(&self) -> u32 {
        self.get_size() as u32
    }

    fn finalize_spawn(
        &self,
        _world: &Arc<World>,
        group_data: Option<super::spawn::SpawnGroupData>,
    ) -> Option<super::spawn::SpawnGroupData> {
        let group_data = self.finalize_ageable_spawn(group_data);
        self.randomize_size();
        group_data
    }

    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Size", self.get_size() - 1);
        nbt.put_bool("wasOnGround", self.was_on_ground.load(Ordering::Relaxed));
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.was_on_ground.store(
            nbt.get_bool("wasOnGround").unwrap_or(false),
            Ordering::Relaxed,
        );
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        Some(crate::entity::mob::MobMetadata::new().slime_size(self.get_size()))
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        if self.get_entity().world.load().level_info.load().difficulty == Difficulty::Peaceful {
            self.get_entity().remove();
            return;
        }
        self.o_squish.store(self.squish.load());
        self.squish
            .store(self.squish.load() + (self.target_squish.load() - self.squish.load()) * 0.5);
    }

    fn post_tick(&self) {
        let on_ground = self
            .entity
            .living_entity
            .entity
            .on_ground
            .load(Ordering::Relaxed);
        let was_on_ground = self.was_on_ground.load(Ordering::Relaxed);

        if on_ground && !was_on_ground {
            // Landing particles are generated by the Java client's AbstractCubeMob.tick.
            self.play_sound(
                self.get_squish_sound(),
                self.get_sound_volume(),
                ((rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0) / 0.8,
            );

            self.target_squish.store(-0.5);
        } else if !on_ground && was_on_ground {
            self.target_squish.store(1.0);
        }

        self.was_on_ground.store(on_ground, Ordering::Relaxed);
        self.target_squish
            .store(self.target_squish.load() * if self.is_magma_cube() { 0.9 } else { 0.6 });
    }

    fn mob_player_collision(&self, player: &Arc<crate::entity::player::Player>) {
        self.deal_damage(self, &**player);
    }

    fn on_removed(&self) {
        if self.entity.living_entity.is_dead_or_dying() && self.get_size() > 1 {
            let size = self.get_size();
            let world = self.entity.living_entity.entity.world.load();
            let pos = self.entity.living_entity.entity.pos.load();
            let half_size = size / 2;
            let count = 2 + rand::random_range(0..3);
            let team = self.get_team_name();
            let mut event = crate::plugin::api::events::entity::slime_split::SlimeSplitEvent {
                entity_id: self.get_entity().entity_id,
                count,
                cancelled: false,
            };
            if let Some(server) = world.server.upgrade() {
                server.plugin_manager.fire_blocking(&server, &mut event);
            }
            if event.cancelled {
                return;
            }

            let width = self
                .entity
                .living_entity
                .entity
                .entity_dimension
                .load()
                .width;
            let xz_offset = width / 2.0;

            for i in 0..event.count.max(0) {
                let xd = ((i % 2) as f32 - 0.5) * xz_offset;
                let zd = ((i / 2) as f32 - 0.5) * xz_offset;

                let new_pos = pumpkin_util::math::vector3::Vector3::new(
                    pos.x + xd as f64,
                    pos.y + 0.5,
                    pos.z + zd as f64,
                );
                let child = crate::entity::r#type::from_type(
                    self.get_entity().entity_type,
                    new_pos,
                    &world,
                    uuid::Uuid::new_v4(),
                );
                if let Some(cube) = child.get_mob().and_then(|mob| mob.as_cube_mob()) {
                    self.set_up_split_cube(cube, half_size, team.as_deref());
                }
                child
                    .get_entity()
                    .set_rotation(rand::random_range(0.0..360.0), 0.0);
                world.spawn_entity(child);
            }
        }
    }
}

pub struct SlimeMoveControl {
    slime: Weak<SlimeEntity>,
}

impl SlimeMoveControl {
    #[must_use]
    pub const fn new(slime: Weak<SlimeEntity>) -> Self {
        Self { slime }
    }
}

impl Control for SlimeMoveControl {}

impl MoveControlTrait for SlimeMoveControl {
    fn set_wanted_position(&mut self, _x: f64, _y: f64, _z: f64, speed_modifier: f64) {
        if let Some(slime) = self.slime.upgrade() {
            slime.speed_modifier.store(Some(speed_modifier));
        }
    }

    fn strafe(&mut self, _forward: f32, _right: f32) {
        if let Some(slime) = self.slime.upgrade() {
            slime.speed_modifier.store(None);
        }
    }

    fn has_wanted(&self) -> bool {
        self.slime
            .upgrade()
            .is_some_and(|slime| slime.speed_modifier.load().is_some())
    }

    fn tick(&mut self, mob: &dyn Mob) {
        let Some(slime) = self.slime.upgrade() else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let living_entity = &mob_entity.living_entity;
        let entity = &living_entity.entity;

        let current_yaw = entity.yaw.load();
        let new_yaw =
            MoveControl::rotlerp(current_yaw, slime.target_yaw.load(), MoveControl::MAX_TURN);
        entity.yaw.store(new_yaw);
        entity.head_yaw.store(new_yaw);
        entity.body_yaw.store(new_yaw);

        let Some(speed_modifier) = slime.speed_modifier.take() else {
            let input = living_entity.movement_input.load();
            living_entity
                .movement_input
                .store(Vector3::new(input.x, input.y, 0.0));
            return;
        };
        mob_entity.set_speed(
            (speed_modifier * living_entity.get_attribute_value(&Attributes::MOVEMENT_SPEED))
                as f32,
        );
        if entity.on_ground.load(Ordering::Relaxed) {
            let delay = slime.jump_delay.fetch_sub(1, Ordering::Relaxed);
            if delay <= 0 {
                let mut next_delay = slime.get_jump_delay();
                if slime.is_aggressive.load(Ordering::Relaxed) {
                    next_delay /= 3;
                }
                slime.jump_delay.store(next_delay, Ordering::Relaxed);
                mob_entity
                    .jump_control
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .jump();
                if slime.do_play_jump_sound() {
                    slime.play_sound(
                        slime.get_jump_sound(),
                        slime.get_sound_volume(),
                        slime.get_sound_pitch(),
                    );
                }
            } else {
                mob_entity.set_speed(0.0);
                let input = living_entity.movement_input.load();
                living_entity
                    .movement_input
                    .store(Vector3::new(0.0, input.y, 0.0));
            }
        }
    }
}

pub struct SlimeFloatGoal {
    slime: Arc<SlimeEntity>,
}

impl SlimeFloatGoal {
    pub const fn new(slime: Arc<SlimeEntity>) -> Self {
        Self { slime }
    }
}

impl Goal for SlimeFloatGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        let entity = &self.slime.entity.living_entity.entity;
        entity.touching_water.load(Ordering::Relaxed)
            || entity.touching_lava.load(Ordering::Relaxed)
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        if rand::random_range(0.0..1.0) < 0.8 {
            self.slime
                .entity
                .jump_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .jump();
        }
        self.slime.speed_modifier.store(Some(1.2));
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::JUMP | crate::entity::ai::goal::Controls::MOVE
    }
}

pub struct SlimeAttackGoal {
    slime: Arc<SlimeEntity>,
    grow_tired_timer: i32,
}

impl SlimeAttackGoal {
    pub const fn new(slime: Arc<SlimeEntity>) -> Self {
        Self {
            slime,
            grow_tired_timer: 0,
        }
    }
}

impl Goal for SlimeAttackGoal {
    fn can_start(&mut self, mob: &dyn Mob) -> bool {
        self.slime
            .entity
            .get_target()
            .is_some_and(|target| mob.can_attack(&*target))
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.grow_tired_timer = to_goal_ticks(300);
    }

    fn should_continue(&mut self, mob: &dyn Mob) -> bool {
        self.grow_tired_timer -= 1;
        self.can_start(mob) && self.grow_tired_timer > 0
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        if let Some(target) = self.slime.entity.get_target() {
            let entity = self.slime.get_entity();
            let pos = target.get_entity().pos.load();
            let my_pos = entity.pos.load();
            let dx = pos.x - my_pos.x;
            let dz = pos.z - my_pos.z;
            let dy = target.get_entity().get_eye_y() - entity.get_eye_y();
            let yaw = MoveControl::rotlerp(
                entity.yaw.load(),
                dz.atan2(dx).to_degrees() as f32 - 90.0,
                10.0,
            );
            entity.set_pitch(MoveControl::rotlerp(
                entity.pitch.load(),
                -dy.atan2(dx.hypot(dz)).to_degrees() as f32,
                10.0,
            ));
            entity.yaw.store(yaw);
            self.slime.target_yaw.store(yaw);
        }
        self.slime
            .is_aggressive
            .store(self.slime.can_deal_damage(), Ordering::Relaxed);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::LOOK
    }
}

pub struct SlimeRandomDirectionGoal {
    slime: Arc<SlimeEntity>,
    chosen_degrees: f32,
    next_randomize_time: i32,
}

impl SlimeRandomDirectionGoal {
    pub const fn new(slime: Arc<SlimeEntity>) -> Self {
        Self {
            slime,
            chosen_degrees: 0.0,
            next_randomize_time: 0,
        }
    }
}

impl Goal for SlimeRandomDirectionGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.slime.entity.get_target().is_none()
            && (self
                .slime
                .entity
                .living_entity
                .entity
                .on_ground
                .load(Ordering::Relaxed)
                || self
                    .slime
                    .entity
                    .living_entity
                    .entity
                    .touching_water
                    .load(Ordering::Relaxed)
                || self
                    .slime
                    .entity
                    .living_entity
                    .entity
                    .touching_lava
                    .load(Ordering::Relaxed)
                || self
                    .slime
                    .entity
                    .living_entity
                    .has_effect(&StatusEffect::LEVITATION))
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.next_randomize_time -= 1;
        if self.next_randomize_time <= 0 {
            self.next_randomize_time = self.get_tick_count(rand::random_range(40..100));
            self.chosen_degrees = rand::random_range(0..360) as f32;
        }
        self.slime.target_yaw.store(self.chosen_degrees);
        self.slime.is_aggressive.store(false, Ordering::Relaxed);
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::LOOK
    }
}

pub struct SlimeKeepOnJumpingGoal {
    slime: Arc<SlimeEntity>,
}

impl SlimeKeepOnJumpingGoal {
    #[must_use]
    pub const fn new(slime: Arc<SlimeEntity>) -> Self {
        Self { slime }
    }
}

impl Goal for SlimeKeepOnJumpingGoal {
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        !self.slime.entity.living_entity.entity.has_vehicle()
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.slime.speed_modifier.store(Some(1.0));
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::JUMP | crate::entity::ai::goal::Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;

    #[tokio::test]
    async fn magma_move_control_and_squish_use_magma_timing()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let cube = SlimeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::MAGMA_CUBE,
        ));
        cube.get_entity().on_ground.store(true, Ordering::Relaxed);
        cube.speed_modifier.store(Some(1.0));
        cube.entity
            .move_control
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .tick(&*cube);
        let delay = cube.jump_delay.load(Ordering::Relaxed);
        assert!((40..120).contains(&delay));
        assert_eq!(delay % 4, 0);
        cube.mob_tick(&*cube);
        cube.post_tick();
        assert_eq!(cube.target_squish.load(), -0.45);
        cube.get_entity().on_ground.store(false, Ordering::Relaxed);
        cube.mob_tick(&*cube);
        cube.post_tick();
        assert_eq!(cube.target_squish.load(), 0.9);
        world.level.shutdown().await;
        Ok(())
    }

    #[tokio::test]
    async fn zero_speed_requests_are_consumed_as_movement_operations()
    -> Result<(), Box<dyn std::error::Error>> {
        let folder = tempfile::tempdir()?;
        let world = world(&folder);
        let slime = SlimeEntity::new(Entity::new(
            world.clone(),
            Vector3::new(0.0, 64.0, 0.0),
            &EntityType::SLIME,
        ));
        let living = &slime.entity.living_entity;
        living.entity.on_ground.store(true, Ordering::Relaxed);
        slime.jump_delay.store(10, Ordering::Relaxed);
        slime.entity.set_speed(0.5);
        let (speed, delay, pending) = {
            let mut control = slime
                .entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            control.set_wanted_position(0.0, 64.0, 8.0, 0.0);
            control.tick(&*slime);
            (
                living.get_speed(),
                slime.jump_delay.load(Ordering::Relaxed),
                control.has_wanted(),
            )
        };
        slime.entity.set_speed(0.5);
        let waiting_speed = {
            let mut control = slime
                .entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            control.tick(&*slime);
            living.get_speed()
        };
        world.level.shutdown().await;
        assert_eq!((speed, delay, pending), (0.0, 9, false));
        assert_eq!(waiting_speed, 0.5);
        assert_eq!(living.movement_input.load().z, 0.0);
        Ok(())
    }
}
