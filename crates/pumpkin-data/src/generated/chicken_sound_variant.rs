/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum ChickenSoundVariant {
    #[default]
    Classic = 0u8,
    Picky = 1u8,
}
impl ChickenSoundVariant {
    pub const ALL: &'static [Self] = &[Self::Classic, Self::Picky];
    #[doc = "Returns the sound variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:classic" | "classic" => Some(Self::Classic),
            "minecraft:picky" | "picky" => Some(Self::Picky),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the sound variant in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u8 {
        *self as u8
    }
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0u8 => Some(Self::Classic),
            1u8 => Some(Self::Picky),
            _ => None,
        }
    }
    #[doc = "Returns the bare string name of the sound variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Picky => "picky",
        }
    }
    #[must_use]
    pub const fn ambient_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyChickenAmbient
                } else {
                    crate::sound::Sound::EntityChickenAmbient
                }
            }
            Self::Picky => {
                if is_baby {
                    crate::sound::Sound::EntityBabyChickenAmbient
                } else {
                    crate::sound::Sound::EntityChickenPickyAmbient
                }
            }
        }
    }
    #[must_use]
    pub const fn death_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyChickenDeath
                } else {
                    crate::sound::Sound::EntityChickenDeath
                }
            }
            Self::Picky => {
                if is_baby {
                    crate::sound::Sound::EntityBabyChickenDeath
                } else {
                    crate::sound::Sound::EntityChickenPickyDeath
                }
            }
        }
    }
    #[must_use]
    pub const fn hurt_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyChickenHurt
                } else {
                    crate::sound::Sound::EntityChickenHurt
                }
            }
            Self::Picky => {
                if is_baby {
                    crate::sound::Sound::EntityBabyChickenHurt
                } else {
                    crate::sound::Sound::EntityChickenPickyHurt
                }
            }
        }
    }
    #[must_use]
    pub const fn step_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyChickenStep
                } else {
                    crate::sound::Sound::EntityChickenStep
                }
            }
            Self::Picky => {
                if is_baby {
                    crate::sound::Sound::EntityBabyChickenStep
                } else {
                    crate::sound::Sound::EntityChickenStep
                }
            }
        }
    }
    #[must_use]
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }
}
