use pumpkin_data::attributes::Attributes;
use pumpkin_data::sound::{Sound, SoundCategory};
use pumpkin_util::Difficulty;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::RandomImpl;

use crate::entity::mob::cube_mob::{
    ContactDamage, CubeKind, CubeSounds, SizedSound, random_spawn_size, random_split_count,
    register_hostile_goals,
};
use crate::world::World;

pub const SLIME: CubeKind = CubeKind {
    sounds: CubeSounds {
        jump: SizedSound {
            normal: Sound::EntitySlimeJump,
            small: Sound::EntitySlimeJumpSmall,
        },
        squish: SizedSound {
            normal: Sound::EntitySlimeSquish,
            small: Sound::EntitySlimeSquishSmall,
        },
        hurt: SizedSound {
            normal: Sound::EntitySlimeHurt,
            small: Sound::EntitySlimeHurtSmall,
        },
        death: SizedSound {
            normal: Sound::EntitySlimeDeath,
            small: Sound::EntitySlimeDeathSmall,
        },
    },
    sound_category: SoundCategory::Hostile,
    health: |size| f64::from(size * size),
    size_attributes: &[(Attributes::ATTACK_DAMAGE, |size| f64::from(size))],
    jump_delay_multiplier: 1,
    contact_damage: ContactDamage::UnlessTiny,
    split_count: random_split_count,
    spawn_size: random_spawn_size,
    register_goals: register_hostile_goals,
};

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
