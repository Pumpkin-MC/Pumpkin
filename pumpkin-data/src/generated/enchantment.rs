/* This file is generated. Do not edit manually. */
use crate::data_component_impl::EnchantmentsImpl;
use crate::item::Item;
use crate::tag::Enchantment as EnchantmentTag;
use crate::tag::Item as ItemTag;
use crate::tag::{RegistryKey, Tag, Taggable};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;
use std::hash::{Hash, Hasher};
use std::slice::Iter;
pub struct Enchantment {
    pub id: u8,
    pub name: &'static str,
    pub registry_key: &'static str,
    pub description: &'static str,
    pub anvil_cost: u32,
    pub supported_items: &'static Tag,
    pub exclusive_set: Option<&'static Tag>,
    pub max_level: i32,
    pub slots: &'static [AttributeModifierSlot],
    pub weight: i32,
    pub min_cost: Cost,
    pub max_cost: Cost,
}
#[derive(Clone, Copy, Debug)]
pub struct Cost {
    pub base: i32,
    pub per_level_above_first: i32,
}
impl Cost {
    pub fn calculate(&self, level: i32) -> i32 {
        self.base + self.per_level_above_first * (level - 1)
    }
}
impl Taggable for Enchantment {
    #[inline]
    fn tag_key() -> RegistryKey {
        RegistryKey::Enchantment
    }
    #[inline]
    fn registry_key(&self) -> &str {
        self.registry_key
    }
    #[inline]
    fn registry_id(&self) -> u16 {
        self.id as u16
    }
}
impl PartialEq for Enchantment {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Enchantment {}
impl Hash for Enchantment {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
#[derive(Debug, Clone, Hash, PartialEq)]
pub enum AttributeModifierSlot {
    Any,
    MainHand,
    OffHand,
    Hand,
    Feet,
    Legs,
    Chest,
    Head,
    Armor,
    Body,
    Saddle,
}
impl Enchantment {
    pub const ALL: &'static [&'static Self] = &[
        &Self::AQUA_AFFINITY,
        &Self::BANE_OF_ARTHROPODS,
        &Self::BINDING_CURSE,
        &Self::BLAST_PROTECTION,
        &Self::BREACH,
        &Self::CHANNELING,
        &Self::DENSITY,
        &Self::DEPTH_STRIDER,
        &Self::EFFICIENCY,
        &Self::FEATHER_FALLING,
        &Self::FIRE_ASPECT,
        &Self::FIRE_PROTECTION,
        &Self::FLAME,
        &Self::FORTUNE,
        &Self::FROST_WALKER,
        &Self::IMPALING,
        &Self::INFINITY,
        &Self::KNOCKBACK,
        &Self::LOOTING,
        &Self::LOYALTY,
        &Self::LUCK_OF_THE_SEA,
        &Self::LUNGE,
        &Self::LURE,
        &Self::MENDING,
        &Self::MULTISHOT,
        &Self::PIERCING,
        &Self::POWER,
        &Self::PROJECTILE_PROTECTION,
        &Self::PROTECTION,
        &Self::PUNCH,
        &Self::QUICK_CHARGE,
        &Self::RESPIRATION,
        &Self::RIPTIDE,
        &Self::SHARPNESS,
        &Self::SILK_TOUCH,
        &Self::SMITE,
        &Self::SOUL_SPEED,
        &Self::SWEEPING_EDGE,
        &Self::SWIFT_SNEAK,
        &Self::THORNS,
        &Self::UNBREAKING,
        &Self::VANISHING_CURSE,
        &Self::WIND_BURST,
    ];
    pub fn all() -> Iter<'static, &'static Self> {
        Self::ALL.iter()
    }
    pub const AQUA_AFFINITY: Self = Self {
        id: 0u8,
        name: "minecraft:aqua_affinity",
        description: "enchantment.minecraft.aqua_affinity",
        registry_key: "aqua_affinity",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_HEAD_ARMOR,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Head],
        weight: 2i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 41i32,
            per_level_above_first: 0i32,
        },
    };
    pub const BANE_OF_ARTHROPODS: Self = Self {
        id: 1u8,
        name: "minecraft:bane_of_arthropods",
        registry_key: "bane_of_arthropods",
        description: "enchantment.minecraft.bane_of_arthropods",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_WEAPON,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 8i32,
        },
    };
    pub const BINDING_CURSE: Self = Self {
        id: 2u8,
        name: "minecraft:binding_curse",
        description: "enchantment.minecraft.binding_curse",
        registry_key: "binding_curse",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_EQUIPPABLE,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 1i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const BLAST_PROTECTION: Self = Self {
        id: 3u8,
        name: "minecraft:blast_protection",
        registry_key: "blast_protection",
        description: "enchantment.minecraft.blast_protection",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_ARMOR),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 2i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 13i32,
            per_level_above_first: 8i32,
        },
    };
    pub const BREACH: Self = Self {
        id: 4u8,
        name: "minecraft:breach",
        registry_key: "breach",
        description: "enchantment.minecraft.breach",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MACE,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
    };
    pub const CHANNELING: Self = Self {
        id: 5u8,
        name: "minecraft:channeling",
        description: "enchantment.minecraft.channeling",
        registry_key: "channeling",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_TRIDENT,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 1i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const DENSITY: Self = Self {
        id: 6u8,
        name: "minecraft:density",
        registry_key: "density",
        description: "enchantment.minecraft.density",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MACE,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 8i32,
        },
    };
    pub const DEPTH_STRIDER: Self = Self {
        id: 7u8,
        name: "minecraft:depth_strider",
        registry_key: "depth_strider",
        description: "enchantment.minecraft.depth_strider",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FOOT_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_BOOTS),
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Feet],
        weight: 2i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 10i32,
        },
    };
    pub const EFFICIENCY: Self = Self {
        id: 8u8,
        name: "minecraft:efficiency",
        description: "enchantment.minecraft.efficiency",
        registry_key: "efficiency",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MINING,
        exclusive_set: None,
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 51i32,
            per_level_above_first: 10i32,
        },
    };
    pub const FEATHER_FALLING: Self = Self {
        id: 9u8,
        name: "minecraft:feather_falling",
        description: "enchantment.minecraft.feather_falling",
        registry_key: "feather_falling",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FOOT_ARMOR,
        exclusive_set: None,
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 6i32,
        },
        max_cost: Cost {
            base: 11i32,
            per_level_above_first: 6i32,
        },
    };
    pub const FIRE_ASPECT: Self = Self {
        id: 10u8,
        name: "minecraft:fire_aspect",
        description: "enchantment.minecraft.fire_aspect",
        registry_key: "fire_aspect",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FIRE_ASPECT,
        exclusive_set: None,
        max_level: 2i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 60i32,
            per_level_above_first: 20i32,
        },
    };
    pub const FIRE_PROTECTION: Self = Self {
        id: 11u8,
        name: "minecraft:fire_protection",
        registry_key: "fire_protection",
        description: "enchantment.minecraft.fire_protection",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_ARMOR),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 5i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 18i32,
            per_level_above_first: 8i32,
        },
    };
    pub const FLAME: Self = Self {
        id: 12u8,
        name: "minecraft:flame",
        description: "enchantment.minecraft.flame",
        registry_key: "flame",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_BOW,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 20i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const FORTUNE: Self = Self {
        id: 13u8,
        name: "minecraft:fortune",
        registry_key: "fortune",
        description: "enchantment.minecraft.fortune",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MINING_LOOT,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_MINING),
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
    };
    pub const FROST_WALKER: Self = Self {
        id: 14u8,
        name: "minecraft:frost_walker",
        registry_key: "frost_walker",
        description: "enchantment.minecraft.frost_walker",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FOOT_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_BOOTS),
        max_level: 2i32,
        slots: &[AttributeModifierSlot::Feet],
        weight: 2i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 10i32,
        },
    };
    pub const IMPALING: Self = Self {
        id: 15u8,
        name: "minecraft:impaling",
        registry_key: "impaling",
        description: "enchantment.minecraft.impaling",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_TRIDENT,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 21i32,
            per_level_above_first: 8i32,
        },
    };
    pub const INFINITY: Self = Self {
        id: 16u8,
        name: "minecraft:infinity",
        registry_key: "infinity",
        description: "enchantment.minecraft.infinity",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_BOW,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_BOW),
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 1i32,
        min_cost: Cost {
            base: 20i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const KNOCKBACK: Self = Self {
        id: 17u8,
        name: "minecraft:knockback",
        description: "enchantment.minecraft.knockback",
        registry_key: "knockback",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MELEE_WEAPON,
        exclusive_set: None,
        max_level: 2i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 55i32,
            per_level_above_first: 20i32,
        },
    };
    pub const LOOTING: Self = Self {
        id: 18u8,
        name: "minecraft:looting",
        description: "enchantment.minecraft.looting",
        registry_key: "looting",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MELEE_WEAPON,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
    };
    pub const LOYALTY: Self = Self {
        id: 19u8,
        name: "minecraft:loyalty",
        description: "enchantment.minecraft.loyalty",
        registry_key: "loyalty",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_TRIDENT,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 12i32,
            per_level_above_first: 7i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const LUCK_OF_THE_SEA: Self = Self {
        id: 20u8,
        name: "minecraft:luck_of_the_sea",
        description: "enchantment.minecraft.luck_of_the_sea",
        registry_key: "luck_of_the_sea",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FISHING,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
    };
    pub const LUNGE: Self = Self {
        id: 21u8,
        name: "minecraft:lunge",
        description: "enchantment.minecraft.lunge",
        registry_key: "lunge",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_LUNGE,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Hand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 8i32,
        },
    };
    pub const LURE: Self = Self {
        id: 22u8,
        name: "minecraft:lure",
        description: "enchantment.minecraft.lure",
        registry_key: "lure",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FISHING,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
    };
    pub const MENDING: Self = Self {
        id: 23u8,
        name: "minecraft:mending",
        description: "enchantment.minecraft.mending",
        registry_key: "mending",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_DURABILITY,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Any],
        weight: 2i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 25i32,
        },
        max_cost: Cost {
            base: 75i32,
            per_level_above_first: 25i32,
        },
    };
    pub const MULTISHOT: Self = Self {
        id: 24u8,
        name: "minecraft:multishot",
        registry_key: "multishot",
        description: "enchantment.minecraft.multishot",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_CROSSBOW,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_CROSSBOW),
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 20i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const PIERCING: Self = Self {
        id: 25u8,
        name: "minecraft:piercing",
        registry_key: "piercing",
        description: "enchantment.minecraft.piercing",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_CROSSBOW,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_CROSSBOW),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const POWER: Self = Self {
        id: 26u8,
        name: "minecraft:power",
        description: "enchantment.minecraft.power",
        registry_key: "power",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_BOW,
        exclusive_set: None,
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 16i32,
            per_level_above_first: 10i32,
        },
    };
    pub const PROJECTILE_PROTECTION: Self = Self {
        id: 27u8,
        name: "minecraft:projectile_protection",
        registry_key: "projectile_protection",
        description: "enchantment.minecraft.projectile_protection",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_ARMOR),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 5i32,
        min_cost: Cost {
            base: 3i32,
            per_level_above_first: 6i32,
        },
        max_cost: Cost {
            base: 9i32,
            per_level_above_first: 6i32,
        },
    };
    pub const PROTECTION: Self = Self {
        id: 28u8,
        name: "minecraft:protection",
        registry_key: "protection",
        description: "enchantment.minecraft.protection",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_ARMOR),
        max_level: 4i32,
        slots: &[AttributeModifierSlot::Armor],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 11i32,
        },
        max_cost: Cost {
            base: 12i32,
            per_level_above_first: 11i32,
        },
    };
    pub const PUNCH: Self = Self {
        id: 29u8,
        name: "minecraft:punch",
        description: "enchantment.minecraft.punch",
        registry_key: "punch",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_BOW,
        exclusive_set: None,
        max_level: 2i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 12i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 37i32,
            per_level_above_first: 20i32,
        },
    };
    pub const QUICK_CHARGE: Self = Self {
        id: 30u8,
        name: "minecraft:quick_charge",
        description: "enchantment.minecraft.quick_charge",
        registry_key: "quick_charge",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_CROSSBOW,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[
            AttributeModifierSlot::MainHand,
            AttributeModifierSlot::OffHand,
        ],
        weight: 5i32,
        min_cost: Cost {
            base: 12i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const RESPIRATION: Self = Self {
        id: 31u8,
        name: "minecraft:respiration",
        description: "enchantment.minecraft.respiration",
        registry_key: "respiration",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_HEAD_ARMOR,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Head],
        weight: 2i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 40i32,
            per_level_above_first: 10i32,
        },
    };
    pub const RIPTIDE: Self = Self {
        id: 32u8,
        name: "minecraft:riptide",
        registry_key: "riptide",
        description: "enchantment.minecraft.riptide",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_TRIDENT,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_RIPTIDE),
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Hand],
        weight: 2i32,
        min_cost: Cost {
            base: 17i32,
            per_level_above_first: 7i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const SHARPNESS: Self = Self {
        id: 33u8,
        name: "minecraft:sharpness",
        registry_key: "sharpness",
        description: "enchantment.minecraft.sharpness",
        anvil_cost: 1u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_SHARP_WEAPON,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 10i32,
        min_cost: Cost {
            base: 1i32,
            per_level_above_first: 11i32,
        },
        max_cost: Cost {
            base: 21i32,
            per_level_above_first: 11i32,
        },
    };
    pub const SILK_TOUCH: Self = Self {
        id: 34u8,
        name: "minecraft:silk_touch",
        registry_key: "silk_touch",
        description: "enchantment.minecraft.silk_touch",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MINING_LOOT,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_MINING),
        max_level: 1i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 1i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 0i32,
        },
    };
    pub const SMITE: Self = Self {
        id: 35u8,
        name: "minecraft:smite",
        registry_key: "smite",
        description: "enchantment.minecraft.smite",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_WEAPON,
        exclusive_set: Some(&EnchantmentTag::MINECRAFT_EXCLUSIVE_SET_DAMAGE),
        max_level: 5i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 8i32,
        },
    };
    pub const SOUL_SPEED: Self = Self {
        id: 36u8,
        name: "minecraft:soul_speed",
        description: "enchantment.minecraft.soul_speed",
        registry_key: "soul_speed",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_FOOT_ARMOR,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Feet],
        weight: 1i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 10i32,
        },
        max_cost: Cost {
            base: 25i32,
            per_level_above_first: 10i32,
        },
    };
    pub const SWEEPING_EDGE: Self = Self {
        id: 37u8,
        name: "minecraft:sweeping_edge",
        description: "enchantment.minecraft.sweeping_edge",
        registry_key: "sweeping_edge",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_SWEEPING,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 20i32,
            per_level_above_first: 9i32,
        },
    };
    pub const SWIFT_SNEAK: Self = Self {
        id: 38u8,
        name: "minecraft:swift_sneak",
        description: "enchantment.minecraft.swift_sneak",
        registry_key: "swift_sneak",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_LEG_ARMOR,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Legs],
        weight: 1i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 25i32,
        },
        max_cost: Cost {
            base: 75i32,
            per_level_above_first: 25i32,
        },
    };
    pub const THORNS: Self = Self {
        id: 39u8,
        name: "minecraft:thorns",
        description: "enchantment.minecraft.thorns",
        registry_key: "thorns",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_ARMOR,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Any],
        weight: 1i32,
        min_cost: Cost {
            base: 10i32,
            per_level_above_first: 20i32,
        },
        max_cost: Cost {
            base: 60i32,
            per_level_above_first: 20i32,
        },
    };
    pub const UNBREAKING: Self = Self {
        id: 40u8,
        name: "minecraft:unbreaking",
        description: "enchantment.minecraft.unbreaking",
        registry_key: "unbreaking",
        anvil_cost: 2u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_DURABILITY,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::Any],
        weight: 5i32,
        min_cost: Cost {
            base: 5i32,
            per_level_above_first: 8i32,
        },
        max_cost: Cost {
            base: 55i32,
            per_level_above_first: 8i32,
        },
    };
    pub const VANISHING_CURSE: Self = Self {
        id: 41u8,
        name: "minecraft:vanishing_curse",
        description: "enchantment.minecraft.vanishing_curse",
        registry_key: "vanishing_curse",
        anvil_cost: 8u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_VANISHING,
        exclusive_set: None,
        max_level: 1i32,
        slots: &[AttributeModifierSlot::Any],
        weight: 1i32,
        min_cost: Cost {
            base: 25i32,
            per_level_above_first: 0i32,
        },
        max_cost: Cost {
            base: 50i32,
            per_level_above_first: 0i32,
        },
    };
    pub const WIND_BURST: Self = Self {
        id: 42u8,
        name: "minecraft:wind_burst",
        description: "enchantment.minecraft.wind_burst",
        registry_key: "wind_burst",
        anvil_cost: 4u32,
        supported_items: &ItemTag::MINECRAFT_ENCHANTABLE_MACE,
        exclusive_set: None,
        max_level: 3i32,
        slots: &[AttributeModifierSlot::MainHand],
        weight: 2i32,
        min_cost: Cost {
            base: 15i32,
            per_level_above_first: 9i32,
        },
        max_cost: Cost {
            base: 65i32,
            per_level_above_first: 9i32,
        },
    };
    pub fn from_name(name: &str) -> Option<&'static Self> {
        match name {
            "minecraft:aqua_affinity" => Some(&Self::AQUA_AFFINITY),
            "minecraft:bane_of_arthropods" => Some(&Self::BANE_OF_ARTHROPODS),
            "minecraft:binding_curse" => Some(&Self::BINDING_CURSE),
            "minecraft:blast_protection" => Some(&Self::BLAST_PROTECTION),
            "minecraft:breach" => Some(&Self::BREACH),
            "minecraft:channeling" => Some(&Self::CHANNELING),
            "minecraft:density" => Some(&Self::DENSITY),
            "minecraft:depth_strider" => Some(&Self::DEPTH_STRIDER),
            "minecraft:efficiency" => Some(&Self::EFFICIENCY),
            "minecraft:feather_falling" => Some(&Self::FEATHER_FALLING),
            "minecraft:fire_aspect" => Some(&Self::FIRE_ASPECT),
            "minecraft:fire_protection" => Some(&Self::FIRE_PROTECTION),
            "minecraft:flame" => Some(&Self::FLAME),
            "minecraft:fortune" => Some(&Self::FORTUNE),
            "minecraft:frost_walker" => Some(&Self::FROST_WALKER),
            "minecraft:impaling" => Some(&Self::IMPALING),
            "minecraft:infinity" => Some(&Self::INFINITY),
            "minecraft:knockback" => Some(&Self::KNOCKBACK),
            "minecraft:looting" => Some(&Self::LOOTING),
            "minecraft:loyalty" => Some(&Self::LOYALTY),
            "minecraft:luck_of_the_sea" => Some(&Self::LUCK_OF_THE_SEA),
            "minecraft:lunge" => Some(&Self::LUNGE),
            "minecraft:lure" => Some(&Self::LURE),
            "minecraft:mending" => Some(&Self::MENDING),
            "minecraft:multishot" => Some(&Self::MULTISHOT),
            "minecraft:piercing" => Some(&Self::PIERCING),
            "minecraft:power" => Some(&Self::POWER),
            "minecraft:projectile_protection" => Some(&Self::PROJECTILE_PROTECTION),
            "minecraft:protection" => Some(&Self::PROTECTION),
            "minecraft:punch" => Some(&Self::PUNCH),
            "minecraft:quick_charge" => Some(&Self::QUICK_CHARGE),
            "minecraft:respiration" => Some(&Self::RESPIRATION),
            "minecraft:riptide" => Some(&Self::RIPTIDE),
            "minecraft:sharpness" => Some(&Self::SHARPNESS),
            "minecraft:silk_touch" => Some(&Self::SILK_TOUCH),
            "minecraft:smite" => Some(&Self::SMITE),
            "minecraft:soul_speed" => Some(&Self::SOUL_SPEED),
            "minecraft:sweeping_edge" => Some(&Self::SWEEPING_EDGE),
            "minecraft:swift_sneak" => Some(&Self::SWIFT_SNEAK),
            "minecraft:thorns" => Some(&Self::THORNS),
            "minecraft:unbreaking" => Some(&Self::UNBREAKING),
            "minecraft:vanishing_curse" => Some(&Self::VANISHING_CURSE),
            "minecraft:wind_burst" => Some(&Self::WIND_BURST),
            _ => None,
        }
    }
    pub fn from_id(id: u8) -> Option<&'static Self> {
        match id {
            0u8 => Some(&Self::AQUA_AFFINITY),
            1u8 => Some(&Self::BANE_OF_ARTHROPODS),
            2u8 => Some(&Self::BINDING_CURSE),
            3u8 => Some(&Self::BLAST_PROTECTION),
            4u8 => Some(&Self::BREACH),
            5u8 => Some(&Self::CHANNELING),
            6u8 => Some(&Self::DENSITY),
            7u8 => Some(&Self::DEPTH_STRIDER),
            8u8 => Some(&Self::EFFICIENCY),
            9u8 => Some(&Self::FEATHER_FALLING),
            10u8 => Some(&Self::FIRE_ASPECT),
            11u8 => Some(&Self::FIRE_PROTECTION),
            12u8 => Some(&Self::FLAME),
            13u8 => Some(&Self::FORTUNE),
            14u8 => Some(&Self::FROST_WALKER),
            15u8 => Some(&Self::IMPALING),
            16u8 => Some(&Self::INFINITY),
            17u8 => Some(&Self::KNOCKBACK),
            18u8 => Some(&Self::LOOTING),
            19u8 => Some(&Self::LOYALTY),
            20u8 => Some(&Self::LUCK_OF_THE_SEA),
            21u8 => Some(&Self::LUNGE),
            22u8 => Some(&Self::LURE),
            23u8 => Some(&Self::MENDING),
            24u8 => Some(&Self::MULTISHOT),
            25u8 => Some(&Self::PIERCING),
            26u8 => Some(&Self::POWER),
            27u8 => Some(&Self::PROJECTILE_PROTECTION),
            28u8 => Some(&Self::PROTECTION),
            29u8 => Some(&Self::PUNCH),
            30u8 => Some(&Self::QUICK_CHARGE),
            31u8 => Some(&Self::RESPIRATION),
            32u8 => Some(&Self::RIPTIDE),
            33u8 => Some(&Self::SHARPNESS),
            34u8 => Some(&Self::SILK_TOUCH),
            35u8 => Some(&Self::SMITE),
            36u8 => Some(&Self::SOUL_SPEED),
            37u8 => Some(&Self::SWEEPING_EDGE),
            38u8 => Some(&Self::SWIFT_SNEAK),
            39u8 => Some(&Self::THORNS),
            40u8 => Some(&Self::UNBREAKING),
            41u8 => Some(&Self::VANISHING_CURSE),
            42u8 => Some(&Self::WIND_BURST),
            _ => None,
        }
    }
    pub fn can_enchant(&self, item: &'static Item) -> bool {
        self.supported_items.1.contains(&item.id)
    }
    pub fn are_compatible(&self, other: &'static Enchantment) -> bool {
        if self == other {
            return false;
        }
        if let Some(tag) = self.exclusive_set
            && tag.1.contains(&(other.id as u16))
        {
            return false;
        }
        if let Some(tag) = other.exclusive_set
            && tag.1.contains(&(self.id as u16))
        {
            return false;
        }
        true
    }
    pub fn is_enchantment_compatible(&self, other: &EnchantmentsImpl) -> bool {
        for (i, _) in other.enchantment.iter() {
            if !self.are_compatible(i) {
                return false;
            }
        }
        true
    }
    pub fn get_fullname(&self, level: i32) -> TextComponent {
        let mut ret = TextComponent::translate(self.description, []).color_named(
            if self.has_tag(&EnchantmentTag::MINECRAFT_CURSE) {
                NamedColor::Red
            } else {
                NamedColor::Gray
            },
        );
        if level != 1 || self.max_level != 1 {
            ret = ret.add_text(" ").add_child(TextComponent::translate(
                "enchantment.level.".to_string() + &level.to_string(),
                [],
            ));
        }
        ret
    }
}
