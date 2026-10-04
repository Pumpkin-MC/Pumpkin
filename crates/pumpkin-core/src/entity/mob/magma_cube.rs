use pumpkin_data::attributes::Attributes;
use pumpkin_data::sound::Sound;

use crate::entity::mob::cube_mob::{ContactDamage, CubeKind, CubeSounds, SizedSound};
use crate::entity::mob::slime::SLIME;

pub const MAGMA_CUBE: CubeKind = CubeKind {
    sounds: CubeSounds {
        jump: SizedSound {
            normal: Sound::EntityMagmaCubeJump,
            small: Sound::EntityMagmaCubeJump,
        },
        squish: SizedSound {
            normal: Sound::EntityMagmaCubeSquish,
            small: Sound::EntityMagmaCubeSquishSmall,
        },
        hurt: SizedSound {
            normal: Sound::EntityMagmaCubeHurt,
            small: Sound::EntityMagmaCubeHurtSmall,
        },
        death: SizedSound {
            normal: Sound::EntityMagmaCubeDeath,
            small: Sound::EntityMagmaCubeDeathSmall,
        },
    },
    // Vanilla adds the 2 in getAttackDamage instead of the attribute.
    size_attributes: &[
        (Attributes::ATTACK_DAMAGE, |size| f64::from(size + 2)),
        (Attributes::ARMOR, |size| f64::from(size * 3)),
    ],
    jump_delay_multiplier: 4,
    contact_damage: ContactDamage::Always,
    ..SLIME
};
