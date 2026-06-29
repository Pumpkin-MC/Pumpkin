use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Arc, Weak};

use crossbeam::atomic::AtomicCell;
use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::boundingbox::{BoundingBox, EntityDimensions};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::{
    Entity, EntityBase, NBTStorage, NbtFuture,
    ai::control::{Control, MoveControlTrait},
    ai::goal::{Goal, GoalFuture, active_target::ActiveTargetGoal},
    mob::{Mob, MobEntity},
};
use crate::world::World;
use pumpkin_util::random::RandomImpl;
use rand::RngExt;

pub struct SlimeEntity {
    entity: Arc<MobEntity>,
    jump_delay: AtomicI32,
    target_yaw: AtomicCell<f32>,
    is_aggressive: AtomicBool,
    was_on_ground: AtomicBool,
    pub squish: AtomicCell<f32>,
    pub target_squish: AtomicCell<f32>,
    pub o_squish: AtomicCell<f32>,
    speed_modifier: AtomicCell<f64>,
    has_split: AtomicBool,
}

impl SlimeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let slime = Self {
            entity: Arc::new(mob_entity),
            jump_delay: AtomicI32::new(0),
            target_yaw: AtomicCell::new(0.0),
            is_aggressive: AtomicBool::new(false),
            was_on_ground: AtomicBool::new(false),
            squish: AtomicCell::new(0.0),
            target_squish: AtomicCell::new(0.0),
            o_squish: AtomicCell::new(0.0),
            speed_modifier: AtomicCell::new(0.0),
            has_split: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(slime);

        {
            let mut move_control = mob_arc.entity.move_control.lock().unwrap();
            *move_control = Box::new(SlimeMoveControl::new(Arc::downgrade(&mob_arc)));

            let mut goal_selector = mob_arc.entity.goals_selector.lock().unwrap();
            let mut target_selector = mob_arc.entity.target_selector.lock().unwrap();

            goal_selector.add_goal(1, Box::new(SlimeFloatGoal::new(mob_arc.clone())));
            goal_selector.add_goal(2, Box::new(SlimeAttackGoal::new(mob_arc.clone())));
            goal_selector.add_goal(3, Box::new(SlimeRandomDirectionGoal::new(mob_arc.clone())));
            goal_selector.add_goal(5, Box::new(SlimeKeepOnJumpingGoal::new(mob_arc.clone())));

            target_selector.add_goal(
                1,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::PLAYER, true),
            );
            target_selector.add_goal(
                3,
                ActiveTargetGoal::with_default(&mob_arc.entity, &EntityType::IRON_GOLEM, true),
            );
        };

        mob_arc.randomize_size();

        mob_arc
    }

    pub fn randomize_size(&self) {
        let mut size_scale = rand::random_range(0..3);
        if size_scale < 2 && rand::random_range(0.0..1.0) < 0.5 {
            size_scale += 1;
        }
        let size = 1 << size_scale;
        self.set_size(size, true);
    }

    pub fn set_size(&self, size: i32, update_health: bool) {
        let actual_size = size.clamp(1, 127);
        let entity = &self.entity.living_entity.entity;
        entity.data.store(actual_size, Ordering::Relaxed);

        // Update attributes
        {
            let mut attributes = self.entity.living_entity.attributes.write().unwrap();
            if let Some(health) = attributes.get_mut(&Attributes::MAX_HEALTH.id) {
                health.base_value = (actual_size * actual_size) as f64;
                health.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(speed) = attributes.get_mut(&Attributes::MOVEMENT_SPEED.id) {
                speed.base_value = (0.2 + 0.1 * actual_size as f32) as f64;
                speed.dirty.store(true, Ordering::Relaxed);
            }
            if let Some(damage) = attributes.get_mut(&Attributes::ATTACK_DAMAGE.id) {
                damage.base_value = actual_size as f64;
                damage.dirty.store(true, Ordering::Relaxed);
            }
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
        entity.entity_dimension.store(scaled_dimensions);

        let pos = entity.pos.load();
        let new_bb = BoundingBox::new_from_pos(pos.x, pos.y, pos.z, &scaled_dimensions);
        entity.bounding_box.store(new_bb);
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

    pub(crate) const fn hurt_sound_for_size(size: i32) -> Sound {
        if size == 1 {
            Sound::EntitySlimeHurtSmall
        } else {
            Sound::EntitySlimeHurt
        }
    }

    fn get_jump_delay() -> i32 {
        rand::random_range(10..30)
    }

    fn rot_lerp(start: f32, end: f32, max_step: f32) -> f32 {
        let mut diff = (end - start).rem_euclid(360.0);
        if diff > 180.0 {
            diff -= 360.0;
        }
        start + diff.clamp(-max_step, max_step)
    }

    fn do_play_jump_sound(&self) -> bool {
        self.get_size() > 0
    }

    fn get_jump_sound(&self) -> Sound {
        if self.is_tiny() {
            Sound::EntitySlimeJumpSmall
        } else {
            Sound::EntitySlimeJump
        }
    }

    fn get_squish_sound(&self) -> Sound {
        if self.is_tiny() {
            Sound::EntitySlimeSquishSmall
        } else {
            Sound::EntitySlimeSquish
        }
    }

    fn get_sound_volume(&self) -> f32 {
        0.4 * self.get_size() as f32
    }

    fn get_sound_pitch(&self) -> f32 {
        let pitch_adjuster = if self.is_tiny() { 1.4 } else { 0.8 };
        (rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0 * pitch_adjuster
    }
}

impl NBTStorage for SlimeEntity {
    fn write_nbt<'a>(&'a self, nbt: &'a mut NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.living_entity.write_nbt(nbt).await;
            nbt.put_int("Size", self.get_size() - 1);
            nbt.put_bool("wasOnGround", self.was_on_ground.load(Ordering::Relaxed));
        })
    }

    fn read_nbt_non_mut<'a>(&'a self, nbt: &'a NbtCompound) -> NbtFuture<'a, ()> {
        Box::pin(async move {
            self.entity.living_entity.read_nbt_non_mut(nbt).await;
            self.set_size(nbt.get_int("Size").unwrap_or(0) + 1, false);
            self.was_on_ground.store(
                nbt.get_bool("wasOnGround").unwrap_or(false),
                Ordering::Relaxed,
            );
        })
    }
}

impl Mob for SlimeEntity {
    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    fn mob_tick<'a>(
        &'a self,
        _caller: &'a Arc<dyn EntityBase>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            self.o_squish.store(self.squish.load());
            self.squish
                .store(self.squish.load() + (self.target_squish.load() - self.squish.load()) * 0.5);

            let on_ground = self
                .entity
                .living_entity
                .entity
                .on_ground
                .load(Ordering::Relaxed);
            let was_on_ground = self.was_on_ground.load(Ordering::Relaxed);

            if on_ground && !was_on_ground {
                // TODO: particles

                let world = self.entity.living_entity.entity.world.load();
                world.play_sound_fine(
                    self.get_squish_sound(),
                    SoundCategory::Hostile,
                    &self.entity.living_entity.entity.pos.load(),
                    self.get_sound_volume(),
                    ((rand::random_range(0.0..1.0) - rand::random_range(0.0..1.0)) * 0.2 + 1.0)
                        / 0.8,
                );

                self.target_squish.store(-0.5);
            } else if !on_ground && was_on_ground {
                self.target_squish.store(1.0);
            }

            self.was_on_ground.store(on_ground, Ordering::Relaxed);
            self.target_squish.store(self.target_squish.load() * 0.6);

            self.is_aggressive.store(false, Ordering::Relaxed);
            self.speed_modifier.store(0.0);
        })
    }

    fn mob_player_collision<'a>(
        &'a self,
        player: &'a Arc<crate::entity::player::Player>,
    ) -> crate::entity::EntityBaseFuture<'a, ()> {
        Box::pin(async move {
            if !self.is_tiny() {
                // dealDamage
                self.entity.try_attack(self, &**player).await;
            }
        })
    }

    fn post_tick(&self) -> crate::entity::EntityBaseFuture<'_, ()> {
        Box::pin(async move {
            if self.entity.living_entity.dead.load(Ordering::Relaxed)
                && self.get_size() > 1
                && self
                    .has_split
                    .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
                    .is_ok()
            {
                let size = self.get_size();
                let world = self.entity.living_entity.entity.world.load();
                let pos = self.entity.living_entity.entity.pos.load();
                let half_size = size / 2;
                let count = 2 + rand::random_range(0..3);

                let width = self
                    .entity
                    .living_entity
                    .entity
                    .entity_dimension
                    .load()
                    .width;
                let xz_offset = width / 4.0;

                for i in 0..count {
                    let xd = ((i % 2) as f32 - 0.5) * xz_offset;
                    let zd = ((i / 2) as f32 - 0.5) * xz_offset;

                    let new_pos = pumpkin_util::math::vector3::Vector3::new(
                        pos.x + xd as f64,
                        pos.y + 0.5,
                        pos.z + zd as f64,
                    );
                    let new_entity = Entity::new(world.clone(), new_pos, &EntityType::SLIME);
                    let slime = Self::new(new_entity);
                    slime.set_size(half_size, true);
                    slime
                        .entity
                        .living_entity
                        .entity
                        .yaw
                        .store(rand::random_range(0.0..360.0));
                    world.spawn_entity(slime).await;
                }
            }
        })
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
    fn tick(&mut self, mob: &dyn Mob) {
        let Some(slime) = self.slime.upgrade() else {
            return;
        };
        let mob_entity = mob.get_mob_entity();
        let living_entity = &mob_entity.living_entity;
        let entity = &living_entity.entity;

        let current_yaw = entity.yaw.load();
        let new_yaw = SlimeEntity::rot_lerp(current_yaw, slime.target_yaw.load(), 90.0);
        entity.yaw.store(new_yaw);
        entity.head_yaw.store(new_yaw);
        entity.body_yaw.store(new_yaw);

        let speed_modifier = slime.speed_modifier.load();
        let mut movement_input = Vector3::new(0.0, 0.0, 0.0);

        let on_ground = entity.on_ground.load(Ordering::Relaxed);

        if on_ground {
            if speed_modifier > 0.0 {
                let current_delay = slime.jump_delay.load(Ordering::Relaxed);
                if current_delay <= 0 {
                    // Start jump
                    let mut next_delay = SlimeEntity::get_jump_delay();
                    if slime.is_aggressive.load(Ordering::Relaxed) {
                        next_delay /= 3;
                    }
                    slime.jump_delay.store(next_delay, Ordering::Relaxed);
                    living_entity.jumping.store(true, Ordering::SeqCst);
                    if slime.do_play_jump_sound() {
                        let world = entity.world.load();
                        world.play_sound_fine(
                            slime.get_jump_sound(),
                            SoundCategory::Hostile,
                            &entity.pos.load(),
                            slime.get_sound_volume(),
                            slime.get_sound_pitch(),
                        );
                    }
                    movement_input.z = speed_modifier;
                } else {
                    slime.jump_delay.store(current_delay - 1, Ordering::Relaxed);
                    living_entity.jumping.store(false, Ordering::SeqCst);
                }
            } else {
                living_entity.jumping.store(false, Ordering::SeqCst);
            }
        } else {
            // In air: move forward but don't "jump" again
            if speed_modifier > 0.0 {
                movement_input.z = speed_modifier;
            }
            living_entity.jumping.store(false, Ordering::SeqCst);
        }
        living_entity.movement_input.store(movement_input);
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
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let entity = &self.slime.entity.living_entity.entity;
            entity.touching_water.load(Ordering::Relaxed)
                || entity.touching_lava.load(Ordering::Relaxed)
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            if rand::random_range(0.0..1.0) < 0.8 {
                self.slime
                    .entity
                    .living_entity
                    .jumping
                    .store(true, Ordering::SeqCst);
            }
            self.slime.speed_modifier.store(1.2);
        })
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
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = self.slime.entity.target.lock().await;
            target.is_some()
        })
    }

    fn start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.grow_tired_timer = 300;
        })
    }

    fn should_continue<'a>(&'a self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = self.slime.entity.target.lock().await;
            target.is_some() && self.grow_tired_timer > 0
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.grow_tired_timer -= 1;
            let target_guard = self.slime.entity.target.lock().await;
            if let Some(target) = target_guard.as_ref() {
                let pos = target.get_entity().pos.load();
                let my_pos = self.slime.entity.living_entity.entity.pos.load();
                let dx = pos.x - my_pos.x;
                let dz = pos.z - my_pos.z;
                let yaw = dx.atan2(dz).to_degrees() as f32;
                self.slime.target_yaw.store(yaw);
            }
            self.slime.is_aggressive.store(true, Ordering::Relaxed);
        })
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
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let target = self.slime.entity.target.lock().await;
            target.is_none()
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
                        .load(Ordering::Relaxed))
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.next_randomize_time -= 1;
            if self.next_randomize_time <= 0 {
                self.next_randomize_time = rand::random_range(40..100);
                self.chosen_degrees = rand::random_range(0.0..360.0);
            }
            self.slime.target_yaw.store(self.chosen_degrees);
            self.slime.is_aggressive.store(false, Ordering::Relaxed);
        })
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::LOOK
    }
}

pub struct SlimeKeepOnJumpingGoal {
    slime: Arc<SlimeEntity>,
}

impl SlimeKeepOnJumpingGoal {
    pub const fn new(slime: Arc<SlimeEntity>) -> Self {
        Self { slime }
    }
}

impl Goal for SlimeKeepOnJumpingGoal {
    fn can_start<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, bool> {
        Box::pin(async move {
            let vehicle = self.slime.entity.living_entity.entity.vehicle.lock().await;
            vehicle.is_none()
        })
    }

    fn tick<'a>(&'a mut self, _mob: &'a dyn Mob) -> GoalFuture<'a, ()> {
        Box::pin(async move {
            self.slime.speed_modifier.store(1.0);
        })
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::JUMP | crate::entity::ai::goal::Controls::MOVE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_small_hurt_sound_only_for_smallest_slimes() {
        assert_eq!(
            SlimeEntity::hurt_sound_for_size(1),
            Sound::EntitySlimeHurtSmall
        );
        assert_eq!(SlimeEntity::hurt_sound_for_size(0), Sound::EntitySlimeHurt);
        assert_eq!(SlimeEntity::hurt_sound_for_size(2), Sound::EntitySlimeHurt);
    }
}
