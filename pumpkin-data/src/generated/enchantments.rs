#[derive(Debug)]
pub struct Enchantment {
    anvil_cost: u32,
    supported_items: String,
    max_level: i32,
    slots: &[AttributeModifierSlot],
}
pub enum AttributeModifierSlot {
    Any,
    MainHand,
    OffHand,
    Feet,
    Legs,
    Chest,
    Head,
    Body,
    Saddle,
}
impl Enchantment {
    pub const MINECRAFT_PIERCING: Enchantment = Enchantment {
        anvil_cost: 1u32,
        supported_items: "#minecraft:enchantable/crossbow",
        max_level: 4i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_PROJECTILE_PROTECTION: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/armor",
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
    };
    pub const MINECRAFT_PUNCH: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/bow",
        max_level: 2i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_DEPTH_STRIDER: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/foot_armor",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Feet],
    };
    pub const MINECRAFT_IMPALING: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/trident",
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_MENDING: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/durability",
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Any],
    };
    pub const MINECRAFT_MULTISHOT: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/crossbow",
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_LOOTING: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/sword",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_BREACH: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/mace",
        max_level: 4i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_SHARPNESS: Enchantment = Enchantment {
        anvil_cost: 1u32,
        supported_items: "#minecraft:enchantable/sharp_weapon",
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_SILK_TOUCH: Enchantment = Enchantment {
        anvil_cost: 8u32,
        supported_items: "#minecraft:enchantable/mining_loot",
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_THORNS: Enchantment = Enchantment {
        anvil_cost: 8u32,
        supported_items: "#minecraft:enchantable/armor",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Any],
    };
    pub const MINECRAFT_QUICK_CHARGE: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/crossbow",
        max_level: 3i32,
        slots: &[
            AttributeModifierSlot::MainHand,
            AttributeModifierSlot::OffHand,
        ],
    };
    pub const MINECRAFT_WIND_BURST: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/mace",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_RIPTIDE: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/trident",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Hand],
    };
    pub const MINECRAFT_KNOCKBACK: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/sword",
        max_level: 2i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_FIRE_PROTECTION: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/armor",
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
    };
    pub const MINECRAFT_FLAME: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/bow",
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_BLAST_PROTECTION: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/armor",
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
    };
    pub const MINECRAFT_VANISHING_CURSE: Enchantment = Enchantment {
        anvil_cost: 8u32,
        supported_items: "#minecraft:enchantable/vanishing",
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Any],
    };
    pub const MINECRAFT_LURE: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/fishing",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_CHANNELING: Enchantment = Enchantment {
        anvil_cost: 8u32,
        supported_items: "#minecraft:enchantable/trident",
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_EFFICIENCY: Enchantment = Enchantment {
        anvil_cost: 1u32,
        supported_items: "#minecraft:enchantable/mining",
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_UNBREAKING: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/durability",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Any],
    };
    pub const MINECRAFT_SWIFT_SNEAK: Enchantment = Enchantment {
        anvil_cost: 8u32,
        supported_items: "#minecraft:enchantable/leg_armor",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Legs],
    };
    pub const MINECRAFT_FIRE_ASPECT: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/fire_aspect",
        max_level: 2i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_BINDING_CURSE: Enchantment = Enchantment {
        anvil_cost: 8u32,
        supported_items: "#minecraft:enchantable/equippable",
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Armor],
    };
    pub const MINECRAFT_SOUL_SPEED: Enchantment = Enchantment {
        anvil_cost: 8u32,
        supported_items: "#minecraft:enchantable/foot_armor",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Feet],
    };
    pub const MINECRAFT_SMITE: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/weapon",
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_FEATHER_FALLING: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/foot_armor",
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
    };
    pub const MINECRAFT_INFINITY: Enchantment = Enchantment {
        anvil_cost: 8u32,
        supported_items: "#minecraft:enchantable/bow",
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_BANE_OF_ARTHROPODS: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/weapon",
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_PROTECTION: Enchantment = Enchantment {
        anvil_cost: 1u32,
        supported_items: "#minecraft:enchantable/armor",
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
    };
    pub const MINECRAFT_AQUA_AFFINITY: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/head_armor",
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Head],
    };
    pub const MINECRAFT_FORTUNE: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/mining_loot",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_FROST_WALKER: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/foot_armor",
        max_level: 2i32,
        slots: &[AttributeModifierSlot::Feet],
    };
    pub const MINECRAFT_POWER: Enchantment = Enchantment {
        anvil_cost: 1u32,
        supported_items: "#minecraft:enchantable/bow",
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_DENSITY: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/mace",
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_LOYALTY: Enchantment = Enchantment {
        anvil_cost: 2u32,
        supported_items: "#minecraft:enchantable/trident",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_SWEEPING_EDGE: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/sword",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub const MINECRAFT_RESPIRATION: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/head_armor",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Head],
    };
    pub const MINECRAFT_LUCK_OF_THE_SEA: Enchantment = Enchantment {
        anvil_cost: 4u32,
        supported_items: "#minecraft:enchantable/fishing",
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
    };
    pub fn from_name(name: &str) -> Option<&'static Self> {
        match name {
            "minecraft:piercing" => Some(&Self::MINECRAFT_PIERCING),
            "minecraft:projectile_protection" => Some(&Self::MINECRAFT_PROJECTILE_PROTECTION),
            "minecraft:punch" => Some(&Self::MINECRAFT_PUNCH),
            "minecraft:depth_strider" => Some(&Self::MINECRAFT_DEPTH_STRIDER),
            "minecraft:impaling" => Some(&Self::MINECRAFT_IMPALING),
            "minecraft:mending" => Some(&Self::MINECRAFT_MENDING),
            "minecraft:multishot" => Some(&Self::MINECRAFT_MULTISHOT),
            "minecraft:looting" => Some(&Self::MINECRAFT_LOOTING),
            "minecraft:breach" => Some(&Self::MINECRAFT_BREACH),
            "minecraft:sharpness" => Some(&Self::MINECRAFT_SHARPNESS),
            "minecraft:silk_touch" => Some(&Self::MINECRAFT_SILK_TOUCH),
            "minecraft:thorns" => Some(&Self::MINECRAFT_THORNS),
            "minecraft:quick_charge" => Some(&Self::MINECRAFT_QUICK_CHARGE),
            "minecraft:wind_burst" => Some(&Self::MINECRAFT_WIND_BURST),
            "minecraft:riptide" => Some(&Self::MINECRAFT_RIPTIDE),
            "minecraft:knockback" => Some(&Self::MINECRAFT_KNOCKBACK),
            "minecraft:fire_protection" => Some(&Self::MINECRAFT_FIRE_PROTECTION),
            "minecraft:flame" => Some(&Self::MINECRAFT_FLAME),
            "minecraft:blast_protection" => Some(&Self::MINECRAFT_BLAST_PROTECTION),
            "minecraft:vanishing_curse" => Some(&Self::MINECRAFT_VANISHING_CURSE),
            "minecraft:lure" => Some(&Self::MINECRAFT_LURE),
            "minecraft:channeling" => Some(&Self::MINECRAFT_CHANNELING),
            "minecraft:efficiency" => Some(&Self::MINECRAFT_EFFICIENCY),
            "minecraft:unbreaking" => Some(&Self::MINECRAFT_UNBREAKING),
            "minecraft:swift_sneak" => Some(&Self::MINECRAFT_SWIFT_SNEAK),
            "minecraft:fire_aspect" => Some(&Self::MINECRAFT_FIRE_ASPECT),
            "minecraft:binding_curse" => Some(&Self::MINECRAFT_BINDING_CURSE),
            "minecraft:soul_speed" => Some(&Self::MINECRAFT_SOUL_SPEED),
            "minecraft:smite" => Some(&Self::MINECRAFT_SMITE),
            "minecraft:feather_falling" => Some(&Self::MINECRAFT_FEATHER_FALLING),
            "minecraft:infinity" => Some(&Self::MINECRAFT_INFINITY),
            "minecraft:bane_of_arthropods" => Some(&Self::MINECRAFT_BANE_OF_ARTHROPODS),
            "minecraft:protection" => Some(&Self::MINECRAFT_PROTECTION),
            "minecraft:aqua_affinity" => Some(&Self::MINECRAFT_AQUA_AFFINITY),
            "minecraft:fortune" => Some(&Self::MINECRAFT_FORTUNE),
            "minecraft:frost_walker" => Some(&Self::MINECRAFT_FROST_WALKER),
            "minecraft:power" => Some(&Self::MINECRAFT_POWER),
            "minecraft:density" => Some(&Self::MINECRAFT_DENSITY),
            "minecraft:loyalty" => Some(&Self::MINECRAFT_LOYALTY),
            "minecraft:sweeping_edge" => Some(&Self::MINECRAFT_SWEEPING_EDGE),
            "minecraft:respiration" => Some(&Self::MINECRAFT_RESPIRATION),
            "minecraft:luck_of_the_sea" => Some(&Self::MINECRAFT_LUCK_OF_THE_SEA),
            _ => None,
        }
    }
}
