use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use pumpkin_util::math::{experience::Experience, vector3::Vector3};
use quote::{format_ident, quote, ToTokens};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    io::{Cursor, Read},
    panic,
};
use syn::{Ident, LitInt, LitStr};

use crate::{
    bitsets::{gen_u16_bitset, Bitset},
    loot::LootTableStruct,
};

/// Converts a sparse index-value list into a dense array, filling gaps with `None` tokens.
///
/// # Arguments
/// – `array` – pairs of `(index, value)` where `index` is the position in the output array.
fn fill_array<T: Clone + ToTokens>(array: Vec<(u16, T)>) -> Vec<TokenStream> {
    let max_index = array.iter().map(|(index, _)| index).max().unwrap();
    let mut raw_id_from_state_id_ordered = vec![quote! { None }; (max_index + 1) as usize];

    for (state_id, id_lit) in array {
        raw_id_from_state_id_ordered[state_id as usize] = quote! { #id_lit };
    }

    raw_id_from_state_id_ordered
}

/// Converts a sparse `(block_ident, state_index, state_id)` list into a dense `TokenStream` array.
///
/// # Arguments
/// – `array` – triples of `(block_ident, state_index, state_id)` mapping each state ID to the block constant and local index.
fn fill_state_array(array: Vec<(Ident, usize, u16)>) -> Vec<TokenStream> {
    let max_index = array.iter().map(|(_, _, index)| index).max().unwrap();
    let mut ret = vec![quote! { Missed State }; (max_index + 1) as usize];

    for (block, index, state_id) in array {
        let index_lit = LitInt::new(&index.to_string(), Span::call_site());
        ret[state_id as usize] = quote! { &Self::#block.states[#index_lit] };
    }

    ret
}

/// Converts a block registry name into the SCREAMING_SNAKE_CASE constant identifier.
///
/// # Arguments
/// – `block` – the lowercase registry name (e.g., `"stone_slab"`).
fn const_block_name_from_block_name(block: &str) -> String {
    block.to_shouty_snake_case()
}

/// Derives the UpperCamelCase struct name for a block-property group from a derived base name.
///
/// # Arguments
/// – `name` – a base name such as `"oak_slab_like"` which is converted to e.g., `"OakSlabLikeProperties"`.
fn property_group_name_from_derived_name(name: &str) -> String {
    format!("{name}_properties").to_upper_camel_case()
}

/// Discriminates between the two runtime representations of a block property.
enum PropertyType {
    /// The property is a simple boolean (`true`/`false`).
    Bool,
    /// The property is an enum with a generated Rust type identified by `name`.
    Enum { name: String },
}

/// Maps a single block-property field to its Rust type representation.
struct PropertyVariantMapping {
    /// The serialized property name as it appears in JSON (e.g., `"facing"`).
    original_name: String,
    /// The Rust type used to represent this property in the generated code.
    property_type: PropertyType,
}

/// Accumulated data for a group of blocks that share the same set of block properties.
struct PropertyCollectionData {
    /// The ordered list of property-to-type mappings that form the generated struct fields.
    variant_mappings: Vec<PropertyVariantMapping>,
    /// All blocks (by name and numeric ID) that belong to this property group.
    blocks: Vec<(String, u16)>,
}

impl PropertyCollectionData {
    /// Registers an additional block that shares this property group.
    ///
    /// # Arguments
    /// – `block_name` – the registry name of the block.
    /// – `block_id` – the numeric block ID.
    pub fn add_block(&mut self, block_name: String, block_id: u16) {
        self.blocks.push((block_name, block_id));
    }

    /// Creates a new `PropertyCollectionData` from an ordered list of property mappings with no blocks yet registered.
    ///
    /// # Arguments
    /// – `variant_mappings` – the property-to-type mappings for this group.
    pub const fn from_mappings(variant_mappings: Vec<PropertyVariantMapping>) -> Self {
        Self {
            variant_mappings,
            blocks: Vec::new(),
        }
    }

    /// Derives the base name for this property group from the first registered block.
    pub fn derive_name(&self) -> String {
        format!("{}_like", self.blocks[0].0)
    }
}

/// Deserialized representation of a single block property enum, used to emit the Rust enum definition.
#[derive(Deserialize, Clone, Debug)]
pub struct PropertyStruct {
    /// The UpperCamelCase name used for the generated Rust enum (e.g., `"Facing"`).
    pub name: String,
    /// The ordered list of variant value strings (e.g., `["north", "south", ...]`).
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

/// Code-generation wrapper that emits the full `BlockProperties` impl for one property group.
struct BlockPropertyStruct {
    /// The property group data used to generate the struct and its trait implementation.
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
                #[allow(clippy::manual_range_patterns)]
                fn handles_block_id(block_id: u16) -> bool where Self: Sized {
                    matches!(block_id, #(#block_ids)|*)
                }

                fn to_state_id(&self, block: &Block) -> u16 {
                    if !Self::handles_block_id(block.id) {
                        panic!("{} is not a valid block for {}", &block.name, #struct_name);
                    }
                    block.states[0].id + self.to_index()
                }

                fn from_state_id(state_id: u16, block: &Block) -> Self {
                    debug_assert!(
                        Self::handles_block_id(block.id),
                        "{} is not a valid block for {}", &block.name, #struct_name
                    );

                    let min_id = block.states[0].id;
                    let max_id = block.states.last().map(|s| s.id).unwrap_or(min_id);

                    if (min_id..=max_id).contains(&state_id) {
                        Self::from_index(state_id - min_id)
                    } else {
                        #[cfg(debug_assertions)]
                        panic!("State ID {} does not exist for {}", state_id, &block.name);

                        #[cfg(not(debug_assertions))]
                        Self::from_index(0)
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

                #[allow(clippy::manual_range_patterns)]
                fn from_props(props: &[(&str, &str)], block: &Block) -> Self {
                    #[cfg(debug_assertions)]
                    if !matches!(block.id, #(#block_ids)|*) {
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

/// Deserialized flammability data for a block.
#[derive(Deserialize)]
pub struct FlammableStruct {
    /// Chance (0–300) that fire spreads to neighbouring blocks.
    pub spread_chance: u8,
    /// Chance (0–300) that the block itself burns away.
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

/// Axis-aligned bounding box used to describe block collision and outline shapes.
#[derive(Deserialize, Clone, Copy)]
pub struct BoundingBox {
    /// The minimum corner of the bounding box.
    pub min: Vector3<f64>,
    /// The maximum corner of the bounding box.
    pub max: Vector3<f64>,
}

impl ToTokens for BoundingBox {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_x = &self.min.x;
        let min_y = &self.min.y;
        let min_z = &self.min.z;

        let max_x = &self.max.x;
        let max_y = &self.max.y;
        let max_z = &self.max.z;

        tokens.extend(quote! {
            BoundingBox {
                min: Vector3::new(#min_x, #min_y, #min_z),
                max: Vector3::new(#max_x, #max_y, #max_z),
            }
        });
    }
}

/// Deserialized representation of a single block state as stored in `blocks.json`.
#[derive(Deserialize, Clone)]
pub struct BlockState {
    /// Globally unique numeric ID for this block state.
    pub id: u16,
    /// Bitfield encoding boolean state properties (air, random ticks, etc.).
    pub state_flags: u16,
    /// Bitfield encoding which sides of the block are solid.
    pub side_flags: u8,
    /// Name of the note-block instrument for this state. TODO: make this an enum
    pub instrument: String,
    /// Light level emitted by this state (0–15).
    pub luminance: u8,
    /// How pistons interact with this block state.
    pub piston_behavior: PistonBehavior,
    /// Mining hardness of this block state.
    pub hardness: f32,
    /// Indices into the global shapes array for collision shape segments.
    pub collision_shapes: Vec<u16>,
    /// Indices into the global shapes array for outline (selection) shape segments.
    pub outline_shapes: Vec<u16>,
    /// Opacity value for light propagation, if non-full.
    pub opacity: Option<u8>,
    /// Associated block-entity type ID, if any.
    pub block_entity_type: Option<u16>,
}

/// Describes how a piston interacts with a block.
#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PistonBehavior {
    /// The block can be pushed and pulled normally.
    Normal,
    /// The block is destroyed when pushed.
    Destroy,
    /// The block prevents piston movement.
    Block,
    /// The piston ignores the block entirely.
    Ignore,
    /// The block can only be pushed, not pulled.
    PushOnly,
}

impl PistonBehavior {
    fn to_tokens(&self) -> TokenStream {
        match self {
            Self::Normal => quote! { PistonBehavior::Normal },
            Self::Destroy => quote! { PistonBehavior::Destroy },
            Self::Block => quote! { PistonBehavior::Block },
            Self::Ignore => quote! { PistonBehavior::Ignore },
            Self::PushOnly => quote! { PistonBehavior::PushOnly },
        }
    }
}

impl BlockState {
    /// Bit flag indicating this state is an air block.
    const IS_AIR: u16 = 1 << 0;
    /// Bit flag indicating this state receives random tick events.
    const HAS_RANDOM_TICKS: u16 = 1 << 9;

    /// Returns `true` if this state receives random tick events.
    const fn has_random_ticks(&self) -> bool {
        self.state_flags & Self::HAS_RANDOM_TICKS != 0
    }

    /// Returns `true` if this state is an air variant.
    pub const fn is_air(&self) -> bool {
        self.state_flags & Self::IS_AIR != 0
    }

    /// Emits the `BlockState { … }` strut literal token stream for code generation.
    fn to_tokens(&self) -> TokenStream {
        let mut tokens = TokenStream::new();
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let state_flags = LitInt::new(&self.state_flags.to_string(), Span::call_site());
        let side_flags = LitInt::new(&self.side_flags.to_string(), Span::call_site());
        let instrument = format_ident!("{}", self.instrument.to_upper_camel_case());
        let luminance = LitInt::new(&self.luminance.to_string(), Span::call_site());
        let hardness = self.hardness;
        let opacity = if let Some(opacity) = self.opacity {
            let opacity = LitInt::new(&opacity.to_string(), Span::call_site());
            quote! { #opacity }
        } else {
            quote! { 0 }
        };
        let block_entity_type = if let Some(block_entity_type) = self.block_entity_type {
            let block_entity_type = LitInt::new(&block_entity_type.to_string(), Span::call_site());
            quote! { #block_entity_type }
        } else {
            quote! { u16::MAX }
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

/// Deserialized representation of a Minecraft block as stored in `blocks.json`.
#[derive(Deserialize)]
pub struct Block {
    /// Numeric block ID used in the protocol.
    pub id: u16,
    /// Registry name without the `minecraft:` namespace prefix.
    pub name: String,
    /// Translation key for the block's display name.
    pub translation_key: String,
    /// Mining hardness; affects how long the block takes to break.
    pub hardness: f32,
    /// Blast resistance against explosions.
    pub blast_resistance: f32,
    /// Numeric ID of the corresponding item, if any.
    pub item_id: u16,
    /// Flammability data, present only if the block can catch fire.
    pub flammable: Option<FlammableStruct>,
    /// Loot table used when the block is broken, if any.
    pub loot_table: Option<LootTableStruct>,
    /// Friction applied to entities walking on this block.
    pub slipperiness: f32,
    /// Horizontal velocity multiplier for entities inside this block.
    pub velocity_multiplier: f32,
    /// Jump velocity multiplier applied when jumping from this block.
    pub jump_velocity_multiplier: f32,
    /// Hash keys referencing the properties defined for this block.
    pub properties: Vec<i32>,
    /// State ID of the default (canonical) block state.
    pub default_state_id: u16,
    /// All possible states for this block in state-ID order.
    pub states: Vec<BlockState>,
    /// Experience points dropped when the block is mined, if any.
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
        let experience = if let Some(exp) = &self.experience {
            let exp_tokens = exp.to_token_stream();
            quote! { Some(#exp_tokens) }
        } else {
            quote! { None }
        };
        // Generate state tokens
        let states = self.states.iter().map(BlockState::to_tokens);
        let loot_table = if let Some(table) = &self.loot_table {
            let table_tokens = table.to_token_stream();
            quote! { Some(#table_tokens) }
        } else {
            quote! { None }
        };

        let default_state_ref: &BlockState = self
            .states
            .iter()
            .find(|state| state.id == self.default_state_id)
            .unwrap();
        let mut default_state = default_state_ref.clone();
        default_state.id = default_state_ref.id;
        let default_state = default_state.to_tokens();
        let flammable = if let Some(flammable) = &self.flammable {
            let flammable_tokens = flammable.to_token_stream();
            quote! { Some(#flammable_tokens) }
        } else {
            quote! { None }
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

/// The underlying data type of generated block property as classified in `properties.json`.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum GeneratedPropertyType {
    /// A simple `true`/`false` boolean property.
    #[serde(rename = "boolean")]
    Boolean,
    /// An integer property with an inclusive range.
    #[serde(rename = "int")]
    Int {
        /// Minimum value (inclusive).
        min: u8,
        /// Maximum value (inclusive).
        max: u8,
    },
    /// A named-variant enum property.
    #[serde(rename = "enum")]
    Enum {
        /// The ordered list of variant strings.
        values: Vec<String>,
    },
}

/// A single block property entry as deserialized from `properties.json`.
#[derive(Deserialize, Clone)]
pub struct GeneratedProperty {
    /// A stable integer hash used to identify this property across blocks.
    hash_key: i32,
    /// The name used for the generated Rust enum type.
    enum_name: String,
    /// The property name as it appears in block state JSON.
    serialized_name: String,
    /// The kind and possible values of this property.
    #[serde(rename = "type")]
    #[serde(flatten)]
    property_type: GeneratedPropertyType,
}

impl GeneratedProperty {
    /// Converts this deserialized property into the intermediate `Property` used during code generation.
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

/// Intermediate representation of a block property used while building the property group map.
#[derive(Clone)]
struct Property {
    /// The Rust enum type name for this property.
    enum_name: String,
    /// The JSON key name for this property.
    serialized_name: String,
    /// All possible string values this property can take.
    values: Vec<String>,
}

/// Top-level container for all block assets loaded from `blocks.json`.
#[derive(Deserialize)]
pub struct BlockAssets {
    /// All blocks with their states and properties.
    pub blocks: Vec<Block>,
    /// All unique bounding boxes referenced by block states.
    pub shapes: Vec<BoundingBox>,
    /// Registry names of all block entity types.
    pub block_entity_types: Vec<String>,
}

/// Reads all block assets and generates the complete block registry `TokenStream`.
pub fn build() -> TokenStream {
    let be_blocks_data = fs::read("../assets/bedrock_block_states.nbt").unwrap();
    let mut be_blocks_cursor = Cursor::new(be_blocks_data);
    let be_blocks = get_be_data_from_nbt(&mut be_blocks_cursor);

    let blocks_assets: BlockAssets =
        serde_json::from_str(&fs::read_to_string("../assets/blocks.json").unwrap())
            .expect("Failed to parse blocks.json");

    let generated_properties: Vec<GeneratedProperty> =
        serde_json::from_str(&fs::read_to_string("../assets/properties.json").unwrap())
            .expect("Failed to parse properties.json");

    let generated_prop_map: BTreeMap<i32, &GeneratedProperty> = generated_properties
        .iter()
        .map(|p| (p.hash_key, p))
        .collect();

    let mut random_tick_states = Vec::new();
    let mut air_states = Vec::new();

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
    let mut existing_item_ids: HashSet<u16> = HashSet::new();

    for block in blocks_assets.blocks {
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

        let be_state_list = be_blocks.get(be_name);

        for (i, state) in block.states.iter().enumerate() {
            if state.has_random_ticks() {
                let state_id = LitInt::new(&state.id.to_string(), Span::call_site());
                random_tick_states.push(state_id);
            }
            if state.is_air() {
                let state_id = LitInt::new(&state.id.to_string(), Span::call_site());
                air_states.push(state_id);
            }

            let mut matched_be_id = 1;

            if let Some(be_variants) = be_state_list {
                let mut temp_index = i as u16;
                let mut java_props_for_this_state = BTreeMap::new();

                for mapping in property_mapping.iter().rev() {
                    match &mapping.property_type {
                        PropertyType::Bool => {
                            let val = temp_index % 2;
                            temp_index /= 2;
                            java_props_for_this_state.insert(
                                mapping.original_name.clone(),
                                if val == 0 { "true" } else { "false" }.to_string(),
                            );
                        }
                        PropertyType::Enum { name } => {
                            let enum_info = property_enums.get(name).unwrap();
                            let count = enum_info.values.len() as u16;
                            let val_idx = temp_index % count;
                            temp_index /= count;

                            let raw_val = &enum_info.values[val_idx as usize];
                            let val_str = if raw_val.starts_with('L') {
                                raw_val.strip_prefix('L').unwrap().to_string()
                            } else {
                                raw_val.clone()
                            };
                            java_props_for_this_state
                                .insert(mapping.original_name.clone(), val_str);
                        }
                    }
                }

                matched_be_id = be_variants
                    .iter()
                    .find(|(_, be_props)| {
                        java_props_for_this_state
                            .iter()
                            .all(|(k, v)| be_props.get(k).is_some_and(|be_v| be_v == v))
                    })
                    .map_or_else(
                        || be_variants.first().map_or(1, |(id, _)| *id),
                        |(id, _)| *id,
                    );
            }

            block_state_to_bedrock.push((state.id, matched_be_id));
            raw_id_from_state_id_array.push((state.id, id_lit.clone()));
            state_from_state_id_array.push((const_ident.clone(), i, state.id));
        }

        if !property_collection.is_empty() {
            let mut property_collection_vec: Vec<i32> = property_collection.into_iter().collect();
            property_collection_vec.sort_unstable();

            property_collection_map
                .entry(property_collection_vec)
                .or_insert_with(|| PropertyCollectionData::from_mappings(property_mapping))
                .add_block(block.name.clone(), block.id);
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

    let shapes = blocks_assets.shapes.iter().map(ToTokens::to_token_stream);

    let air_state_ids = quote! { #(#air_states)|* };

    let block_props = block_properties.iter().map(ToTokens::to_token_stream);
    let properties = property_enums.values().map(ToTokens::to_token_stream);

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

    let Bitset {
        items,
        mod_ident,
        contains_ident,
    } = &gen_u16_bitset(
        "RANDOM_TICKS",
        &random_tick_states
            .iter()
            .map(|it| it.base10_parse().unwrap())
            .collect::<Vec<u16>>(),
    );

    quote! {
        use pumpkin_util::math::boundingbox::BoundingBox;

        use crate::{BlockState, Block, blocks::Flammable};
        use crate::block_state::PistonBehavior;
        use pumpkin_util::math::int_provider::{UniformIntProvider, IntProvider, NormalIntProvider};
        use pumpkin_util::loot_table::*;
        use pumpkin_util::math::experience::Experience;
        use pumpkin_util::math::vector3::Vector3;
        use std::collections::BTreeMap;
        use phf;

        #items

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

        pub const COLLISION_SHAPES: &[BoundingBox] = &[
            #(#shapes),*
        ];

        pub const BLOCK_ENTITY_TYPES: &[&str] = &[
            #(#block_entity_types),*
        ];

        #[inline(always)]
        pub fn is_air(state_id: u16) -> bool {
            matches!(state_id, #air_state_ids)
        }

        #[inline(always)]
        pub fn has_random_ticks(state_id: u16) -> bool {
            #mod_ident::#contains_ident(state_id)
        }

        pub fn blocks_movement(block_state: &BlockState, block: u16) -> bool {
            if block_state.is_solid() {
                return block != Block::COBWEB && block != Block::BAMBOO_SAPLING;
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
                // In debug, this avoids the slow range-checking logic
                unsafe {
                    Block::STATE_FROM_STATE_ID.get_unchecked(id as usize)
                }
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
           pub fn get_raw_id_from_state_id(state_id: u16) -> u16 {
                let index = state_id as usize;
                if index >= Self::RAW_ID_FROM_STATE_ID.len() {
                    0
                } else {
                    unsafe { *Self::RAW_ID_FROM_STATE_ID.get_unchecked(index) }
                }
            }

            #[doc = r" Get a block from a state id."]
            #[inline]
            pub fn from_state_id(id: u16) -> &'static Self {
                let index = id as usize;
                if index >= Self::RAW_ID_FROM_STATE_ID.len() {
                    return &Self::AIR;
                }
                let raw_id = unsafe { *Self::RAW_ID_FROM_STATE_ID.get_unchecked(index) };
                Self::from_id(raw_id)
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

/// Parses the Bedrock Edition block palette NBT file into a map of block names to their state variants.
///
/// # Arguments
/// – `reader` – a readable byte source positioned at the start of the NBT data.
#[expect(clippy::type_complexity)]
fn get_be_data_from_nbt<R: Read>(
    reader: &mut R,
) -> BTreeMap<String, Vec<(u32, BTreeMap<String, String>)>> {
    let mut block_data: BTreeMap<String, Vec<(u32, BTreeMap<String, String>)>> = BTreeMap::new();
    let mut current_id = 0;

    let read_nbt_string = |reader: &mut R| -> String {
        let len = read_varint(reader);
        let mut buf = vec![0; len as usize];
        reader.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    };

    let read_byte_safe = |reader: &mut R| -> Option<u8> {
        let mut buf = [0; 1];
        reader.read_exact(&mut buf).is_ok().then(|| buf[0])
    };

    while let Some(tag_id) = read_byte_safe(reader) {
        if tag_id != 10 {
            break;
        } // Tag_Compound (10) required

        // Read Root Name (usually empty string in palette)
        let _root_name = read_nbt_string(reader);

        let mut block_name = String::new();
        let mut properties = BTreeMap::new();

        loop {
            let field_type = read_byte(reader);
            if field_type == 0 {
                break;
            } // Tag_End (0)

            let field_name = read_nbt_string(reader);

            match field_name.as_str() {
                "name" => {
                    let raw_name = read_nbt_string(reader);
                    block_name = raw_name
                        .strip_prefix("minecraft:")
                        .unwrap_or(&raw_name)
                        .to_string();
                }
                "states" => loop {
                    let prop_type = read_byte(reader);
                    if prop_type == 0 {
                        break;
                    }

                    let prop_key = read_nbt_string(reader);

                    let prop_val = match prop_type {
                        1 => {
                            let val = read_byte(reader);
                            if val == 1 {
                                "true".to_string()
                            } else {
                                "false".to_string()
                            }
                        }
                        3 => read_varint(reader).to_string(),
                        8 => read_nbt_string(reader),
                        _ => panic!("Unknown property type {prop_type} for key {prop_key}"),
                    };

                    properties.insert(prop_key, prop_val);
                },
                "version" => {
                    read_varint(reader);
                }
                _ => panic!("Unexpected root field: {field_name}"),
            }
        }

        if !block_name.is_empty() {
            block_data
                .entry(block_name)
                .or_default()
                .push((current_id, properties));
        }

        current_id += 1;
    }

    block_data
}

/// Reads a variable-length encoded 32-bit integer from the reader.
///
/// # Arguments
/// – `reader` – the byte source to read from.
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

/// Reads a single byte from the reader, returning `0` on failure.
///
/// # Arguments
/// – `reader` – the byte source to read from.
fn read_byte<W: Read>(reader: &mut W) -> u8 {
    let byte = &mut [0];
    reader.read_exact(byte).unwrap_or_default();
    byte[0]
}
