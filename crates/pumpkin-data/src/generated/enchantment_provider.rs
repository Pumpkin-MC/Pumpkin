/* This file is generated. Do not edit manually. */
use crate::Enchantment;
use crate::tag::Enchantment as EnchantmentTag;
use crate::tag::Tag;
#[doc = r" How something that is not a player at an enchanting table decides what to put on"]
#[doc = r" an item. Vanilla calls these enchantment providers and ships them as datapack"]
#[doc = r" entries; this is that data."]
#[derive(Clone, Copy)]
pub enum EnchantmentProvider {
    #[doc = r" One enchantment at one level, upgraded onto the item."]
    Single {
        enchantment: &'static Enchantment,
        level: i32,
    },
    #[doc = r" A draw from `enchantments` at a cost between `min_cost` and"]
    #[doc = r" `min_cost + difficulty * max_cost_span`."]
    ByCostWithDifficulty {
        enchantments: &'static Tag,
        min_cost: i32,
        max_cost_span: i32,
    },
}
impl EnchantmentProvider {
    pub const ENDERMAN_LOOT_DROP: Self = Self::Single {
        enchantment: &Enchantment::SILK_TOUCH,
        level: 1i32,
    };
    pub const MOB_SPAWN_EQUIPMENT: Self = Self::ByCostWithDifficulty {
        enchantments: &EnchantmentTag::MINECRAFT_ON_MOB_SPAWN_EQUIPMENT,
        min_cost: 5i32,
        max_cost_span: 17i32,
    };
    pub const PILLAGER_SPAWN_CROSSBOW: Self = Self::Single {
        enchantment: &Enchantment::PIERCING,
        level: 1i32,
    };
    pub const RAID_PILLAGER_POST_WAVE_3: Self = Self::Single {
        enchantment: &Enchantment::QUICK_CHARGE,
        level: 1i32,
    };
    pub const RAID_PILLAGER_POST_WAVE_5: Self = Self::Single {
        enchantment: &Enchantment::QUICK_CHARGE,
        level: 2i32,
    };
    pub const RAID_VINDICATOR: Self = Self::Single {
        enchantment: &Enchantment::SHARPNESS,
        level: 1i32,
    };
    pub const RAID_VINDICATOR_POST_WAVE_5: Self = Self::Single {
        enchantment: &Enchantment::SHARPNESS,
        level: 2i32,
    };
}
