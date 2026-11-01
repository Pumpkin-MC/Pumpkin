/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u8)]
pub enum CatVariant {
    AllBlack,
    Black,
    BritishShorthair,
    Calico,
    Jellie,
    Persian,
    Ragdoll,
    Red,
    Siamese,
    Tabby,
    White,
}
impl CatVariant {
    #[doc = "Returns the cat variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:all_black" | "all_black" => Some(Self::AllBlack),
            "minecraft:black" | "black" => Some(Self::Black),
            "minecraft:british_shorthair" | "british_shorthair" => Some(Self::BritishShorthair),
            "minecraft:calico" | "calico" => Some(Self::Calico),
            "minecraft:jellie" | "jellie" => Some(Self::Jellie),
            "minecraft:persian" | "persian" => Some(Self::Persian),
            "minecraft:ragdoll" | "ragdoll" => Some(Self::Ragdoll),
            "minecraft:red" | "red" => Some(Self::Red),
            "minecraft:siamese" | "siamese" => Some(Self::Siamese),
            "minecraft:tabby" | "tabby" => Some(Self::Tabby),
            "minecraft:white" | "white" => Some(Self::White),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the cat variant in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u8 {
        *self as u8
    }
    #[doc = "Returns the bare string name of the cat variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::AllBlack => "all_black",
            Self::Black => "black",
            Self::BritishShorthair => "british_shorthair",
            Self::Calico => "calico",
            Self::Jellie => "jellie",
            Self::Persian => "persian",
            Self::Ragdoll => "ragdoll",
            Self::Red => "red",
            Self::Siamese => "siamese",
            Self::Tabby => "tabby",
            Self::White => "white",
        }
    }
    #[doc = "Returns the fully-qualified asset id of the cat variant."]
    #[must_use]
    pub const fn asset_id(&self) -> &'static str {
        match self {
            Self::AllBlack => "minecraft:all_black",
            Self::Black => "minecraft:black",
            Self::BritishShorthair => "minecraft:british_shorthair",
            Self::Calico => "minecraft:calico",
            Self::Jellie => "minecraft:jellie",
            Self::Persian => "minecraft:persian",
            Self::Ragdoll => "minecraft:ragdoll",
            Self::Red => "minecraft:red",
            Self::Siamese => "minecraft:siamese",
            Self::Tabby => "minecraft:tabby",
            Self::White => "minecraft:white",
        }
    }
    #[doc = "Returns the texture asset path for the cat variant."]
    #[must_use]
    pub const fn texture(&self) -> &'static str {
        match self {
            Self::AllBlack => "minecraft:entity/cat/cat_all_black",
            Self::Black => "minecraft:entity/cat/cat_black",
            Self::BritishShorthair => "minecraft:entity/cat/cat_british_shorthair",
            Self::Calico => "minecraft:entity/cat/cat_calico",
            Self::Jellie => "minecraft:entity/cat/cat_jellie",
            Self::Persian => "minecraft:entity/cat/cat_persian",
            Self::Ragdoll => "minecraft:entity/cat/cat_ragdoll",
            Self::Red => "minecraft:entity/cat/cat_red",
            Self::Siamese => "minecraft:entity/cat/cat_siamese",
            Self::Tabby => "minecraft:entity/cat/cat_tabby",
            Self::White => "minecraft:entity/cat/cat_white",
        }
    }
    #[doc = "Returns all vanilla cat variants."]
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::AllBlack,
            Self::Black,
            Self::BritishShorthair,
            Self::Calico,
            Self::Jellie,
            Self::Persian,
            Self::Ragdoll,
            Self::Red,
            Self::Siamese,
            Self::Tabby,
            Self::White,
        ]
    }
}
