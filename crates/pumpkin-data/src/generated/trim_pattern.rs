/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[repr(u32)]
pub enum TrimPattern {
    Bolt,
    Coast,
    Dune,
    Eye,
    Flow,
    Host,
    Raiser,
    Rib,
    Sentry,
    Shaper,
    Silence,
    Snout,
    Spire,
    Tide,
    Vex,
    Ward,
    Wayfinder,
    Wild,
}
impl TrimPattern {
    #[doc = "Returns the trim pattern from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:bolt" | "bolt" => Some(Self::Bolt),
            "minecraft:coast" | "coast" => Some(Self::Coast),
            "minecraft:dune" | "dune" => Some(Self::Dune),
            "minecraft:eye" | "eye" => Some(Self::Eye),
            "minecraft:flow" | "flow" => Some(Self::Flow),
            "minecraft:host" | "host" => Some(Self::Host),
            "minecraft:raiser" | "raiser" => Some(Self::Raiser),
            "minecraft:rib" | "rib" => Some(Self::Rib),
            "minecraft:sentry" | "sentry" => Some(Self::Sentry),
            "minecraft:shaper" | "shaper" => Some(Self::Shaper),
            "minecraft:silence" | "silence" => Some(Self::Silence),
            "minecraft:snout" | "snout" => Some(Self::Snout),
            "minecraft:spire" | "spire" => Some(Self::Spire),
            "minecraft:tide" | "tide" => Some(Self::Tide),
            "minecraft:vex" | "vex" => Some(Self::Vex),
            "minecraft:ward" | "ward" => Some(Self::Ward),
            "minecraft:wayfinder" | "wayfinder" => Some(Self::Wayfinder),
            "minecraft:wild" | "wild" => Some(Self::Wild),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the trim pattern in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u32 {
        *self as u32
    }
    #[doc = "Returns the bare string name of the trim pattern."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Bolt => "bolt",
            Self::Coast => "coast",
            Self::Dune => "dune",
            Self::Eye => "eye",
            Self::Flow => "flow",
            Self::Host => "host",
            Self::Raiser => "raiser",
            Self::Rib => "rib",
            Self::Sentry => "sentry",
            Self::Shaper => "shaper",
            Self::Silence => "silence",
            Self::Snout => "snout",
            Self::Spire => "spire",
            Self::Tide => "tide",
            Self::Vex => "vex",
            Self::Ward => "ward",
            Self::Wayfinder => "wayfinder",
            Self::Wild => "wild",
        }
    }
    #[doc = "Returns the fully-qualified asset id of the trim pattern."]
    #[must_use]
    pub const fn asset_id(&self) -> &'static str {
        match self {
            Self::Bolt => "minecraft:bolt",
            Self::Coast => "minecraft:coast",
            Self::Dune => "minecraft:dune",
            Self::Eye => "minecraft:eye",
            Self::Flow => "minecraft:flow",
            Self::Host => "minecraft:host",
            Self::Raiser => "minecraft:raiser",
            Self::Rib => "minecraft:rib",
            Self::Sentry => "minecraft:sentry",
            Self::Shaper => "minecraft:shaper",
            Self::Silence => "minecraft:silence",
            Self::Snout => "minecraft:snout",
            Self::Spire => "minecraft:spire",
            Self::Tide => "minecraft:tide",
            Self::Vex => "minecraft:vex",
            Self::Ward => "minecraft:ward",
            Self::Wayfinder => "minecraft:wayfinder",
            Self::Wild => "minecraft:wild",
        }
    }
    #[doc = "Returns whether this trim pattern is a decal."]
    #[must_use]
    pub const fn is_decal(&self) -> bool {
        match self {
            Self::Bolt => false,
            Self::Coast => false,
            Self::Dune => false,
            Self::Eye => false,
            Self::Flow => false,
            Self::Host => false,
            Self::Raiser => false,
            Self::Rib => false,
            Self::Sentry => false,
            Self::Shaper => false,
            Self::Silence => false,
            Self::Snout => false,
            Self::Spire => false,
            Self::Tide => false,
            Self::Vex => false,
            Self::Ward => false,
            Self::Wayfinder => false,
            Self::Wild => false,
        }
    }
    #[doc = "Returns the translation key describing this trim pattern."]
    #[must_use]
    pub const fn translation_key(&self) -> &'static str {
        match self {
            Self::Bolt => "trim_pattern.minecraft.bolt",
            Self::Coast => "trim_pattern.minecraft.coast",
            Self::Dune => "trim_pattern.minecraft.dune",
            Self::Eye => "trim_pattern.minecraft.eye",
            Self::Flow => "trim_pattern.minecraft.flow",
            Self::Host => "trim_pattern.minecraft.host",
            Self::Raiser => "trim_pattern.minecraft.raiser",
            Self::Rib => "trim_pattern.minecraft.rib",
            Self::Sentry => "trim_pattern.minecraft.sentry",
            Self::Shaper => "trim_pattern.minecraft.shaper",
            Self::Silence => "trim_pattern.minecraft.silence",
            Self::Snout => "trim_pattern.minecraft.snout",
            Self::Spire => "trim_pattern.minecraft.spire",
            Self::Tide => "trim_pattern.minecraft.tide",
            Self::Vex => "trim_pattern.minecraft.vex",
            Self::Ward => "trim_pattern.minecraft.ward",
            Self::Wayfinder => "trim_pattern.minecraft.wayfinder",
            Self::Wild => "trim_pattern.minecraft.wild",
        }
    }
    #[doc = "Returns all vanilla trim patterns."]
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Bolt,
            Self::Coast,
            Self::Dune,
            Self::Eye,
            Self::Flow,
            Self::Host,
            Self::Raiser,
            Self::Rib,
            Self::Sentry,
            Self::Shaper,
            Self::Silence,
            Self::Snout,
            Self::Spire,
            Self::Tide,
            Self::Vex,
            Self::Ward,
            Self::Wayfinder,
            Self::Wild,
        ]
    }
}
