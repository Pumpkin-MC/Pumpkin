use pumpkin_data::item::Item;

/// A single entry in a trade pool from which trades are randomly selected.
pub struct TradePoolEntry {
    pub input1_item: &'static Item,
    pub input1_count: i32,
    pub input2_item: Option<&'static Item>,
    pub input2_count: i32,
    pub output_item: &'static Item,
    pub output_count: i32,
    pub max_uses: i32,
    pub xp_reward: i32,
    pub price_multiplier: f32,
}

/// A pool of possible trades for a given profession + level.
pub struct TradePool {
    pub entries: &'static [TradePoolEntry],
}

/// Get the trade pool for a profession at a specific level.
/// Returns None if the profession has no trades (None/Nitwit) or level is invalid.
pub fn get_trade_pool(profession: i32, level: i32) -> Option<&'static TradePool> {
    match profession {
        5 => FARMER_TRADES.get((level - 1) as usize),    // Farmer
        9 => LIBRARIAN_TRADES.get((level - 1) as usize),  // Librarian
        1 => ARMORER_TRADES.get((level - 1) as usize),    // Armorer
        2 => BUTCHER_TRADES.get((level - 1) as usize),    // Butcher
        4 => CLERIC_TRADES.get((level - 1) as usize),     // Cleric
        7 => FLETCHER_TRADES.get((level - 1) as usize),   // Fletcher
        14 => WEAPONSMITH_TRADES.get((level - 1) as usize), // Weaponsmith
        13 => TOOLSMITH_TRADES.get((level - 1) as usize),  // Toolsmith
        12 => SHEPHERD_TRADES.get((level - 1) as usize),   // Shepherd
        6 => FISHERMAN_TRADES.get((level - 1) as usize),   // Fisherman
        3 => CARTOGRAPHER_TRADES.get((level - 1) as usize), // Cartographer
        8 => LEATHERWORKER_TRADES.get((level - 1) as usize), // Leatherworker
        10 => MASON_TRADES.get((level - 1) as usize),     // Mason
        _ => None,
    }
}

/// Select random trades from a pool (up to `count` trades).
pub fn select_random_trades(pool: &TradePool, count: usize, seed: u64) -> Vec<usize> {
    if pool.entries.len() <= count {
        return (0..pool.entries.len()).collect();
    }

    let mut indices: Vec<usize> = (0..pool.entries.len()).collect();
    // Simple shuffle using seed
    let mut rng = seed;
    for i in (1..indices.len()).rev() {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let j = (rng >> 33) as usize % (i + 1);
        indices.swap(i, j);
    }
    indices.truncate(count);
    indices
}

// === FARMER TRADES ===
static FARMER_TRADES: [TradePool; 5] = [
    // Level 1
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::WHEAT, input1_count: 20, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::POTATO, input1_count: 26, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::CARROT, input1_count: 22, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::BEETROOT, input1_count: 15, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::BREAD, output_count: 6, max_uses: 16, xp_reward: 1, price_multiplier: 0.05 },
    ]},
    // Level 2
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::PUMPKIN, input1_count: 6, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::PUMPKIN_PIE, output_count: 4, max_uses: 12, xp_reward: 5, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::APPLE, output_count: 4, max_uses: 16, xp_reward: 5, price_multiplier: 0.05 },
    ]},
    // Level 3
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::MELON, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 20, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 3, input2_item: None, input2_count: 0, output_item: &Item::COOKIE, output_count: 18, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    // Level 4
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::CAKE, output_count: 1, max_uses: 12, xp_reward: 15, price_multiplier: 0.05 },
    ]},
    // Level 5
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 3, input2_item: None, input2_count: 0, output_item: &Item::GOLDEN_CARROT, output_count: 3, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::GLISTERING_MELON_SLICE, output_count: 3, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];

// === LIBRARIAN TRADES ===
static LIBRARIAN_TRADES: [TradePool; 5] = [
    // Level 1
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::PAPER, input1_count: 24, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 9, input2_item: Some(&Item::BOOK), input2_count: 1, output_item: &Item::BOOKSHELF, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.05 },
    ]},
    // Level 2
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::BOOK, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::LANTERN, output_count: 1, max_uses: 12, xp_reward: 5, price_multiplier: 0.05 },
    ]},
    // Level 3
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::INK_SAC, input1_count: 5, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 20, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::GLASS, output_count: 4, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    // Level 4
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 5, input2_item: None, input2_count: 0, output_item: &Item::CLOCK, output_count: 1, max_uses: 12, xp_reward: 15, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::COMPASS, output_count: 1, max_uses: 12, xp_reward: 15, price_multiplier: 0.05 },
    ]},
    // Level 5
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 20, input2_item: None, input2_count: 0, output_item: &Item::NAME_TAG, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];

// === ARMORER TRADES ===
static ARMORER_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::COAL, input1_count: 15, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 7, input2_item: None, input2_count: 0, output_item: &Item::IRON_LEGGINGS, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::IRON_BOOTS, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 5, input2_item: None, input2_count: 0, output_item: &Item::IRON_HELMET, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::IRON_INGOT, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 9, input2_item: None, input2_count: 0, output_item: &Item::IRON_CHESTPLATE, output_count: 1, max_uses: 12, xp_reward: 5, price_multiplier: 0.2 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::LAVA_BUCKET, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 20, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::SHIELD, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.2 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::DIAMOND, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 19, input2_item: None, input2_count: 0, output_item: &Item::DIAMOND_CHESTPLATE, output_count: 1, max_uses: 3, xp_reward: 30, price_multiplier: 0.2 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 13, input2_item: None, input2_count: 0, output_item: &Item::DIAMOND_LEGGINGS, output_count: 1, max_uses: 3, xp_reward: 30, price_multiplier: 0.2 },
    ]},
];

// === BUTCHER TRADES ===
static BUTCHER_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::CHICKEN, input1_count: 14, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::PORKCHOP, input1_count: 7, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::RABBIT, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::COAL, input1_count: 15, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::COOKED_PORKCHOP, output_count: 5, max_uses: 16, xp_reward: 5, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::COOKED_CHICKEN, output_count: 8, max_uses: 16, xp_reward: 5, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::MUTTON, input1_count: 7, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 20, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::BEEF, input1_count: 10, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 20, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::DRIED_KELP_BLOCK, input1_count: 10, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::SWEET_BERRIES, input1_count: 10, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];

// === CLERIC TRADES ===
static CLERIC_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::ROTTEN_FLESH, input1_count: 32, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::REDSTONE, output_count: 2, max_uses: 12, xp_reward: 1, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::GOLD_INGOT, input1_count: 3, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::LAPIS_LAZULI, output_count: 1, max_uses: 12, xp_reward: 5, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::RABBIT_FOOT, input1_count: 2, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 20, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::GLOWSTONE, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::TURTLE_SCUTE, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 5, input2_item: None, input2_count: 0, output_item: &Item::ENDER_PEARL, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];

// === FLETCHER TRADES ===
static FLETCHER_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::STICK, input1_count: 32, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::ARROW, output_count: 16, max_uses: 12, xp_reward: 1, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::FLINT, input1_count: 26, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 2, input2_item: None, input2_count: 0, output_item: &Item::BOW, output_count: 1, max_uses: 12, xp_reward: 5, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::STRING, input1_count: 14, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 20, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 3, input2_item: None, input2_count: 0, output_item: &Item::CROSSBOW, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::FEATHER, input1_count: 24, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 30, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::TRIPWIRE_HOOK, input1_count: 8, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];

// === WEAPONSMITH TRADES ===
static WEAPONSMITH_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::COAL, input1_count: 15, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 3, input2_item: None, input2_count: 0, output_item: &Item::IRON_AXE, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 2, input2_item: None, input2_count: 0, output_item: &Item::IRON_SWORD, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::IRON_INGOT, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::FLINT, input1_count: 24, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 20, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::DIAMOND, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 8, input2_item: None, input2_count: 0, output_item: &Item::DIAMOND_AXE, output_count: 1, max_uses: 3, xp_reward: 30, price_multiplier: 0.2 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 8, input2_item: None, input2_count: 0, output_item: &Item::DIAMOND_SWORD, output_count: 1, max_uses: 3, xp_reward: 30, price_multiplier: 0.2 },
    ]},
];

// === TOOLSMITH TRADES ===
static TOOLSMITH_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::COAL, input1_count: 15, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::STONE_AXE, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::STONE_SHOVEL, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::STONE_PICKAXE, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::STONE_HOE, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::IRON_INGOT, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::FLINT, input1_count: 30, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 20, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::DIAMOND, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 13, input2_item: None, input2_count: 0, output_item: &Item::DIAMOND_AXE, output_count: 1, max_uses: 3, xp_reward: 30, price_multiplier: 0.2 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 11, input2_item: None, input2_count: 0, output_item: &Item::DIAMOND_PICKAXE, output_count: 1, max_uses: 3, xp_reward: 30, price_multiplier: 0.2 },
    ]},
];

// === SHEPHERD TRADES ===
static SHEPHERD_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::WHITE_WOOL, input1_count: 18, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 2, input2_item: None, input2_count: 0, output_item: &Item::SHEARS, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::WHITE_CARPET, output_count: 4, max_uses: 16, xp_reward: 5, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::WHITE_BED, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 2, input2_item: None, input2_count: 0, output_item: &Item::WHITE_BANNER, output_count: 1, max_uses: 12, xp_reward: 15, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 3, input2_item: None, input2_count: 0, output_item: &Item::PAINTING, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];

// === FISHERMAN TRADES ===
static FISHERMAN_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::STRING, input1_count: 20, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::COAL, input1_count: 10, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 3, input2_item: Some(&Item::COD), input2_count: 6, output_item: &Item::COOKED_COD, output_count: 6, max_uses: 16, xp_reward: 1, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::COD, input1_count: 15, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::SALMON, input1_count: 13, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 20, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::TROPICAL_FISH, input1_count: 6, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::PUFFERFISH, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];

// === CARTOGRAPHER TRADES ===
static CARTOGRAPHER_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::PAPER, input1_count: 24, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::GLASS_PANE, input1_count: 11, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::COMPASS, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 20, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 7, input2_item: None, input2_count: 0, output_item: &Item::ITEM_FRAME, output_count: 1, max_uses: 12, xp_reward: 15, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 8, input2_item: None, input2_count: 0, output_item: &Item::GLOBE_BANNER_PATTERN, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];

// === LEATHERWORKER TRADES ===
static LEATHERWORKER_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::LEATHER, input1_count: 6, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 3, input2_item: None, input2_count: 0, output_item: &Item::LEATHER_LEGGINGS, output_count: 1, max_uses: 12, xp_reward: 1, price_multiplier: 0.2 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::FLINT, input1_count: 26, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 10, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 7, input2_item: None, input2_count: 0, output_item: &Item::LEATHER_CHESTPLATE, output_count: 1, max_uses: 12, xp_reward: 5, price_multiplier: 0.2 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::RABBIT_HIDE, input1_count: 9, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 20, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::TURTLE_SCUTE, input1_count: 4, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 6, input2_item: None, input2_count: 0, output_item: &Item::SADDLE, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];

// === MASON TRADES ===
static MASON_TRADES: [TradePool; 5] = [
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::CLAY_BALL, input1_count: 10, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 2, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::BRICK, output_count: 10, max_uses: 16, xp_reward: 1, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::STONE, input1_count: 20, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 10, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::GRANITE, input1_count: 16, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 20, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::ANDESITE, input1_count: 16, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 20, price_multiplier: 0.05 },
        TradePoolEntry { input1_item: &Item::DIORITE, input1_count: 16, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 16, xp_reward: 20, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::QUARTZ, input1_count: 12, input2_item: None, input2_count: 0, output_item: &Item::EMERALD, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
    TradePool { entries: &[
        TradePoolEntry { input1_item: &Item::EMERALD, input1_count: 1, input2_item: None, input2_count: 0, output_item: &Item::QUARTZ_PILLAR, output_count: 1, max_uses: 12, xp_reward: 30, price_multiplier: 0.05 },
    ]},
];
