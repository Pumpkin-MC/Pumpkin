/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum PigSoundVariant {
    Big = 0u8,
    #[default]
    Classic = 1u8,
    Mini = 2u8,
}
impl PigSoundVariant {
    pub const ALL: &'static [Self] = &[Self::Big, Self::Classic, Self::Mini];
    #[doc = "Returns the sound variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:big" | "big" => Some(Self::Big),
            "minecraft:classic" | "classic" => Some(Self::Classic),
            "minecraft:mini" | "mini" => Some(Self::Mini),
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
            0u8 => Some(Self::Big),
            1u8 => Some(Self::Classic),
            2u8 => Some(Self::Mini),
            _ => None,
        }
    }
    #[doc = "Returns the bare string name of the sound variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Big => "big",
            Self::Classic => "classic",
            Self::Mini => "mini",
        }
    }
    #[must_use]
    pub const fn ambient_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigAmbient
                } else {
                    crate::sound::Sound::EntityPigBigAmbient
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigAmbient
                } else {
                    crate::sound::Sound::EntityPigAmbient
                }
            }
            Self::Mini => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigAmbient
                } else {
                    crate::sound::Sound::EntityPigMiniAmbient
                }
            }
        }
    }
    #[must_use]
    pub const fn death_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigDeath
                } else {
                    crate::sound::Sound::EntityPigBigDeath
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigDeath
                } else {
                    crate::sound::Sound::EntityPigDeath
                }
            }
            Self::Mini => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigDeath
                } else {
                    crate::sound::Sound::EntityPigMiniDeath
                }
            }
        }
    }
    #[must_use]
    pub const fn hurt_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigHurt
                } else {
                    crate::sound::Sound::EntityPigBigHurt
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigHurt
                } else {
                    crate::sound::Sound::EntityPigHurt
                }
            }
            Self::Mini => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigHurt
                } else {
                    crate::sound::Sound::EntityPigMiniHurt
                }
            }
        }
    }
    #[must_use]
    pub const fn step_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigStep
                } else {
                    crate::sound::Sound::EntityPigStep
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigStep
                } else {
                    crate::sound::Sound::EntityPigStep
                }
            }
            Self::Mini => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigStep
                } else {
                    crate::sound::Sound::EntityPigStep
                }
            }
        }
    }
    #[must_use]
    pub const fn eat_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigEat
                } else {
                    crate::sound::Sound::EntityPigBigEat
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigEat
                } else {
                    crate::sound::Sound::EntityPigEat
                }
            }
            Self::Mini => {
                if is_baby {
                    crate::sound::Sound::EntityBabyPigEat
                } else {
                    crate::sound::Sound::EntityPigMiniEat
                }
            }
        }
    }
    #[must_use]
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }
}
