/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum WolfVariant {
    Ashen,
    Black,
    Chestnut,
    Pale,
    Rusty,
    Snowy,
    Spotted,
    Striped,
    Woods,
}
impl WolfVariant {
    #[doc = "Returns the wolf variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:ashen" | "ashen" => Some(Self::Ashen),
            "minecraft:black" | "black" => Some(Self::Black),
            "minecraft:chestnut" | "chestnut" => Some(Self::Chestnut),
            "minecraft:pale" | "pale" => Some(Self::Pale),
            "minecraft:rusty" | "rusty" => Some(Self::Rusty),
            "minecraft:snowy" | "snowy" => Some(Self::Snowy),
            "minecraft:spotted" | "spotted" => Some(Self::Spotted),
            "minecraft:striped" | "striped" => Some(Self::Striped),
            "minecraft:woods" | "woods" => Some(Self::Woods),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the wolf variant in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u8 {
        *self as u8
    }
    #[doc = "Returns the bare string name of the wolf variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Ashen => "ashen",
            Self::Black => "black",
            Self::Chestnut => "chestnut",
            Self::Pale => "pale",
            Self::Rusty => "rusty",
            Self::Snowy => "snowy",
            Self::Spotted => "spotted",
            Self::Striped => "striped",
            Self::Woods => "woods",
        }
    }
    #[doc = "Returns the fully-qualified asset id of the wolf variant."]
    #[must_use]
    pub const fn asset_id(&self) -> &'static str {
        match self {
            Self::Ashen => "minecraft:ashen",
            Self::Black => "minecraft:black",
            Self::Chestnut => "minecraft:chestnut",
            Self::Pale => "minecraft:pale",
            Self::Rusty => "minecraft:rusty",
            Self::Snowy => "minecraft:snowy",
            Self::Spotted => "minecraft:spotted",
            Self::Striped => "minecraft:striped",
            Self::Woods => "minecraft:woods",
        }
    }
    #[doc = "Returns the wild texture asset path."]
    #[must_use]
    pub const fn wild_texture(&self) -> &'static str {
        match self {
            Self::Ashen => "minecraft:entity/wolf/wolf_ashen",
            Self::Black => "minecraft:entity/wolf/wolf_black",
            Self::Chestnut => "minecraft:entity/wolf/wolf_chestnut",
            Self::Pale => "minecraft:entity/wolf/wolf",
            Self::Rusty => "minecraft:entity/wolf/wolf_rusty",
            Self::Snowy => "minecraft:entity/wolf/wolf_snowy",
            Self::Spotted => "minecraft:entity/wolf/wolf_spotted",
            Self::Striped => "minecraft:entity/wolf/wolf_striped",
            Self::Woods => "minecraft:entity/wolf/wolf_woods",
        }
    }
    #[doc = "Returns the tame texture asset path."]
    #[must_use]
    pub const fn tame_texture(&self) -> &'static str {
        match self {
            Self::Ashen => "minecraft:entity/wolf/wolf_ashen_tame",
            Self::Black => "minecraft:entity/wolf/wolf_black_tame",
            Self::Chestnut => "minecraft:entity/wolf/wolf_chestnut_tame",
            Self::Pale => "minecraft:entity/wolf/wolf_tame",
            Self::Rusty => "minecraft:entity/wolf/wolf_rusty_tame",
            Self::Snowy => "minecraft:entity/wolf/wolf_snowy_tame",
            Self::Spotted => "minecraft:entity/wolf/wolf_spotted_tame",
            Self::Striped => "minecraft:entity/wolf/wolf_striped_tame",
            Self::Woods => "minecraft:entity/wolf/wolf_woods_tame",
        }
    }
    #[doc = "Returns the angry texture asset path."]
    #[must_use]
    pub const fn angry_texture(&self) -> &'static str {
        match self {
            Self::Ashen => "minecraft:entity/wolf/wolf_ashen_angry",
            Self::Black => "minecraft:entity/wolf/wolf_black_angry",
            Self::Chestnut => "minecraft:entity/wolf/wolf_chestnut_angry",
            Self::Pale => "minecraft:entity/wolf/wolf_angry",
            Self::Rusty => "minecraft:entity/wolf/wolf_rusty_angry",
            Self::Snowy => "minecraft:entity/wolf/wolf_snowy_angry",
            Self::Spotted => "minecraft:entity/wolf/wolf_spotted_angry",
            Self::Striped => "minecraft:entity/wolf/wolf_striped_angry",
            Self::Woods => "minecraft:entity/wolf/wolf_woods_angry",
        }
    }
    #[doc = "Returns all vanilla wolf variants."]
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Ashen,
            Self::Black,
            Self::Chestnut,
            Self::Pale,
            Self::Rusty,
            Self::Snowy,
            Self::Spotted,
            Self::Striped,
            Self::Woods,
        ]
    }
}
