use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use pumpkin_util::math::{experience::Experience, vector3::Vector3};
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{Cursor, Read},
    panic,
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

fn fill_state_array(array: Vec<(Ident, usize, u16)>) -> Vec<TokenStream> {
    let max_index = array.iter().map(|(_, _, index)| index).max().unwrap();
    let mut ret = vec![quote! { Missed State }; (max_index + 1) as usize];

    for (block, index, state_id) in array {
        let index_lit = LitInt::new(&index.to_string(), Span::call_site());
        ret[state_id as usize] = quote! { &Self::#block.states[#index_lit] };
    }

    ret
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
        if self.values[0] == "true" && self.values[1] == "false" {
            return;
        }

        let name = Ident::new(&self.name, Span::call_site());
        let count = self.values.len();
        let variant_count = count as u16;

        let is_number_values = !self.values.is_empty()
            && self.values.iter().all(|v| v.starts_with('L'))
            && self.values.iter().any(|v| v == "L1");

        let mut variants = Vec::with_capacity(count);
        let mut literals = Vec::with_capacity(count);
        let mut indices = Vec::with_capacity(count);

        for (i, raw_value) in self.values.iter().enumerate() {
            let ident = Ident::new(&raw_value.to_upper_camel_case(), Span::call_site());

            let literal_str = if is_number_values {
                raw_value.strip_prefix('L').unwrap_or(raw_value)
            } else {
                raw_value.as_str()
            };

            variants.push(ident);
            literals.push(literal_str);
            indices.push(i as u16);
        }
        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub enum #name {
                #(#variants),*
            }

            impl EnumVariants for #name {
                fn variant_count() -> u16 {
                    #variant_count
                }

                fn to_index(&self) -> u16 {
                    match self {
                        #(Self::#variants => #indices),*
                    }
                }

                fn from_index(index: u16) -> Self {
                    match index {
                        #(#indices => Self::#variants,)*
                        _ => panic!("Invalid index: {index}"),
                    }
                }

                fn to_value(&self) -> &'static str {
                    match self {
                        #(Self::#variants => #literals),*
                    }
                }

                fn from_value(value: &str) -> Self {
                    match value {
                        #(#literals => Self::#variants),*,
                        _ => panic!("Invalid value: {value}"),
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

        let to_index_logic = self.data.variant_mappings.iter().rev().map(|entry| {
            let field = Ident::new_raw(&entry.original_name, Span::call_site());
            match &entry.property_type {
                PropertyType::Bool => quote! { (!self.#field as u16, 2) },
                PropertyType::Enum { name } => {
                    let ty = Ident::new(name, Span::call_site());
                    quote! { (self.#field.to_index(), #ty::variant_count()) }
                }
            }
        });

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

        let to_props_entries = self.data.variant_mappings.iter().map(|entry| {
            let key_str = &entry.original_name;
            let field = Ident::new_raw(&entry.original_name, Span::call_site());
            match &entry.property_type {
                PropertyType::Bool => quote! {
                    (#key_str, if self.#field { "true" } else { "false" })
                },
                PropertyType::Enum { .. } => quote! {
                    (#key_str, self.#field.to_value())
                },
            }
        });

        let from_props_values = self.data.variant_mappings.iter().map(|entry| {
            let key = &entry.original_name;
            let field_name = Ident::new_raw(&entry.original_name, Span::call_site());
            match &entry.property_type {
                PropertyType::Bool => quote! {
                    #key => {
                        block_props.#field_name = matches!(*value, "true")
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
                    let (index, _) = [#(#to_index_logic),*]
                        .iter()
                        .fold((0, 1), |(curr, mul), &(val, count)| (curr + val * mul, mul * count));
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

               fn to_props(&self) -> Vec<(&'static str, &'static str)> {
                   vec![ #(#to_props_entries),* ]
                }

                fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
                    if ![#(#block_ids),*].contains(&block.id) {
                        panic!("{} is not a valid block for {}", &block.name, #struct_name);
                    }
                    let mut block_props = Self::default(block);
                    for (key, value) in props {
                        match *key {
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

#[derive(Deserialize)]
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

#[derive(Deserialize, Clone, Copy)]
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

#[derive(Deserialize, Clone)]
pub struct BlockState {
    pub id: u16,
    pub state_flags: u16,
    pub side_flags: u8,
    pub instrument: String, // TODO: make this an enum
    pub luminance: u8,
    pub piston_behavior: PistonBehavior,
    pub hardness: f32,
    pub collision_shapes: Vec<u16>,
    pub outline_shapes: Vec<u16>,
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
    const HAS_RANDOM_TICKS: u16 = 1 << 9;

    fn has_random_ticks(&self) -> bool {
        self.state_flags & Self::HAS_RANDOM_TICKS != 0
    }

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
            None => quote! { 0 },
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
                opacity: #opacity,
                block_entity_type: #block_entity_type,
            }
        });
        tokens
    }
}

#[derive(Deserialize)]
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

impl ToTokens for Block {
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

#[derive(Deserialize, Clone)]
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

#[derive(Clone)]
struct Property {
    enum_name: String,
    serialized_name: String,
    values: Vec<String>,
}

#[derive(Deserialize)]
pub struct BlockAssets {
    pub blocks: Vec<Block>,
    pub shapes: Vec<CollisionShape>,
    pub block_entity_types: Vec<String>,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/blocks.json");
    println!("cargo:rerun-if-changed=../assets/bedrock_block_states.nbt");
    println!("cargo:rerun-if-changed=../assets/properties.json");

    let be_blocks_data = fs::read("../assets/bedrock_block_states.nbt").unwrap();
    let mut be_blocks_cursor = Cursor::new(be_blocks_data);
    let be_blocks = get_be_data_from_nbt(&mut be_blocks_cursor);

    let blocks_assets: BlockAssets =
        serde_json::from_str(&fs::read_to_string("../assets/blocks.json").unwrap())
            .expect("Failed to parse blocks.json");

    let generated_properties: Vec<GeneratedProperty> =
        serde_json::from_str(&fs::read_to_string("../assets/properties.json").unwrap())
            .expect("Failed to parse properties.json");

    let generated_prop_map: std::collections::HashMap<i32, &GeneratedProperty> =
        generated_properties
            .iter()
            .map(|p| (p.hash_key, p))
            .collect();

    let mut random_tick_states = Vec::new();
    let mut constants_list = Vec::new();
    let mut block_from_name_entries = Vec::new();
    let mut block_from_item_id_arms = Vec::new();
    let mut block_state_to_bedrock = Vec::new();

    let mut raw_id_from_state_id_array = Vec::new();
    let mut type_from_raw_id_array = Vec::new();
    let mut state_from_state_id_array = Vec::<(Ident, usize, u16)>::new();

    let mut property_enums: BTreeMap<String, PropertyStruct> = BTreeMap::new();
    let mut block_properties: Vec<BlockPropertyStruct> = Vec::new();
    let mut property_collection_map: BTreeMap<Vec<i32>, PropertyCollectionData> = BTreeMap::new();
    let mut existing_item_ids: std::collections::HashSet<u16> = std::collections::HashSet::new();

    for block in blocks_assets.blocks {
        for state in &block.states {
            if state.has_random_ticks() {
                let state_id = LitInt::new(&state.id.to_string(), Span::call_site());
                random_tick_states.push(state_id);
            }
        }

        let mut property_collection = HashSet::new();
        let mut property_mapping = Vec::new();

        for property_hash in &block.properties {
            let generated_property = generated_prop_map
                .get(property_hash)
                .expect("Property hash not found in generated_properties");

            property_collection.insert(generated_property.hash_key);

            let property = generated_property.to_property();
            let renamed_property = property.enum_name.to_upper_camel_case();

            let property_type = if property.values.len() == 2
                && property.values.contains(&"true".to_string())
                && property.values.contains(&"false".to_string())
            {
                PropertyType::Bool
            } else {
                PropertyType::Enum {
                    name: renamed_property.clone(),
                }
            };

            if let PropertyType::Enum { name } = &property_type {
                property_enums
                    .entry(name.clone())
                    .or_insert_with(|| PropertyStruct {
                        name: name.clone(),
                        values: property.values.clone(),
                    });
            }

            property_mapping.push(PropertyVariantMapping {
                original_name: property.serialized_name,
                property_type,
            });
        }

        if !property_collection.is_empty() {
            let mut property_collection_vec: Vec<i32> = property_collection.into_iter().collect();
            property_collection_vec.sort_unstable();

            property_collection_map
                .entry(property_collection_vec)
                .or_insert_with(|| PropertyCollectionData::from_mappings(property_mapping))
                .add_block(block.name.clone(), block.id);
        }

        let const_ident = format_ident!("{}", const_block_name_from_block_name(&block.name));
        let name_str = &block.name;
        let id_lit = LitInt::new(&block.id.to_string(), Span::call_site());
        let item_id = block.item_id;

        constants_list.push(quote! {
            pub const #const_ident: Block = #block;
        });

        type_from_raw_id_array.push((block.id, quote! { &Self::#const_ident }));

        block_from_name_entries.push(quote! {
            #name_str => Self::#const_ident,
        });

        let be_name = match block.name.as_str() {
            "dead_bush" => "deadbush",
            "tripwire" => "trip_wire",
            "note_block" => "noteblock",
            "powered_rail" => "golden_rail",
            "cobweb" => "web",
            "tall_seagrass" => "seagrass",
            "wall_torch" => "torch",
            "spawner" => "mob_spawner",
            "snow" => "snow_layer",
            "snow_block" => "snow",
            name => name,
        };

        let (state_count, id) = *be_blocks.get(be_name).unwrap_or(&(0, 1));

        for (i, state) in block.states.iter().enumerate() {
            if state_count != 0 {
                let bedrock_val = if state_count > i as u32 {
                    id as u16 + i as u16
                } else {
                    id as u16
                };
                block_state_to_bedrock.push((state.id, bedrock_val));
            }

            raw_id_from_state_id_array.push((state.id, id_lit.clone()));
            state_from_state_id_array.push((const_ident.clone(), i, state.id));
        }

        if existing_item_ids.insert(item_id) {
            block_from_item_id_arms.push(quote! {
                #item_id => Some(&Self::#const_ident),
            });
        }
    }

    let mut block_properties_from_state_and_block_id_arms = Vec::new();
    let mut block_properties_from_props_and_name_arms = Vec::new();

    for property_group in property_collection_map.into_values() {
        let property_name = Ident::new(
            &property_group_name_from_derived_name(&property_group.derive_name()),
            Span::call_site(),
        );

        for (block_name, id) in &property_group.blocks {
            let const_block_name = Ident::new(
                &const_block_name_from_block_name(block_name),
                Span::call_site(),
            );
            let id_lit = LitInt::new(&id.to_string(), Span::call_site());

            block_properties_from_state_and_block_id_arms.push(quote! {
                #id_lit => Box::new(#property_name::from_state_id(state_id, &Block::#const_block_name)),
            });

            block_properties_from_props_and_name_arms.push(quote! {
                #id_lit => Box::new(#property_name::from_props(props, &Block::#const_block_name)),
            });
        }

        block_properties.push(BlockPropertyStruct {
            data: property_group,
        });
    }

    let shapes = blocks_assets
        .shapes
        .iter()
        .map(|shape| shape.to_token_stream());

    let random_tick_state_ids = quote! { #(#random_tick_states)|* };

    let block_props = block_properties.iter().map(|prop| prop.to_token_stream());
    let properties = property_enums.values().map(|prop| prop.to_token_stream());

    let block_entity_types = blocks_assets
        .block_entity_types
        .iter()
        .map(|entity_type| LitStr::new(entity_type, Span::call_site()));

    let raw_id_from_state_id_ordered = fill_array(raw_id_from_state_id_array);
    let max_state_id = raw_id_from_state_id_ordered.len();
    let raw_id_from_state_id = quote! { #(#raw_id_from_state_id_ordered),* };

    let max_index = block_state_to_bedrock
        .iter()
        .map(|(idx, _)| *idx)
        .max()
        .unwrap_or(0);
    let mut state_to_bedrock_tokens = vec![quote! { 1 }; (max_index + 1) as usize];
    for (state_id, id_lit) in block_state_to_bedrock {
        let lit = LitInt::new(&id_lit.to_string(), Span::call_site());
        state_to_bedrock_tokens[state_id as usize] = quote! { #lit };
    }
    let block_state_to_bedrock_t = quote! { #(#state_to_bedrock_tokens),* };

    let type_from_raw_id_vec = fill_array(type_from_raw_id_array);
    let max_type_id = type_from_raw_id_vec.len();
    let type_from_raw_id_items = quote! { #(#type_from_raw_id_vec),* };

    let state_from_state_id_vec = fill_state_array(state_from_state_id_array);
    let max_state_id_2 = state_from_state_id_vec.len();
    let state_from_state_id = quote! { #(#state_from_state_id_vec),* };

    assert_eq!(max_state_id, max_state_id_2);

    quote! {
        use crate::{BlockState, Block, CollisionShape, blocks::Flammable};
        use crate::block_state::PistonBehavior;
        use pumpkin_util::math::int_provider::{UniformIntProvider, IntProvider, NormalIntProvider};
        use pumpkin_util::loot_table::*;
        use pumpkin_util::math::experience::Experience;
        use pumpkin_util::math::vector3::Vector3;
        use std::collections::BTreeMap;
        use phf;

        #[derive(Clone, Copy, Debug)]
        pub struct BlockProperty {
            pub name: &'static str,
            pub values: &'static [&'static str],
        }

        pub trait BlockProperties where Self: 'static {
            fn to_index(&self) -> u16;
            fn from_index(index: u16) -> Self where Self: Sized;
            fn handles_block_id(block_id: u16) -> bool where Self: Sized;
            fn to_state_id(&self, block: &Block) -> u16;
            fn from_state_id(state_id: u16, block: &Block) -> Self where Self: Sized;
            fn default(block: &Block) -> Self where Self: Sized;
            fn to_props(&self) -> Vec<(&'static str, &'static str)>;
            fn from_props(props: &[(&str, &str)], block: &Block) -> Self where Self: Sized;
        }

        pub trait EnumVariants {
            fn variant_count() -> u16;
            fn to_index(&self) -> u16;
            fn from_index(index: u16) -> Self;
            fn to_value(&self) -> &'static str;
            fn from_value(value: &str) -> Self;
        }

        pub const COLLISION_SHAPES: &[CollisionShape] = &[
            #(#shapes),*
        ];

        pub const BLOCK_ENTITY_TYPES: &[&str] = &[
            #(#block_entity_types),*
        ];

        pub fn has_random_ticks(state_id: u16) -> bool {
            matches!(state_id, #random_tick_state_ids)
        }

        pub fn blocks_movement(block_state: &BlockState, block: &Block) -> bool {
            if block_state.is_solid() {
                return block != &Block::COBWEB && block != &Block::BAMBOO_SAPLING;
            }
            false
        }

        impl BlockState {
            const STATE_ID_TO_BEDROCK: &[u16] = &[
                #block_state_to_bedrock_t
            ];

            #[doc = r" Get a block state from a state id."]
            #[doc = r" If you need access to the block use `BlockState::from_id_with_block` instead."]
            #[inline]
            pub fn from_id(id: u16) -> &'static Self {
                Block::STATE_FROM_STATE_ID[id as usize]
            }

            #[doc = r" Get a block state from a state id and the corresponding block."]
            #[inline]
            pub fn from_id_with_block(id: u16) -> (&'static Block, &'static Self) {
                let block = Block::from_state_id(id);
                let state: &Self = Block::STATE_FROM_STATE_ID[id as usize];
                (block, state)
            }

            pub fn to_be_network_id(id: u16) -> u16 {
                Self::STATE_ID_TO_BEDROCK[id as usize]
            }
        }

        impl Block {
            #(#constants_list)*

            const BLOCK_FROM_NAME_MAP: phf::Map<&'static str, Block> = phf::phf_map!{
                #(#block_from_name_entries)*
            };

            const RAW_ID_FROM_STATE_ID: [u16; #max_state_id] = [
                #raw_id_from_state_id
            ];

            const TYPE_FROM_RAW_ID: [&'static Block; #max_type_id] = [
                #type_from_raw_id_items
            ];

            const STATE_FROM_STATE_ID: [&'static BlockState; #max_state_id] = [
                #state_from_state_id
            ];

            #[doc = r" Try to parse a block from a resource location string."]
            #[inline]
            pub fn from_registry_key(name: &str) -> Option<&'static Self> {
                Self::BLOCK_FROM_NAME_MAP.get(name)
            }

            #[doc = r" Try to get a block from a namespace prefixed name."]
            pub fn from_name(name: &str) -> Option<&'static Self> {
                let key = name.strip_prefix("minecraft:").unwrap_or(name);
                Self::BLOCK_FROM_NAME_MAP.get(key)
            }

            #[doc = r" Get a block from a raw block id."]
            #[inline]
            pub const fn from_id(id: u16) -> &'static Self {
                if id as usize >= Self::RAW_ID_FROM_STATE_ID.len() {
                    &Self::AIR
                } else {
                    Self::TYPE_FROM_RAW_ID[id as usize]
                }
            }

             #[doc = r" Get a raw ID from an State ID."]
            #[inline]
            pub const fn get_raw_id_from_state_id(state_id: u16) -> u16 {
                if state_id as usize >= Self::RAW_ID_FROM_STATE_ID.len() {
                    0
                } else {
                    Self::RAW_ID_FROM_STATE_ID[state_id as usize]
                }
            }

            #[doc = r" Get a block from a state id."]
            #[inline]
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
                    #(#block_from_item_id_arms)*
                    _ => None
                }
            }

            #[track_caller]
            #[doc = r" Get the properties of the block."]
            pub fn properties(&self, state_id: u16) -> Option<Box<dyn BlockProperties>> {
                Some(match self.id {
                    #(#block_properties_from_state_and_block_id_arms)*
                    _ => return None,
                })
            }

            #[track_caller]
            #[doc = r" Get the properties of the block."]
            pub fn from_properties(&self, props: &[(&str, &str)]) -> Box<dyn BlockProperties> {
                match self.id {
                    #(#block_properties_from_props_and_name_arms)*
                    _ => panic!("Invalid props")
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

fn get_be_data_from_nbt<R: Read>(reader: &mut R) -> BTreeMap<String, (u32, u32)> {
    let mut block_data: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    let mut current_id = 0;

    while read_byte(reader) == 10 {
        let len = read_varint(reader);
        let mut buf = vec![0; len as usize];
        reader.read_exact(&mut buf).unwrap();

        let mut name = String::new();
        let mut byte = read_byte(reader);

        while byte != 0 {
            let mut name_buf = vec![0; read_varint(reader) as usize];
            reader.read_exact(&mut name_buf).unwrap();
            let cp_name = String::from_utf8(name_buf).unwrap();

            match cp_name.as_str() {
                "name" => {
                    let mut name_buf = vec![0; read_varint(reader) as usize];
                    reader.read_exact(&mut name_buf).unwrap();
                    name = String::from_utf8(name_buf)
                        .unwrap()
                        .strip_prefix("minecraft:")
                        .unwrap()
                        .to_string();
                }
                "states" => {
                    let mut byte = read_byte(reader);
                    while byte != 0 {
                        let b = &mut vec![0; read_varint(reader) as usize];
                        reader.read_exact(b).unwrap();

                        match byte {
                            8 => {
                                let b = &mut vec![0; read_varint(reader) as usize];
                                reader.read_exact(b).unwrap();
                            }
                            3 => {
                                read_varint(reader);
                            }
                            1 => {
                                read_byte(reader);
                            }
                            _ => panic!("{}", byte),
                        }
                        byte = read_byte(reader);
                    }
                }
                "version" => {
                    read_varint(reader);
                }
                _ => panic!(),
            }
            byte = read_byte(reader);
        }

        block_data
            .entry(name)
            .and_modify(|(v, _)| *v += 1)
            .or_insert((1, current_id));
        current_id += 1;
    }
    block_data
}

fn read_varint<W: Read>(reader: &mut W) -> u32 {
    let mut val = 0;
    for i in 0..5u32 {
        let byte = &mut [0];
        reader.read_exact(byte).unwrap();
        val |= (u32::from(byte[0]) & 0x7F) << (i * 7);
        if byte[0] & 0x80 == 0 {
            return val;
        }
    }
    panic!()
}

fn read_byte<W: Read>(reader: &mut W) -> u8 {
    let byte = &mut [0];
    reader.read_exact(byte).unwrap_or_default();
    byte[0]
}
