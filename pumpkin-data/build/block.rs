use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use pumpkin_util::math::{experience::Experience, vector3::Vector3};
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use syn::{Ident, LitInt, LitStr};

fn const_block_name_from_block_name(block: &str) -> String {
    block.to_shouty_snake_case()
}

fn property_group_name_from_derived_name(name: &str) -> String {
    format!("{name}_properties").to_upper_camel_case()
}

enum PropertyType {
    Bool,
    Enum { name: String },
}

struct PropertyVariantMapping {
    original_name: String,
    property_type: PropertyType,
}

struct PropertyCollectionData {
    variant_mappings: Vec<PropertyVariantMapping>,
    blocks: Vec<(String, u16)>,
}

impl PropertyCollectionData {
    pub fn add_block(&mut self, block_name: String, block_id: u16) {
        self.blocks.push((block_name, block_id));
    }

    pub fn from_mappings(variant_mappings: Vec<PropertyVariantMapping>) -> Self {
        Self {
            variant_mappings,
            blocks: Vec::new(),
        }
    }

    pub fn derive_name(&self) -> String {
        format!("{}_like", self.blocks[0].0)
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct PropertyStruct {
    pub name: String,
    pub values: Vec<String>,
}

impl ToTokens for PropertyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.values == vec!["true".to_string(), "false".to_string()] {
            // For boolean properties, we'll use Rust's built-in bool type
            return;
        }

        let name = Ident::new(&self.name, Span::call_site());

        let variant_count = self.values.clone().len() as u16;
        let values_index = (0..self.values.clone().len() as u16).collect::<Vec<_>>();

        let ident_values = self
            .values
            .iter()
            .map(|value| Ident::new(&(value).to_upper_camel_case(), Span::call_site()));

        let values_2 = ident_values.clone();
        let values_3 = ident_values.clone();

        let is_number_values =
            self.values.iter().all(|v| v.starts_with("L")) && self.values.iter().any(|v| v == "L1");

        let from_values = self.values.iter().map(|value| {
            let ident = Ident::new(&(value).to_upper_camel_case(), Span::call_site());
            let value = if is_number_values {
                value.strip_prefix("L").unwrap()
            } else {
                value
            };
            quote! {
                #value => Self::#ident
            }
        });
        let to_values = self.values.iter().map(|value| {
            let ident = Ident::new(&(value).to_upper_camel_case(), Span::call_site());
            let value = if is_number_values {
                value.strip_prefix("L").unwrap()
            } else {
                value
            };
            quote! {
                Self::#ident => #value
            }
        });

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub enum #name {
                #(#ident_values),*
            }

            impl EnumVariants for #name {
                fn variant_count() -> u16 {
                    #variant_count
                }

                fn to_index(&self) -> u16 {
                    match self {
                        #(Self::#values_2 => #values_index),*
                    }
                }

                fn from_index(index: u16) -> Self {
                    match index {
                        #(#values_index => Self::#values_3,)*
                        _ => panic!("Invalid index: {}", index),
                    }
                }

                fn to_value(&self) -> &str {
                    match self {
                        #(#to_values),*
                    }
                }

                fn from_value(value: &str) -> Self {
                    match value {
                        #(#from_values),*,
                        _ => panic!("Invalid value: {:?}", value),
                    }
                }

            }
        });
    }
}

struct BlockPropertyStruct {
    data: PropertyCollectionData,
}

impl ToTokens for BlockPropertyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let struct_name = property_group_name_from_derived_name(&self.data.derive_name());
        let name = Ident::new(&struct_name, Span::call_site());

        // Strukturfelder generieren
        let fields = self.data.variant_mappings.iter().map(|entry| {
            let key = Ident::new_raw(&entry.original_name, Span::call_site());
            match &entry.property_type {
                PropertyType::Bool => quote! { pub #key: bool },
                PropertyType::Enum { name } => {
                    let value = Ident::new(name, Span::call_site());
                    quote! { pub #key: #value }
                }
            }
        });

        let block_ids = self
            .data
            .blocks
            .iter()
            .map(|(_, id)| *id)
            .collect::<Vec<_>>();

        let to_index_body = self
            .data
            .variant_mappings
            .iter()
            .rev()
            .map(|entry| {
                let field_name = Ident::new_raw(&entry.original_name, Span::call_site());
                match &entry.property_type {
                    PropertyType::Bool => quote! {
                        index += !self.#field_name as u16 * multiplier;
                        multiplier *= 2;
                    },
                    PropertyType::Enum { name } => {
                        let enum_ident = Ident::new(name, Span::call_site());
                        quote! {
                            index += self.#field_name.to_index() * multiplier;
                            multiplier *= #enum_ident::variant_count();
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        let from_index_body = self
            .data
            .variant_mappings
            .iter()
            .rev()
            .map(|entry| {
                let field_name = Ident::new_raw(&entry.original_name, Span::call_site());
                match &entry.property_type {
                    PropertyType::Bool => quote! {
                        #field_name: {
                            let value = index % 2;
                            index /= 2;
                            value == 0
                        }
                    },
                    PropertyType::Enum { name } => {
                        let enum_ident = Ident::new(name, Span::call_site());
                        quote! {
                            #field_name: {
                                let value = index % #enum_ident::variant_count();
                                index /= #enum_ident::variant_count();
                                #enum_ident::from_index(value)
                            }
                        }
                    }
                }
            })
            .collect::<Vec<_>>();

        let to_props_values = self.data.variant_mappings.iter().map(|entry| {
            let key = &entry.original_name;
            let field_name = Ident::new_raw(&entry.original_name, Span::call_site());
            match &entry.property_type {
                PropertyType::Bool => quote! {
                    props.push((#key.to_string(), self.#field_name.to_string()));
                },
                PropertyType::Enum { name: _ } => quote! {
                    props.push((#key.to_string(), self.#field_name.to_value().to_string()));
                },
            }
        });

        let from_props_values = self.data.variant_mappings.iter().map(|entry| {
            let key = &entry.original_name;
            let field_name = Ident::new_raw(&entry.original_name, Span::call_site());
            match &entry.property_type {
                PropertyType::Bool => quote! {
                    #key => {
                        block_props.#field_name = matches!(value, "true")
                    }
                },
                PropertyType::Enum { name } => {
                    let enum_ident = Ident::new(name, Span::call_site());
                    quote! {
                        #key => {
                            block_props.#field_name = #enum_ident::from_value(value)
                        }
                    }
                }
            }
        });

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct #name {
                #(#fields),*
            }

            impl BlockProperties for #name {
                #[allow(unused_assignments)]
                fn to_index(&self) -> u16 {
                    let mut index = 0;
                    let mut multiplier = 1;
                    #(#to_index_body)*
                    index
                }

                #[allow(unused_assignments)]
                fn from_index(mut index: u16) -> Self {
                    Self {
                        #(#from_index_body),*
                    }
                }

                #[inline]
                fn handles_block_id(block_id: u16) -> bool where Self: Sized {
                    [#(#block_ids),*].contains(&block_id)
                }

                fn to_state_id(&self, block: &Block) -> u16 {
                    if !Self::handles_block_id(block.id) {
                        panic!("{} is not a valid block for {}", &block.name, #struct_name);
                    }
                    block.states[0].id + self.to_index()
                }

                fn from_state_id(state_id: u16, block: &Block) -> Self {
                    if !Self::handles_block_id(block.id) {
                        panic!("{} is not a valid block for {}", &block.name, #struct_name);
                    }
                    if state_id >= block.states[0].id && state_id <= block.states.last().unwrap().id {
                        let index = state_id - block.states[0].id;
                        Self::from_index(index)
                    } else {
                        panic!("State ID {} does not exist for {}", state_id, &block.name);
                    }
                }

                fn default(block: &Block) -> Self {
                    if !Self::handles_block_id(block.id) {
                        panic!("{} is not a valid block for {}", &block.name, #struct_name);
                    }
                    Self::from_state_id(block.default_state_id, block)
                }

                #[allow(clippy::vec_init_then_push)]
                fn to_props(&self) -> Vec<(String, String)> {
                    let mut props = vec![];
                    #(#to_props_values)*
                    props
                }
                fn from_props(props: Vec<(&str, &str)>, block: &Block) -> Self {
                    if ![#(#block_ids),*].contains(&block.id) {
                        panic!("{} is not a valid block for {}", &block.name, #struct_name);
                    }
                    let mut block_props = Self::default(block);
                    for (key, value) in props {
                        match key {
                            #(#from_props_values),*,
                            _ => panic!("Invalid key: {}", key),
                        }
                    }
                    block_props
                }
            }
        });
    }
}

#[derive(Deserialize, Clone, Copy, Debug)]
pub struct CollisionShape {
    pub min: Vector3<f64>,
    pub max: Vector3<f64>,
}

impl ToTokens for CollisionShape {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_x = &self.min.x;
        let min_y = &self.min.y;
        let min_z = &self.min.z;

        let max_x = &self.max.x;
        let max_y = &self.max.y;
        let max_z = &self.max.z;

        tokens.extend(quote! {
            CollisionShape {
                min: Vector3::new(#min_x, #min_y, #min_z),
                max: Vector3::new(#max_x, #max_y, #max_z),
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockState {
    pub id: u16,
    pub state_flags: u8,
    pub side_flags: u8,
    pub instrument: String, // TODO: make this an enum
    pub luminance: u8,
    pub piston_behavior: PistonBehavior,
    pub hardness: f32,
    pub collision_shapes: Vec<u16>,
    pub opacity: Option<u8>,
    pub block_entity_type: Option<u16>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PistonBehavior {
    Normal,
    Destroy,
    Block,
    Ignore,
    PushOnly,
}

impl PistonBehavior {
    fn to_tokens(&self) -> TokenStream {
        match self {
            PistonBehavior::Normal => quote! { PistonBehavior::Normal },
            PistonBehavior::Destroy => quote! { PistonBehavior::Destroy },
            PistonBehavior::Block => quote! { PistonBehavior::Block },
            PistonBehavior::Ignore => quote! { PistonBehavior::Ignore },
            PistonBehavior::PushOnly => quote! { PistonBehavior::PushOnly },
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockStateRef {
    pub id: u16,
    pub state_idx: u16,
}

impl BlockState {
    fn to_tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let state_flags = LitInt::new(&self.state_flags.to_string(), Span::call_site());
        let side_flags = LitInt::new(&self.side_flags.to_string(), Span::call_site());
        let instrument = self.instrument.clone();
        let luminance = LitInt::new(&self.luminance.to_string(), Span::call_site());
        let hardness = self.hardness;
        let opacity = match self.opacity {
            Some(opacity) => {
                let opacity = LitInt::new(&opacity.to_string(), Span::call_site());
                quote! { #opacity }
            }
            None => quote! { u8::MAX },
        };
        let block_entity_type = match self.block_entity_type {
            Some(block_entity_type) => {
                let block_entity_type =
                    LitInt::new(&block_entity_type.to_string(), Span::call_site());
                quote! { #block_entity_type }
            }
            None => quote! { u16::MAX },
        };

        let collision_shapes = self
            .collision_shapes
            .iter()
            .map(|shape_id| LitInt::new(&shape_id.to_string(), Span::call_site()));
        let piston_behavior = &self.piston_behavior.to_tokens();

        tokens.extend(quote! {
            BlockState {
                id: #id,
                state_flags: #state_flags,
                side_flags: #side_flags,
                instrument: #instrument,
                luminance: #luminance,
                piston_behavior: #piston_behavior,
                hardness: #hardness,
                collision_shapes: &[#(#collision_shapes),*],
                opacity: #opacity,
                block_entity_type: #block_entity_type,
            }
        });
        tokens
    }
}

impl ToTokens for BlockStateRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let state_idx = LitInt::new(&self.state_idx.to_string(), Span::call_site());

        tokens.extend(quote! {
            BlockStateRef {
                id: #id,
                state_idx: #state_idx,
            }
        });
    }
}

/// These are required to be defined twice because serde can't deseralize into static context for obvious reasons.
#[derive(Deserialize, Clone, Debug)]
pub struct LootTableStruct {
    r#type: LootTableTypeStruct,
    random_sequence: Option<String>,
    pools: Option<Vec<LootPoolStruct>>,
}

impl ToTokens for LootTableStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let loot_table_type = self.r#type.to_token_stream();
        let random_sequence = match &self.random_sequence {
            Some(seq) => quote! { Some(#seq) },
            None => quote! { None },
        };
        let pools = match &self.pools {
            Some(pools) => {
                let pool_tokens: Vec<_> = pools.iter().map(|pool| pool.to_token_stream()).collect();
                quote! { Some(&[#(#pool_tokens),*]) }
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
pub struct LootPoolStruct {
    entries: Vec<LootPoolEntryStruct>,
    rolls: f32, // TODO
    bonus_rolls: f32,
}

impl ToTokens for LootPoolStruct {
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
                entries: &[#(#entries_tokens),*],
                rolls: #rolls,
                bonus_rolls: #bonus_rolls,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct ItemEntryStruct {
    name: String,
}

impl ToTokens for ItemEntryStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = LitStr::new(&self.name, Span::call_site());

        tokens.extend(quote! {
            ItemEntry {
                name: #name,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct AlternativeEntryStruct {
    children: Vec<LootPoolEntryStruct>,
}

impl ToTokens for AlternativeEntryStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let children = self.children.iter().map(|entry| entry.to_token_stream());

        tokens.extend(quote! {
            AlternativeEntry {
                children: &[#(#children),*],
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum LootPoolEntryTypesStruct {
    #[serde(rename = "minecraft:empty")]
    Empty,
    #[serde(rename = "minecraft:item")]
    Item(ItemEntryStruct),
    #[serde(rename = "minecraft:loot_table")]
    LootTable,
    #[serde(rename = "minecraft:dynamic")]
    Dynamic,
    #[serde(rename = "minecraft:tag")]
    Tag,
    #[serde(rename = "minecraft:alternatives")]
    Alternatives(AlternativeEntryStruct),
    #[serde(rename = "minecraft:sequence")]
    Sequence,
    #[serde(rename = "minecraft:group")]
    Group,
}

impl ToTokens for LootPoolEntryTypesStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            LootPoolEntryTypesStruct::Empty => {
                tokens.extend(quote! { LootPoolEntryTypes::Empty });
            }
            LootPoolEntryTypesStruct::Item(item) => {
                tokens.extend(quote! { LootPoolEntryTypes::Item(#item) });
            }
            LootPoolEntryTypesStruct::LootTable => {
                tokens.extend(quote! { LootPoolEntryTypes::LootTable });
            }
            LootPoolEntryTypesStruct::Dynamic => {
                tokens.extend(quote! { LootPoolEntryTypes::Dynamic });
            }
            LootPoolEntryTypesStruct::Tag => {
                tokens.extend(quote! { LootPoolEntryTypes::Tag });
            }
            LootPoolEntryTypesStruct::Alternatives(alt) => {
                tokens.extend(quote! { LootPoolEntryTypes::Alternatives(#alt) });
            }
            LootPoolEntryTypesStruct::Sequence => {
                tokens.extend(quote! { LootPoolEntryTypes::Sequence });
            }
            LootPoolEntryTypesStruct::Group => {
                tokens.extend(quote! { LootPoolEntryTypes::Group });
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "condition")]
pub enum LootConditionStruct {
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
    BlockStateProperty {
        block: String,
        properties: HashMap<String, String>,
    },
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

impl ToTokens for LootConditionStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootConditionStruct::Inverted => quote! { LootCondition::Inverted },
            LootConditionStruct::AnyOf => quote! { LootCondition::AnyOf },
            LootConditionStruct::AllOf => quote! { LootCondition::AllOf },
            LootConditionStruct::RandomChance => quote! { LootCondition::RandomChance },
            LootConditionStruct::RandomChanceWithEnchantedBonus => {
                quote! { LootCondition::RandomChanceWithEnchantedBonus }
            }
            LootConditionStruct::EntityProperties => quote! { LootCondition::EntityProperties },
            LootConditionStruct::KilledByPlayer => quote! { LootCondition::KilledByPlayer },
            LootConditionStruct::EntityScores => quote! { LootCondition::EntityScores },
            LootConditionStruct::BlockStateProperty { block, properties } => {
                let properties: Vec<_> = properties
                    .iter()
                    .map(|(k, v)| quote! { (#k, #v) })
                    .collect();
                quote! { LootCondition::BlockStateProperty { block: #block, properties: &[#(#properties),*] } }
            }
            LootConditionStruct::MatchTool => quote! { LootCondition::MatchTool },
            LootConditionStruct::TableBonus => quote! { LootCondition::TableBonus },
            LootConditionStruct::SurvivesExplosion => quote! { LootCondition::SurvivesExplosion },
            LootConditionStruct::DamageSourceProperties => {
                quote! { LootCondition::DamageSourceProperties }
            }
            LootConditionStruct::LocationCheck => quote! { LootCondition::LocationCheck },
            LootConditionStruct::WeatherCheck => quote! { LootCondition::WeatherCheck },
            LootConditionStruct::Reference => quote! { LootCondition::Reference },
            LootConditionStruct::TimeCheck => quote! { LootCondition::TimeCheck },
            LootConditionStruct::ValueCheck => quote! { LootCondition::ValueCheck },
            LootConditionStruct::EnchantmentActiveCheck => {
                quote! { LootCondition::EnchantmentActiveCheck }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootFunctionStruct {
    #[serde(flatten)]
    content: LootFunctionTypesStruct,
    conditions: Option<Vec<LootConditionStruct>>,
}

impl ToTokens for LootFunctionStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let functions_tokens = &self.content.to_token_stream();

        let conditions_tokens = match &self.conditions {
            Some(conds) => {
                let cond_tokens: Vec<_> = conds.iter().map(|c| c.to_token_stream()).collect();
                quote! { Some(&[#(#cond_tokens),*]) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            LootFunction {
                content: #functions_tokens,
                conditions: #conditions_tokens,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "function")]
pub enum LootFunctionTypesStruct {
    #[serde(rename = "minecraft:set_count")]
    SetCount {
        count: LootFunctionNumberProviderStruct,
        add: Option<bool>,
    },
    #[serde(rename = "minecraft:limit_count")]
    LimitCount { limit: LootFunctionLimitCountStruct },
    #[serde(rename = "minecraft:apply_bonus")]
    ApplyBonus {
        enchantment: String,
        formula: String,
        parameters: Option<LootFunctionBonusParameterStruct>,
    },
    #[serde(rename = "minecraft:copy_components")]
    CopyComponents {
        source: String,
        include: Vec<String>,
    },
    #[serde(rename = "minecraft:copy_state")]
    CopyState {
        block: String,
        properties: Vec<String>,
    },
    #[serde(rename = "minecraft:explosion_decay")]
    ExplosionDecay,
}

impl ToTokens for LootFunctionTypesStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootFunctionTypesStruct::SetCount { count, add } => {
                let count = count.to_token_stream();
                let add = add.unwrap_or(false);
                quote! { LootFunctionTypes::SetCount { count: #count, add: #add } }
            }
            LootFunctionTypesStruct::LimitCount { limit } => {
                let min = match limit.min {
                    Some(min) => quote! { Some(#min) },
                    None => quote! { None },
                };
                let max = match limit.max {
                    Some(max) => quote! { Some(#max) },
                    None => quote! { None },
                };
                quote! { LootFunctionTypes::LimitCount { min: #min, max: #max } }
            }
            LootFunctionTypesStruct::ApplyBonus {
                enchantment,
                formula,
                parameters,
            } => {
                let parameters = match parameters {
                    Some(params) => {
                        let params = params.to_token_stream();
                        quote! { Some(#params) }
                    }
                    None => quote! { None },
                };

                quote! {
                    LootFunctionTypes::ApplyBonus {
                        enchantment: #enchantment,
                        formula: #formula,
                        parameters: #parameters,
                    }
                }
            }
            LootFunctionTypesStruct::CopyComponents { source, include } => {
                quote! {
                    LootFunctionTypes::CopyComponents {
                        source: #source,
                        include: &[#(#include),*],
                    }
                }
            }
            LootFunctionTypesStruct::CopyState { block, properties } => {
                quote! {
                    LootFunctionTypes::CopyState {
                        block: #block,
                        properties: &[#(#properties),*],
                    }
                }
            }
            LootFunctionTypesStruct::ExplosionDecay => {
                quote! { LootFunctionTypes::ExplosionDecay }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum LootFunctionNumberProviderStruct {
    #[serde(rename = "minecraft:uniform")]
    Uniform { min: f32, max: f32 },
    #[serde(rename = "minecraft:binomial")]
    Binomial { n: f32, p: f32 },
    #[serde(rename = "minecraft:constant", untagged)]
    Constant(f32),
}

impl ToTokens for LootFunctionNumberProviderStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::Constant(value) => {
                quote! { LootFunctionNumberProvider::Constant { value: #value } }
            }
            Self::Uniform { min, max } => {
                quote! { LootFunctionNumberProvider::Uniform { min: #min, max: #max } }
            }
            Self::Binomial { n, p } => {
                quote! { LootFunctionNumberProvider::Binomial { n: #n, p: #p } }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootFunctionLimitCountStruct {
    min: Option<f32>,
    max: Option<f32>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum LootFunctionBonusParameterStruct {
    Multiplier {
        #[serde(rename = "bonusMultiplier")]
        bonus_multiplier: i32,
    },
    Probability {
        extra: i32,
        probability: f32,
    },
}

impl ToTokens for LootFunctionBonusParameterStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::Multiplier { bonus_multiplier } => {
                quote! { LootFunctionBonusParameter::Multiplier { bonus_multiplier: #bonus_multiplier } }
            }
            Self::Probability { extra, probability } => {
                quote! { LootFunctionBonusParameter::Probability { extra: #extra, probability: #probability } }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct LootPoolEntryStruct {
    #[serde(flatten)]
    content: LootPoolEntryTypesStruct,
    conditions: Option<Vec<LootConditionStruct>>,
    functions: Option<Vec<LootFunctionStruct>>,
}

impl ToTokens for LootPoolEntryStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let content = &self.content;
        let conditions_tokens = match &self.conditions {
            Some(conds) => {
                let cond_tokens: Vec<_> = conds.iter().map(|c| c.to_token_stream()).collect();
                quote! { Some(&[#(#cond_tokens),*]) }
            }
            None => quote! { None },
        };
        let functions_tokens = match &self.functions {
            Some(fns) => {
                let cond_tokens: Vec<_> = fns.iter().map(|c| c.to_token_stream()).collect();
                quote! { Some(&[#(#cond_tokens),*]) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            LootPoolEntry {
                content: #content,
                conditions: #conditions_tokens,
                functions: #functions_tokens,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename = "snake_case")]
pub enum LootTableTypeStruct {
    #[serde(rename = "minecraft:empty")]
    /// Nothing will be dropped.
    Empty,
    #[serde(rename = "minecraft:block")]
    /// A block will be dropped.
    Block,
    #[serde(rename = "minecraft:chest")]
    /// An item will be dropped.
    Chest,
}

impl ToTokens for LootTableTypeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            LootTableTypeStruct::Empty => quote! { LootTableType::Empty },
            LootTableTypeStruct::Block => quote! { LootTableType::Block },
            LootTableTypeStruct::Chest => quote! { LootTableType::Chest },
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct Block {
    pub id: u16,
    pub name: String,
    pub translation_key: String,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub item_id: u16,
    pub loot_table: Option<LootTableStruct>,
    pub slipperiness: f32,
    pub velocity_multiplier: f32,
    pub jump_velocity_multiplier: f32,
    pub properties: Vec<i32>,
    pub default_state_id: u16,
    pub states: Vec<BlockState>,
    pub experience: Option<Experience>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OptimizedBlock {
    pub id: u16,
    pub name: String,
    pub translation_key: String,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub item_id: u16,
    pub loot_table: Option<LootTableStruct>,
    pub slipperiness: f32,
    pub velocity_multiplier: f32,
    pub jump_velocity_multiplier: f32,
    pub default_state_id: u16,
    pub states: Vec<BlockStateRef>,
    pub experience: Option<Experience>,
}

impl ToTokens for OptimizedBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let name = LitStr::new(&self.name, Span::call_site());
        let translation_key = LitStr::new(&self.translation_key, Span::call_site());
        let hardness = &self.hardness;
        let blast_resistance = &self.blast_resistance;
        let item_id = LitInt::new(&self.item_id.to_string(), Span::call_site());
        let default_state_id = LitInt::new(&self.default_state_id.to_string(), Span::call_site());
        let slipperiness = &self.slipperiness;
        let velocity_multiplier = &self.velocity_multiplier;
        let jump_velocity_multiplier = &self.jump_velocity_multiplier;
        let experience = match &self.experience {
            Some(exp) => {
                let exp_tokens = exp.to_token_stream();
                quote! { Some(#exp_tokens) }
            }
            None => quote! { None },
        };
        // Generate state tokens
        let states = self.states.iter().map(|state| state.to_token_stream());
        let loot_table = match &self.loot_table {
            Some(table) => {
                let table_tokens = table.to_token_stream();
                quote! { Some(#table_tokens) }
            }
            None => quote! { None },
        };

        tokens.extend(quote! {
            Block {
                id: #id,
                name: #name,
                translation_key: #translation_key,
                hardness: #hardness,
                blast_resistance: #blast_resistance,
                slipperiness: #slipperiness,
                velocity_multiplier: #velocity_multiplier,
                jump_velocity_multiplier: #jump_velocity_multiplier,
                item_id: #item_id,
                default_state_id: #default_state_id,
                states: &[#(#states),*],
                loot_table: #loot_table,
                experience: #experience,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum GeneratedPropertyType {
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "int")]
    Int { min: u8, max: u8 },
    #[serde(rename = "enum")]
    Enum { values: Vec<String> },
}

#[derive(Deserialize, Clone, Debug)]
pub struct GeneratedProperty {
    hash_key: i32,
    enum_name: String,
    serialized_name: String,
    #[serde(rename = "type")]
    #[serde(flatten)]
    property_type: GeneratedPropertyType,
}

impl GeneratedProperty {
    fn to_property(&self) -> Property {
        let enum_name = match &self.property_type {
            GeneratedPropertyType::Boolean => "boolean".to_string(),
            GeneratedPropertyType::Int { min, max } => format!("integer_{min}_to_{max}"),
            GeneratedPropertyType::Enum { .. } => self.enum_name.clone(),
        };

        let values = match &self.property_type {
            GeneratedPropertyType::Boolean => {
                vec!["true".to_string(), "false".to_string()]
            }
            GeneratedPropertyType::Int { min, max } => {
                let mut values = Vec::new();
                for i in *min..=*max {
                    values.push(format!("L{i}"));
                }
                values
            }
            GeneratedPropertyType::Enum { values } => values.clone(),
        };

        Property {
            enum_name,
            serialized_name: self.serialized_name.clone(),
            values,
        }
    }
}

#[derive(Clone, Debug)]
struct Property {
    enum_name: String,
    serialized_name: String,
    values: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockAssets {
    pub blocks: Vec<Block>,
    pub shapes: Vec<CollisionShape>,
    pub block_entity_types: Vec<String>,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/blocks.json");
    println!("cargo:rerun-if-changed=../assets/properties.json");

    let blocks_assets: BlockAssets = serde_json::from_str(include_str!("../../assets/blocks.json"))
        .expect("Failed to parse blocks.json");

    let generated_properties: Vec<GeneratedProperty> =
        serde_json::from_str(include_str!("../../assets/properties.json"))
            .expect("Failed to parse properties.json");

    let mut type_from_raw_id_arms = TokenStream::new();
    let mut type_from_name = TokenStream::new();
    let mut block_from_state_id = TokenStream::new();
    let mut block_from_item_id = TokenStream::new();
    let mut block_properties_from_state_and_block_id = TokenStream::new();
    let mut block_properties_from_props_and_name = TokenStream::new();
    let mut existing_item_ids: Vec<u16> = Vec::new();
    let mut constants = TokenStream::new();

    // Collect unique block states to create partial block states to save memory.
    let mut unique_states = Vec::new();
    for block in blocks_assets.blocks.clone() {
        for state in block.states.clone() {
            // Check if this state is already in `unique_states` by comparing all fields except `id`.
            let already_exists = unique_states.iter().any(|s: &BlockState| {
                s.state_flags == state.state_flags
                    && s.luminance == state.luminance
                    && s.hardness == state.hardness
                    && s.collision_shapes == state.collision_shapes
            });

            if !already_exists {
                unique_states.push(state);
            }
        }
    }

    // Used to create property `enum`s.
    let mut property_enums: HashMap<String, PropertyStruct> = HashMap::new();
    // Property implementation for a block.
    let mut block_properties: Vec<BlockPropertyStruct> = Vec::new();
    // Mapping of a collection of property hashes -> blocks that have these properties.
    let mut property_collection_map: HashMap<Vec<i32>, PropertyCollectionData> = HashMap::new();
    // Validator that we have no `enum` collisions.
    let mut optimized_blocks: Vec<(String, OptimizedBlock)> = Vec::new();
    for block in blocks_assets.blocks.clone() {
        let optimized_block = OptimizedBlock {
            id: block.id,
            name: block.name.clone(),
            translation_key: block.translation_key.clone(),
            hardness: block.hardness,
            blast_resistance: block.blast_resistance,
            item_id: block.item_id,
            default_state_id: block.default_state_id,
            slipperiness: block.slipperiness,
            velocity_multiplier: block.velocity_multiplier,
            jump_velocity_multiplier: block.jump_velocity_multiplier,
            loot_table: block.loot_table,
            experience: block.experience,
            states: block
                .states
                .iter()
                .map(|state| {
                    // Find the index in `unique_states` by comparing all fields except `id`.
                    let state_idx = unique_states
                        .iter()
                        .position(|s| {
                            s.state_flags == state.state_flags
                                && s.luminance == state.luminance
                                && s.hardness == state.hardness
                                && s.collision_shapes == state.collision_shapes
                        })
                        .unwrap() as u16;

                    BlockStateRef {
                        id: state.id,
                        state_idx,
                    }
                })
                .collect(),
        };

        optimized_blocks.push((block.name.clone(), optimized_block));

        let mut property_collection = HashSet::new();
        let mut property_mapping = Vec::new();
        for property in block.properties {
            let generated_property = generated_properties
                .iter()
                .find(|p| p.hash_key == property)
                .unwrap();
            property_collection.insert(generated_property.hash_key);
            let property = generated_property.to_property();
            let renamed_property = property.enum_name.to_upper_camel_case();

            let property_type = if property.values == vec!["true".to_string(), "false".to_string()]
            {
                PropertyType::Bool
            } else {
                PropertyType::Enum {
                    name: renamed_property.clone(),
                }
            };

            if let PropertyType::Enum { name } = &property_type {
                let _ = property_enums
                    .entry(name.clone())
                    .or_insert_with(|| PropertyStruct {
                        name: name.clone(),
                        values: property.values.clone(),
                    });
            }

            property_mapping.push(PropertyVariantMapping {
                original_name: property.serialized_name.clone(),
                property_type,
            });
        }

        // The Minecraft Java state manager deterministically produces an index given a set of properties. We must use
        // the original property names here when checking for unique combinations of properties, and
        // sort them to make a deterministic hash.

        if !property_collection.is_empty() {
            let mut property_collection = Vec::from_iter(property_collection);
            property_collection.sort();
            property_collection_map
                .entry(property_collection)
                .or_insert_with(|| PropertyCollectionData::from_mappings(property_mapping))
                .add_block(block.name, block.id);
        }
    }

    for property_group in property_collection_map.into_values() {
        for (block_name, id) in &property_group.blocks {
            let const_block_name = Ident::new(
                &const_block_name_from_block_name(block_name),
                Span::call_site(),
            );
            let property_name = Ident::new(
                &property_group_name_from_derived_name(&property_group.derive_name()),
                Span::call_site(),
            );
            let id_lit = LitInt::new(&id.to_string(), Span::call_site());

            block_properties_from_state_and_block_id.extend(quote! {
                #id_lit => Some(Box::new(#property_name::from_state_id(state_id, &Block::#const_block_name))),
            });

            block_properties_from_props_and_name.extend(quote! {
                #id_lit => Some(Box::new(#property_name::from_props(props, &Block::#const_block_name))),
            });
        }

        block_properties.push(BlockPropertyStruct {
            data: property_group,
        });
    }

    // Generate the collision shapes array.
    let shapes = blocks_assets
        .shapes
        .iter()
        .map(|shape| shape.to_token_stream());

    let unique_states = unique_states.iter().map(|state| state.to_tokens());

    let block_props = block_properties.iter().map(|prop| prop.to_token_stream());
    let properties = property_enums.values().map(|prop| prop.to_token_stream());

    // Generate the block entity types array.
    let block_entity_types = blocks_assets
        .block_entity_types
        .iter()
        .map(|entity_type| LitStr::new(entity_type, Span::call_site()));

    // Generate constants and `match` arms for each block.
    for (name, block) in optimized_blocks {
        let const_ident = format_ident!("{}", const_block_name_from_block_name(&name));
        let block_tokens = block.to_token_stream();
        let id_lit = LitInt::new(&block.id.to_string(), Span::call_site());
        let state_start = block.states.iter().map(|state| state.id).min().unwrap();
        let state_end = block.states.iter().map(|state| state.id).max().unwrap();
        let item_id = block.item_id;

        constants.extend(quote! {
            pub const #const_ident: Block = #block_tokens;

        });

        type_from_raw_id_arms.extend(quote! {
            #id_lit => Some(Self::#const_ident),
        });

        type_from_name.extend(quote! {
            #name => Some(Self::#const_ident),
        });

        block_from_state_id.extend(quote! {
            #state_start..=#state_end => Some(Self::#const_ident),
        });

        if !existing_item_ids.contains(&item_id) {
            block_from_item_id.extend(quote! {
                #item_id => Some(Self::#const_ident),
            });
            existing_item_ids.push(item_id);
        }
    }

    quote! {
        use crate::{BlockState, BlockStateRef, Block, CollisionShape};
        use crate::block_state::PistonBehavior;
        use pumpkin_util::math::int_provider::{UniformIntProvider, IntProvider, NormalIntProvider};
        use pumpkin_util::loot_table::*;
        use pumpkin_util::math::experience::Experience;
        use pumpkin_util::math::vector3::Vector3;

        #[derive(Clone, Copy, Debug)]
        pub struct BlockProperty {
            pub name: &'static str,
            pub values: &'static [&'static str],
        }

        pub trait BlockProperties where Self: 'static {
            // Convert properties to an index (`0` to `N-1`).
            fn to_index(&self) -> u16;
            // Convert an index back to properties.
            fn from_index(index: u16) -> Self where Self: Sized;

            // Check if a block uses this property
            fn handles_block_id(block_id: u16) -> bool where Self: Sized;

            // Convert properties to a state id.
            fn to_state_id(&self, block: &Block) -> u16;
            // Convert a state id back to properties.
            fn from_state_id(state_id: u16, block: &Block) -> Self where Self: Sized;
            // Get the default properties.
            fn default(block: &Block) -> Self where Self: Sized;

            // Convert properties to a `Vec` of `(name, value)`
            fn to_props(&self) -> Vec<(String, String)>;

            // Convert properties to a block state, and add them onto the default state.
            fn from_props(props: Vec<(&str, &str)>, block: &Block) -> Self where Self: Sized;
        }

        pub trait EnumVariants {
            fn variant_count() -> u16;
            fn to_index(&self) -> u16;
            fn from_index(index: u16) -> Self;
            fn to_value(&self) -> &str;
            fn from_value(value: &str) -> Self;
        }



        pub static COLLISION_SHAPES: &[CollisionShape] = &[
            #(#shapes),*
        ];

        pub static BLOCK_STATES: &[BlockState] = &[
            #(#unique_states),*
        ];

        pub static BLOCK_ENTITY_TYPES: &[&str] = &[
            #(#block_entity_types),*
        ];



        impl Block {
            #constants

            #[doc = r" Try to parse a block from a resource location string."]
            pub fn from_registry_key(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }

            #[doc = r" Try to parse a block from a raw id."]
            pub const fn from_id(id: u16) -> Option<Self> {
                match id {
                    #type_from_raw_id_arms
                    _ => None
                }
            }

            #[doc = r" Try to parse a block from a state id."]
            pub const fn from_state_id(id: u16) -> Option<Self> {
                match id {
                    #block_from_state_id
                    _ => None
                }
            }

            #[doc = r" Try to parse a block from an item id."]
            pub const fn from_item_id(id: u16) -> Option<Self> {
                #[allow(unreachable_patterns)]
                match id {
                    #block_from_item_id
                    _ => None
                }
            }

            #[doc = r" Get the properties of the block."]
            pub fn properties(&self, state_id: u16) -> Option<Box<dyn BlockProperties>> {
                match self.id {
                    #block_properties_from_state_and_block_id
                    _ => None
                }
            }

            #[doc = r" Get the properties of the block."]
            pub fn from_properties(&self, props: Vec<(&str, &str)>) -> Option<Box<dyn BlockProperties>> {
                match self.id {
                    #block_properties_from_props_and_name
                    _ => None
                }
            }
        }

        #(#properties)*

        #(#block_props)*

        impl BlockStateRef {
            pub fn get_state(&self) -> BlockState {
                let mut state = BLOCK_STATES[self.state_idx as usize].clone();
                state.id = self.id;
                state
            }
        }

        impl Facing {
            pub fn opposite(&self) -> Self {
                match self {
                    Facing::North => Facing::South,
                    Facing::South => Facing::North,
                    Facing::East => Facing::West,
                    Facing::West => Facing::East,
                    Facing::Up => Facing::Down,
                    Facing::Down => Facing::Up,
                }
            }
        }

        impl HorizontalFacing {
            pub fn all() -> [HorizontalFacing; 4] {
                [
                    HorizontalFacing::North,
                    HorizontalFacing::South,
                    HorizontalFacing::West,
                    HorizontalFacing::East,
                ]
            }

            pub fn to_offset(&self) -> Vector3<i32> {
                match self {
                    Self::North => (0, 0, -1),
                    Self::South => (0, 0, 1),
                    Self::West => (-1, 0, 0),
                    Self::East => (1, 0, 0),
                }
                .into()
            }

            pub fn opposite(&self) -> Self {
                match self {
                    Self::North => Self::South,
                    Self::South => Self::North,
                    Self::West => Self::East,
                    Self::East => Self::West,
                }
            }

            pub fn rotate_clockwise(&self) -> Self {
                match self {
                    Self::North => Self::East,
                    Self::South => Self::West,
                    Self::West => Self::North,
                    Self::East => Self::South,
                }
            }

            pub fn rotate_counter_clockwise(&self) -> Self {
                match self {
                    Self::North => Self::West,
                    Self::South => Self::East,
                    Self::West => Self::South,
                    Self::East => Self::North,
                }
            }
        }
    }
}
