use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum RawProcessorListWrapper {
    Object { processors: Vec<RawProcessor> },
    Array(Vec<RawProcessor>),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "processor_type")]
pub enum RawProcessor {
    #[serde(rename = "minecraft:rule", alias = "rule")]
    Rule { rules: Vec<RawProcessorRule> },
    #[serde(rename = "minecraft:block_rot", alias = "block_rot")]
    BlockRot {
        integrity: f32,
        #[serde(default)]
        rottable_blocks: Option<RawRottableBlocks>,
    },
    #[serde(rename = "minecraft:block_age", alias = "block_age")]
    BlockAge { mossiness: f32 },
    #[serde(rename = "minecraft:block_ignore", alias = "block_ignore")]
    BlockIgnore { blocks: Vec<RawBlockStateOrName> },
    #[serde(rename = "minecraft:gravity", alias = "gravity")]
    Gravity {
        heightmap: String,
        #[serde(default)]
        offset: i32,
    },
    #[serde(rename = "minecraft:protected_blocks", alias = "protected_blocks")]
    ProtectedBlocks { value: String },
    #[serde(rename = "minecraft:blackstone_replace", alias = "blackstone_replace")]
    BlackstoneReplace,
    #[serde(rename = "minecraft:jigsaw_replacement", alias = "jigsaw_replacement")]
    JigsawReplacement,
    #[serde(
        rename = "minecraft:lava_submerged_block",
        alias = "lava_submerged_block"
    )]
    LavaSubmergedBlock,
    #[serde(rename = "minecraft:capped", alias = "capped")]
    Capped {
        limit: RawCappedLimit,
        delegate: Box<RawProcessor>,
    },
    #[serde(rename = "minecraft:nop", alias = "nop")]
    Nop,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RawCappedLimit {
    Int(i32),
    Obj {
        #[serde(default)]
        value: i32,
    },
}

impl RawCappedLimit {
    fn as_i32(&self) -> i32 {
        match self {
            Self::Int(i) => *i,
            Self::Obj { value } => *value,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RawRottableBlocks {
    Tag(String),
    List(Vec<String>),
    Single(String),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RawBlockStateOrName {
    State(RawBlockState),
    Name(String),
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawBlockState {
    #[serde(rename = "Name", alias = "id")]
    pub name: String,
    #[serde(rename = "Properties", default)]
    pub properties: BTreeMap<String, String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawProcessorRule {
    #[serde(default)]
    pub position_predicate: Option<RawPosRuleTest>,
    pub input_predicate: RawRuleTest,
    #[serde(default)]
    pub location_predicate: Option<RawRuleTest>,
    pub output_state: RawOutputState,
    #[serde(default)]
    pub block_entity_modifier: Option<RawBlockEntityModifier>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RawOutputState {
    Name(String),
    State {
        #[serde(rename = "Name", alias = "id")]
        name: String,
        #[serde(rename = "Properties", default)]
        properties: BTreeMap<String, String>,
    },
}

impl RawOutputState {
    fn name(&self) -> &str {
        match self {
            Self::Name(n) => n,
            Self::State { name, .. } => name,
        }
    }

    fn properties(&self) -> Option<&BTreeMap<String, String>> {
        match self {
            Self::Name(_) => None,
            Self::State { properties, .. } => {
                if properties.is_empty() {
                    None
                } else {
                    Some(properties)
                }
            }
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "predicate_type")]
pub enum RawRuleTest {
    #[serde(rename = "minecraft:always_true", alias = "always_true")]
    AlwaysTrue,
    #[serde(rename = "minecraft:block_match", alias = "block_match")]
    BlockMatch { block: String },
    #[serde(rename = "minecraft:blockstate_match", alias = "blockstate_match")]
    BlockStateMatch {
        block: Option<String>,
        block_state: Option<RawBlockState>,
    },
    #[serde(rename = "minecraft:random_block_match", alias = "random_block_match")]
    RandomBlockMatch { block: String, probability: f32 },
    #[serde(
        rename = "minecraft:random_blockstate_match",
        alias = "random_blockstate_match"
    )]
    RandomBlockStateMatch {
        block: Option<String>,
        block_state: Option<RawBlockState>,
        probability: f32,
    },
    #[serde(rename = "minecraft:tag_match", alias = "tag_match")]
    TagMatch { tag: String },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "predicate_type")]
pub enum RawPosRuleTest {
    #[serde(rename = "minecraft:always_true", alias = "always_true")]
    AlwaysTrue,
    #[serde(rename = "minecraft:linear_pos", alias = "linear_pos")]
    LinearPos {
        #[serde(default)]
        min_dist: i32,
        #[serde(default)]
        max_dist: i32,
        #[serde(default)]
        min_chance: f32,
        #[serde(default)]
        max_chance: f32,
    },
    #[serde(
        rename = "minecraft:axis_aligned_linear_pos",
        alias = "axis_aligned_linear_pos"
    )]
    AxisAlignedLinearPos {
        #[serde(default = "default_axis")]
        axis: String,
        #[serde(default)]
        min_dist: i32,
        #[serde(default)]
        max_dist: i32,
        #[serde(default)]
        min_chance: f32,
        #[serde(default)]
        max_chance: f32,
    },
}

fn default_axis() -> String {
    "y".to_string()
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum RawBlockEntityModifier {
    #[serde(rename = "minecraft:append_loot", alias = "append_loot")]
    AppendLoot { loot_table: String },
    #[serde(rename = "minecraft:append_static", alias = "append_static")]
    AppendStatic { data: serde_json::Value },
    #[serde(rename = "minecraft:clear", alias = "clear")]
    Clear,
    #[serde(rename = "minecraft:passthrough", alias = "passthrough")]
    Passthrough,
}

fn block_ident(name: &str) -> syn::Ident {
    let clean = name.strip_prefix("minecraft:").unwrap_or(name);
    format_ident!("{}", clean.to_uppercase().replace([':', '-', '.'], "_"))
}

fn rule_test_to_tokens(rule_test: &RawRuleTest) -> TokenStream {
    match rule_test {
        RawRuleTest::AlwaysTrue => quote!(StaticRuleTest::AlwaysTrue),
        RawRuleTest::BlockMatch { block } => {
            let ident = block_ident(block);
            quote!(StaticRuleTest::BlockMatch(crate::Block::#ident.id))
        }
        RawRuleTest::BlockStateMatch { block, block_state } => {
            if let Some(state) = block_state {
                let ident = block_ident(&state.name);
                let props: Vec<TokenStream> = state
                    .properties
                    .iter()
                    .map(|(k, v)| quote!((#k, #v)))
                    .collect();
                quote!(StaticRuleTest::BlockStateMatch {
                    block_id: crate::Block::#ident.id,
                    properties: &[#(#props),*],
                })
            } else if let Some(b) = block {
                let ident = block_ident(b);
                quote!(StaticRuleTest::BlockMatch(crate::Block::#ident.id))
            } else {
                quote!(StaticRuleTest::AlwaysTrue)
            }
        }
        RawRuleTest::RandomBlockMatch { block, probability } => {
            let ident = block_ident(block);
            quote!(StaticRuleTest::RandomBlockMatch {
                block: crate::Block::#ident.id,
                probability: #probability,
            })
        }
        RawRuleTest::RandomBlockStateMatch {
            block,
            block_state,
            probability,
        } => {
            if let Some(state) = block_state {
                let ident = block_ident(&state.name);
                let props: Vec<TokenStream> = state
                    .properties
                    .iter()
                    .map(|(k, v)| quote!((#k, #v)))
                    .collect();
                quote!(StaticRuleTest::RandomBlockStateMatch {
                    block_id: crate::Block::#ident.id,
                    properties: &[#(#props),*],
                    probability: #probability,
                })
            } else if let Some(b) = block {
                let ident = block_ident(b);
                quote!(StaticRuleTest::RandomBlockMatch {
                    block: crate::Block::#ident.id,
                    probability: #probability,
                })
            } else {
                quote!(StaticRuleTest::AlwaysTrue)
            }
        }
        RawRuleTest::TagMatch { tag } => {
            let tag_name = tag.strip_prefix('#').unwrap_or(tag);
            let tag_name = tag_name.strip_prefix("minecraft:").unwrap_or(tag_name);
            quote!(StaticRuleTest::TagMatch(#tag_name))
        }
    }
}

fn pos_rule_test_to_tokens(pos: Option<&RawPosRuleTest>) -> TokenStream {
    let Some(pos) = pos else {
        return quote!(StaticPosRuleTest::AlwaysTrue);
    };
    match pos {
        RawPosRuleTest::AlwaysTrue => quote!(StaticPosRuleTest::AlwaysTrue),
        RawPosRuleTest::LinearPos {
            min_dist,
            max_dist,
            min_chance,
            max_chance,
        } => quote!(StaticPosRuleTest::LinearPos {
            min_dist: #min_dist,
            max_dist: #max_dist,
            min_chance: #min_chance,
            max_chance: #max_chance,
        }),
        RawPosRuleTest::AxisAlignedLinearPos {
            axis,
            min_dist,
            max_dist,
            min_chance,
            max_chance,
        } => {
            let axis_tok = match axis.to_ascii_lowercase().as_str() {
                "x" => quote!(StaticAxis::X),
                "z" => quote!(StaticAxis::Z),
                _ => quote!(StaticAxis::Y),
            };
            quote!(StaticPosRuleTest::AxisAlignedLinearPos {
                axis: #axis_tok,
                min_dist: #min_dist,
                max_dist: #max_dist,
                min_chance: #min_chance,
                max_chance: #max_chance,
            })
        }
    }
}

fn output_state_to_tokens(state: &RawOutputState) -> TokenStream {
    let name = state.name();
    let ident = block_ident(name);
    let props = state.properties();
    if let Some(props_map) = props {
        let props_tokens: Vec<TokenStream> =
            props_map.iter().map(|(k, v)| quote!((#k, #v))).collect();
        quote!(StaticOutputState {
            default_state: crate::Block::#ident.default_state,
            properties: &[#(#props_tokens),*],
        })
    } else {
        quote!(StaticOutputState {
            default_state: crate::Block::#ident.default_state,
            properties: &[],
        })
    }
}

fn modifier_to_tokens(modifier: Option<&RawBlockEntityModifier>) -> TokenStream {
    let Some(modifier) = modifier else {
        return quote!(None);
    };
    match modifier {
        RawBlockEntityModifier::AppendLoot { loot_table } => {
            quote!(Some(StaticBlockEntityModifier::AppendLoot { loot_table: #loot_table }))
        }
        RawBlockEntityModifier::Clear => quote!(Some(StaticBlockEntityModifier::Clear)),
        RawBlockEntityModifier::Passthrough => {
            quote!(Some(StaticBlockEntityModifier::Passthrough))
        }
        RawBlockEntityModifier::AppendStatic { .. } => quote!(None),
    }
}

fn processor_to_tokens(proc: &RawProcessor) -> TokenStream {
    match proc {
        RawProcessor::Rule { rules } => {
            let rule_tokens: Vec<TokenStream> = rules
                .iter()
                .map(|r| {
                    let pos = pos_rule_test_to_tokens(r.position_predicate.as_ref());
                    let input = rule_test_to_tokens(&r.input_predicate);
                    let loc = r
                        .location_predicate
                        .as_ref()
                        .map_or_else(|| quote!(StaticRuleTest::AlwaysTrue), rule_test_to_tokens);
                    let output = output_state_to_tokens(&r.output_state);
                    let modifier = modifier_to_tokens(r.block_entity_modifier.as_ref());
                    quote!(StaticProcessorRule {
                        position_predicate: #pos,
                        input_predicate: #input,
                        location_predicate: #loc,
                        output_state: #output,
                        block_entity_modifier: #modifier,
                    })
                })
                .collect();
            quote!(StaticProcessor::Rule(&[#(#rule_tokens),*]))
        }
        RawProcessor::BlockRot {
            integrity,
            rottable_blocks,
        } => {
            let rottable_tokens = match rottable_blocks {
                None => quote!(None),
                Some(RawRottableBlocks::Tag(tag)) => {
                    let clean = tag.strip_prefix('#').unwrap_or(tag);
                    let clean = clean.strip_prefix("minecraft:").unwrap_or(clean);
                    quote!(Some(StaticRottableBlocks::Tag(#clean)))
                }
                Some(RawRottableBlocks::Single(block)) => {
                    let ident = block_ident(block);
                    quote!(Some(StaticRottableBlocks::Block(crate::Block::#ident.id)))
                }
                Some(RawRottableBlocks::List(list)) => {
                    let idents: Vec<TokenStream> = list
                        .iter()
                        .map(|b| {
                            let ident = block_ident(b);
                            quote!(crate::Block::#ident.id)
                        })
                        .collect();
                    quote!(Some(StaticRottableBlocks::List(&[#(#idents),*])))
                }
            };
            quote!(StaticProcessor::BlockRot {
                integrity: #integrity,
                rottable_blocks: #rottable_tokens,
            })
        }
        RawProcessor::BlockAge { mossiness } => {
            quote!(StaticProcessor::BlockAge { mossiness: #mossiness })
        }
        RawProcessor::BlockIgnore { blocks } => {
            let ignored_tokens: Vec<TokenStream> = blocks
                .iter()
                .map(|b| match b {
                    RawBlockStateOrName::Name(name) => {
                        let ident = block_ident(name);
                        quote!(StaticIgnoredBlock {
                            block_id: crate::Block::#ident.id,
                            properties: None,
                        })
                    }
                    RawBlockStateOrName::State(state) => {
                        let ident = block_ident(&state.name);
                        if state.properties.is_empty() {
                            quote!(StaticIgnoredBlock {
                                block_id: crate::Block::#ident.id,
                                properties: None,
                            })
                        } else {
                            let props: Vec<TokenStream> = state
                                .properties
                                .iter()
                                .map(|(k, v)| quote!((#k, #v)))
                                .collect();
                            quote!(StaticIgnoredBlock {
                                block_id: crate::Block::#ident.id,
                                properties: Some(&[#(#props),*]),
                            })
                        }
                    }
                })
                .collect();
            quote!(StaticProcessor::BlockIgnore(&[#(#ignored_tokens),*]))
        }
        RawProcessor::Gravity { heightmap, offset } => {
            let heightmap_tok = match heightmap.as_str() {
                "WORLD_SURFACE" => quote!(StaticHeightmapType::WorldSurface),
                "OCEAN_FLOOR_WG" => quote!(StaticHeightmapType::OceanFloorWg),
                "OCEAN_FLOOR" => quote!(StaticHeightmapType::OceanFloor),
                "MOTION_BLOCKING" => quote!(StaticHeightmapType::MotionBlocking),
                "MOTION_BLOCKING_NO_LEAVES" => quote!(StaticHeightmapType::MotionBlockingNoLeaves),
                _ => quote!(StaticHeightmapType::WorldSurfaceWg),
            };
            quote!(StaticProcessor::Gravity {
                heightmap: #heightmap_tok,
                offset: #offset,
            })
        }
        RawProcessor::ProtectedBlocks { value } => {
            let clean = value.strip_prefix('#').unwrap_or(value);
            let clean = clean.strip_prefix("minecraft:").unwrap_or(clean);
            quote!(StaticProcessor::ProtectedBlocks(#clean))
        }
        RawProcessor::BlackstoneReplace => quote!(StaticProcessor::BlackstoneReplace),
        RawProcessor::JigsawReplacement => quote!(StaticProcessor::JigsawReplacement),
        RawProcessor::LavaSubmergedBlock => quote!(StaticProcessor::LavaSubmergedBlock),
        RawProcessor::Capped { limit, delegate } => {
            let limit_val = limit.as_i32();
            let delegate_tok = processor_to_tokens(delegate);
            quote!(StaticProcessor::Capped {
                limit: #limit_val,
                delegate: &#delegate_tok,
            })
        }
        RawProcessor::Nop => quote!(StaticProcessor::Nop),
    }
}

pub fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../../assets/datapack/data/minecraft/worldgen/processor_list");

    let dir = Path::new("../../assets/datapack/data/minecraft/worldgen/processor_list");
    let mut files: Vec<_> = fs::read_dir(dir)
        .expect("Failed to read processor_list directory")
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort_by_key(|e| e.file_name());

    let mut constants = Vec::new();
    let mut match_arms = Vec::new();
    let mut all_names = Vec::new();

    for file in &files {
        let path = file.path();
        let stem = path
            .file_stem()
            .expect("missing stem")
            .to_str()
            .expect("invalid utf8");
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

        let wrapper: RawProcessorListWrapper = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));

        let raw_processors = match wrapper {
            RawProcessorListWrapper::Object { processors }
            | RawProcessorListWrapper::Array(processors) => processors,
        };

        let const_ident = format_ident!("{}", stem.to_uppercase().replace('-', "_"));
        let full_id = format!("minecraft:{stem}");
        let bare_id = stem.to_string();

        let proc_tokens: Vec<TokenStream> =
            raw_processors.iter().map(processor_to_tokens).collect();

        constants.push(quote! {
            pub const #const_ident: &'static [StaticProcessor] = &[#(#proc_tokens),*];
        });

        match_arms.push(quote! {
            #bare_id | #full_id => Some(Self::#const_ident),
        });

        all_names.push(full_id);
    }

    quote! {
        /* This file is generated. Do not edit manually. */

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum StaticHeightmapType {
            WorldSurfaceWg,
            WorldSurface,
            OceanFloorWg,
            OceanFloor,
            MotionBlocking,
            MotionBlockingNoLeaves,
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum StaticAxis {
            X,
            Y,
            Z,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum StaticPosRuleTest {
            AlwaysTrue,
            LinearPos {
                min_dist: i32,
                max_dist: i32,
                min_chance: f32,
                max_chance: f32,
            },
            AxisAlignedLinearPos {
                axis: StaticAxis,
                min_dist: i32,
                max_dist: i32,
                min_chance: f32,
                max_chance: f32,
            },
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum StaticRuleTest {
            AlwaysTrue,
            BlockMatch(crate::BlockId),
            BlockStateMatch {
                block_id: crate::BlockId,
                properties: &'static [(&'static str, &'static str)],
            },
            RandomBlockMatch {
                block: crate::BlockId,
                probability: f32,
            },
            RandomBlockStateMatch {
                block_id: crate::BlockId,
                properties: &'static [(&'static str, &'static str)],
                probability: f32,
            },
            TagMatch(&'static str),
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct StaticIgnoredBlock {
            pub block_id: crate::BlockId,
            pub properties: Option<&'static [(&'static str, &'static str)]>,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum StaticRottableBlocks {
            Tag(&'static str),
            Block(crate::BlockId),
            List(&'static [crate::BlockId]),
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum StaticBlockEntityModifier {
            AppendLoot { loot_table: &'static str },
            Clear,
            Passthrough,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct StaticOutputState {
            pub default_state: &'static crate::BlockState,
            pub properties: &'static [(&'static str, &'static str)],
        }

        #[derive(Clone, Debug, PartialEq)]
        pub struct StaticProcessorRule {
            pub position_predicate: StaticPosRuleTest,
            pub input_predicate: StaticRuleTest,
            pub location_predicate: StaticRuleTest,
            pub output_state: StaticOutputState,
            pub block_entity_modifier: Option<StaticBlockEntityModifier>,
        }

        #[derive(Clone, Debug, PartialEq)]
        pub enum StaticProcessor {
            Rule(&'static [StaticProcessorRule]),
            BlockRot {
                integrity: f32,
                rottable_blocks: Option<StaticRottableBlocks>,
            },
            BlockAge {
                mossiness: f32,
            },
            BlockIgnore(&'static [StaticIgnoredBlock]),
            Gravity {
                heightmap: StaticHeightmapType,
                offset: i32,
            },
            ProtectedBlocks(&'static str),
            BlackstoneReplace,
            JigsawReplacement,
            LavaSubmergedBlock,
            Capped {
                limit: i32,
                delegate: &'static StaticProcessor,
            },
            Nop,
        }

        pub struct StaticProcessorList;

        impl StaticProcessorList {
            #(#constants)*

            #[must_use]
            pub fn get(id: &str) -> Option<&'static [StaticProcessor]> {
                let trimmed = id.strip_prefix("minecraft:").unwrap_or(id);
                match trimmed {
                    #(#match_arms)*
                    _ => None,
                }
            }

            #[must_use]
            pub const fn all_names() -> &'static [&'static str] {
                &[
                    #(#all_names),*
                ]
            }
        }
    }
}
