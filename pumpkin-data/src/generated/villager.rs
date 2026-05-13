/* This file is generated. Do not edit manually. */
use serde::Serialize;
#[derive(Clone, Copy, PartialEq)]
pub struct VillagerTradeItem {
    pub item: &'static crate::item::Item,
    pub count: i32,
}
#[derive(Clone, Copy, PartialEq)]
pub struct VillagerTrade {
    pub wants: VillagerTradeItem,
    pub wants_b: Option<VillagerTradeItem>,
    pub gives: VillagerTradeItem,
    pub max_uses: i32,
    pub xp: i32,
    pub price_multiplier: f32,
}
#[derive(Clone, Copy, PartialEq)]
pub struct VillagerTradeSet {
    pub trades: &'static [VillagerTrade],
    pub amount: i32,
}
pub const TRADES_ARMORER_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::IRON_BOOTS,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 9i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::IRON_CHESTPLATE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 5i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::IRON_HELMET,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 7i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::IRON_LEGGINGS,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_ARMORER_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CHAINMAIL_BOOTS,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CHAINMAIL_LEGGINGS,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_ARMORER_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CHAINMAIL_CHESTPLATE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CHAINMAIL_HELMET,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 5i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::SHIELD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::LAVA_BUCKET,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_ARMORER_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND_BOOTS,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 14i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND_LEGGINGS,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_ARMORER_LEVEL_5: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 16i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND_CHESTPLATE,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND_HELMET,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_BUTCHER_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::CHICKEN,
            count: 14i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::RABBIT_STEW,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::PORKCHOP,
            count: 7i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::RABBIT,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_BUTCHER_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::COAL,
            count: 15i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::COOKED_CHICKEN,
            count: 8i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::COOKED_PORKCHOP,
            count: 5i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_BUTCHER_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::BEEF,
            count: 10i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::MUTTON,
            count: 7i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_BUTCHER_LEVEL_4: &[VillagerTrade] = &[VillagerTrade {
    wants: VillagerTradeItem {
        item: &crate::item::Item::DRIED_KELP_BLOCK,
        count: 10i32,
    },
    wants_b: None,
    gives: VillagerTradeItem {
        item: &crate::item::Item::EMERALD,
        count: 1i32,
    },
    max_uses: 12i32,
    xp: 30i32,
    price_multiplier: 0.05f32,
}];
pub const TRADES_BUTCHER_LEVEL_5: &[VillagerTrade] = &[VillagerTrade {
    wants: VillagerTradeItem {
        item: &crate::item::Item::SWEET_BERRIES,
        count: 10i32,
    },
    wants_b: None,
    gives: VillagerTradeItem {
        item: &crate::item::Item::EMERALD,
        count: 1i32,
    },
    max_uses: 12i32,
    xp: 30i32,
    price_multiplier: 0.05f32,
}];
pub const TRADES_CARTOGRAPHER_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 7i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::PAPER,
            count: 24i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_CARTOGRAPHER_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::GLASS_PANE,
            count: 11i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_CARTOGRAPHER_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::COMPASS,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 13i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_CARTOGRAPHER_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLACK_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLUE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BROWN_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CYAN_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GRAY_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GREEN_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 7i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ITEM_FRAME,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_BLUE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIME_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAGENTA_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ORANGE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PINK_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PURPLE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::RED_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::WHITE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::YELLOW_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_CARTOGRAPHER_LEVEL_5: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 14i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAP,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GLOBE_BANNER_PATTERN,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_CLERIC_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::REDSTONE,
            count: 2i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::ROTTEN_FLESH,
            count: 32i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_CLERIC_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LAPIS_LAZULI,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::GOLD_INGOT,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_CLERIC_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GLOWSTONE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::RABBIT_FOOT,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_CLERIC_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 5i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ENDER_PEARL,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::GLASS_BOTTLE,
            count: 9i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::TURTLE_SCUTE,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_CLERIC_LEVEL_5: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EXPERIENCE_BOTTLE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::NETHER_WART,
            count: 22i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FARMER_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::BEETROOT,
            count: 15i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::CARROT,
            count: 22i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BREAD,
            count: 6i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::POTATO,
            count: 26i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::WHEAT,
            count: 20i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FARMER_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::APPLE,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PUMPKIN_PIE,
            count: 4i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::PUMPKIN,
            count: 6i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FARMER_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::COOKIE,
            count: 18i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::MELON,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FARMER_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CAKE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::SUSPICIOUS_STEW,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FARMER_LEVEL_5: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GLISTERING_MELON_SLICE,
            count: 3i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GOLDEN_CARROT,
            count: 3i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FISHERMAN_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::COAL,
            count: 10i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::COD_BUCKET,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::COD,
            count: 6i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::COOKED_COD,
            count: 6i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::STRING,
            count: 20i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FISHERMAN_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::COD,
            count: 15i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CAMPFIRE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::SALMON,
            count: 6i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::COOKED_SALMON,
            count: 6i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FISHERMAN_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::FISHING_ROD,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::SALMON,
            count: 13i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FISHERMAN_LEVEL_4: &[VillagerTrade] = &[VillagerTrade {
    wants: VillagerTradeItem {
        item: &crate::item::Item::TROPICAL_FISH,
        count: 6i32,
    },
    wants_b: None,
    gives: VillagerTradeItem {
        item: &crate::item::Item::EMERALD,
        count: 1i32,
    },
    max_uses: 12i32,
    xp: 30i32,
    price_multiplier: 0.05f32,
}];
pub const TRADES_FISHERMAN_LEVEL_5: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::ACACIA_BOAT,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::DARK_OAK_BOAT,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::JUNGLE_BOAT,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::OAK_BOAT,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::PUFFERFISH,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::SPRUCE_BOAT,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FLETCHER_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ARROW,
            count: 16i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::GRAVEL,
            count: 10i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::FLINT,
            count: 10i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::STICK,
            count: 32i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FLETCHER_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BOW,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::FLINT,
            count: 26i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FLETCHER_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CROSSBOW,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::STRING,
            count: 14i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FLETCHER_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BOW,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::FEATHER,
            count: 24i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_FLETCHER_LEVEL_5: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::ARROW,
            count: 5i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::TIPPED_ARROW,
            count: 5i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CROSSBOW,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::TRIPWIRE_HOOK,
            count: 8i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LEATHERWORKER_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 7i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LEATHER_CHESTPLATE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LEATHER_LEGGINGS,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::LEATHER,
            count: 6i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LEATHERWORKER_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LEATHER_BOOTS,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 5i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LEATHER_HELMET,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::FLINT,
            count: 26i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LEATHERWORKER_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 7i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LEATHER_CHESTPLATE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::RABBIT_HIDE,
            count: 9i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LEATHERWORKER_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 6i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LEATHER_HORSE_ARMOR,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::TURTLE_SCUTE,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LEATHERWORKER_LEVEL_5: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 5i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LEATHER_HELMET,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 6i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::SADDLE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LIBRARIAN_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 0i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ENCHANTED_BOOK,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 9i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BOOKSHELF,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::PAPER,
            count: 24i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LIBRARIAN_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::BOOK,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 0i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ENCHANTED_BOOK,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LANTERN,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LIBRARIAN_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 0i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ENCHANTED_BOOK,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GLASS,
            count: 4i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::INK_SAC,
            count: 5i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LIBRARIAN_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 0i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ENCHANTED_BOOK,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 5i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CLOCK,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::COMPASS,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::WRITABLE_BOOK,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_LIBRARIAN_LEVEL_5: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::RED_CANDLE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::YELLOW_CANDLE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_MASON_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::CLAY_BALL,
            count: 10i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BRICK,
            count: 10i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_MASON_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CHISELED_STONE_BRICKS,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::STONE,
            count: 20i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_MASON_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::ANDESITE,
            count: 16i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::DIORITE,
            count: 16i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::DRIPSTONE_BLOCK,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::POLISHED_ANDESITE,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::POLISHED_DIORITE,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::POLISHED_GRANITE,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::GRANITE,
            count: 16i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_MASON_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLACK_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLACK_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLUE_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLUE_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BROWN_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BROWN_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CYAN_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CYAN_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GRAY_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GRAY_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GREEN_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GREEN_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_BLUE_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_BLUE_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_GRAY_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_GRAY_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIME_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIME_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAGENTA_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAGENTA_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ORANGE_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ORANGE_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PINK_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PINK_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PURPLE_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PURPLE_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::RED_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::RED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::WHITE_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::WHITE_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::YELLOW_GLAZED_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::YELLOW_TERRACOTTA,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::QUARTZ,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_MASON_LEVEL_5: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::QUARTZ_BLOCK,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::QUARTZ_PILLAR,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_SHEPHERD_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::BLACK_WOOL,
            count: 18i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::BROWN_WOOL,
            count: 18i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::SHEARS,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::GRAY_WOOL,
            count: 18i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::WHITE_WOOL,
            count: 18i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_SHEPHERD_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::BLACK_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLACK_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLACK_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLUE_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLUE_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BROWN_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BROWN_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CYAN_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CYAN_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GRAY_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GRAY_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GREEN_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GREEN_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_BLUE_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_BLUE_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_GRAY_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_GRAY_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIME_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIME_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAGENTA_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAGENTA_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ORANGE_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ORANGE_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PINK_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PINK_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PURPLE_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PURPLE_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::RED_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::RED_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::WHITE_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::WHITE_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::YELLOW_CARPET,
            count: 4i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::YELLOW_WOOL,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::GRAY_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_BLUE_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::LIME_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::WHITE_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_SHEPHERD_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLACK_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLUE_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BROWN_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CYAN_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GRAY_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GREEN_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_BLUE_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_GRAY_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIME_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAGENTA_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ORANGE_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PINK_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PURPLE_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::RED_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::WHITE_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::YELLOW_BED,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_GRAY_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::ORANGE_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::PINK_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::RED_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::YELLOW_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_SHEPHERD_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::BLUE_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::BROWN_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::CYAN_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLACK_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BLUE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BROWN_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::CYAN_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GRAY_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::GREEN_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_BLUE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIGHT_GRAY_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::LIME_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::MAGENTA_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::ORANGE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PINK_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::PURPLE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::RED_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::WHITE_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::YELLOW_BANNER,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::GREEN_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::MAGENTA_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::PURPLE_DYE,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 16i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_SHEPHERD_LEVEL_5: &[VillagerTrade] = &[VillagerTrade {
    wants: VillagerTradeItem {
        item: &crate::item::Item::EMERALD,
        count: 2i32,
    },
    wants_b: None,
    gives: VillagerTradeItem {
        item: &crate::item::Item::PAINTING,
        count: 3i32,
    },
    max_uses: 12i32,
    xp: 30i32,
    price_multiplier: 0.05f32,
}];
pub const TRADES_TOOLSMITH_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::STONE_AXE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::STONE_HOE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::STONE_PICKAXE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::STONE_SHOVEL,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_TOOLSMITH_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 36i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BELL,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::IRON_INGOT,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_TOOLSMITH_LEVEL_3: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND_HOE,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::IRON_AXE,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::IRON_PICKAXE,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::IRON_SHOVEL,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::FLINT,
            count: 30i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 20i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_TOOLSMITH_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND_AXE,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 5i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND_SHOVEL,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_TOOLSMITH_LEVEL_5: &[VillagerTrade] = &[VillagerTrade {
    wants: VillagerTradeItem {
        item: &crate::item::Item::EMERALD,
        count: 13i32,
    },
    wants_b: None,
    gives: VillagerTradeItem {
        item: &crate::item::Item::DIAMOND_PICKAXE,
        count: 1i32,
    },
    max_uses: 3i32,
    xp: 30i32,
    price_multiplier: 0.05f32,
}];
pub const TRADES_WEAPONSMITH_LEVEL_1: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 2i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::IRON_SWORD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 3i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::IRON_AXE,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 2i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_WEAPONSMITH_LEVEL_2: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 36i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::BELL,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 5i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::IRON_INGOT,
            count: 4i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 10i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_WEAPONSMITH_LEVEL_3: &[VillagerTrade] = &[VillagerTrade {
    wants: VillagerTradeItem {
        item: &crate::item::Item::FLINT,
        count: 24i32,
    },
    wants_b: None,
    gives: VillagerTradeItem {
        item: &crate::item::Item::EMERALD,
        count: 1i32,
    },
    max_uses: 12i32,
    xp: 20i32,
    price_multiplier: 0.05f32,
}];
pub const TRADES_WEAPONSMITH_LEVEL_4: &[VillagerTrade] = &[
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND,
            count: 1i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 1i32,
        },
        max_uses: 12i32,
        xp: 30i32,
        price_multiplier: 0.05f32,
    },
    VillagerTrade {
        wants: VillagerTradeItem {
            item: &crate::item::Item::EMERALD,
            count: 12i32,
        },
        wants_b: None,
        gives: VillagerTradeItem {
            item: &crate::item::Item::DIAMOND_AXE,
            count: 1i32,
        },
        max_uses: 3i32,
        xp: 15i32,
        price_multiplier: 0.05f32,
    },
];
pub const TRADES_WEAPONSMITH_LEVEL_5: &[VillagerTrade] = &[VillagerTrade {
    wants: VillagerTradeItem {
        item: &crate::item::Item::EMERALD,
        count: 8i32,
    },
    wants_b: None,
    gives: VillagerTradeItem {
        item: &crate::item::Item::DIAMOND_SWORD,
        count: 1i32,
    },
    max_uses: 3i32,
    xp: 30i32,
    price_multiplier: 0.05f32,
}];
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[repr(i32)]
pub enum VillagerProfession {
    None,
    Armorer,
    Butcher,
    Cartographer,
    Cleric,
    Farmer,
    Fisherman,
    Fletcher,
    Leatherworker,
    Librarian,
    Mason,
    Nitwit,
    Shepherd,
    Toolsmith,
    Weaponsmith,
}
impl VillagerProfession {
    #[must_use]
    pub fn from_i32(id: i32) -> Option<Self> {
        match id {
            0i32 => Some(Self::None),
            1i32 => Some(Self::Armorer),
            2i32 => Some(Self::Butcher),
            3i32 => Some(Self::Cartographer),
            4i32 => Some(Self::Cleric),
            5i32 => Some(Self::Farmer),
            6i32 => Some(Self::Fisherman),
            7i32 => Some(Self::Fletcher),
            8i32 => Some(Self::Leatherworker),
            9i32 => Some(Self::Librarian),
            10i32 => Some(Self::Mason),
            11i32 => Some(Self::Nitwit),
            12i32 => Some(Self::Shepherd),
            13i32 => Some(Self::Toolsmith),
            14i32 => Some(Self::Weaponsmith),
            _ => None,
        }
    }
    #[must_use]
    pub fn work_sound(&self) -> Option<crate::sound::Sound> {
        match self {
            Self::None => None,
            Self::Armorer => Some(crate::sound::Sound::EntityVillagerWorkArmorer),
            Self::Butcher => Some(crate::sound::Sound::EntityVillagerWorkButcher),
            Self::Cartographer => Some(crate::sound::Sound::EntityVillagerWorkCartographer),
            Self::Cleric => Some(crate::sound::Sound::EntityVillagerWorkCleric),
            Self::Farmer => Some(crate::sound::Sound::EntityVillagerWorkFarmer),
            Self::Fisherman => Some(crate::sound::Sound::EntityVillagerWorkFisherman),
            Self::Fletcher => Some(crate::sound::Sound::EntityVillagerWorkFletcher),
            Self::Leatherworker => Some(crate::sound::Sound::EntityVillagerWorkLeatherworker),
            Self::Librarian => Some(crate::sound::Sound::EntityVillagerWorkLibrarian),
            Self::Mason => Some(crate::sound::Sound::EntityVillagerWorkMason),
            Self::Nitwit => None,
            Self::Shepherd => Some(crate::sound::Sound::EntityVillagerWorkShepherd),
            Self::Toolsmith => Some(crate::sound::Sound::EntityVillagerWorkToolsmith),
            Self::Weaponsmith => Some(crate::sound::Sound::EntityVillagerWorkWeaponsmith),
        }
    }
    #[must_use]
    pub fn requested_items(&self) -> &'static [&'static crate::item::Item] {
        match self {
            Self::None => &[],
            Self::Armorer => &[],
            Self::Butcher => &[],
            Self::Cartographer => &[],
            Self::Cleric => &[],
            Self::Farmer => &[
                &crate::item::Item::WHEAT,
                &crate::item::Item::WHEAT_SEEDS,
                &crate::item::Item::BEETROOT_SEEDS,
                &crate::item::Item::BONE_MEAL,
            ],
            Self::Fisherman => &[],
            Self::Fletcher => &[],
            Self::Leatherworker => &[],
            Self::Librarian => &[],
            Self::Mason => &[],
            Self::Nitwit => &[],
            Self::Shepherd => &[],
            Self::Toolsmith => &[],
            Self::Weaponsmith => &[],
        }
    }
    #[must_use]
    pub fn translation_key(&self) -> &'static str {
        match self {
            Self::None => "entity.minecraft.villager.none",
            Self::Armorer => "entity.minecraft.villager.armorer",
            Self::Butcher => "entity.minecraft.villager.butcher",
            Self::Cartographer => "entity.minecraft.villager.cartographer",
            Self::Cleric => "entity.minecraft.villager.cleric",
            Self::Farmer => "entity.minecraft.villager.farmer",
            Self::Fisherman => "entity.minecraft.villager.fisherman",
            Self::Fletcher => "entity.minecraft.villager.fletcher",
            Self::Leatherworker => "entity.minecraft.villager.leatherworker",
            Self::Librarian => "entity.minecraft.villager.librarian",
            Self::Mason => "entity.minecraft.villager.mason",
            Self::Nitwit => "entity.minecraft.villager.nitwit",
            Self::Shepherd => "entity.minecraft.villager.shepherd",
            Self::Toolsmith => "entity.minecraft.villager.toolsmith",
            Self::Weaponsmith => "entity.minecraft.villager.weaponsmith",
        }
    }
    #[must_use]
    pub fn trade_set(&self, level: i32) -> Option<VillagerTradeSet> {
        match self {
            Self::None => None,
            Self::Armorer => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_ARMORER_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_ARMORER_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_ARMORER_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_ARMORER_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_ARMORER_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Butcher => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_BUTCHER_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_BUTCHER_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_BUTCHER_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_BUTCHER_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_BUTCHER_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Cartographer => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_CARTOGRAPHER_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_CARTOGRAPHER_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_CARTOGRAPHER_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_CARTOGRAPHER_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_CARTOGRAPHER_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Cleric => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_CLERIC_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_CLERIC_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_CLERIC_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_CLERIC_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_CLERIC_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Farmer => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_FARMER_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_FARMER_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_FARMER_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_FARMER_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_FARMER_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Fisherman => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_FISHERMAN_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_FISHERMAN_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_FISHERMAN_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_FISHERMAN_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_FISHERMAN_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Fletcher => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_FLETCHER_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_FLETCHER_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_FLETCHER_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_FLETCHER_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_FLETCHER_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Leatherworker => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_LEATHERWORKER_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_LEATHERWORKER_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_LEATHERWORKER_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_LEATHERWORKER_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_LEATHERWORKER_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Librarian => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_LIBRARIAN_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_LIBRARIAN_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_LIBRARIAN_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_LIBRARIAN_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_LIBRARIAN_LEVEL_5,
                    amount: 3i32,
                }),
                _ => None,
            },
            Self::Mason => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_MASON_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_MASON_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_MASON_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_MASON_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_MASON_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Nitwit => None,
            Self::Shepherd => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_SHEPHERD_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_SHEPHERD_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_SHEPHERD_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_SHEPHERD_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_SHEPHERD_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Toolsmith => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_TOOLSMITH_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_TOOLSMITH_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_TOOLSMITH_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_TOOLSMITH_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_TOOLSMITH_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
            Self::Weaponsmith => match level {
                1i32 => Some(VillagerTradeSet {
                    trades: TRADES_WEAPONSMITH_LEVEL_1,
                    amount: 2i32,
                }),
                2i32 => Some(VillagerTradeSet {
                    trades: TRADES_WEAPONSMITH_LEVEL_2,
                    amount: 2i32,
                }),
                3i32 => Some(VillagerTradeSet {
                    trades: TRADES_WEAPONSMITH_LEVEL_3,
                    amount: 2i32,
                }),
                4i32 => Some(VillagerTradeSet {
                    trades: TRADES_WEAPONSMITH_LEVEL_4,
                    amount: 2i32,
                }),
                5i32 => Some(VillagerTradeSet {
                    trades: TRADES_WEAPONSMITH_LEVEL_5,
                    amount: 2i32,
                }),
                _ => None,
            },
        }
    }
}
impl TryFrom<i32> for VillagerProfession {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[repr(i32)]
pub enum VillagerType {
    Desert,
    Jungle,
    Plains,
    Savanna,
    Snow,
    Swamp,
    Taiga,
}
impl VillagerType {
    #[must_use]
    pub fn from_i32(id: i32) -> Option<Self> {
        match id {
            0i32 => Some(Self::Desert),
            1i32 => Some(Self::Jungle),
            2i32 => Some(Self::Plains),
            3i32 => Some(Self::Savanna),
            4i32 => Some(Self::Snow),
            5i32 => Some(Self::Swamp),
            6i32 => Some(Self::Taiga),
            _ => None,
        }
    }
}
impl TryFrom<i32> for VillagerType {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_i32(value).ok_or(())
    }
}
