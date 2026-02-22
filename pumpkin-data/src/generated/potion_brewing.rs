/* This file is generated. Do not edit manually. */
#![allow(dead_code)]
use crate::item::Item;
use crate::potion::Potion;
pub struct PotionRecipe {
    from: &'static Potion,
    ingredient: &'static [&'static Item],
    to: &'static Potion,
}
pub struct ItemRecipe {
    from: &'static Item,
    ingredient: &'static [&'static Item],
    to: &'static Item,
}
impl PotionRecipe {
    pub fn from(&self) -> &'static Potion {
        self.from
    }
    pub fn ingredient(&self) -> &'static [&'static Item] {
        self.ingredient
    }
    pub fn to(&self) -> &'static Potion {
        self.to
    }
}
impl ItemRecipe {
    pub fn from(&self) -> &'static Item {
        self.from
    }
    pub fn ingredient(&self) -> &'static [&'static Item] {
        self.ingredient
    }
    pub fn to(&self) -> &'static Item {
        self.to
    }
}
pub const ITEM_RECIPES: [ItemRecipe; 2usize] = [
    ItemRecipe {
        from: &Item::POTION,
        ingredient: &[&Item::GUNPOWDER],
        to: &Item::SPLASH_POTION,
    },
    ItemRecipe {
        from: &Item::SPLASH_POTION,
        ingredient: &[&Item::DRAGON_BREATH],
        to: &Item::LINGERING_POTION,
    },
];
pub const POTION_RECIPES: [PotionRecipe; 63usize] = [
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::THICK,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::NETHER_WART],
        to: &Potion::AWKWARD,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::BREEZE_ROD],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::BREEZE_ROD],
        to: &Potion::WIND_CHARGED,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::SLIME_BLOCK],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::SLIME_BLOCK],
        to: &Potion::OOZING,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::STONE],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::STONE],
        to: &Potion::INFESTED,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::COBWEB],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::COBWEB],
        to: &Potion::WEAVING,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::GOLDEN_CARROT],
        to: &Potion::NIGHT_VISION,
    },
    PotionRecipe {
        from: &Potion::NIGHT_VISION,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_NIGHT_VISION,
    },
    PotionRecipe {
        from: &Potion::NIGHT_VISION,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::INVISIBILITY,
    },
    PotionRecipe {
        from: &Potion::LONG_NIGHT_VISION,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::LONG_INVISIBILITY,
    },
    PotionRecipe {
        from: &Potion::INVISIBILITY,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_INVISIBILITY,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::MAGMA_CREAM],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::MAGMA_CREAM],
        to: &Potion::FIRE_RESISTANCE,
    },
    PotionRecipe {
        from: &Potion::FIRE_RESISTANCE,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_FIRE_RESISTANCE,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::RABBIT_FOOT],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::RABBIT_FOOT],
        to: &Potion::LEAPING,
    },
    PotionRecipe {
        from: &Potion::LEAPING,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_LEAPING,
    },
    PotionRecipe {
        from: &Potion::LEAPING,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::STRONG_LEAPING,
    },
    PotionRecipe {
        from: &Potion::LEAPING,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::SLOWNESS,
    },
    PotionRecipe {
        from: &Potion::LONG_LEAPING,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::LONG_SLOWNESS,
    },
    PotionRecipe {
        from: &Potion::SLOWNESS,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_SLOWNESS,
    },
    PotionRecipe {
        from: &Potion::SLOWNESS,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::STRONG_SLOWNESS,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::TURTLE_HELMET],
        to: &Potion::TURTLE_MASTER,
    },
    PotionRecipe {
        from: &Potion::TURTLE_MASTER,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_TURTLE_MASTER,
    },
    PotionRecipe {
        from: &Potion::TURTLE_MASTER,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::STRONG_TURTLE_MASTER,
    },
    PotionRecipe {
        from: &Potion::SWIFTNESS,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::SLOWNESS,
    },
    PotionRecipe {
        from: &Potion::LONG_SWIFTNESS,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::LONG_SLOWNESS,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::SUGAR],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::SUGAR],
        to: &Potion::SWIFTNESS,
    },
    PotionRecipe {
        from: &Potion::SWIFTNESS,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_SWIFTNESS,
    },
    PotionRecipe {
        from: &Potion::SWIFTNESS,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::STRONG_SWIFTNESS,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::PUFFERFISH],
        to: &Potion::WATER_BREATHING,
    },
    PotionRecipe {
        from: &Potion::WATER_BREATHING,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_WATER_BREATHING,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::GLISTERING_MELON_SLICE],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::GLISTERING_MELON_SLICE],
        to: &Potion::HEALING,
    },
    PotionRecipe {
        from: &Potion::HEALING,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::STRONG_HEALING,
    },
    PotionRecipe {
        from: &Potion::HEALING,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::HARMING,
    },
    PotionRecipe {
        from: &Potion::STRONG_HEALING,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::STRONG_HARMING,
    },
    PotionRecipe {
        from: &Potion::HARMING,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::STRONG_HARMING,
    },
    PotionRecipe {
        from: &Potion::POISON,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::HARMING,
    },
    PotionRecipe {
        from: &Potion::LONG_POISON,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::HARMING,
    },
    PotionRecipe {
        from: &Potion::STRONG_POISON,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::STRONG_HARMING,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::SPIDER_EYE],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::SPIDER_EYE],
        to: &Potion::POISON,
    },
    PotionRecipe {
        from: &Potion::POISON,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_POISON,
    },
    PotionRecipe {
        from: &Potion::POISON,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::STRONG_POISON,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::GHAST_TEAR],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::GHAST_TEAR],
        to: &Potion::REGENERATION,
    },
    PotionRecipe {
        from: &Potion::REGENERATION,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_REGENERATION,
    },
    PotionRecipe {
        from: &Potion::REGENERATION,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::STRONG_REGENERATION,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::BLAZE_POWDER],
        to: &Potion::MUNDANE,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::BLAZE_POWDER],
        to: &Potion::STRENGTH,
    },
    PotionRecipe {
        from: &Potion::STRENGTH,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_STRENGTH,
    },
    PotionRecipe {
        from: &Potion::STRENGTH,
        ingredient: &[&Item::GLOWSTONE_DUST],
        to: &Potion::STRONG_STRENGTH,
    },
    PotionRecipe {
        from: &Potion::WATER,
        ingredient: &[&Item::FERMENTED_SPIDER_EYE],
        to: &Potion::WEAKNESS,
    },
    PotionRecipe {
        from: &Potion::WEAKNESS,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_WEAKNESS,
    },
    PotionRecipe {
        from: &Potion::AWKWARD,
        ingredient: &[&Item::PHANTOM_MEMBRANE],
        to: &Potion::SLOW_FALLING,
    },
    PotionRecipe {
        from: &Potion::SLOW_FALLING,
        ingredient: &[&Item::REDSTONE],
        to: &Potion::LONG_SLOW_FALLING,
    },
];
