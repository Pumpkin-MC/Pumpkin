use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::boundingbox::EntityDimensions;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;

use crate::entity::ai::behavior::hunting::{self, Hunt};
use crate::entity::ai::target_match::TargetMatch;
use crate::entity::custom_sound::CustomSound;
use crate::entity::{
    Entity, EntityBase,
    ai::goal::Goal,
    mob::{
        Mob, MobEntity,
        cube_mob::{
            CubeMob, CubeMobData, CubeMobFloatGoal, CubeMobKeepOnJumpingGoal, CubeMobMoveControl,
            CubeMobRandomDirectionGoal,
        },
        spawn::EntitySpawnReason,
    },
};
use crate::world::World;
use pumpkin_util::random::RandomImpl;
use rand::RngExt;

const PASSENGER_SINK_PER_SIZE: f64 = 0.015_625;

const SLIME_HUNTS: &[Hunt] = &[
    Hunt::new(1, TargetMatch::Type(&EntityType::PLAYER), true)
        .matching(hunting::within_four_blocks_height),
    Hunt::new(3, TargetMatch::Type(&EntityType::IRON_GOLEM), true),
];

pub struct SlimeEntity {
    entity: Arc<MobEntity>,
    cube: CubeMobData,
    has_split: AtomicBool,
}

impl SlimeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let mob_entity = MobEntity::new(entity);
        let slime = Self {
            entity: Arc::new(mob_entity),
            cube: CubeMobData::default(),
            has_split: AtomicBool::new(false),
        };
        let mob_arc = Arc::new(slime);
        let weak = {
            let cube: Arc<dyn CubeMob> = mob_arc.clone();
            Arc::downgrade(&cube)
        };

        {
            let mut move_control = mob_arc
                .entity
                .move_control
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *move_control = Box::new(CubeMobMoveControl::new(weak.clone()));

            let goal_selector = &mob_arc.entity.goals_selector;
            let target_selector = &mob_arc.entity.target_selector;

            goal_selector.add_goal(1, Box::new(CubeMobFloatGoal::new(weak.clone())));
            goal_selector.add_goal(2, Box::new(SlimeAttackGoal::new(mob_arc.clone())));
            goal_selector.add_goal(3, Box::new(CubeMobRandomDirectionGoal::new(weak.clone())));
            goal_selector.add_goal(5, Box::new(CubeMobKeepOnJumpingGoal::new(weak)));

            hunting::apply(target_selector, &mob_arc.entity, SLIME_HUNTS);
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
            let mut attributes = self
                .entity
                .living_entity
                .attributes
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        entity.set_default_dimensions(scaled_dimensions);

        entity.set_synced_data(pumpkin_data::tracked_data::slime::ID_SIZE, actual_size);
        if let Some(meta) = self.mob_metadata() {
            entity.send_bedrock_actor_data(&meta.to_bedrock_actor_data());
        }
    }

    /// Vanilla `AbstractCubeMob.dealDamage`: touching isn't enough, the player must be in
    /// melee range and in sight. Sight is checked fresh, not through the sensing cache, which
    /// `NoAI` cubes never clear.
    pub fn deal_damage(&self, player: &crate::entity::player::Player) {
        if self.entity.is_in_attack_range(player)
            && self.get_entity().has_line_of_sight(player.get_entity())
            && self.entity.try_attack(self, player)
        {
            self.on_attack(player);
        }
    }

    pub fn get_size(&self) -> i32 {
        CubeMob::get_size(self)
    }

    pub fn is_tiny(&self) -> bool {
        CubeMob::is_tiny(self)
    }

    pub fn check_slime_spawn_rules(
        entity_type: &'static EntityType,
        world: &World,
        pos: &BlockPos,
        reason: EntitySpawnReason,
    ) -> bool {
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }
        if reason.is_spawner() {
            return MobEntity::check_mob_spawn_rules(entity_type, world, pos, reason);
        }

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
            return MobEntity::check_mob_spawn_rules(entity_type, world, pos, reason);
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
}

impl CubeMob for SlimeEntity {
    fn cube_data(&self) -> &CubeMobData {
        &self.cube
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
}

impl CustomSound for SlimeEntity {
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

impl Mob for SlimeEntity {
    fn xp_reward(&self) -> u32 {
        u32::try_from(self.get_size()).unwrap_or(0)
    }

    fn as_custom_sound(&self) -> Option<&dyn crate::entity::custom_sound::CustomSound> {
        Some(self)
    }

    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        nbt.put_int("Size", self.get_size() - 1);
        nbt.put_bool(
            "wasOnGround",
            self.cube.was_on_ground.load(Ordering::Relaxed),
        );
    }

    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.set_size(nbt.get_int("Size").unwrap_or(0) + 1, false);
        self.cube.was_on_ground.store(
            nbt.get_bool("wasOnGround").unwrap_or(false),
            Ordering::Relaxed,
        );
    }

    fn get_mob_entity(&self) -> &MobEntity {
        &self.entity
    }

    fn mob_passenger_attachment_point(
        &self,
        _passenger: &dyn EntityBase,
        _index: usize,
    ) -> Option<Vector3<f64>> {
        let living = &self.entity.living_entity;
        let scale = living.get_attribute_value(&Attributes::SCALE);
        Some(Vector3::new(
            0.0,
            f64::from(living.entity.height())
                - PASSENGER_SINK_PER_SIZE * f64::from(self.get_size()) * scale,
            0.0,
        ))
    }

    fn mob_metadata(&self) -> Option<crate::entity::mob::MobMetadata> {
        Some(crate::entity::mob::MobMetadata::new().slime_size(self.get_size()))
    }

    fn mob_tick(&self, _caller: &dyn EntityBase) {
        crate::entity::mob::cube_mob::tick_cube_mob(self);
    }

    fn mob_receives_player_touch(&self) -> bool {
        true
    }

    fn mob_player_collision(&self, player: &Arc<crate::entity::player::Player>) {
        if !self.is_tiny() {
            self.deal_damage(player);
        }
    }

    fn post_tick(&self) {
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
                let new_entity = Entity::new(
                    world.clone(),
                    new_pos,
                    self.entity.living_entity.entity.entity_type,
                );
                let slime_like = Self::new(new_entity);
                slime_like.set_size(half_size, true);
                slime_like
                    .entity
                    .living_entity
                    .entity
                    .yaw
                    .store(rand::random_range(0.0..360.0));
                world.spawn_entity_non_save(slime_like as Arc<dyn EntityBase>);
            }
        }
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
    fn can_start(&mut self, _mob: &dyn Mob) -> bool {
        self.slime.entity.get_target().is_some()
    }

    fn start(&mut self, _mob: &dyn Mob) {
        self.grow_tired_timer = 300;
    }

    fn should_continue(&mut self, _mob: &dyn Mob) -> bool {
        self.slime.entity.get_target().is_some() && self.grow_tired_timer > 0
    }

    fn tick(&mut self, _mob: &dyn Mob) {
        self.grow_tired_timer -= 1;
        if let Some(target) = self.slime.entity.get_target() {
            let pos = target.get_entity().pos.load();
            let my_pos = self.slime.entity.living_entity.entity.pos.load();
            let dx = pos.x - my_pos.x;
            let dz = pos.z - my_pos.z;
            let yaw = dx.atan2(dz).to_degrees() as f32;
            self.slime.cube_data().target_yaw.store(yaw);
        }
        self.slime
            .cube_data()
            .is_aggressive
            .store(true, Ordering::Relaxed);
    }

    fn should_run_every_tick(&self) -> bool {
        true
    }

    fn controls(&self) -> crate::entity::ai::goal::Controls {
        crate::entity::ai::goal::Controls::LOOK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::ai::test_support::world;

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
        slime.cube.jump_delay.store(10, Ordering::Relaxed);
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
                slime.cube.jump_delay.load(Ordering::Relaxed),
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
