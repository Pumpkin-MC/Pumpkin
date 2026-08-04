use heck::{ToPascalCase, ToShoutySnakeCase};
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct VillagerDataJson {
    professions: IndexMap<String, ProfessionJson>,
    types: IndexMap<String, String>,
    trade_sets: IndexMap<String, TradeSetJson>,
    villager_trades: IndexMap<String, TradeJson>,
}

#[derive(Deserialize)]
struct ProfessionJson {
    name: NameJson,
    requested_items: Vec<String>,
    work_sound: Option<String>,
    trade_sets: IndexMap<String, String>,
}

#[derive(Deserialize)]
struct NameJson {
    translate: String,
}

#[derive(Deserialize)]
struct TradeSetJson {
    trades: String, // Tag like "#minecraft:armorer/level_1"
    amount: f32,
}

#[derive(Deserialize)]
struct TradeJson {
    wants: TradeItemJson,
    wants_b: Option<TradeItemJson>,
    gives: TradeItemJson,
    max_uses: Option<f32>,
    xp: Option<f32>,
    price_multiplier: Option<f32>,
}

#[derive(Deserialize)]
struct TradeItemJson {
    id: String,
    count: Option<f32>,
}

/// `data/minecraft/tags/villager_trade/wandering_trader/buying.json` (vanilla 26.2 decompiled
/// source). Unlike profession trade sets, whose vanilla trade keys are named
/// `<profession>/level_<n>/<trade>` (so a simple key-prefix match recovers tag membership),
/// wandering trader trade keys are flat (`wandering_trader/<trade>`) with no level/category
/// segment -- the buying/common/uncommon split is a tag, not derivable from the key name, so
/// membership must be listed explicitly from the actual tag file.
const WANDERING_TRADER_BUYING_KEYS: &[&str] = &[
    "wandering_trader/water_bottle_emerald",
    "wandering_trader/water_bucket_emerald",
    "wandering_trader/milk_bucket_emerald",
    "wandering_trader/fermented_spider_eye_emerald",
    "wandering_trader/baked_potato_emerald",
    "wandering_trader/hay_block_emerald",
];

/// `data/minecraft/tags/villager_trade/wandering_trader/common.json`.
const WANDERING_TRADER_COMMON_KEYS: &[&str] = &[
    "wandering_trader/emerald_white_dye",
    "wandering_trader/emerald_orange_dye",
    "wandering_trader/emerald_magenta_dye",
    "wandering_trader/emerald_light_blue_dye",
    "wandering_trader/emerald_yellow_dye",
    "wandering_trader/emerald_lime_dye",
    "wandering_trader/emerald_pink_dye",
    "wandering_trader/emerald_gray_dye",
    "wandering_trader/emerald_light_gray_dye",
    "wandering_trader/emerald_cyan_dye",
    "wandering_trader/emerald_purple_dye",
    "wandering_trader/emerald_blue_dye",
    "wandering_trader/emerald_brown_dye",
    "wandering_trader/emerald_green_dye",
    "wandering_trader/emerald_red_dye",
    "wandering_trader/emerald_black_dye",
    "wandering_trader/emerald_fish_bucket",
    "wandering_trader/emerald_pufferfish_bucket",
    "wandering_trader/emerald_sea_pickle",
    "wandering_trader/emerald_slime_ball",
    "wandering_trader/emerald_glowstone",
    "wandering_trader/emerald_nautilus_shell",
    "wandering_trader/emerald_fern",
    "wandering_trader/emerald_sugar_cane",
    "wandering_trader/emerald_pumpkin",
    "wandering_trader/emerald_kelp",
    "wandering_trader/emerald_cactus",
    "wandering_trader/emerald_dandelion",
    "wandering_trader/emerald_poppy",
    "wandering_trader/emerald_blue_orchid",
    "wandering_trader/emerald_allium",
    "wandering_trader/emerald_azure_bluet",
    "wandering_trader/emerald_red_tulip",
    "wandering_trader/emerald_orange_tulip",
    "wandering_trader/emerald_white_tulip",
    "wandering_trader/emerald_pink_tulip",
    "wandering_trader/emerald_oxeye_daisy",
    "wandering_trader/emerald_cornflower",
    "wandering_trader/emerald_lily_of_the_valley",
    "wandering_trader/emerald_open_eyeblossom",
    "wandering_trader/emerald_wheat_seeds",
    "wandering_trader/emerald_beetroot_seeds",
    "wandering_trader/emerald_pumpkin_seeds",
    "wandering_trader/emerald_melon_seeds",
    "wandering_trader/emerald_acacia_sapling",
    "wandering_trader/emerald_birch_sapling",
    "wandering_trader/emerald_dark_oak_sapling",
    "wandering_trader/emerald_jungle_sapling",
    "wandering_trader/emerald_oak_sapling",
    "wandering_trader/emerald_spruce_sapling",
    "wandering_trader/emerald_cherry_sapling",
    "wandering_trader/emerald_pale_oak_sapling",
    "wandering_trader/emerald_mangrove_propagule",
    "wandering_trader/emerald_brain_coral_block",
    "wandering_trader/emerald_bubble_coral_block",
    "wandering_trader/emerald_fire_coral_block",
    "wandering_trader/emerald_horn_coral_block",
    "wandering_trader/emerald_tube_coral_block",
    "wandering_trader/emerald_vine",
    "wandering_trader/emerald_pale_hanging_moss",
    "wandering_trader/emerald_brown_mushroom",
    "wandering_trader/emerald_red_mushroom",
    "wandering_trader/emerald_lily_pad",
    "wandering_trader/emerald_small_dripleaf",
    "wandering_trader/emerald_sand",
    "wandering_trader/emerald_red_sand",
    "wandering_trader/emerald_pointed_dripstone",
    "wandering_trader/emerald_sulfur_spike",
    "wandering_trader/emerald_rooted_dirt",
    "wandering_trader/emerald_moss_block",
    "wandering_trader/emerald_pale_moss_block",
    "wandering_trader/emerald_wildflowers",
    "wandering_trader/emerald_dry_tall_grass",
    "wandering_trader/emerald_firefly_bush",
    "wandering_trader/emerald_golden_dandelion",
    "wandering_trader/emerald_name_tag",
];

/// `data/minecraft/tags/villager_trade/wandering_trader/uncommon.json`.
const WANDERING_TRADER_UNCOMMON_KEYS: &[&str] = &[
    "wandering_trader/emerald_packed_ice",
    "wandering_trader/emerald_blue_ice",
    "wandering_trader/emerald_gunpowder",
    "wandering_trader/emerald_podzol",
    "wandering_trader/emerald_acacia_log",
    "wandering_trader/emerald_birch_log",
    "wandering_trader/emerald_dark_oak_log",
    "wandering_trader/emerald_jungle_log",
    "wandering_trader/emerald_oak_log",
    "wandering_trader/emerald_spruce_log",
    "wandering_trader/emerald_cherry_log",
    "wandering_trader/emerald_mangrove_log",
    "wandering_trader/emerald_pale_oak_log",
    "wandering_trader/emerald_enchanted_iron_pickaxe",
    "wandering_trader/emerald_long_invisibility_potion",
];

pub fn build() -> TokenStream {
    let data: VillagerDataJson =
        serde_json::from_str(&fs::read_to_string("../assets/villager_data.json").unwrap())
            .expect("Failed to parse villager_data.json");

    let mut profession_variants = Vec::new();
    let mut type_variants = Vec::new();

    let mut work_sounds = Vec::new();
    let mut requested_items = Vec::new();
    let mut profession_names = Vec::new();

    let mut profession_from_i32 = Vec::new();
    let mut type_from_i32 = Vec::new();

    let mut trade_set_data = Vec::new();
    let mut generated_trade_sets = IndexMap::new();
    let mut wandering_trader_trade_sets = Vec::new();

    // Helper to format a trade into TokenStream
    let format_trade = |trade: &TradeJson| {
        let wants_item = format_ident!(
            "{}",
            trade
                .wants
                .id
                .strip_prefix("minecraft:")
                .unwrap_or(&trade.wants.id)
                .to_shouty_snake_case()
        );
        let wants_count = trade.wants.count.unwrap_or(1.0) as i32;
        let wants = quote! { VillagerTradeItem { item: &crate::item::Item::#wants_item, count: #wants_count } };

        let wants_b = if let Some(b) = &trade.wants_b {
            let item = format_ident!(
                "{}",
                b.id.strip_prefix("minecraft:")
                    .unwrap_or(&b.id)
                    .to_shouty_snake_case()
            );
            let count = b.count.unwrap_or(1.0) as i32;
            quote! { Some(VillagerTradeItem { item: &crate::item::Item::#item, count: #count }) }
        } else {
            quote! { None }
        };

        let gives_item = format_ident!(
            "{}",
            trade
                .gives
                .id
                .strip_prefix("minecraft:")
                .unwrap_or(&trade.gives.id)
                .to_shouty_snake_case()
        );
        let gives_count = trade.gives.count.unwrap_or(1.0) as i32;
        let gives = quote! { VillagerTradeItem { item: &crate::item::Item::#gives_item, count: #gives_count } };

        let max_uses = trade.max_uses.unwrap_or(16.0) as i32;
        let xp = trade.xp.unwrap_or(2.0) as i32;
        let price_multiplier = trade.price_multiplier.unwrap_or(0.05);

        quote! {
            VillagerTrade {
                wants: #wants,
                wants_b: #wants_b,
                gives: #gives,
                max_uses: #max_uses,
                xp: #xp,
                price_multiplier: #price_multiplier,
            }
        }
    };

    // Pre-process all trade sets mentioned in trade_sets map
    for (_set_key, set_data) in &data.trade_sets {
        let tag = &set_data.trades;
        if !tag.starts_with("#minecraft:") {
            continue;
        }
        let tag_content = tag.strip_prefix("#minecraft:").unwrap();
        let parts: Vec<&str> = tag_content.split('/').collect();
        if parts.len() < 2 {
            continue;
        }
        let prof = parts[0];
        let level_str = parts[1].strip_prefix("level_").unwrap_or(parts[1]);

        let mut matching_trades = Vec::new();

        if prof == "wandering_trader" {
            let keys: &[&str] = match level_str {
                "buying" => WANDERING_TRADER_BUYING_KEYS,
                "common" => WANDERING_TRADER_COMMON_KEYS,
                "uncommon" => WANDERING_TRADER_UNCOMMON_KEYS,
                _ => &[],
            };
            for key in keys {
                if let Some(trade) = data.villager_trades.get(*key) {
                    matching_trades.push(format_trade(trade));
                }
            }
        } else {
            let prefix = format!("{prof}/{level_str}/");
            for (key, trade) in &data.villager_trades {
                if key.starts_with(&prefix) {
                    matching_trades.push(format_trade(trade));
                }
            }

            // Fallback for smiths
            if matching_trades.is_empty()
                && (prof == "armorer" || prof == "toolsmith" || prof == "weaponsmith")
            {
                let smith_prefix = format!("smith/{level_str}/");
                for (key, trade) in &data.villager_trades {
                    if key.starts_with(&smith_prefix) {
                        matching_trades.push(format_trade(trade));
                    }
                }
            }
        }

        if !matching_trades.is_empty() {
            let ident_name = tag_content.replace('/', "_").to_shouty_snake_case();
            let ident = format_ident!("TRADES_{}", ident_name);
            trade_set_data.push(quote! {
                pub const #ident: &[VillagerTrade] = &[
                    #(#matching_trades),*
                ];
            });
            if prof == "wandering_trader" {
                let amount = set_data.amount as i32;
                let const_ident =
                    format_ident!("WANDERING_TRADER_TRADE_SET_{}", level_str.to_shouty_snake_case());
                wandering_trader_trade_sets.push(quote! {
                    pub const #const_ident: VillagerTradeSet = VillagerTradeSet {
                        trades: #ident,
                        amount: #amount,
                    };
                });
            }

            generated_trade_sets.insert(tag.clone(), ident);
        }
    }

    let mut profession_trade_sets = Vec::new();

    for (i, (name, prof_data)) in data.professions.iter().enumerate() {
        let ident = format_ident!("{}", name.to_pascal_case());
        profession_variants.push(quote! { #ident });

        let sound = if let Some(sound) = &prof_data.work_sound {
            let sound_ident = format_ident!(
                "{}",
                sound
                    .strip_prefix("minecraft:")
                    .unwrap_or(sound)
                    .replace('.', "_")
                    .to_pascal_case()
            );
            quote! { Some(crate::sound::Sound::#sound_ident) }
        } else {
            quote! { None }
        };
        work_sounds.push(quote! { Self::#ident => #sound });

        let items: Vec<_> = prof_data
            .requested_items
            .iter()
            .map(|i| {
                let item_ident = format_ident!(
                    "{}",
                    i.strip_prefix("minecraft:")
                        .unwrap_or(i)
                        .to_shouty_snake_case()
                );
                quote! { &crate::item::Item::#item_ident }
            })
            .collect();
        requested_items.push(quote! { Self::#ident => &[#(#items),*] });

        let translate = &prof_data.name.translate;
        profession_names.push(quote! { Self::#ident => #translate });

        let i = i as i32;
        profession_from_i32.push(quote! { #i => Some(Self::#ident) });

        let mut level_matches = Vec::new();
        for (level_str, set_key) in &prof_data.trade_sets {
            let level = level_str.parse::<i32>().unwrap();
            let set_key_clean = set_key.strip_prefix("minecraft:").unwrap_or(set_key);
            if let Some(trades_ident) = data
                .trade_sets
                .get(set_key_clean)
                .and_then(|set| generated_trade_sets.get(&set.trades))
            {
                let set = data.trade_sets.get(set_key_clean).unwrap();
                let amount = set.amount as i32;
                level_matches.push(quote! { #level => Some(VillagerTradeSet { trades: #trades_ident, amount: #amount }) });
            }
        }
        let profession_trade_set = if level_matches.is_empty() {
            quote! { Self::#ident => None }
        } else {
            quote! {
                Self::#ident => match level {
                    #(#level_matches,)*
                    _ => None,
                }
            }
        };
        profession_trade_sets.push(profession_trade_set);
    }

    for (i, name) in data.types.keys().enumerate() {
        let ident = format_ident!("{}", name.to_pascal_case());
        type_variants.push(quote! { #ident });

        let i = i as i32;
        type_from_i32.push(quote! { #i => Some(Self::#ident) });
    }

    quote! {
        use serde::Serialize;

        #[derive(Clone, Copy, PartialEq, Eq)]
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

        #(#trade_set_data)*

        // `WanderingTrader::updateTrades` (`WanderingTrader.java:129-135`): pulls buying, then
        // uncommon, then common trade sets via `AbstractVillager.addOffersFromTradeSet`. Unlike
        // profession trade sets these aren't keyed by a `VillagerProfession`/level pair, so they
        // are exposed as top-level constants instead of through `trade_set(level)`.
        #(#wandering_trader_trade_sets)*

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
        #[repr(i32)]
        pub enum VillagerProfession {
            #(#profession_variants),*
        }

        impl VillagerProfession {
            #[must_use]
            pub const fn from_i32(id: i32) -> Option<Self> {
                match id {
                    #(#profession_from_i32,)*
                    _ => None,
                }
            }

            #[must_use]
            #[allow(clippy::match_same_arms)]
            pub const fn work_sound(&self) -> Option<crate::sound::Sound> {
                match self {
                    #(#work_sounds),*
                }
            }

            #[must_use]
            #[allow(clippy::match_same_arms)]
            pub const fn requested_items(&self) -> &'static [&'static crate::item::Item] {
                match self {
                    #(#requested_items),*
                }
            }

            #[must_use]
            pub const fn translation_key(&self) -> &'static str {
                match self {
                    #(#profession_names),*
                }
            }

            #[must_use]
            #[allow(clippy::too_many_lines, clippy::match_same_arms)]
            pub const fn trade_set(&self, level: i32) -> Option<VillagerTradeSet> {
                match self {
                    #(#profession_trade_sets,)*
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
            #(#type_variants),*
        }

        impl VillagerType {
            #[must_use]
            pub const fn from_i32(id: i32) -> Option<Self> {
                match id {
                    #(#type_from_i32,)*
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
    }
}
