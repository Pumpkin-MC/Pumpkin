use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use serde::Deserialize;
use syn::LitStr;

#[derive(Deserialize, Clone, Debug)]
pub struct LootTable<'a> {
    pub r#type: LootTableType,
    pub random_sequence: Option<Cow<'a, str>>,
    pub pools: Option<Cow<'a, [LootPool<'a>]>>,
}

impl ToTokens for LootTable<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let loot_table_type = self.r#type.to_token_stream();
        let random_sequence = match &self.random_sequence {
            Some(seq) => {
                let lit = LitStr::new(seq, Span::call_site());
                quote! { Some(Cow::Borrowed(#lit)) }
            }
            None => quote! { None },
        };
        let pools = match &self.pools {
            Some(pools) => {
                let pool_tokens: Vec<_> = pools.iter().map(|pool| pool.to_token_stream()).collect();
                quote! { Some(Cow::Borrowed(&[#(#pool_tokens),*])) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            LootTable {
                r#type: #loot_table_type,
                random_sequence: #random_sequence,
                pools: #pools,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootPool<'a> {
    pub entries: Cow<'a, [LootPoolEntry<'a>]>,
    pub rolls: f32,
    pub bonus_rolls: f32,
}

impl ToTokens for LootPool<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let entries_tokens: Vec<_> = self
            .entries
            .iter()
            .map(|entry| entry.to_token_stream())
            .collect();
        let rolls = &self.rolls;
        let bonus_rolls = &self.bonus_rolls;

        tokens.extend(quote! {
            LootPool {
                entries: Cow::Borrowed(&[#(#entries_tokens),*]),
                rolls: #rolls,
                bonus_rolls: #bonus_rolls,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemEntry<'a> {
    pub name: Cow<'a, str>,
}

impl ToTokens for ItemEntry<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = LitStr::new(&self.name, Span::call_site());
        tokens.extend(quote! {
            ItemEntry {
                name: Cow::Borrowed(#name),
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct AlternativeEntry<'a> {
    pub children: Cow<'a, [LootPoolEntry<'a>]>,
}

impl ToTokens for AlternativeEntry<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let children_tokens: Vec<_> = self
            .children
            .iter()
            .map(|entry| entry.to_token_stream())
            .collect();

        tokens.extend(quote! {
            AlternativeEntry {
                children: Cow::Borrowed(&[#(#children_tokens),*]),
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum LootPoolEntryTypes<'a> {
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:item")]
    Item(ItemEntry<'a>),
    #[serde(rename = "minecraft:loot_table")]
    LootTable,
    #[serde(rename = "minecraft:dynamic")]
    Dynamic,
    #[serde(rename = "minecraft:tag")]
    Tag,
    #[serde(rename = "minecraft:alternatives")]
    Alternatives(AlternativeEntry<'a>),
    #[serde(rename = "minecraft:sequence")]
    Sequence,
    #[serde(rename = "minecraft:group")]
    Group,
}

impl ToTokens for LootPoolEntryTypes<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            LootPoolEntryTypes::Empty => {
                tokens.extend(quote! { LootPoolEntryTypes::Empty });
            }
            LootPoolEntryTypes::Item(item) => {
                tokens.extend(quote! { LootPoolEntryTypes::Item(#item) });
            }
            LootPoolEntryTypes::LootTable => {
                tokens.extend(quote! { LootPoolEntryTypes::LootTable });
            }
            LootPoolEntryTypes::Dynamic => {
                tokens.extend(quote! { LootPoolEntryTypes::Dynamic });
            }
            LootPoolEntryTypes::Tag => {
                tokens.extend(quote! { LootPoolEntryTypes::Tag });
            }
            LootPoolEntryTypes::Alternatives(alt) => {
                tokens.extend(quote! { LootPoolEntryTypes::Alternatives(#alt) });
            }
            LootPoolEntryTypes::Sequence => {
                tokens.extend(quote! { LootPoolEntryTypes::Sequence });
            }
            LootPoolEntryTypes::Group => {
                tokens.extend(quote! { LootPoolEntryTypes::Group });
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "condition")]
pub enum LootCondition {
    #[serde(rename = "minecraft:inverted")]
    Inverted,
    #[serde(rename = "minecraft:any_of")]
    AnyOf,
    #[serde(rename = "minecraft:all_of")]
    AllOf,
    #[serde(rename = "minecraft:random_chance")]
    RandomChance,
    #[serde(rename = "minecraft:random_chance_with_enchanted_bonus")]
    RandomChanceWithEnchantedBonus,
    #[serde(rename = "minecraft:entity_properties")]
    EntityProperties,
    #[serde(rename = "minecraft:killed_by_player")]
    KilledByPlayer,
    #[serde(rename = "minecraft:entity_scores")]
    EntityScores,
    #[serde(rename = "minecraft:block_state_property")]
    BlockStateProperty,
    #[serde(rename = "minecraft:match_tool")]
    MatchTool,
    #[serde(rename = "minecraft:table_bonus")]
    TableBonus,
    #[serde(rename = "minecraft:survives_explosion")]
    SurvivesExplosion,
    #[serde(rename = "minecraft:damage_source_properties")]
    DamageSourceProperties,
    #[serde(rename = "minecraft:location_check")]
    LocationCheck,
    #[serde(rename = "minecraft:weather_check")]
    WeatherCheck,
    #[serde(rename = "minecraft:reference")]
    Reference,
    #[serde(rename = "minecraft:time_check")]
    TimeCheck,
    #[serde(rename = "minecraft:value_check")]
    ValueCheck,
    #[serde(rename = "minecraft:enchantment_active_check")]
    EnchantmentActiveCheck,
}

impl ToTokens for LootCondition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootCondition::Inverted => quote! { LootCondition::Inverted },
            LootCondition::AnyOf => quote! { LootCondition::AnyOf },
            LootCondition::AllOf => quote! { LootCondition::AllOf },
            LootCondition::RandomChance => quote! { LootCondition::RandomChance },
            LootCondition::RandomChanceWithEnchantedBonus => {
                quote! { LootCondition::RandomChanceWithEnchantedBonus }
            }
            LootCondition::EntityProperties => quote! { LootCondition::EntityProperties },
            LootCondition::KilledByPlayer => quote! { LootCondition::KilledByPlayer },
            LootCondition::EntityScores => quote! { LootCondition::EntityScores },
            LootCondition::BlockStateProperty => quote! { LootCondition::BlockStateProperty },
            LootCondition::MatchTool => quote! { LootCondition::MatchTool },
            LootCondition::TableBonus => quote! { LootCondition::TableBonus },
            LootCondition::SurvivesExplosion => quote! { LootCondition::SurvivesExplosion },
            LootCondition::DamageSourceProperties => {
                quote! { LootCondition::DamageSourceProperties }
            }
            LootCondition::LocationCheck => quote! { LootCondition::LocationCheck },
            LootCondition::WeatherCheck => quote! { LootCondition::WeatherCheck },
            LootCondition::Reference => quote! { LootCondition::Reference },
            LootCondition::TimeCheck => quote! { LootCondition::TimeCheck },
            LootCondition::ValueCheck => quote! { LootCondition::ValueCheck },
            LootCondition::EnchantmentActiveCheck => {
                quote! { LootCondition::EnchantmentActiveCheck }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootPoolEntry<'a> {
    #[serde(flatten)]
    pub content: LootPoolEntryTypes<'a>,
    pub conditions: Option<Cow<'a, [LootCondition]>>,
}

impl ToTokens for LootPoolEntry<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let content = &self.content;
        let conditions_tokens = match &self.conditions {
            Some(conds) => {
                let cond_tokens: Vec<_> = conds.iter().map(|c| c.to_token_stream()).collect();
                quote! { Some(Cow::Borrowed(&[#(#cond_tokens),*])) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            LootPoolEntry {
                content: #content,
                conditions: #conditions_tokens,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename = "snake_case")]
pub enum LootTableType {
    #[serde(rename = "minecraft:empty")]
    /// Nothing will be dropped
    Empty,
    #[serde(rename = "minecraft:block")]
    /// A Block will be dropped
    Block,
    #[serde(rename = "minecraft:chest")]
    /// A Item will be dropped
    Chest,
}

impl ToTokens for LootTableType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootTableType::Empty => quote! { LootTableType::Empty },
            LootTableType::Block => quote! { LootTableType::Block },
            LootTableType::Chest => quote! { LootTableType::Chest },
        };

        tokens.extend(name);
    }
}

type ItemName = String;
type ItemCount = u16;

impl LootTable<'_> {
    pub fn get_loot(&self) -> Vec<(ItemName, ItemCount)> {
        let mut items = vec![];
        if let Some(pools) = &self.pools {
            for i in 0..pools.len() {
                let pool = &pools[i];
                items.extend_from_slice(&pool.get_loot());
            }
        }
        items
    }
}

impl LootPool<'_> {
    pub fn get_loot(&self) -> Vec<(ItemName, ItemCount)> {
        let i = self.rolls.round() as i32 + self.bonus_rolls.floor() as i32; // TODO: mul by luck
        let mut items = vec![];
        for _ in 0..i {
            for entry_idx in 0..self.entries.len() {
                let entry = &self.entries[entry_idx];
                if let Some(conditions) = &entry.conditions {
                    if !conditions.iter().all(|condition| condition.test()) {
                        continue;
                    }
                }
                items.extend_from_slice(&entry.content.get_items());
            }
        }
        items
    }
}

impl ItemEntry<'_> {
    pub fn get_items(&self) -> Vec<(ItemName, ItemCount)> {
        let item = &self.name.replace("minecraft:", "");
        vec![(item.to_string(), 1)]
    }
}

impl AlternativeEntry<'_> {
    pub fn get_items(&self) -> Vec<(ItemName, ItemCount)> {
        let mut items = vec![];
        for i in 0..self.children.len() {
            let child = &self.children[i];
            if let Some(conditions) = &child.conditions {
                if !conditions.iter().all(|condition| condition.test()) {
                    continue;
                }
            }
            items.extend_from_slice(&child.content.get_items());
        }
        items
    }
}

impl LootPoolEntryTypes<'_> {
    pub fn get_items(&self) -> Vec<(ItemName, ItemCount)> {
        match self {
            LootPoolEntryTypes::Empty => todo!(),
            LootPoolEntryTypes::Item(item_entry) => item_entry.get_items(),
            LootPoolEntryTypes::LootTable => todo!(),
            LootPoolEntryTypes::Dynamic => todo!(),
            LootPoolEntryTypes::Tag => todo!(),
            LootPoolEntryTypes::Alternatives(alternative) => alternative.get_items(),
            LootPoolEntryTypes::Sequence => todo!(),
            LootPoolEntryTypes::Group => todo!(),
        }
    }
}

#[expect(clippy::match_like_matches_macro)]
impl LootCondition {
    // TODO: This is trash, Make this right
    pub fn test(&self) -> bool {
        match self {
            LootCondition::SurvivesExplosion => true,
            _ => false,
        }
    }
}
