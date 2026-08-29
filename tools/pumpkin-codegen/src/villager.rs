use heck::{ToPascalCase, ToShoutySnakeCase};
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use serde_json::Value;
use std::fs;

#[derive(Deserialize)]
struct VillagerDataJson {
    professions: IndexMap<String, ProfessionJson>,
    types: IndexMap<String, String>,
}

#[derive(Deserialize)]
struct ProfessionJson {
    name: NameJson,
    requested_items: Vec<String>,
    work_sound: Option<String>,
    #[serde(default)]
    trade_sets: IndexMap<String, String>,
}

#[derive(Deserialize)]
struct NameJson {
    translate: String,
}

#[derive(Deserialize, Clone)]
struct TradeSetJson {
    trades: String, // Tag like "#minecraft:armorer/level_1"
    #[serde(default = "default_amount")]
    amount: f32,
}

fn default_amount() -> f32 {
    1.0
}

#[derive(Deserialize, Clone)]
struct TradeJson {
    wants: TradeItemJson,
    #[serde(alias = "wants_b")]
    additional_wants: Option<TradeItemJson>,
    gives: TradeItemJson,
    max_uses: Option<f32>,
    xp: Option<f32>,
    #[serde(alias = "price_multiplier")]
    reputation_discount: Option<f32>,
    #[serde(default)]
    given_item_modifiers: Vec<Value>,
    merchant_predicate: Option<Value>,
}

#[derive(Deserialize, Clone)]
struct TradeItemJson {
    id: String,
    count: Option<f32>,
}

fn walk_json_files<F>(dir: &std::path::Path, base_dir: &std::path::Path, callback: &mut F)
where
    F: FnMut(String, String),
{
    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                walk_json_files(&path, base_dir, callback);
            } else if path.extension().is_some_and(|ext| ext == "json") {
                let rel = path.strip_prefix(base_dir).unwrap();
                let stem = rel.with_extension("");
                let key = stem.to_string_lossy().replace('\\', "/");
                let content = fs::read_to_string(&path).expect("Failed to read JSON file");
                callback(key, content);
            }
        }
    }
}

pub fn build() -> TokenStream {
    let data: VillagerDataJson =
        serde_json::from_str(&fs::read_to_string("../../assets/villager_data.json").unwrap())
            .expect("Failed to parse villager_data.json");

    // Load trade sets from datapack
    let trade_set_dir =
        std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/trade_set");
    let mut trade_sets: IndexMap<String, TradeSetJson> = IndexMap::new();
    walk_json_files(trade_set_dir, trade_set_dir, &mut |key, content| {
        if let Ok(trade_set) = serde_json::from_str::<TradeSetJson>(&content) {
            trade_sets.insert(format!("minecraft:{key}"), trade_set.clone());
            trade_sets.insert(key, trade_set);
        }
    });

    // Load trade tags from datapack
    let trade_tags_dir =
        std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/tags/villager_trade");
    let mut raw_trade_tags: IndexMap<String, Vec<String>> = IndexMap::new();
    walk_json_files(trade_tags_dir, trade_tags_dir, &mut |key, content| {
        #[derive(Deserialize)]
        struct TagFile {
            values: Vec<Value>,
        }
        if let Ok(tag_file) = serde_json::from_str::<TagFile>(&content) {
            let values = tag_file
                .values
                .into_iter()
                .filter_map(|v| match v {
                    Value::String(s) => Some(s),
                    Value::Object(obj) => obj.get("id").and_then(Value::as_str).map(String::from),
                    _ => None,
                })
                .collect();
            raw_trade_tags.insert(format!("minecraft:{key}"), values);
        }
    });

    fn resolve_trade_tag(
        tag: &str,
        raw_tags: &IndexMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Vec<String> {
        if visited.contains(tag) {
            return Vec::new();
        }
        visited.insert(tag.to_string());
        let mut result = Vec::new();
        let full_tag = if tag.starts_with("minecraft:") {
            tag.to_string()
        } else {
            format!("minecraft:{tag}")
        };
        if let Some(values) = raw_tags.get(&full_tag) {
            for v in values {
                if let Some(sub) = v.strip_prefix('#') {
                    let mut sub_visited = visited.clone();
                    for child in resolve_trade_tag(sub, raw_tags, &mut sub_visited) {
                        if !result.contains(&child) {
                            result.push(child);
                        }
                    }
                } else {
                    let full_v = if v.starts_with("minecraft:") {
                        v.clone()
                    } else {
                        format!("minecraft:{v}")
                    };
                    if !result.contains(&full_v) {
                        result.push(full_v);
                    }
                }
            }
        }
        result
    }

    // Load individual villager trades from datapack
    let trades_dir =
        std::path::Path::new("../../assets/datapacks/26_2/data/minecraft/villager_trade");
    let mut villager_trades: IndexMap<String, TradeJson> = IndexMap::new();
    walk_json_files(trades_dir, trades_dir, &mut |key, content| {
        if let Ok(trade) = serde_json::from_str::<TradeJson>(&content) {
            villager_trades.insert(format!("minecraft:{key}"), trade.clone());
            villager_trades.insert(key, trade);
        }
    });

    let mut profession_variants = Vec::new();
    let mut type_variants = Vec::new();

    let mut work_sounds = Vec::new();
    let mut requested_items = Vec::new();
    let mut profession_names = Vec::new();

    let mut profession_from_i32 = Vec::new();
    let mut type_from_i32 = Vec::new();

    let mut trade_set_data = Vec::new();
    let mut generated_trade_sets = IndexMap::new();

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

        let wants_b = if let Some(b) = &trade.additional_wants {
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
        let price_multiplier = trade.reputation_discount.unwrap_or(0.05);

        let modifier = trade
            .given_item_modifiers
            .iter()
            .find_map(|modifier| {
                let function = modifier.get("function")?.as_str()?;
                Some(match function {
                    "minecraft:enchant_randomly" => quote! { VillagerTradeModifier::EnchantRandomly },
                    "minecraft:enchant_with_levels" => {
                        let levels = modifier.get("levels")?;
                        let min = levels.get("min")?.as_f64()? as i32;
                        let max = levels.get("max")?.as_f64()? as i32;
                        quote! { VillagerTradeModifier::EnchantWithLevels { min: #min, max: #max } }
                    }
                    "minecraft:exploration_map" => {
                        let destination = modifier.get("destination")?.as_str()?;
                        quote! { VillagerTradeModifier::ExplorationMap { destination: #destination } }
                    }
                    "minecraft:set_random_dyes" => quote! { VillagerTradeModifier::RandomDyes },
                    "minecraft:set_random_potion" => quote! { VillagerTradeModifier::RandomPotion },
                    "minecraft:set_stew_effect" => quote! { VillagerTradeModifier::SuspiciousStew },
                    "minecraft:set_potion" => {
                        let potion = modifier.get("id")?.as_str()?;
                        quote! { VillagerTradeModifier::Potion(#potion) }
                    }
                    _ => return None,
                })
            })
            .unwrap_or_else(|| quote! { VillagerTradeModifier::None });

        let allowed_types = trade
            .merchant_predicate
            .as_ref()
            .and_then(|predicate| {
                predicate.pointer("/predicate/minecraft:predicates/minecraft:villager~1variant")
            })
            .map(|variants| {
                let variants: Vec<_> = variants
                    .as_array()
                    .map_or_else(|| vec![variants], |variants| variants.iter().collect())
                    .into_iter()
                    .filter_map(Value::as_str)
                    .map(|variant| {
                        let ident = format_ident!(
                            "{}",
                            variant
                                .strip_prefix("minecraft:")
                                .unwrap_or(variant)
                                .to_pascal_case()
                        );
                        quote! { VillagerType::#ident }
                    })
                    .collect();
                quote! { &[#(#variants),*] }
            })
            .unwrap_or_else(|| quote! { &[] });

        quote! {
            VillagerTrade {
                wants: #wants,
                wants_b: #wants_b,
                gives: #gives,
                max_uses: #max_uses,
                xp: #xp,
                price_multiplier: #price_multiplier,
                modifier: #modifier,
                allowed_types: #allowed_types,
            }
        }
    };

    // Pre-process all trade sets mentioned in trade_sets map
    for (_set_key, set_data) in &trade_sets {
        let tag = &set_data.trades;
        if generated_trade_sets.contains_key(tag) {
            continue;
        }
        let tag_content = tag.strip_prefix('#').unwrap_or(tag);
        let tag_clean = tag_content
            .strip_prefix("minecraft:")
            .unwrap_or(tag_content);

        let mut visited = std::collections::HashSet::new();
        let trade_ids = resolve_trade_tag(tag_clean, &raw_trade_tags, &mut visited);

        let mut matching_trades = Vec::new();
        for trade_id in trade_ids {
            if let Some(trade) = villager_trades.get(&trade_id) {
                matching_trades.push(format_trade(trade));
            }
        }

        if !matching_trades.is_empty() {
            let ident_name = tag_clean.replace('/', "_").to_shouty_snake_case();
            let ident = format_ident!("TRADES_{}", ident_name);
            trade_set_data.push(quote! {
                pub const #ident: &[VillagerTrade] = &[
                    #(#matching_trades),*
                ];
            });
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
            if let Some(trades_ident) = trade_sets
                .get(set_key_clean)
                .and_then(|set| generated_trade_sets.get(&set.trades))
            {
                let set = trade_sets.get(set_key_clean).unwrap();
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
            pub modifier: VillagerTradeModifier,
            pub allowed_types: &'static [VillagerType],
        }

        #[derive(Clone, Copy, PartialEq, Eq)]
        pub enum VillagerTradeModifier {
            None,
            EnchantRandomly,
            EnchantWithLevels { min: i32, max: i32 },
            ExplorationMap { destination: &'static str },
            RandomDyes,
            RandomPotion,
            SuspiciousStew,
            Potion(&'static str),
        }

        #[derive(Clone, Copy, PartialEq)]
        pub struct VillagerTradeSet {
            pub trades: &'static [VillagerTrade],
            pub amount: i32,
        }

        #(#trade_set_data)*

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
