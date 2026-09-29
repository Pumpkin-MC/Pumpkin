use std::sync::atomic::Ordering;
use std::sync::{Arc, Weak};

use pumpkin_data::attributes::Attributes;
use pumpkin_data::entity::EntityType;
use pumpkin_data::sound::Sound;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomImpl;

use crate::entity::ai::goal::active_target::ActiveTargetGoal;
use crate::entity::mob::cube_mob::{AbstractCubeMob, CubeMobHooks};
use crate::entity::mob::{Mob, MobEntity};
use crate::entity::player::Player;
use crate::entity::{Entity, EntityBase, custom_sound::CustomSound};
use crate::world::World;

pub struct SlimeEntity {
    pub cube: AbstractCubeMob,
}

impl SlimeEntity {
    pub fn new(entity: Entity) -> Arc<Self> {
        let slime = Self::new_without_size(entity);
        slime.randomize_size();
        slime
    }

    pub fn new_with_size(entity: Entity, size: i32) -> Arc<Self> {
        let slime = Self::new_without_size(entity);
        slime.set_size(size, true);
        slime
    }

    fn new_without_size(entity: Entity) -> Arc<Self> {
        let slime = Arc::new(Self {
            cube: AbstractCubeMob::new(entity),
        });
        let weak: Weak<dyn CubeMobHooks> = {
            let mob: Arc<dyn CubeMobHooks> = slime.clone();
            Arc::downgrade(&mob)
        };
        slime.cube.register_common_goals(weak.clone());
        slime.cube.add_attack_goal(weak);
        {
            let mut targets = slime
                .cube
                .mob_entity
                .target_selector
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            targets.add_goal(
                1,
                ActiveTargetGoal::with_default(&slime.cube.mob_entity, &EntityType::PLAYER, true),
            );
            targets.add_goal(
                3,
                ActiveTargetGoal::with_default(
                    &slime.cube.mob_entity,
                    &EntityType::IRON_GOLEM,
                    true,
                ),
            );
        };
        slime
    }

    pub fn get_size(&self) -> i32 {
        self.cube.get_size()
    }
    pub fn is_tiny(&self) -> bool {
        self.cube.is_tiny()
    }

    pub fn randomize_size(&self) {
        let mut size_scale = rand::random_range(0..3);
        if size_scale < 2 && rand::random_range(0.0..1.0) < 0.5 {
            size_scale += 1;
        }
        self.set_size(1 << size_scale, true);
    }

    pub fn check_slime_spawn_rules(world: &World, pos: &BlockPos) -> bool {
        if world.level_info.load().difficulty == Difficulty::Peaceful {
            return false;
        }
        let chunk_pos = pos.chunk_position();
        let slime_seed = pumpkin_util::random::seed_slime_chunk(
            chunk_pos.x,
            chunk_pos.y,
            world.level.seed.0,
            987_234_911,
        );
        let mut slime_rand = pumpkin_util::random::legacy_rand::LegacyRand::from_seed(slime_seed);
        rand::random_range(0..10) == 0 && slime_rand.next_bounded_i32(10) == 0 && pos.0.y < 40
    }
}

impl CubeMobHooks for SlimeEntity {
    fn cube_mob(&self) -> &AbstractCubeMob {
        &self.cube
    }
    fn jump_sound(&self) -> Sound {
        if self.is_tiny() {
            Sound::EntitySlimeJumpSmall
        } else {
            Sound::EntitySlimeJump
        }
    }
    fn squish_sound(&self) -> Sound {
        if self.is_tiny() {
            Sound::EntitySlimeSquishSmall
        } else {
            Sound::EntitySlimeSquish
        }
    }
    fn set_size(&self, size: i32, update_health: bool) {
        self.cube.set_size(self, size, update_health);
        let actual_size = self.get_size();
        let mut attributes = self
            .cube
            .mob_entity
            .living_entity
            .attributes
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(damage) = attributes.get_mut(&Attributes::ATTACK_DAMAGE.id) {
            damage.base_value = actual_size as f64;
            damage.dirty.store(true, Ordering::Relaxed);
        }
    }
    fn create_split_child(&self, entity: Entity, size: i32) -> Arc<dyn EntityBase> {
        Self::new_with_size(entity, size)
    }
}

impl CustomSound for SlimeEntity {
    fn death_sound(&self) -> Option<Sound> {
        Some(if self.is_tiny() {
            Sound::EntitySlimeDeathSmall
        } else {
            Sound::EntitySlimeDeath
        })
    }
    fn hurt_sound(&self) -> Option<Sound> {
        Some(if self.is_tiny() {
            Sound::EntitySlimeHurtSmall
        } else {
            Sound::EntitySlimeHurt
        })
    }
}

impl Mob for SlimeEntity {
    fn as_custom_sound(&self) -> Option<&dyn CustomSound> {
        Some(self)
    }
    fn mob_write_nbt(&self, nbt: &mut NbtCompound) {
        self.cube.mob_write_nbt(nbt);
    }
    fn mob_read_nbt(&self, nbt: &NbtCompound) {
        self.cube.mob_read_nbt(self, nbt);
    }
    fn get_mob_entity(&self) -> &MobEntity {
        &self.cube.mob_entity
    }
    fn mob_tick(&self, _caller: &dyn EntityBase) {
        self.cube.mob_tick(self);
    }
    fn post_tick(&self) {
        self.cube.post_tick(self);
    }
    fn mob_player_collision(&self, player: &Arc<Player>) {
        if self.can_deal_damage() {
            self.cube.mob_entity.try_attack(self, &**player);
        }
    }
}
