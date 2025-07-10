use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
};
use syn::{Ident, LitInt, LitStr};

fn const_fluid_name_from_fluid_name(fluid: &str) -> String {
    fluid.to_shouty_snake_case()
}

fn property_group_name_from_derived_name(name: &str) -> String {
    format!("{name}_fluid_properties").to_upper_camel_case()
}

struct PropertyVariantMapping {
    original_name: String,
    property_enum: String,
}

struct PropertyCollectionData {
    variant_mappings: Vec<PropertyVariantMapping>,
    fluid_names: Vec<String>,
}

impl PropertyCollectionData {
    pub fn add_fluid_name(&mut self, fluid_name: String) {
        self.fluid_names.push(fluid_name);
    }

    pub fn from_mappings(variant_mappings: Vec<PropertyVariantMapping>) -> Self {
        Self {
            variant_mappings,
            fluid_names: Vec::new(),
        }
    }

    pub fn derive_name(&self) -> String {
        format!("{}_like", self.fluid_names[0])
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct PropertyStruct {
    pub name: String,
    pub values: Vec<String>,
}

impl ToTokens for PropertyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(&self.name, Span::call_site());

        let variant_count = self.values.clone().len() as u16;
        let values_index = (0..self.values.clone().len() as u16).collect::<Vec<_>>();

        let ident_values = self.values.iter().map(|value| {
            let value_str = if value.chars().all(|c| c.is_numeric()) {
                format!("L{value}")
            } else {
                value.clone()
            };
            Ident::new(&value_str.to_upper_camel_case(), Span::call_site())
        });

        let values_2 = ident_values.clone();
        let values_3 = ident_values.clone();

        let from_values = self.values.iter().map(|value| {
            let value_str = if value.chars().all(|c| c.is_numeric()) {
                format!("L{value}")
            } else {
                value.clone()
            };
            let ident = Ident::new(&value_str.to_upper_camel_case(), Span::call_site());
            quote! {
                #value => Self::#ident
            }
        });
        let to_values = self.values.iter().map(|value| {
            let value_str = if value.chars().all(|c| c.is_numeric()) {
                format!("L{value}")
            } else {
                value.clone()
            };
            let ident = Ident::new(&value_str.to_upper_camel_case(), Span::call_site());
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

struct FluidPropertyStruct {
    data: PropertyCollectionData,
}

impl ToTokens for FluidPropertyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let struct_name = property_group_name_from_derived_name(&self.data.derive_name());
        let name = Ident::new(&struct_name, Span::call_site());

        let values = self.data.variant_mappings.iter().map(|entry| {
            let key = Ident::new_raw(&entry.original_name, Span::call_site());
            let value = Ident::new(&entry.property_enum, Span::call_site());

            quote! {
                #key: #value
            }
        });

        let fluid_names = &self.data.fluid_names;

        let field_names: Vec<_> = self
            .data
            .variant_mappings
            .iter()
            .rev()
            .map(|entry| Ident::new_raw(&entry.original_name, Span::call_site()))
            .collect();

        let field_types: Vec<_> = self
            .data
            .variant_mappings
            .iter()
            .rev()
            .map(|entry| Ident::new(&entry.property_enum, Span::call_site()))
            .collect();

        let to_props_values = self.data.variant_mappings.iter().map(|entry| {
            let key = &entry.original_name;
            let key2 = Ident::new_raw(&entry.original_name, Span::call_site());

            quote! {
                (#key.to_string(), self.#key2.to_value().to_string()),
            }
        });

        let from_props_values = self.data.variant_mappings.iter().map(|entry| {
            let key = &entry.original_name;
            let key2 = Ident::new_raw(&entry.original_name, Span::call_site());
            let value = Ident::new(&entry.property_enum, Span::call_site());

            quote! {
                #key => fluid_props.#key2 = #value::from_value(&value)
            }
        });

        tokens.extend(quote! {
            #[derive(Clone, Copy, Debug, Eq, PartialEq)]
            pub struct #name {
                #(pub #values),*
            }

            impl FluidProperties for #name {
                #[allow(unused_assignments)]
                fn to_index(&self) -> u16 {
                    let mut index = 0;
                    let mut multiplier = 1;

                    #(
                        index += self.#field_names.to_index() * multiplier;
                        multiplier *= #field_types::variant_count();
                    )*

                    index
                }

                #[allow(unused_assignments)]
                fn from_index(mut index: u16) -> Self {
                    Self {
                        #(
                            #field_names: {
                                let value = index % #field_types::variant_count();
                                index /= #field_types::variant_count();
                                #field_types::from_index(value)
                            }
                        ),*
                    }
                }

                fn to_state_id(&self, fluid: &Fluid) -> u16 {
                    if ![#(#fluid_names),*].contains(&fluid.name) {
                        panic!("{} is not a valid fluid for {}", &fluid.name, #struct_name);
                    }

                    let prop_index = self.to_index();
                    if prop_index < fluid.states.len() as u16 {
                        fluid.states[prop_index as usize].block_state_id
                    } else {
                        fluid.states[fluid.default_state_index as usize].block_state_id
                    }
                }

                fn from_state_id(state_id: u16, fluid: &Fluid) -> Self {
                    if ![#(#fluid_names),*].contains(&fluid.name) {
                        panic!("{} is not a valid fluid for {}", &fluid.name, #struct_name);
                    }

                    for (idx, state) in fluid.states.iter().enumerate() {
                        if state.block_state_id == state_id {
                            return Self::from_index(idx as u16);
                        }
                    }

                    Self::from_index(fluid.default_state_index)
                }

                fn default(fluid: &Fluid) -> Self {
                    if ![#(#fluid_names),*].contains(&fluid.name) {
                        panic!("{} is not a valid fluid for {}", &fluid.name, #struct_name);
                    }

                    Self::from_state_id(fluid.default_state_index, fluid)
                }

                fn to_props(&self) -> Vec<(String, String)> {
                   vec![#(#to_props_values)*]
                }

                fn from_props(props: Vec<(String, String)>, fluid: &Fluid) -> Self {
                    if ![#(#fluid_names),*].contains(&fluid.name) {
                        panic!("{} is not a valid fluid for {}", &fluid.name, #struct_name);
                    }

                    let mut fluid_props = Self::default(fluid);

                    for (key, value) in props {
                        match key.as_str() {
                            #(#from_props_values),*,
                            _ => panic!("Invalid key: {key}"),
                        }
                    }

                    fluid_props
                }
            }
        });
    }
}

#[derive(Deserialize, Clone)]
struct FluidState {
    height: f32,
    level: i16,
    is_empty: bool,
    blast_resistance: f32,
    block_state_id: u16,
    is_still: bool,
    // We'll derive is_source and falling from existing fields instead of requiring them in JSON
}

#[derive(Clone, Debug)]
struct FluidStateRef {
    pub id: u16,
    pub state_idx: u16,
}
impl ToTokens for FluidStateRef {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let state_idx = LitInt::new(&self.state_idx.to_string(), Span::call_site());
        tokens.extend(quote! {
            FluidStateRef {
                id: #id,
                state_idx: #state_idx,
            }
        });
    }
}

#[derive(Deserialize, Clone)]
struct Property {
    name: String,
    values: Vec<String>,
}

#[derive(Deserialize, Clone)]
struct Fluid {
    name: String,
    id: u16,
    properties: Vec<Property>,
    default_state_index: u16,
    states: Vec<FluidState>,
    #[serde(default = "default_flow_speed")]
    flow_speed: u32,
    #[serde(default = "default_flow_distance")]
    flow_distance: u32,
    #[serde(default)]
    can_convert_to_source: bool,
}

fn default_flow_speed() -> u32 {
    5 // Default to water's speed
}

fn default_flow_distance() -> u32 {
    4 // Default to water's distance
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/fluids.json");

    let fluids: Vec<Fluid> =
        match serde_json::from_str(&fs::read_to_string("../assets/fluids.json").unwrap()) {
            Ok(fluids) => fluids,
            Err(e) => panic!("Failed to parse fluids.json: {e}"),
        };

    let mut constants = TokenStream::new();
    let mut id_matches = Vec::new();
    let mut type_from_name = TokenStream::new();
    let mut type_from_raw_id_arms = TokenStream::new();
    let mut fluid_from_state_id = TokenStream::new();

    let mut fluid_properties_from_state_and_name = TokenStream::new();
    let mut fluid_properties_from_props_and_name = TokenStream::new();

    // Used to create property `enum`s.
    let mut property_enums: HashMap<String, PropertyStruct> = HashMap::new();
    // Property implementation for a fluid.
    let mut fluid_properties: Vec<FluidPropertyStruct> = Vec::new();
    // Mapping of a collection of property names -> fluids that have these properties.
    let mut property_collection_map: HashMap<Vec<String>, PropertyCollectionData> = HashMap::new();
    // Validator that we have no `enum` collisions.
    let mut enum_to_values: HashMap<String, Vec<String>> = HashMap::new();

    // Collect unique fluid states to create partial states
    let mut unique_states = Vec::new();
    let mut optimized_fluids: Vec<(String, FluidStateRef)> = Vec::new();

    for fluid in fluids.clone() {
        let id_name = LitStr::new(&fluid.name, proc_macro2::Span::call_site());
        let const_ident = format_ident!("{}", fluid.name.to_shouty_snake_case());
        let state_id_start = fluid
            .states
            .iter()
            .map(|state| state.block_state_id)
            .min()
            .unwrap();
        let state_id_end = fluid
            .states
            .iter()
            .map(|state| state.block_state_id)
            .max()
            .unwrap();

        let id_lit = LitInt::new(&fluid.id.to_string(), proc_macro2::Span::call_site());
        let mut properties = TokenStream::new();
        if fluid.properties.is_empty() {
            properties.extend(quote!(None));
        } else {
            let internal_properties = fluid.properties.iter().map(|property| {
                let key = LitStr::new(&property.name, proc_macro2::Span::call_site());
                let values = property
                    .values
                    .iter()
                    .map(|value| LitStr::new(value, proc_macro2::Span::call_site()));

                quote! {
                    (#key, &[
                        #(#values),*
                    ])
                }
            });
            properties.extend(quote! {
                Some(&[
                    #(#internal_properties),*
                ])
            });
        }

        for (idx, state) in fluid.states.iter().enumerate() {
            // Check if this state is already in `unique_states` by comparing key fields
            let already_exists = unique_states.iter().any(|s: &FluidState| {
                s.height == state.height
                    && s.level == state.level
                    && s.is_empty == state.is_empty
                    && s.blast_resistance == state.blast_resistance
                    && s.is_still == state.is_still
            });
            if !already_exists {
                unique_states.push(state.clone());
            }
            // Create a reference to the state
            let state_idx = unique_states
                .iter()
                .position(|s| {
                    s.height == state.height
                        && s.level == state.level
                        && s.is_empty == state.is_empty
                        && s.blast_resistance == state.blast_resistance
                        && s.is_still == state.is_still
                })
                .unwrap() as u16;
            optimized_fluids.push((
                fluid.name.clone(),
                FluidStateRef {
                    id: idx as u16,
                    state_idx,
                },
            ));
        }
        fluid_from_state_id.extend(quote! {
            #state_id_start..=#state_id_end => Some(&Fluid::#const_ident),
        });

        type_from_name.extend(quote! {
            #id_name => Some(&Self::#const_ident),
        });

        type_from_raw_id_arms.extend(quote! {
            #id_lit => Some(&Self::#const_ident),
        });

        let fluid_states = fluid.states.iter().map(|state| {
            let height = state.height;
            let level = state.level;
            let is_empty = state.is_empty;
            let blast_resistance = state.blast_resistance;
            let block_state_id = state.block_state_id;
            let is_still = state.is_still;
            // Derive these values based on existing fields
            let is_source = level == 0 && is_still; // Level 0 and still means it's a source
            let falling = false; // Default to false - we'll handle falling in the fluid behavior code

            quote! {
                FluidState {
                    height: #height,
                    level: #level,
                    is_empty: #is_empty,
                    blast_resistance: #blast_resistance,
                    block_state_id: #block_state_id,
                    is_still: #is_still,
                    is_source: #is_source,
                    falling: #falling,
                }
            }
        });
        let state_id = fluid.default_state_index;
        let flow_speed = fluid.flow_speed;
        let flow_distance = fluid.flow_distance;
        let can_convert_to_source = fluid.can_convert_to_source;

        id_matches.push(quote! {
            #id_name => Some(#id_lit),
        });

        constants.extend(quote! {
            pub const #const_ident: Fluid = Fluid {
                id: #id_lit,
                name: #id_name,
                properties: #properties,
                states: &[#(#fluid_states),*],
                default_state_index: #state_id,
                flow_speed: #flow_speed,
                flow_distance: #flow_distance,
                can_convert_to_source: #can_convert_to_source,
            };
        });

        let mut property_collection = HashSet::new();
        let mut property_mapping = Vec::new();
        for property in &fluid.properties {
            property_collection.insert(property.name.clone());

            // Get mapped property `enum` name
            let renamed_property = property.name.to_upper_camel_case();

            let expected_values = enum_to_values
                .entry(renamed_property.clone())
                .or_insert_with(|| property.values.clone());

            if expected_values != &property.values {
                panic!(
                    "Enum overlap for '{}' ({:?} vs {:?})",
                    property.name, &property.values, expected_values
                );
            };

            property_mapping.push(PropertyVariantMapping {
                original_name: property.name.clone(),
                property_enum: renamed_property.clone(),
            });

            // If this property doesn't have an `enum` yet, make one.
            let _ = property_enums
                .entry(renamed_property.clone())
                .or_insert_with(|| PropertyStruct {
                    name: renamed_property,
                    values: property.values.clone(),
                });
        }

        if !property_collection.is_empty() {
            let mut property_collection = Vec::from_iter(property_collection);
            property_collection.sort();
            property_collection_map
                .entry(property_collection)
                .or_insert_with(|| PropertyCollectionData::from_mappings(property_mapping))
                .add_fluid_name(fluid.name.clone());
        }
    }

    let unique_fluid_states = unique_states.iter().map(|state| {
        let height = state.height;
        let level = state.level;
        let is_empty = state.is_empty;
        let blast_resistance = state.blast_resistance;
        let block_state_id = state.block_state_id;
        let is_still = state.is_still;
        let is_source = level == 0 && is_still;
        let falling = false;
        quote! {
            PartialFluidState {
                height: #height,
                level: #level,
                is_empty: #is_empty,
                blast_resistance: #blast_resistance,
                block_state_id: #block_state_id,
                is_still: #is_still,
                is_source: #is_source,
                falling: #falling,
            }
        }
    });

    for property_group in property_collection_map.into_values() {
        for fluid_name in &property_group.fluid_names {
            let const_fluid_name = Ident::new(
                &const_fluid_name_from_fluid_name(fluid_name),
                Span::call_site(),
            );
            let property_name = Ident::new(
                &property_group_name_from_derived_name(&property_group.derive_name()),
                Span::call_site(),
            );

            fluid_properties_from_state_and_name.extend(quote! {
                #fluid_name => Some(Box::new(#property_name::from_state_id(state_id, &Fluid::#const_fluid_name))),
            });

            fluid_properties_from_props_and_name.extend(quote! {
                #fluid_name => Some(Box::new(#property_name::from_props(props, &Fluid::#const_fluid_name))),
            });
        }

        fluid_properties.push(FluidPropertyStruct {
            data: property_group,
        });
    }

    let fluid_props = fluid_properties.iter().map(|prop| prop.to_token_stream());
    let properties = property_enums.values().map(|prop| prop.to_token_stream());

    quote! {
        use std::hash::{Hash, Hasher};
        use crate::tag::{Tagable, RegistryKey};
        use pumpkin_util::resource_location::{FromResourceLocation, ResourceLocation, ToResourceLocation};

        #[derive(Clone, Debug)]
        pub struct PartialFluidState {
            pub height: f32,
            pub level: i16,
            pub is_empty: bool,
            pub blast_resistance: f32,
            pub block_state_id: u16,
            pub is_still: bool,
            pub is_source: bool,
            pub falling: bool,
        }

        #[derive(Clone, Debug)]
        pub struct FluidState {
            pub height: f32,
            pub level: i16,
            pub is_empty: bool,
            pub blast_resistance: f32,
            pub block_state_id: u16,
            pub is_still: bool,
            pub is_source: bool,
            pub falling: bool,
        }

        #[derive(Clone, Debug)]
        pub struct FluidStateRef {
            pub id: u16,
            pub state_idx: u16,
        }

        #[derive(Clone, Debug)]
        pub struct Fluid {
            pub id: u16,
            pub name: &'static str,
            pub properties: Option<&'static [(&'static str, &'static [&'static str])]>,
            pub states: &'static [FluidState],
            pub default_state_index: u16,
            pub flow_speed: u32,
            pub flow_distance: u32,
            pub can_convert_to_source: bool,
        }

        impl Hash for Fluid {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.id.hash(state);
            }
        }

        impl PartialEq for Fluid {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        impl Eq for Fluid {}

        pub static FLUID_STATES: &[PartialFluidState] = &[
            #(#unique_fluid_states),*
        ];

        pub trait EnumVariants {
            fn variant_count() -> u16;
            fn to_index(&self) -> u16;
            fn from_index(index: u16) -> Self;
            fn to_value(&self) -> &str;
            fn from_value(value: &str) -> Self;
        }

        pub trait FluidProperties where Self: 'static {
            // Convert properties to an index (`0` to `N-1`).
            fn to_index(&self) -> u16;
            // Convert an index back to properties.
            fn from_index(index: u16) -> Self where Self: Sized;

            // Convert properties to a state id.
            fn to_state_id(&self, fluid: &Fluid) -> u16;
            // Convert a state id back to properties.
            fn from_state_id(state_id: u16, fluid: &Fluid) -> Self where Self: Sized;
            // Get the default properties.
            fn default(fluid: &Fluid) -> Self where Self: Sized;

            // Convert properties to a `Vec` of `(name, value)`
            fn to_props(&self) -> Vec<(String, String)>;

            // Convert properties to a fluid state, and add them onto the default state.
            fn from_props(props: Vec<(String, String)>, fluid: &Fluid) -> Self where Self: Sized;
        }

        pub fn get_fluid(registry_id: &str) -> Option<&'static Fluid> {
           let key = registry_id.strip_prefix("minecraft:").unwrap_or(registry_id);
           Fluid::from_registry_key(key)
        }

        impl Fluid {
            #constants

            pub fn from_registry_key(name: &str) -> Option<&'static Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }

            pub const fn from_id(id: u16) -> Option<&'static Self> {
                match id {
                    #type_from_raw_id_arms
                    _ => None
                }
            }
            #[allow(unreachable_patterns)]
            pub const fn from_state_id(id: u16) -> Option<&'static Self> {
                match id {
                    #fluid_from_state_id
                    _ => None
                }
            }


            pub fn ident_to_fluid_id(name: &str) -> Option<u8> {
                match name {
                    #(#id_matches)*
                    _ => None
                }
            }

            #[doc = r" Get the properties of the fluid."]
            pub fn properties(&self, state_id: u16) -> Option<Box<dyn FluidProperties>> {
                match self.name {
                    #fluid_properties_from_state_and_name
                    _ => None
                }
            }

            #[doc = r" Get the properties of the fluid."]
            pub fn from_properties(&self, props: Vec<(String, String)>) -> Option<Box<dyn FluidProperties>> {
                match self.name {
                    #fluid_properties_from_props_and_name
                    _ => None
                }
            }

            // Added helper methods for fluid behavior
            pub fn is_source(&self, state_id: u16) -> bool {
                let idx = (state_id as usize) % self.states.len();
                self.states[idx].is_source
            }

            pub fn is_falling(&self, state_id: u16) -> bool {
                let idx = (state_id as usize) % self.states.len();
                self.states[idx].falling
            }

            pub fn get_level(&self, state_id: u16) -> i16 {
                let idx = (state_id as usize) % self.states.len();
                self.states[idx].level
            }

            pub fn get_height(&self, state_id: u16) -> f32 {
                let idx = (state_id as usize) % self.states.len();
                self.states[idx].height
            }
        }

        impl ToResourceLocation for &'static Fluid {
            fn to_resource_location(&self) -> ResourceLocation {
                ResourceLocation::vanilla(self.name)
            }
        }

        impl FromResourceLocation for &'static Fluid {
            fn from_resource_location(resource_location: &ResourceLocation) -> Option<Self> {
                Fluid::from_registry_key(&resource_location.path)
            }
        }

        impl FluidStateRef {
            pub fn get_state(&self) -> FluidState {
                let partial_state = &FLUID_STATES[self.state_idx as usize];
                FluidState {
                    height: partial_state.height,
                    level: partial_state.level,
                    is_empty: partial_state.is_empty,
                    blast_resistance: partial_state.blast_resistance,
                    block_state_id: partial_state.block_state_id,
                    is_still: partial_state.is_still,
                    is_source: partial_state.is_source,
                    falling: partial_state.falling,
                }
            }
        }

        impl Tagable for Fluid {
            #[inline]
            fn tag_key() -> RegistryKey {
                RegistryKey::Fluid
            }

            #[inline]
            fn registry_key(&self) -> &str {
                self.name
            }
        }

        // Added FluidLevel enum and required constants
        pub const FLUID_LEVEL_SOURCE: i32 = 0;
        pub const FLUID_LEVEL_FLOWING_MAX: i32 = 8;
        pub const FLUID_MIN_HEIGHT: f32 = 0.0;
        pub const FLUID_MAX_HEIGHT: f32 = 1.0;

        #(#properties)*

        #(#fluid_props)*
    }
}
