/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum Instrument {
    AdmireGoatHorn,
    CallGoatHorn,
    DreamGoatHorn,
    FeelGoatHorn,
    PonderGoatHorn,
    SeekGoatHorn,
    SingGoatHorn,
    YearnGoatHorn,
}
impl Instrument {
    #[doc = "Returns the instrument from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:admire_goat_horn" | "admire_goat_horn" => Some(Self::AdmireGoatHorn),
            "minecraft:call_goat_horn" | "call_goat_horn" => Some(Self::CallGoatHorn),
            "minecraft:dream_goat_horn" | "dream_goat_horn" => Some(Self::DreamGoatHorn),
            "minecraft:feel_goat_horn" | "feel_goat_horn" => Some(Self::FeelGoatHorn),
            "minecraft:ponder_goat_horn" | "ponder_goat_horn" => Some(Self::PonderGoatHorn),
            "minecraft:seek_goat_horn" | "seek_goat_horn" => Some(Self::SeekGoatHorn),
            "minecraft:sing_goat_horn" | "sing_goat_horn" => Some(Self::SingGoatHorn),
            "minecraft:yearn_goat_horn" | "yearn_goat_horn" => Some(Self::YearnGoatHorn),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the instrument in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u32 {
        *self as u32
    }
    #[doc = "Returns the bare string name of the instrument."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::AdmireGoatHorn => "admire_goat_horn",
            Self::CallGoatHorn => "call_goat_horn",
            Self::DreamGoatHorn => "dream_goat_horn",
            Self::FeelGoatHorn => "feel_goat_horn",
            Self::PonderGoatHorn => "ponder_goat_horn",
            Self::SeekGoatHorn => "seek_goat_horn",
            Self::SingGoatHorn => "sing_goat_horn",
            Self::YearnGoatHorn => "yearn_goat_horn",
        }
    }
    #[doc = "Returns the fully-qualified asset id of the instrument."]
    #[must_use]
    pub const fn asset_id(&self) -> &'static str {
        match self {
            Self::AdmireGoatHorn => "minecraft:admire_goat_horn",
            Self::CallGoatHorn => "minecraft:call_goat_horn",
            Self::DreamGoatHorn => "minecraft:dream_goat_horn",
            Self::FeelGoatHorn => "minecraft:feel_goat_horn",
            Self::PonderGoatHorn => "minecraft:ponder_goat_horn",
            Self::SeekGoatHorn => "minecraft:seek_goat_horn",
            Self::SingGoatHorn => "minecraft:sing_goat_horn",
            Self::YearnGoatHorn => "minecraft:yearn_goat_horn",
        }
    }
    #[doc = "Returns the sound event played by this instrument."]
    #[must_use]
    pub const fn sound(&self) -> crate::sound::Sound {
        match self {
            Self::AdmireGoatHorn => crate::sound::Sound::ItemGoatHornSound4,
            Self::CallGoatHorn => crate::sound::Sound::ItemGoatHornSound5,
            Self::DreamGoatHorn => crate::sound::Sound::ItemGoatHornSound7,
            Self::FeelGoatHorn => crate::sound::Sound::ItemGoatHornSound3,
            Self::PonderGoatHorn => crate::sound::Sound::ItemGoatHornSound0,
            Self::SeekGoatHorn => crate::sound::Sound::ItemGoatHornSound2,
            Self::SingGoatHorn => crate::sound::Sound::ItemGoatHornSound1,
            Self::YearnGoatHorn => crate::sound::Sound::ItemGoatHornSound6,
        }
    }
    #[doc = "Returns the use duration of the instrument in seconds."]
    #[must_use]
    pub const fn use_duration_seconds(&self) -> f32 {
        match self {
            Self::AdmireGoatHorn => 7f32,
            Self::CallGoatHorn => 7f32,
            Self::DreamGoatHorn => 7f32,
            Self::FeelGoatHorn => 7f32,
            Self::PonderGoatHorn => 7f32,
            Self::SeekGoatHorn => 7f32,
            Self::SingGoatHorn => 7f32,
            Self::YearnGoatHorn => 7f32,
        }
    }
    #[doc = "Returns the use duration of the instrument in game ticks."]
    #[must_use]
    pub const fn use_duration_ticks(&self) -> u32 {
        match self {
            Self::AdmireGoatHorn => 140u32,
            Self::CallGoatHorn => 140u32,
            Self::DreamGoatHorn => 140u32,
            Self::FeelGoatHorn => 140u32,
            Self::PonderGoatHorn => 140u32,
            Self::SeekGoatHorn => 140u32,
            Self::SingGoatHorn => 140u32,
            Self::YearnGoatHorn => 140u32,
        }
    }
    #[doc = "Returns the audibility range of the instrument in blocks."]
    #[must_use]
    pub const fn range(&self) -> f32 {
        match self {
            Self::AdmireGoatHorn => 256f32,
            Self::CallGoatHorn => 256f32,
            Self::DreamGoatHorn => 256f32,
            Self::FeelGoatHorn => 256f32,
            Self::PonderGoatHorn => 256f32,
            Self::SeekGoatHorn => 256f32,
            Self::SingGoatHorn => 256f32,
            Self::YearnGoatHorn => 256f32,
        }
    }
    #[doc = "Returns the translation key describing this instrument."]
    #[must_use]
    pub const fn translation_key(&self) -> &'static str {
        match self {
            Self::AdmireGoatHorn => "instrument.minecraft.admire_goat_horn",
            Self::CallGoatHorn => "instrument.minecraft.call_goat_horn",
            Self::DreamGoatHorn => "instrument.minecraft.dream_goat_horn",
            Self::FeelGoatHorn => "instrument.minecraft.feel_goat_horn",
            Self::PonderGoatHorn => "instrument.minecraft.ponder_goat_horn",
            Self::SeekGoatHorn => "instrument.minecraft.seek_goat_horn",
            Self::SingGoatHorn => "instrument.minecraft.sing_goat_horn",
            Self::YearnGoatHorn => "instrument.minecraft.yearn_goat_horn",
        }
    }
    #[doc = "Returns all vanilla instrument variants."]
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::AdmireGoatHorn,
            Self::CallGoatHorn,
            Self::DreamGoatHorn,
            Self::FeelGoatHorn,
            Self::PonderGoatHorn,
            Self::SeekGoatHorn,
            Self::SingGoatHorn,
            Self::YearnGoatHorn,
        ]
    }
}
