/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u32)]
pub enum FrogVariant {
    Cold,
    #[default]
    Temperate,
    Warm,
}
impl FrogVariant {
    #[doc = "Returns the frog variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:cold" | "cold" => Some(Self::Cold),
            "minecraft:temperate" | "temperate" => Some(Self::Temperate),
            "minecraft:warm" | "warm" => Some(Self::Warm),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the frog variant in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u32 {
        *self as u32
    }
    #[doc = "Returns the frog variant from numeric ID."]
    #[must_use]
    pub const fn from_id(id: u32) -> Self {
        match id {
            0 => Self::Cold,
            2 => Self::Warm,
            _ => Self::Temperate,
        }
    }
    #[doc = "Returns the bare string name of the frog variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Cold => "cold",
            Self::Temperate => "temperate",
            Self::Warm => "warm",
        }
    }
    #[doc = "Returns the fully-qualified asset id of the frog variant."]
    #[must_use]
    pub const fn asset_id(&self) -> &'static str {
        match self {
            Self::Cold => "minecraft:cold",
            Self::Temperate => "minecraft:temperate",
            Self::Warm => "minecraft:warm",
        }
    }
    #[doc = "Returns the texture asset path for the frog variant."]
    #[must_use]
    pub const fn texture(&self) -> &'static str {
        match self {
            Self::Cold => "minecraft:entity/frog/frog_cold",
            Self::Temperate => "minecraft:entity/frog/frog_temperate",
            Self::Warm => "minecraft:entity/frog/frog_warm",
        }
    }
    #[doc = "Returns all vanilla frog variants."]
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Cold, Self::Temperate, Self::Warm]
    }
}
