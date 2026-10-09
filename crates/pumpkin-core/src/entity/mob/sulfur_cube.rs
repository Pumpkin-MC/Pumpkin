use pumpkin_data::sound::{Sound, SoundCategory};

use crate::entity::mob::cube_mob::{ContactDamage, CubeKind, CubeSounds, SizedSound};
use crate::entity::mob::slime::SLIME;

pub const SPLIT_COUNT: i32 = 2;
pub const MAX_SIZE: i32 = 2;

pub const SULFUR_CUBE: CubeKind = CubeKind {
    sounds: CubeSounds {
        jump: SizedSound {
            normal: Sound::EntitySulfurCubeJump,
            small: Sound::EntitySmallSulfurCubeJump,
        },
        squish: SizedSound {
            normal: Sound::EntitySulfurCubeSquish,
            small: Sound::EntitySmallSulfurCubeSquish,
        },
        hurt: SizedSound {
            normal: Sound::EntitySulfurCubeHurt,
            small: Sound::EntitySmallSulfurCubeHurt,
        },
        death: SizedSound {
            normal: Sound::EntitySulfurCubeDeath,
            small: Sound::EntitySmallSulfurCubeDeath,
        },
    },
    sound_category: SoundCategory::Neutral,
    health: |size| f64::from(4 * size),
    size_attributes: &[],
    contact_damage: ContactDamage::Never,
    split_count: || SPLIT_COUNT,
    // Babies, which spawn at size 1, are not implemented yet.
    spawn_size: || MAX_SIZE,
    // TODO: SulfurCubeTemptGoal and SulfurCubeSearchForItemsGoal.
    register_goals: |_| {},
    ..SLIME
};
