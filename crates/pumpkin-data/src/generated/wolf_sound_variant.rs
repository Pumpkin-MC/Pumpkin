/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum WolfSoundVariant {
    Angry = 0u8,
    Big = 1u8,
    #[default]
    Classic = 2u8,
    Cute = 3u8,
    Grumpy = 4u8,
    Puglin = 5u8,
    Sad = 6u8,
}
impl WolfSoundVariant {
    pub const ALL: &'static [Self] = &[
        Self::Angry,
        Self::Big,
        Self::Classic,
        Self::Cute,
        Self::Grumpy,
        Self::Puglin,
        Self::Sad,
    ];
    #[doc = "Returns the sound variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:angry" | "angry" => Some(Self::Angry),
            "minecraft:big" | "big" => Some(Self::Big),
            "minecraft:classic" | "classic" => Some(Self::Classic),
            "minecraft:cute" | "cute" => Some(Self::Cute),
            "minecraft:grumpy" | "grumpy" => Some(Self::Grumpy),
            "minecraft:puglin" | "puglin" => Some(Self::Puglin),
            "minecraft:sad" | "sad" => Some(Self::Sad),
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
            0u8 => Some(Self::Angry),
            1u8 => Some(Self::Big),
            2u8 => Some(Self::Classic),
            3u8 => Some(Self::Cute),
            4u8 => Some(Self::Grumpy),
            5u8 => Some(Self::Puglin),
            6u8 => Some(Self::Sad),
            _ => None,
        }
    }
    #[doc = "Returns the bare string name of the sound variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Angry => "angry",
            Self::Big => "big",
            Self::Classic => "classic",
            Self::Cute => "cute",
            Self::Grumpy => "grumpy",
            Self::Puglin => "puglin",
            Self::Sad => "sad",
        }
    }
    #[must_use]
    pub const fn ambient_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfAngryAmbient
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfBigAmbient
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfAmbient
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfCuteAmbient
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfGrumpyAmbient
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfPuglinAmbient
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfSadAmbient
                }
            }
        }
    }
    #[must_use]
    pub const fn death_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfAngryDeath
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfBigDeath
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfDeath
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfCuteDeath
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfGrumpyDeath
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfPuglinDeath
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfSadDeath
                }
            }
        }
    }
    #[must_use]
    pub const fn hurt_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfAngryHurt
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfBigHurt
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfHurt
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfCuteHurt
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfGrumpyHurt
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfPuglinHurt
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfSadHurt
                }
            }
        }
    }
    #[must_use]
    pub const fn step_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
        }
    }
    #[must_use]
    pub const fn growl_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfAngryGrowl
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfBigGrowl
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfGrowl
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfCuteGrowl
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfGrumpyGrowl
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfPuglinGrowl
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfSadGrowl
                }
            }
        }
    }
    #[must_use]
    pub const fn pant_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfPant
                } else {
                    crate::sound::Sound::EntityWolfAngryPant
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfPant
                } else {
                    crate::sound::Sound::EntityWolfBigPant
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfPant
                } else {
                    crate::sound::Sound::EntityWolfPant
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfPant
                } else {
                    crate::sound::Sound::EntityWolfCutePant
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfPant
                } else {
                    crate::sound::Sound::EntityWolfGrumpyPant
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfPant
                } else {
                    crate::sound::Sound::EntityWolfPuglinPant
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfPant
                } else {
                    crate::sound::Sound::EntityWolfSadPant
                }
            }
        }
    }
    #[must_use]
    pub const fn whine_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfAngryWhine
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfBigWhine
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfWhine
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfCuteWhine
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfGrumpyWhine
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfPuglinWhine
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityBabyWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfSadWhine
                }
            }
        }
    }
    #[must_use]
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }
}
