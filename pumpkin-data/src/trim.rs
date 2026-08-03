use crate::item::Item;

// Registry contents transcribed from net/minecraft/world/item/equipment/trim/TrimMaterials.java
// and the `.trimMaterial(...)` item property calls in net/minecraft/world/item/Items.java.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrimMaterial {
    Quartz,
    Iron,
    Netherite,
    Redstone,
    Copper,
    Gold,
    Emerald,
    Diamond,
    Lapis,
    Amethyst,
    Resin,
}

impl TrimMaterial {
    #[must_use]
    pub const fn registry_key(self) -> &'static str {
        match self {
            Self::Quartz => "quartz",
            Self::Iron => "iron",
            Self::Netherite => "netherite",
            Self::Redstone => "redstone",
            Self::Copper => "copper",
            Self::Gold => "gold",
            Self::Emerald => "emerald",
            Self::Diamond => "diamond",
            Self::Lapis => "lapis",
            Self::Amethyst => "amethyst",
            Self::Resin => "resin",
        }
    }

    #[must_use]
    pub fn from_registry_key(key: &str) -> Option<Self> {
        Some(match key {
            "quartz" => Self::Quartz,
            "iron" => Self::Iron,
            "netherite" => Self::Netherite,
            "redstone" => Self::Redstone,
            "copper" => Self::Copper,
            "gold" => Self::Gold,
            "emerald" => Self::Emerald,
            "diamond" => Self::Diamond,
            "lapis" => Self::Lapis,
            "amethyst" => Self::Amethyst,
            "resin" => Self::Resin,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn item(self) -> &'static Item {
        match self {
            Self::Quartz => &Item::QUARTZ,
            Self::Iron => &Item::IRON_INGOT,
            Self::Netherite => &Item::NETHERITE_INGOT,
            Self::Redstone => &Item::REDSTONE,
            Self::Copper => &Item::COPPER_INGOT,
            Self::Gold => &Item::GOLD_INGOT,
            Self::Emerald => &Item::EMERALD,
            Self::Diamond => &Item::DIAMOND,
            Self::Lapis => &Item::LAPIS_LAZULI,
            Self::Amethyst => &Item::AMETHYST_SHARD,
            Self::Resin => &Item::RESIN_BRICK,
        }
    }

    #[must_use]
    pub fn from_item(item: &Item) -> Option<Self> {
        [
            Self::Quartz,
            Self::Iron,
            Self::Netherite,
            Self::Redstone,
            Self::Copper,
            Self::Gold,
            Self::Emerald,
            Self::Diamond,
            Self::Lapis,
            Self::Amethyst,
            Self::Resin,
        ]
        .into_iter()
        .find(|material| material.item() == item)
    }

    #[must_use]
    pub const fn hover_color(self) -> u32 {
        match self {
            Self::Quartz => 14_931_140,
            Self::Iron => 15_527_148,
            Self::Netherite => 6_445_145,
            Self::Redstone => 9_901_575,
            Self::Copper => 11_823_181,
            Self::Gold => 14_594_349,
            Self::Emerald => 1_155_126,
            Self::Diamond => 7_269_586,
            Self::Lapis => 4_288_151,
            Self::Amethyst => 10_116_294,
            Self::Resin => 16_545_810,
        }
    }
}

// Registry contents transcribed from net/minecraft/world/item/equipment/trim/TrimPatterns.java.
// Each pattern's template item is `<name>_armor_trim_smithing_template`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TrimPattern {
    Sentry,
    Dune,
    Coast,
    Wild,
    Ward,
    Eye,
    Vex,
    Tide,
    Snout,
    Rib,
    Spire,
    Wayfinder,
    Shaper,
    Silence,
    Raiser,
    Host,
    Flow,
    Bolt,
}

impl TrimPattern {
    #[must_use]
    pub const fn registry_key(self) -> &'static str {
        match self {
            Self::Sentry => "sentry",
            Self::Dune => "dune",
            Self::Coast => "coast",
            Self::Wild => "wild",
            Self::Ward => "ward",
            Self::Eye => "eye",
            Self::Vex => "vex",
            Self::Tide => "tide",
            Self::Snout => "snout",
            Self::Rib => "rib",
            Self::Spire => "spire",
            Self::Wayfinder => "wayfinder",
            Self::Shaper => "shaper",
            Self::Silence => "silence",
            Self::Raiser => "raiser",
            Self::Host => "host",
            Self::Flow => "flow",
            Self::Bolt => "bolt",
        }
    }

    #[must_use]
    pub fn from_registry_key(key: &str) -> Option<Self> {
        Some(match key {
            "sentry" => Self::Sentry,
            "dune" => Self::Dune,
            "coast" => Self::Coast,
            "wild" => Self::Wild,
            "ward" => Self::Ward,
            "eye" => Self::Eye,
            "vex" => Self::Vex,
            "tide" => Self::Tide,
            "snout" => Self::Snout,
            "rib" => Self::Rib,
            "spire" => Self::Spire,
            "wayfinder" => Self::Wayfinder,
            "shaper" => Self::Shaper,
            "silence" => Self::Silence,
            "raiser" => Self::Raiser,
            "host" => Self::Host,
            "flow" => Self::Flow,
            "bolt" => Self::Bolt,
            _ => return None,
        })
    }

    #[must_use]
    pub const fn template_item(self) -> &'static Item {
        match self {
            Self::Sentry => &Item::SENTRY_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Dune => &Item::DUNE_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Coast => &Item::COAST_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Wild => &Item::WILD_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Ward => &Item::WARD_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Eye => &Item::EYE_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Vex => &Item::VEX_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Tide => &Item::TIDE_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Snout => &Item::SNOUT_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Rib => &Item::RIB_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Spire => &Item::SPIRE_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Wayfinder => &Item::WAYFINDER_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Shaper => &Item::SHAPER_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Silence => &Item::SILENCE_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Raiser => &Item::RAISER_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Host => &Item::HOST_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Flow => &Item::FLOW_ARMOR_TRIM_SMITHING_TEMPLATE,
            Self::Bolt => &Item::BOLT_ARMOR_TRIM_SMITHING_TEMPLATE,
        }
    }

    #[must_use]
    pub fn from_template_item(item: &Item) -> Option<Self> {
        [
            Self::Sentry,
            Self::Dune,
            Self::Coast,
            Self::Wild,
            Self::Ward,
            Self::Eye,
            Self::Vex,
            Self::Tide,
            Self::Snout,
            Self::Rib,
            Self::Spire,
            Self::Wayfinder,
            Self::Shaper,
            Self::Silence,
            Self::Raiser,
            Self::Host,
            Self::Flow,
            Self::Bolt,
        ]
        .into_iter()
        .find(|pattern| pattern.template_item() == item)
    }
}
