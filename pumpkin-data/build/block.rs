use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use pumpkin_util::math::{experience::Experience, vector3::Vector3};
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
};
use syn::{Ident, LitInt, LitStr};

use crate::loot::LootTableStruct;

// Takes an array of tuples containing indices paired with values,Add commentMore actions
// Outputs an array with the values in the appropriate index, gaps filled with None
fn fill_array<T: Clone + quote::ToTokens>(array: Vec<(u16, T)>) -> Vec<TokenStream> {
    let max_index = array.iter().map(|(index, _)| index).max().unwrap();
    let mut raw_id_from_state_id_ordered = vec![quote! { None }; (max_index + 1) as usize];

    for (state_id, id_lit) in array {
        raw_id_from_state_id_ordered[state_id as usize] = quote! { #id_lit };
    }

    raw_id_from_state_id_ordered
}

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
                        _ => panic!("Invalid index: {index}"),
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
                        _ => panic!("Invalid value: {value:?}"),
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
                        (!self.#field_name as u16, 2)
                    },
                    PropertyType::Enum { name } => {
                        let enum_ident = Ident::new(name, Span::call_site());
                        quote! {
                            (self.#field_name.to_index(), #enum_ident::variant_count())
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
                    (#key.to_string(), self.#field_name.to_string()),
                },
                PropertyType::Enum { name: _ } => quote! {
                    (#key.to_string(), self.#field_name.to_value().to_string()),
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
                fn to_index(&self) -> u16 {
                    let (index, _) = [#(#to_index_body),*]
                    .iter()
                    .fold((0, 1), |(current_index, multiplier), &(value, count)| {
                      (current_index + value * multiplier, multiplier * count)
                    });
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
                    Self::from_state_id(block.default_state.id, block)
                }

                fn to_props(&self) -> HashMap<String, String> {
                   HashMap::from([#(#to_props_values)*])
                }
                fn from_props(props: HashMap<&str, &str>, block: &Block) -> Self {
                    if ![#(#block_ids),*].contains(&block.id) {
                        panic!("{} is not a valid block for {}", &block.name, #struct_name);
                    }
                    let mut block_props = Self::default(block);
                    for (key, value) in props {
                        match key {
                            #(#from_props_values),*,
                            _ => panic!("Invalid key: {key}"),
                        }
                    }
                    block_props
                }
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct FlammableStruct {
    pub spread_chance: u8,
    pub burn_chance: u8,
}

impl ToTokens for FlammableStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let spread_chance = &self.spread_chance;
        let burn_chance = &self.burn_chance;

        tokens.extend(quote! {
            Flammable {
                spread_chance: #spread_chance,
                burn_chance: #burn_chance,
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
    pub outline_shapes: Vec<u16>,
    pub has_random_ticks: bool,
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

impl BlockState {
    fn to_tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let state_flags = LitInt::new(&self.state_flags.to_string(), Span::call_site());
        let side_flags = LitInt::new(&self.side_flags.to_string(), Span::call_site());
        let instrument = format_ident!("{}", self.instrument.to_upper_camel_case());
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
        let outline_shapes = self
            .outline_shapes
            .iter()
            .map(|shape_id| LitInt::new(&shape_id.to_string(), Span::call_site()));
        let has_random_ticks = self.has_random_ticks;
        let piston_behavior = &self.piston_behavior.to_tokens();

        tokens.extend(quote! {
            BlockState {
                id: #id,
                state_flags: #state_flags,
                side_flags: #side_flags,
                instrument: Instrument::#instrument,
                luminance: #luminance,
                piston_behavior: #piston_behavior,
                hardness: #hardness,
                collision_shapes: &[#(#collision_shapes),*],
                outline_shapes: &[#(#outline_shapes),*],
                has_random_tick: #has_random_ticks,
                opacity: #opacity,
                block_entity_type: #block_entity_type,
            }
        });
        tokens
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
    pub flammable: Option<FlammableStruct>,
    pub loot_table: Option<LootTableStruct>,
    pub slipperiness: f32,
    pub velocity_multiplier: f32,
    pub jump_velocity_multiplier: f32,
    pub properties: Vec<i32>,
    pub default_state_id: u16,
    pub states: Vec<BlockState>,
    pub experience: Option<Experience>,
}

impl Block {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let name = LitStr::new(&self.name, Span::call_site());
        let translation_key = LitStr::new(&self.translation_key, Span::call_site());
        let hardness = &self.hardness;
        let blast_resistance = &self.blast_resistance;
        let item_id = LitInt::new(&self.item_id.to_string(), Span::call_site());
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
        let states = self.states.iter().map(|state| state.to_tokens());
        let loot_table = match &self.loot_table {
            Some(table) => {
                let table_tokens = table.to_token_stream();
                quote! { Some(#table_tokens) }
            }
            None => quote! { None },
        };

        let default_state_ref: &BlockState = self
            .states
            .iter()
            .find(|state| state.id == self.default_state_id)
            .unwrap();
        let mut default_state = default_state_ref.clone();
        default_state.id = default_state_ref.id;
        let default_state = default_state.to_tokens();
        let flammable = match &self.flammable {
            Some(flammable) => {
                let flammable_tokens = flammable.to_token_stream();
                quote! { Some(#flammable_tokens) }
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
                default_state: &#default_state,
                states: &[#(#states),*],
                flammable: #flammable,
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

    let blocks_assets: BlockAssets =
        serde_json::from_str(&fs::read_to_string("../assets/blocks.json").unwrap())
            .expect("Failed to parse blocks.json");

    let generated_properties: Vec<GeneratedProperty> =
        serde_json::from_str(&fs::read_to_string("../assets/properties.json").unwrap())
            .expect("Failed to parse properties.json");

    let mut type_from_raw_id_items = TokenStream::new();
    let mut block_from_name = TokenStream::new();
    let mut raw_id_from_state_id = TokenStream::new();
    let mut block_from_item_id = TokenStream::new();
    let mut random_tick_states = Vec::new();
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
    let mut optimized_blocks: Vec<(String, Block)> = Vec::new();
    for block in blocks_assets.blocks.clone() {
        optimized_blocks.push((block.name.clone(), block.clone()));

        // Collect state IDs that have random ticks.
        for state in &block.states {
            if state.has_random_ticks {
                let state_id = LitInt::new(&state.id.to_string(), Span::call_site());
                random_tick_states.push(state_id);
            }
        }

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
                .add_block(block.name.clone(), block.id);
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

    let random_tick_state_ids = quote! {
        #(#random_tick_states)|*
    };

    //let unique_states_tokens = unique_states.iter().map(|state| state.to_tokens());

    let block_props = block_properties.iter().map(|prop| prop.to_token_stream());
    let properties = property_enums.values().map(|prop| prop.to_token_stream());

    // Generate the block entity types array.
    let block_entity_types = blocks_assets
        .block_entity_types
        .iter()
        .map(|entity_type| LitStr::new(entity_type, Span::call_site()));

    let mut raw_id_from_state_id_array = vec![];
    let mut type_from_raw_id_array = vec![];

    // Generate constants and `match` arms for each block.
    for (name, block) in optimized_blocks {
        let const_ident = format_ident!("{}", const_block_name_from_block_name(&name));
        let mut block_tokens = TokenStream::new();
        block.to_tokens(&mut block_tokens);
        let id_lit = LitInt::new(&block.id.to_string(), Span::call_site());

        let item_id = block.item_id;

        constants.extend(quote! {
            pub const #const_ident: Block = #block_tokens;

        });

        type_from_raw_id_array.push((block.id, quote! { &Self::#const_ident }));

        block_from_name.extend(quote! {
            #name => Self::#const_ident,
        });

        for state in &block.states {
            raw_id_from_state_id_array.push((state.id, id_lit.clone()));
        }

        if !existing_item_ids.contains(&item_id) {
            block_from_item_id.extend(quote! {
                #item_id => Some(&Self::#const_ident),
            });
            existing_item_ids.push(item_id);
        }
    }

    let raw_id_from_state_id_ordered = fill_array(raw_id_from_state_id_array);
    let max_state_id = raw_id_from_state_id_ordered.len();
    for id_lit in raw_id_from_state_id_ordered {
        raw_id_from_state_id.extend(quote! {
            #id_lit,
        });
    }
    let type_from_raw_id_array = fill_array(type_from_raw_id_array);
    let max_type_id = type_from_raw_id_array.len();
    for type_lit in type_from_raw_id_array {
        type_from_raw_id_items.extend(quote! {
            #type_lit,
        });
    }

    quote! {
        use crate::{BlockState, Block, CollisionShape, blocks::Flammable};
        use crate::block_state::PistonBehavior;
        use pumpkin_util::math::int_provider::{UniformIntProvider, IntProvider, NormalIntProvider};
        use pumpkin_util::loot_table::*;
        use pumpkin_util::math::experience::Experience;
        use pumpkin_util::math::vector3::Vector3;
        use std::collections::HashMap;
        use phf;


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
            fn to_props(&self) -> HashMap<String, String>;

            // Convert properties to a block state, and add them onto the default state.
            fn from_props(props: HashMap<&str, &str>, block: &Block) -> Self where Self: Sized;
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

        //pub static BLOCK_STATES: &[BlockState] = &[
        //    #(#unique_states_tokens),*
        //];

        pub static BLOCK_ENTITY_TYPES: &[&str] = &[
            #(#block_entity_types),*
        ];

        pub fn get_block(registry_id: &str) -> Option<&'static Block> {
           let key = registry_id.strip_prefix("minecraft:").unwrap_or(registry_id);
           Block::from_registry_key(key)
        }

        pub fn get_block_by_id(id: u16) -> &'static Block {
            Block::from_id(id)
        }

        pub fn get_state_by_state_id(id: u16) -> &'static BlockState {
            let state: &BlockState = Block::from_state_id(id).states.iter().find(|state| state.id == id).unwrap();
            state
        }

        pub fn get_block_by_state_id(id: u16) -> &'static Block {
            Block::from_state_id(id)
        }

        pub fn get_block_and_state_by_state_id(id: u16) -> (&'static Block, &'static BlockState) {
            let block = Block::from_state_id(id);
            let state: &BlockState = block.states.iter().find(|state| state.id == id).unwrap();
            (block, state)
        }

        pub fn get_block_by_item(item_id: u16) -> Option<&'static Block> {
            Block::from_item_id(item_id)
        }

        pub fn has_random_ticks(state_id: u16) -> bool {
            matches!(state_id, #random_tick_state_ids)
        }

        pub fn blocks_movement(block_state: &BlockState) -> bool {
            if block_state.is_solid() {
                let block = get_block_by_state_id(block_state.id);
                return block != &Block::COBWEB && block != &Block::BAMBOO_SAPLING;
            }
            false
        }

        impl Block {
            #constants

            // String name to block struct
            const BLOCK_FROM_NAME_MAP: phf::Map<&'static str, Block> = phf::phf_map!{
                #block_from_name
            };

            // Many state ids map to single raw block id
            const RAW_ID_FROM_STATE_ID: [u16; #max_state_id] = [
                #raw_id_from_state_id
            ];

            const TYPE_FROM_RAW_ID: [&'static Block; #max_type_id] = [
                #type_from_raw_id_items
            ];

            #[doc = r" Try to parse a block from a resource location string."]
            pub fn from_registry_key(name: &str) -> Option<&'static Self> {
                Self::BLOCK_FROM_NAME_MAP.get(name)
            }

            #[doc = r" Try to parse a block from a raw id."]
            pub const fn from_id(id: u16) -> &'static Self {
                if id as usize >= Self::RAW_ID_FROM_STATE_ID.len() {
                    &Self::AIR
                } else {
                    Self::TYPE_FROM_RAW_ID[id as usize]
                }
            }

            #[doc = r" Try to parse a block from a state id."]
            pub const fn from_state_id(id: u16) -> &'static Self {
                if id as usize >= Self::RAW_ID_FROM_STATE_ID.len() {
                    return &Self::AIR;
                }
                Self::from_id(Self::RAW_ID_FROM_STATE_ID[id as usize])
            }

            #[doc = r" Try to parse a block from an item id."]
            pub const fn from_item_id(id: u16) -> Option<&'static Self> {
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
            pub fn from_properties(&self, props: HashMap<&str, &str>) -> Option<Box<dyn BlockProperties>> {
                match self.id {
                    #block_properties_from_props_and_name
                    _ => None
                }
            }
        }

        #(#properties)*

        #(#block_props)*

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

        impl RailShape {
            pub fn is_ascending(&self) -> bool {
                matches!(self, Self::AscendingEast | Self::AscendingWest | Self::AscendingNorth | Self::AscendingSouth)
            }
        }

        impl StraightRailShape {
            pub fn is_ascending(&self) -> bool {
                matches!(self, Self::AscendingEast | Self::AscendingWest | Self::AscendingNorth | Self::AscendingSouth)
            }
        }
    }
}
