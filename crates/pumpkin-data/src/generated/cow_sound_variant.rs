/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum CowSoundVariant {
    #[default]
    Classic = 0u8,
    Moody = 1u8,
}
impl CowSoundVariant {
    pub const ALL: &'static [Self] = &[Self::Classic, Self::Moody];
    #[doc = "Returns the sound variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:classic" | "classic" => Some(Self::Classic),
            "minecraft:moody" | "moody" => Some(Self::Moody),
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
            1u8 => Some(Self::Moody),
            _ => None,
        }
    }
    #[doc = "Returns the bare string name of the sound variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Moody => "moody",
        }
    }
    #[must_use]
    pub const fn ambient_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCowAmbient
                } else {
                    crate::sound::Sound::EntityCowAmbient
                }
            }
            Self::Moody => {
                if is_baby {
                    crate::sound::Sound::EntityCowMoodyAmbient
                } else {
                    crate::sound::Sound::EntityCowMoodyAmbient
                }
            }
        }
    }
    #[must_use]
    pub const fn death_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCowDeath
                } else {
                    crate::sound::Sound::EntityCowDeath
                }
            }
            Self::Moody => {
                if is_baby {
                    crate::sound::Sound::EntityCowMoodyDeath
                } else {
                    crate::sound::Sound::EntityCowMoodyDeath
                }
            }
        }
    }
    #[must_use]
    pub const fn hurt_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCowHurt
                } else {
                    crate::sound::Sound::EntityCowHurt
                }
            }
            Self::Moody => {
                if is_baby {
                    crate::sound::Sound::EntityCowMoodyHurt
                } else {
                    crate::sound::Sound::EntityCowMoodyHurt
                }
            }
        }
    }
    #[must_use]
    pub const fn step_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCowStep
                } else {
                    crate::sound::Sound::EntityCowStep
                }
            }
            Self::Moody => {
                if is_baby {
                    crate::sound::Sound::EntityCowMoodyStep
                } else {
                    crate::sound::Sound::EntityCowMoodyStep
                }
            }
        }
    }
    #[must_use]
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }
}
