use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct TransformerEntryJson {
    block_state_provider: BlockStateProviderWrapperJson,
    #[serde(default)]
    disallowed_faces: Vec<String>,
    #[serde(default = "default_item_damage")]
    item_damage_per_use: u16,
    #[serde(default)]
    sound: Option<String>,
    #[serde(default)]
    particle: Option<String>,
    #[serde(default)]
    loot: Option<String>,
    #[serde(default)]
    drop_strategy: Option<String>,
    #[serde(default)]
    transform_type: Option<String>,
    #[serde(default = "default_true")]
    update_from_neighbors: bool,
}

const fn default_item_damage() -> u16 {
    1
}

const fn default_true() -> bool {
    true
}

#[derive(Deserialize, Debug)]
struct BlockStateProviderWrapperJson {
    #[serde(rename = "type")]
    provider_type: String,
    #[serde(default)]
    rules: Vec<RuleJson>,
}

#[derive(Deserialize, Debug)]
struct RuleJson {
    if_true: PredicateJson,
    then: StateProviderJson,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum BlocksValue {
    Single(String),
    Multiple(Vec<String>),
}

#[derive(Deserialize, Debug)]
struct PredicateJson {
    #[serde(rename = "type")]
    predicate_type: String,
    #[serde(default)]
    blocks: Option<BlocksValue>,
    #[serde(default)]
    tag: Option<String>,
    #[serde(default)]
    offset: Option<[i8; 3]>,
    #[serde(default)]
    predicates: Option<Vec<PredicateJson>>,
}

#[derive(Deserialize, Debug)]
struct StateProviderJson {
    #[serde(rename = "type")]
    provider_type: String,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    source: Option<Box<StateProviderJson>>,
}

fn clean_name(name: &str) -> &str {
    name.strip_prefix("minecraft:").unwrap_or(name)
}

fn block_ident(name: &str) -> proc_macro2::Ident {
    format_ident!("{}", clean_name(name).to_shouty_snake_case())
}

fn sound_ident(name: &str) -> proc_macro2::Ident {
    let clean = clean_name(name);
    format_ident!("{}", clean.replace(['.', '_'], " ").to_pascal_case())
}

fn particle_tokens(particle: &str) -> TokenStream {
    match particle {
        "scrape" => quote! { Some(crate::world::WorldEvent::ParticlesScrape) },
        "wax_off" => quote! { Some(crate::world::WorldEvent::ParticlesWaxOff) },
        "wax_on" => quote! { Some(crate::world::WorldEvent::ParticlesAndSoundWaxOn) },
        _ => quote! { None },
    }
}

fn direction_ident(face: &str) -> proc_macro2::Ident {
    format_ident!("{}", face.to_pascal_case())
}

fn is_predicate_valid(pred: &PredicateJson, valid_blocks: &HashSet<String>) -> bool {
    match pred.predicate_type.as_str() {
        "minecraft:matching_blocks" => match &pred.blocks {
            Some(BlocksValue::Single(s)) => valid_blocks.contains(clean_name(s)),
            Some(BlocksValue::Multiple(list)) => {
                list.iter().any(|s| valid_blocks.contains(clean_name(s)))
            }
            None => false,
        },
        "minecraft:matching_block_tag" => true,
        "minecraft:all_of" => pred.predicates.as_ref().map_or(true, |list| {
            list.iter().all(|p| is_predicate_valid(p, valid_blocks))
        }),
        _ => false,
    }
}

fn is_provider_valid(provider: &StateProviderJson, valid_blocks: &HashSet<String>) -> bool {
    match provider.provider_type.as_str() {
        "minecraft:simple_state_provider" => provider
            .state
            .as_deref()
            .is_some_and(|s| valid_blocks.contains(clean_name(s))),
        "minecraft:copy_properties_provider" => provider
            .source
            .as_ref()
            .and_then(|s| s.state.as_deref())
            .is_some_and(|s| valid_blocks.contains(clean_name(s))),
        _ => false,
    }
}

fn predicate_to_tokens(pred: &PredicateJson, valid_blocks: &HashSet<String>) -> TokenStream {
    let (ox, oy, oz) = pred.offset.map_or((0i8, 0i8, 0i8), |o| (o[0], o[1], o[2]));
    match pred.predicate_type.as_str() {
        "minecraft:matching_blocks" => {
            let blocks: Vec<_> = match &pred.blocks {
                Some(BlocksValue::Single(s)) if valid_blocks.contains(clean_name(s)) => {
                    vec![block_ident(s)]
                }
                Some(BlocksValue::Multiple(list)) => list
                    .iter()
                    .filter(|s| valid_blocks.contains(clean_name(s)))
                    .map(|s| block_ident(s))
                    .collect(),
                _ => Vec::new(),
            };
            quote! {
                BlockPredicate::MatchingBlocks {
                    blocks: &[#(BlockId::#blocks),*],
                    offset: (#ox, #oy, #oz),
                }
            }
        }
        "minecraft:matching_block_tag" => {
            let tag_name = pred.tag.as_deref().unwrap_or("");
            let const_name = format_ident!(
                "{}",
                tag_name.replace([':', '/', '.', '-'], "_").to_uppercase()
            );
            quote! {
                BlockPredicate::MatchingBlockTag {
                    tag: tag::Block::#const_name,
                    offset: (#ox, #oy, #oz),
                }
            }
        }
        "minecraft:all_of" => {
            let sub_tokens: Vec<TokenStream> = pred
                .predicates
                .as_ref()
                .map_or(&[] as &[_], |v| v.as_slice())
                .iter()
                .map(|p| predicate_to_tokens(p, valid_blocks))
                .collect();
            quote! {
                BlockPredicate::AllOf(&[#(#sub_tokens),*])
            }
        }
        _ => panic!("Unsupported predicate type: {}", pred.predicate_type),
    }
}

fn state_provider_to_tokens(provider: &StateProviderJson) -> TokenStream {
    match provider.provider_type.as_str() {
        "minecraft:simple_state_provider" => {
            let state_name = provider
                .state
                .as_deref()
                .expect("simple_state missing state");
            let id = block_ident(state_name);
            quote! {
                BlockTransformerStateProvider::SimpleState(BlockId::#id)
            }
        }
        "minecraft:copy_properties_provider" => {
            let src = provider
                .source
                .as_ref()
                .expect("copy_properties missing source");
            let state_name = src.state.as_deref().expect("copy_properties missing state");
            let id = block_ident(state_name);
            quote! {
                BlockTransformerStateProvider::CopyProperties(BlockId::#id)
            }
        }
        _ => panic!(
            "Unsupported state provider type: {}",
            provider.provider_type
        ),
    }
}

pub fn build() -> TokenStream {
    let blocks_file: BTreeMap<String, serde_json::Value> =
        serde_json::from_str(&fs::read_to_string("../../assets/blocks.json").unwrap())
            .expect("Failed to parse blocks.json");
    let valid_blocks: HashSet<String> = blocks_file.into_keys().collect();

    let dir = Path::new("../../assets/datapacks/26_2/data/minecraft/block_transformer");
    let mut files: Vec<(String, Vec<TransformerEntryJson>)> = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir).expect("Failed to read block_transformer dir") {
            let entry = entry.expect("DirEntry error");
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
                let content = fs::read_to_string(&path)
                    .unwrap_or_else(|_| panic!("Failed to read {}", path.display()));
                let entries: Vec<TransformerEntryJson> = serde_json::from_str(&content)
                    .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));
                files.push((stem, entries));
            }
        }
    }

    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut generated_transformers = Vec::new();
    let mut lookup_arms = Vec::new();

    for (stem, entries) in &files {
        let const_ident = format_ident!("{}", stem.to_shouty_snake_case());
        let mut entry_tokens = Vec::new();

        for entry in entries {
            let disallowed: Vec<_> = entry
                .disallowed_faces
                .iter()
                .map(|f| {
                    let dir_id = direction_ident(f);
                    quote! { BlockDirection::#dir_id }
                })
                .collect();

            let item_damage = entry.item_damage_per_use;
            let sound_tok = entry.sound.as_ref().map_or_else(
                || quote! { None },
                |s| {
                    let snd = sound_ident(s);
                    quote! { Some(crate::sound::Sound::#snd) }
                },
            );

            let particle_tok = entry
                .particle
                .as_ref()
                .map_or_else(|| quote! { None }, |p| particle_tokens(p));

            let loot_tok = entry
                .loot
                .as_ref()
                .map_or_else(|| quote! { None }, |l| quote! { Some(#l) });

            let drop_strategy_tok = entry.drop_strategy.as_ref().map_or_else(
                || quote! { None },
                |d| match d.as_str() {
                    "clicked_face" => quote! { Some(DropStrategy::ClickedFace) },
                    _ => quote! { None },
                },
            );

            let transform_type_tok = entry.transform_type.as_ref().map_or_else(
                || quote! { None },
                |t| match t.as_str() {
                    "copper_chest" => quote! { Some(TransformType::CopperChest) },
                    _ => quote! { None },
                },
            );

            let update_neighbors = entry.update_from_neighbors;

            let rule_tokens: Vec<_> = entry
                .block_state_provider
                .rules
                .iter()
                .filter(|rule| {
                    is_predicate_valid(&rule.if_true, &valid_blocks)
                        && is_provider_valid(&rule.then, &valid_blocks)
                })
                .map(|rule| {
                    let pred = predicate_to_tokens(&rule.if_true, &valid_blocks);
                    let prov = state_provider_to_tokens(&rule.then);
                    quote! {
                        BlockTransformerRule {
                            predicate: #pred,
                            provider: #prov,
                        }
                    }
                })
                .collect();

            entry_tokens.push(quote! {
                BlockTransformerEntry {
                    rules: &[#(#rule_tokens),*],
                    disallowed_faces: &[#(#disallowed),*],
                    item_damage_per_use: #item_damage,
                    sound: #sound_tok,
                    particle: #particle_tok,
                    loot: #loot_tok,
                    drop_strategy: #drop_strategy_tok,
                    transform_type: #transform_type_tok,
                    update_from_neighbors: #update_neighbors,
                }
            });
        }

        generated_transformers.push(quote! {
            pub static #const_ident: BlockTransformer = BlockTransformer {
                entries: &[#(#entry_tokens),*],
            };
        });

        let full_key = format!("minecraft:{stem}");
        lookup_arms.push(quote! {
            #full_key | #stem => Some(&#const_ident),
        });
    }

    quote! {
        use crate::{
            Block, BlockDirection, BlockId, BlockStateId, tag,
        };

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum DropStrategy {
            ClickedFace,
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum TransformType {
            CopperChest,
        }

        #[derive(Debug, Clone, Copy)]
        pub enum BlockPredicate {
            MatchingBlocks {
                blocks: &'static [BlockId],
                offset: (i8, i8, i8),
            },
            MatchingBlockTag {
                tag: tag::Tag,
                offset: (i8, i8, i8),
            },
            AllOf(&'static [BlockPredicate]),
        }

        impl BlockPredicate {
            #[must_use]
            pub fn matches<F>(&self, get_block: &F) -> bool
            where
                F: Fn(i8, i8, i8) -> &'static Block,
            {
                match self {
                    Self::MatchingBlocks { blocks, offset } => {
                        let block = get_block(offset.0, offset.1, offset.2);
                        blocks.contains(&block.id)
                    }
                    Self::MatchingBlockTag { tag, offset } => {
                        let block = get_block(offset.0, offset.1, offset.2);
                        block.id.has_tag(*tag)
                    }
                    Self::AllOf(predicates) => {
                        predicates.iter().all(|p| p.matches(get_block))
                    }
                }
            }
        }

        #[derive(Debug, Clone, Copy)]
        pub enum BlockTransformerStateProvider {
            SimpleState(BlockId),
            CopyProperties(BlockId),
        }

        #[derive(Debug, Clone, Copy)]
        pub struct BlockTransformerRule {
            pub predicate: BlockPredicate,
            pub provider: BlockTransformerStateProvider,
        }

        #[derive(Debug, Clone, Copy)]
        pub struct BlockTransformerEntry {
            pub rules: &'static [BlockTransformerRule],
            pub disallowed_faces: &'static [BlockDirection],
            pub item_damage_per_use: u16,
            pub sound: Option<crate::sound::Sound>,
            pub particle: Option<crate::world::WorldEvent>,
            pub loot: Option<&'static str>,
            pub drop_strategy: Option<DropStrategy>,
            pub transform_type: Option<TransformType>,
            pub update_from_neighbors: bool,
        }

        #[derive(Debug, Clone, Copy)]
        pub struct BlockTransformer {
            pub entries: &'static [BlockTransformerEntry],
        }

        #[derive(Debug, Clone, Copy)]
        pub struct TransformResult {
            pub new_state_id: BlockStateId,
            pub target_block: &'static Block,
            pub entry: &'static BlockTransformerEntry,
        }

        impl BlockTransformer {
            #[must_use]
            pub fn transform<F>(
                &self,
                current_block: &Block,
                current_state_id: BlockStateId,
                face: BlockDirection,
                get_block: &F,
            ) -> Option<TransformResult>
            where
                F: Fn(i8, i8, i8) -> &'static Block,
            {
                for entry in self.entries {
                    if entry.disallowed_faces.contains(&face) {
                        continue;
                    }
                    for rule in entry.rules {
                        if rule.predicate.matches(get_block) {
                            let (new_state_id, target_block) = match rule.provider {
                                BlockTransformerStateProvider::SimpleState(target_id) => {
                                    let target_block = target_id.to_block();
                                    (target_block.default_state.id, target_block)
                                }
                                BlockTransformerStateProvider::CopyProperties(target_id) => {
                                    let target_block = target_id.to_block();
                                    let new_state_id = if target_block.states.len() <= 1 {
                                        target_block.default_state.id
                                    } else if let Some(source_props) = current_block.properties(current_state_id) {
                                        let props = source_props.to_props();
                                        target_block.from_properties(&props).to_state_id(target_block)
                                    } else {
                                        target_block.default_state.id
                                    };
                                    (new_state_id, target_block)
                                }
                            };
                            return Some(TransformResult {
                                new_state_id,
                                target_block,
                                entry,
                            });
                        }
                    }
                }
                None
            }
        }

        #(#generated_transformers)*

        #[must_use]
        pub fn get_block_transformer(key: &str) -> Option<&'static BlockTransformer> {
            match key {
                #(#lookup_arms)*
                _ => None,
            }
        }
    }
}
