/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum TrimMaterial {
    Amethyst,
    Copper,
    Diamond,
    Emerald,
    Gold,
    Iron,
    Lapis,
    Netherite,
    Quartz,
    Redstone,
    Resin,
}
impl TrimMaterial {
    #[doc = "Returns the trim material from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:amethyst" | "amethyst" => Some(Self::Amethyst),
            "minecraft:copper" | "copper" => Some(Self::Copper),
            "minecraft:diamond" | "diamond" => Some(Self::Diamond),
            "minecraft:emerald" | "emerald" => Some(Self::Emerald),
            "minecraft:gold" | "gold" => Some(Self::Gold),
            "minecraft:iron" | "iron" => Some(Self::Iron),
            "minecraft:lapis" | "lapis" => Some(Self::Lapis),
            "minecraft:netherite" | "netherite" => Some(Self::Netherite),
            "minecraft:quartz" | "quartz" => Some(Self::Quartz),
            "minecraft:redstone" | "redstone" => Some(Self::Redstone),
            "minecraft:resin" | "resin" => Some(Self::Resin),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the trim material in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u32 {
        *self as u32
    }
    #[doc = "Returns the bare string name of the trim material."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Amethyst => "amethyst",
            Self::Copper => "copper",
            Self::Diamond => "diamond",
            Self::Emerald => "emerald",
            Self::Gold => "gold",
            Self::Iron => "iron",
            Self::Lapis => "lapis",
            Self::Netherite => "netherite",
            Self::Quartz => "quartz",
            Self::Redstone => "redstone",
            Self::Resin => "resin",
        }
    }
    #[doc = "Returns the fully-qualified asset id of the trim material."]
    #[must_use]
    pub const fn asset_id(&self) -> &'static str {
        match self {
            Self::Amethyst => "minecraft:amethyst",
            Self::Copper => "minecraft:copper",
            Self::Diamond => "minecraft:diamond",
            Self::Emerald => "minecraft:emerald",
            Self::Gold => "minecraft:gold",
            Self::Iron => "minecraft:iron",
            Self::Lapis => "minecraft:lapis",
            Self::Netherite => "minecraft:netherite",
            Self::Quartz => "minecraft:quartz",
            Self::Redstone => "minecraft:redstone",
            Self::Resin => "minecraft:resin",
        }
    }
    #[doc = "Returns the palette id for texture styling."]
    #[must_use]
    pub const fn palette_id(&self) -> &'static str {
        match self {
            Self::Amethyst => "minecraft:trim/amethyst",
            Self::Copper => "minecraft:trim/copper",
            Self::Diamond => "minecraft:trim/diamond",
            Self::Emerald => "minecraft:trim/emerald",
            Self::Gold => "minecraft:trim/gold",
            Self::Iron => "minecraft:trim/iron",
            Self::Lapis => "minecraft:trim/lapis",
            Self::Netherite => "minecraft:trim/netherite",
            Self::Quartz => "minecraft:trim/quartz",
            Self::Redstone => "minecraft:trim/redstone",
            Self::Resin => "minecraft:trim/resin",
        }
    }
    #[doc = "Returns the hex color code for this material."]
    #[must_use]
    pub const fn color(&self) -> &'static str {
        match self {
            Self::Amethyst => "#9A5CC6",
            Self::Copper => "#B4684D",
            Self::Diamond => "#6EECD2",
            Self::Emerald => "#11A036",
            Self::Gold => "#DEB12D",
            Self::Iron => "#ECECEC",
            Self::Lapis => "#416E97",
            Self::Netherite => "#625859",
            Self::Quartz => "#E3D4C4",
            Self::Redstone => "#971607",
            Self::Resin => "#FC7812",
        }
    }
    #[doc = "Returns the translation key describing this trim material."]
    #[must_use]
    pub const fn translation_key(&self) -> &'static str {
        match self {
            Self::Amethyst => "trim_material.minecraft.amethyst",
            Self::Copper => "trim_material.minecraft.copper",
            Self::Diamond => "trim_material.minecraft.diamond",
            Self::Emerald => "trim_material.minecraft.emerald",
            Self::Gold => "trim_material.minecraft.gold",
            Self::Iron => "trim_material.minecraft.iron",
            Self::Lapis => "trim_material.minecraft.lapis",
            Self::Netherite => "trim_material.minecraft.netherite",
            Self::Quartz => "trim_material.minecraft.quartz",
            Self::Redstone => "trim_material.minecraft.redstone",
            Self::Resin => "trim_material.minecraft.resin",
        }
    }
    #[doc = "Returns all vanilla trim materials."]
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Amethyst,
            Self::Copper,
            Self::Diamond,
            Self::Emerald,
            Self::Gold,
            Self::Iron,
            Self::Lapis,
            Self::Netherite,
            Self::Quartz,
            Self::Redstone,
            Self::Resin,
        ]
    }
}
