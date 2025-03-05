use heck::{ToShoutySnakeCase, ToUpperCamelCase};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{Ident, LitBool, LitFloat, LitInt, LitStr};

include!("../src/tag.rs");

#[derive(Deserialize, Clone, Debug)]
pub struct BlockProperty {
    pub name: String,
    pub values: Vec<String>,
}

fn get_enum_name(props: Vec<String>) -> String {
    // Define the mapping of variant sets to new enum names
    let enum_mappings: Vec<(Vec<&str>, &str)> = vec![
        (vec!["true", "false"], "Boolean"),
        (vec!["x", "y", "z"], "Axis"),
        (vec!["0", "1"], "Level0to1"),
        (vec!["0", "1", "2", "3", "4"], "Level0to4"),
        (
            vec![
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15",
            ],
            "Level0to15",
        ),
        (vec!["0", "1", "2", "3"], "Level0to3"),
        (vec!["1", "2", "3", "4", "5", "6", "7"], "Level1to7"),
        (
            vec!["north", "east", "south", "west", "up", "down"],
            "Direction",
        ),
        (
            vec![
                "harp",
                "basedrum",
                "snare",
                "hat",
                "bass",
                "flute",
                "bell",
                "guitar",
                "chime",
                "xylophone",
                "iron_xylophone",
                "cow_bell",
                "didgeridoo",
                "bit",
                "banjo",
                "pling",
                "zombie",
                "skeleton",
                "creeper",
                "dragon",
                "wither_skeleton",
                "piglin",
                "custom_head",
            ],
            "Instrument",
        ),
        (
            vec![
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15", "16", "17", "18", "19", "20", "21", "22", "23", "24",
            ],
            "Level0to24",
        ),
        (vec!["north", "south", "west", "east"], "CardinalDirection"),
        (vec!["head", "foot"], "BedPart"),
        (
            vec![
                "north_south",
                "east_west",
                "ascending_east",
                "ascending_west",
                "ascending_north",
                "ascending_south",
            ],
            "RailShape",
        ),
        (vec!["upper", "lower"], "VerticalHalf"),
        (vec!["normal", "sticky"], "PistonType"),
        (vec!["top", "bottom"], "VerticalBlockHalf"),
        (
            vec![
                "straight",
                "inner_left",
                "inner_right",
                "outer_left",
                "outer_right",
            ],
            "StairShape",
        ),
        (vec!["single", "left", "right"], "ChestType"),
        (vec!["up", "side", "none"], "RedstoneConnection"),
        (vec!["0", "1", "2", "3", "4", "5", "6", "7"], "Level0to7"),
        (vec!["left", "right"], "DoorHinge"),
        (
            vec![
                "north_south",
                "east_west",
                "ascending_east",
                "ascending_west",
                "ascending_north",
                "ascending_south",
                "south_east",
                "south_west",
                "north_west",
                "north_east",
            ],
            "ExtendedRailShape",
        ),
        (vec!["floor", "wall", "ceiling"], "AttachmentFace"),
        (vec!["1", "2", "3", "4", "5", "6", "7", "8"], "Level1to8"),
        (vec!["x", "z"], "XYAxis"),
        (vec!["0", "1", "2", "3", "4", "5", "6"], "Level0to6"),
        (vec!["1", "2", "3", "4"], "Level1to4"),
        (vec!["top", "bottom", "double"], "SlabType"),
        (vec!["none", "low", "tall"], "WallHeight"),
        (vec!["1", "2", "3"], "Level1to3"),
        (vec!["0", "1", "2"], "Level0to2"),
        (vec!["compare", "subtract"], "ComparatorMode"),
        (
            vec!["down", "north", "south", "west", "east"],
            "DirectionNoUp",
        ),
        (vec!["0", "1", "2", "3", "4", "5"], "Level0to5"),
        (
            vec![
                "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
                "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25",
            ],
            "Level0to25",
        ),
        (vec!["none", "small", "large"], "LeafSize"),
        (
            vec!["floor", "ceiling", "single_wall", "double_wall"],
            "BellAttachment",
        ),
        (vec!["save", "load", "corner", "data"], "StructureBlockMode"),
        (
            vec![
                "down_east",
                "down_north",
                "down_south",
                "down_west",
                "up_east",
                "up_north",
                "up_south",
                "up_west",
                "west_up",
                "east_up",
                "north_up",
                "south_up",
            ],
            "Orientation",
        ),
        (
            vec!["0", "1", "2", "3", "4", "5", "6", "7", "8"],
            "Level0to8",
        ),
        (vec!["inactive", "active", "cooldown"], "SculkSensorPhase"),
        (
            vec!["tip_merge", "tip", "frustum", "middle", "base"],
            "DripstoneThickness",
        ),
        (vec!["up", "down"], "VerticalDirection"),
        (vec!["none", "unstable", "partial", "full"], "TiltState"),
        (
            vec![
                "inactive",
                "waiting_for_players",
                "active",
                "waiting_for_reward_ejection",
                "ejecting_reward",
                "cooldown",
            ],
            "TrialSpawnerState",
        ),
        (
            vec!["inactive", "active", "unlocking", "ejecting"],
            "VaultState",
        ),
    ];

    // Convert props to a Vec<&str> for comparison
    let props_set: Vec<&str> = props.iter().map(|s| s.as_str()).collect();

    // Find the matching enum by checking if the props match the variant set
    for (variants, enum_name) in enum_mappings {
        if props_set.len() == variants.len() && props_set.iter().all(|p| variants.contains(p)) {
            return enum_name.to_string();
        }
    }

    panic!("UnknownEnum {:?}", props);
}

#[derive(Deserialize, Clone, Debug)]
pub struct PropertyStruct {
    pub name: String,
    pub values: Vec<String>,
}

impl ToTokens for PropertyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(&self.name, Span::call_site());

        let mut prefix = "";

        if name.to_string().contains("Level") {
            prefix = "L";
        }

        let variant_count = self.values.clone().len() as u16;
        let values_index = (0..self.values.clone().len() as u16).collect::<Vec<_>>();

        let values = self.values.iter().map(|value| {
            Ident::new(
                &(prefix.to_owned() + value).to_upper_camel_case(),
                Span::call_site(),
            )
        });

        let values_2 = values.clone();
        let values_3 = values.clone();

        tokens.extend(quote! {
            #[derive(Clone, Copy)]
            pub enum #name {
                #(#values),*
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
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockPropertyStruct {
    pub block_name: String,
    pub entires: Vec<(String, String)>,
}

impl ToTokens for BlockPropertyStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = Ident::new(
            &(self.block_name.clone() + "_props").to_upper_camel_case(),
            Span::call_site(),
        );
        let block_name = self.block_name.clone();
        let values = self.entires.iter().map(|(key, value)| {
            let key = Ident::new_raw(&key.to_owned(), Span::call_site());
            let value = Ident::new(&value, Span::call_site());

            quote! {
                #key: #value
            }
        });

        let field_names: Vec<_> = self
            .entires
            .iter()
            .map(|(key, _)| Ident::new_raw(key, Span::call_site()))
            .collect();

        let field_types: Vec<_> = self
            .entires
            .iter()
            .map(|(_, ty)| Ident::new(ty, Span::call_site()))
            .collect();

        tokens.extend(quote! {
            pub struct #name {
                #(#values),*
            }

            impl BlockProperties for #name {
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

                fn to_state_id(&self) -> u16 {
                    Block::from_registry_key(#block_name).unwrap().states[0].id + self.to_index()
                }

                fn from_state_id(state_id: u16) -> Option<Self> {
                    let block = Block::from_registry_key(#block_name).unwrap();
                    if state_id >= block.states[0].id && state_id <= block.states.last().unwrap().id {
                        let index = state_id - block.states[0].id;
                        Some(Self::from_index(index))
                    } else {
                        None
                    }
                }
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct CollisionShape {
    pub min: (f32, f32, f32),
    pub max: (f32, f32, f32),
}

impl ToTokens for CollisionShape {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_x = LitFloat::new(&self.min.0.to_string(), Span::call_site());
        let min_y = LitFloat::new(&self.min.1.to_string(), Span::call_site());
        let min_z = LitFloat::new(&self.min.2.to_string(), Span::call_site());

        let max_x = LitFloat::new(&self.max.0.to_string(), Span::call_site());
        let max_y = LitFloat::new(&self.max.1.to_string(), Span::call_site());
        let max_z = LitFloat::new(&self.max.2.to_string(), Span::call_site());

        tokens.extend(quote! {
            CollisionShape {
                min: (#min_x as f32, #min_y as f32, #min_z as f32),
                max: (#max_x as f32, #max_y as f32, #max_z as f32),
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockState {
    pub id: u16,
    pub air: bool,
    pub luminance: u8,
    pub burnable: bool,
    pub tool_required: bool,
    pub hardness: f32,
    pub sided_transparency: bool,
    pub replaceable: bool,
    pub collision_shapes: Vec<u16>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockStateRef {
    pub id: u16,
    pub state_idx: u16,
}

impl ToTokens for BlockState {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        //let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let air = LitBool::new(self.air, Span::call_site());
        let luminance = LitInt::new(&self.luminance.to_string(), Span::call_site());
        let burnable = LitBool::new(self.burnable, Span::call_site());
        let tool_required = LitBool::new(self.tool_required, Span::call_site());
        let hardness = LitFloat::new(&self.hardness.to_string(), Span::call_site());
        let sided_transparency = LitBool::new(self.sided_transparency, Span::call_site());
        let replaceable = LitBool::new(self.replaceable, Span::call_site());

        let collision_shapes = self
            .collision_shapes
            .iter()
            .map(|shape_id| LitInt::new(&shape_id.to_string(), Span::call_site()));

        tokens.extend(quote! {
            BlockState {
                air: #air,
                luminance: #luminance,
                burnable: #burnable,
                tool_required: #tool_required,
                hardness: #hardness as f32,
                sided_transparency: #sided_transparency,
                replaceable: #replaceable,
                collision_shapes: &[#(#collision_shapes),*],
            }
        });
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

#[derive(Deserialize, Clone, Debug)]
pub struct Block {
    pub id: u16,
    pub name: String,
    pub translation_key: String,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub item_id: u16,
    // TODO: pub loot_table: Option<LootTable>,
    pub properties: Vec<BlockProperty>,
    pub default_state_id: u16,
    pub states: Vec<BlockState>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct OptimizedBlock {
    pub id: u16,
    pub name: String,
    pub translation_key: String,
    pub hardness: f32,
    pub blast_resistance: f32,
    pub item_id: u16,
    // TODO: pub loot_table: Option<LootTable>,
    pub default_state_id: u16,
    pub states: Vec<BlockStateRef>,
}

impl ToTokens for OptimizedBlock {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = LitInt::new(&self.id.to_string(), Span::call_site());
        let name = LitStr::new(&self.name, Span::call_site());
        let translation_key = LitStr::new(&self.translation_key, Span::call_site());
        let hardness = LitFloat::new(&self.hardness.to_string(), Span::call_site());
        let blast_resistance = LitFloat::new(&self.blast_resistance.to_string(), Span::call_site());
        let item_id = LitInt::new(&self.item_id.to_string(), Span::call_site());
        let default_state_id = LitInt::new(&self.default_state_id.to_string(), Span::call_site());

        // Generate state tokens
        let states = self.states.iter().map(|state| state.to_token_stream());

        tokens.extend(quote! {
            Block {
                id: #id,
                registry_key: #name,
                translation_key: #translation_key,
                hardness: #hardness as f32,
                blast_resistance: #blast_resistance as f32,
                item_id: #item_id,
                default_state_id: #default_state_id,
                states: &[#(#states),*],
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct BlockAssets {
    pub blocks: Vec<Block>,
    pub shapes: Vec<CollisionShape>,
    pub block_entity_types: Vec<String>,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/blocks.json");

    let blocks_assets: BlockAssets = serde_json::from_str(include_str!("../../assets/blocks.json"))
        .expect("Failed to parse blocks.json");

    let mut type_from_raw_id_arms = TokenStream::new();
    let mut type_from_name = TokenStream::new();
    let mut constants = TokenStream::new();

    let mut unique_states = Vec::new();
    for block in blocks_assets.blocks.clone() {
        for state in block.states.clone() {
            // Check if this state is already in unique_states by comparing all fields except id
            let already_exists = unique_states.iter().any(|s: &BlockState| {
                s.air == state.air
                    && s.luminance == state.luminance
                    && s.burnable == state.burnable
                    && s.tool_required == state.tool_required
                    && s.hardness == state.hardness
                    && s.sided_transparency == state.sided_transparency
                    && s.replaceable == state.replaceable
                    && s.collision_shapes == state.collision_shapes
            });

            if !already_exists {
                unique_states.push(state);
            }
        }
    }

    // Create a map of block name to Block for easier access
    let mut blocks_map: HashMap<String, OptimizedBlock> = HashMap::new();
    let mut block_props: Vec<BlockPropertyStruct> = Vec::new();
    let mut properties: Vec<PropertyStruct> = Vec::new();
    for block in blocks_assets.blocks.clone() {
        blocks_map.insert(
            block.name.clone(),
            OptimizedBlock {
                id: block.id,
                name: block.name.clone(),
                translation_key: block.translation_key.clone(),
                hardness: block.hardness,
                blast_resistance: block.blast_resistance,
                item_id: block.item_id,
                default_state_id: block.default_state_id,
                states: block
                    .states
                    .iter()
                    .map(|state| {
                        // Find the index in unique_states by comparing all fields except id
                        let state_idx = unique_states
                            .iter()
                            .position(|s| {
                                s.air == state.air
                                    && s.luminance == state.luminance
                                    && s.burnable == state.burnable
                                    && s.tool_required == state.tool_required
                                    && s.hardness == state.hardness
                                    && s.sided_transparency == state.sided_transparency
                                    && s.replaceable == state.replaceable
                                    && s.collision_shapes == state.collision_shapes
                            })
                            .unwrap() as u16;

                        BlockStateRef {
                            id: state.id,
                            state_idx,
                        }
                    })
                    .collect(),
            },
        );
        let mut props: HashMap<String, Vec<String>> = HashMap::new();
        for prop in block.properties.clone() {
            props.insert(prop.name.clone(), prop.values.clone());
        }
        if props.len() != 0 {
            block_props.push(BlockPropertyStruct {
                block_name: block.name.clone(),
                entires: props
                    .into_iter()
                    .map(|(key, values)| (key, get_enum_name(values)))
                    .collect(),
            });
        }
        for prop in block.properties.clone() {
            if !properties
                .iter()
                .any(|p| p.name == get_enum_name(prop.values.clone()))
            {
                properties.push(PropertyStruct {
                    name: get_enum_name(prop.values.clone()),
                    values: prop.values.clone(),
                });
            }
        }
    }

    // Generate collision shapes array
    let shapes = blocks_assets
        .shapes
        .iter()
        .map(|shape| shape.to_token_stream());

    let unique_states = unique_states.iter().map(|state| state.to_token_stream());

    let block_props = block_props.iter().map(|prop| prop.to_token_stream());
    let properties = properties.iter().map(|prop| prop.to_token_stream());

    // Generate block entity types array
    let block_entity_types = blocks_assets
        .block_entity_types
        .iter()
        .map(|entity_type| LitStr::new(entity_type, Span::call_site()));

    // Generate constants and match arms for each block
    for (name, block) in blocks_map {
        let const_ident = format_ident!("{}", name.to_shouty_snake_case());
        let block_tokens = block.to_token_stream();
        let id_lit = LitInt::new(&block.id.to_string(), Span::call_site());

        constants.extend(quote! {
            pub const #const_ident: Block = #block_tokens;
        });

        type_from_raw_id_arms.extend(quote! {
            #id_lit => Some(Self::#const_ident),
        });

        type_from_name.extend(quote! {
            #name => Some(Self::#const_ident),
        });
    }

    quote! {
        use crate::tag::{Tagable, RegistryKey};


        #[derive(Clone, Debug)]
        pub struct BlockState {
            pub air: bool,
            pub luminance: u8,
            pub burnable: bool,
            pub tool_required: bool,
            pub hardness: f32,
            pub sided_transparency: bool,
            pub replaceable: bool,
            pub collision_shapes: &'static [u16],
        }

        #[derive(Clone, Debug)]
        pub struct BlockStateRef {
            pub id: u16,
            pub state_idx: u16,
        }

        #[derive(Clone, Debug)]
        pub struct Block {
            pub id: u16,
            pub registry_key: &'static str,
            pub translation_key: &'static str,
            pub hardness: f32,
            pub blast_resistance: f32,
            pub item_id: u16,
            pub default_state_id: u16,
            pub states: &'static [BlockStateRef],
        }

        impl PartialEq for Block {
            fn eq(&self, other: &Self) -> bool {
                self.id == other.id
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct BlockProperty {
            pub name: &'static str,
            pub values: &'static [&'static str],
        }

        #[derive(Clone, Copy, Debug)]
        pub struct CollisionShape {
            pub min: (f32, f32, f32),
            pub max: (f32, f32, f32),
        }

        #[derive(Clone, Copy, Debug)]
        pub struct BlockStateData {
            pub air: bool,
            pub luminance: u8,
            pub burnable: bool,
            pub tool_required: bool,
            pub hardness: f32,
            pub sided_transparency: bool,
            pub replaceable: bool,
            pub collision_shapes: &'static [u16],
        }


        pub trait BlockProperties {
            // Convert properties to an index (0 to N-1)
            fn to_index(&self) -> u16;
            // Convert an index back to properties
            fn from_index(index: u16) -> Self;

            // Convert properties to a state id
            fn to_state_id(&self) -> u16;
            // Convert a state id back to properties
            fn from_state_id(state_id: u16) -> Option<Self> where Self: Sized;
        }

        pub trait EnumVariants {
            fn variant_count() -> u16;
            fn to_index(&self) -> u16;
            fn from_index(index: u16) -> Self;
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

            #[doc = r" Try to parse a Block from a resource location string"]
            pub fn from_registry_key(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }

            #[doc = r" Try to parse a Block from a raw id"]
            pub const fn from_id(id: u16) -> Option<Self> {
                match id {
                    #type_from_raw_id_arms
                    _ => None
                }
            }
        }

        #(#properties)*

        #(#block_props)*

        impl Tagable for Block {
            #[inline]
            fn tag_key() -> RegistryKey {
                RegistryKey::Block
            }

            #[inline]
            fn registry_key(&self) -> &str {
                self.registry_key
            }
        }
    }
}
