use std::{collections::HashSet, fs, path::Path};

use heck::{ToPascalCase, ToShoutySnakeCase};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

use crate::block::BlockAssets;

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
    #[serde(rename = "type", default)]
    provider_type: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    source: Option<Box<StateProviderJson>>,
    #[serde(default)]
    id: Option<String>,
}

impl StateProviderJson {
    fn provider_type(&self) -> &str {
        self.provider_type
            .as_deref()
            .unwrap_or("minecraft:simple_state_provider")
    }

    fn state_name(&self) -> Option<&str> {
        self.state.as_deref().or(self.id.as_deref())
    }
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
    match provider.provider_type() {
        "minecraft:simple_state_provider" => provider
            .state_name()
            .is_some_and(|s| valid_blocks.contains(clean_name(s))),
        "minecraft:copy_properties_provider" | "minecraft:copy_properties" => provider
            .source
            .as_ref()
            .and_then(|s| s.state_name())
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
    match provider.provider_type() {
        "minecraft:simple_state_provider" => {
            let state_name = provider.state_name().expect("simple_state missing state");
            let id = block_ident(state_name);
            quote! {
                BlockTransformerStateProvider::SimpleState(BlockId::#id)
            }
        }
        "minecraft:copy_properties_provider" | "minecraft:copy_properties" => {
            let src = provider
                .source
                .as_ref()
                .expect("copy_properties missing source");
            let state_name = src.state_name().expect("copy_properties missing state");
            let id = block_ident(state_name);
            quote! {
                BlockTransformerStateProvider::CopyProperties(BlockId::#id)
            }
        }
        other => panic!("Unsupported state provider type: {other}"),
    }
}

pub fn build() -> TokenStream {
    build_from_assets(
        &fs::read_to_string("../../assets/blocks.json").unwrap(),
        Path::new("../../assets/datapack/data/minecraft/block_transformer"),
    )
}

fn build_from_assets(blocks_json: &str, dir: &Path) -> TokenStream {
    let blocks_file: BlockAssets =
        serde_json::from_str(blocks_json).expect("Failed to parse blocks.json");
    let valid_blocks: HashSet<String> = blocks_file
        .blocks
        .into_iter()
        .map(|block| block.name)
        .collect();

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

#[cfg(test)]
mod tests {
    use super::build_from_assets;
    use proc_macro2::TokenStream;
    use quote::{ToTokens, quote};
    use std::path::Path;
    use syn::{Expr, Item, ItemStatic, Member};

    // Four complete records from blocks.json at f1c0871f492182a228fa585e9d7c74e683e242f9.
    // Keep the shape prefix through index 142 and all block entity types, without remapping.
    const BLOCKS: &str = include_str!("../tests/fixtures/block_transformer/blocks.json");

    fn generated_hoe(blocks: &str) -> ItemStatic {
        // Unmodified 26.3 hoe.json, SHA-256 a4c6294366b9a4c9c847e3e48c8896686efdfce1e6273f8053168934aed25e69.
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/block_transformer/transformers");
        syn::parse2::<syn::File>(build_from_assets(blocks, &dir))
            .unwrap()
            .items
            .into_iter()
            .find_map(|item| match item {
                Item::Static(item) if item.ident == "HOE" => Some(item),
                _ => None,
            })
            .expect("emitted HOE static")
    }

    fn field<'a>(expr: &'a Expr, name: &str) -> &'a Expr {
        let Expr::Struct(value) = expr else {
            panic!("expected a struct expression");
        };
        &value
            .fields
            .iter()
            .find(|field| matches!(&field.member, Member::Named(ident) if ident == name))
            .unwrap_or_else(|| panic!("missing field {name}"))
            .expr
    }

    fn array(expr: &Expr) -> Vec<&Expr> {
        let Expr::Reference(reference) = expr else {
            panic!("expected a borrowed array");
        };
        let Expr::Array(array) = reference.expr.as_ref() else {
            panic!("expected an array");
        };
        array.elems.iter().collect()
    }

    fn assert_field(expr: &Expr, name: &str, expected: TokenStream) {
        assert_eq!(
            field(expr, name).to_token_stream().to_string(),
            expected.to_string(),
            "{name}",
        );
    }

    fn all_of(rule: &Expr) -> Vec<&Expr> {
        let Expr::Call(call) = field(rule, "predicate") else {
            panic!("expected an all_of predicate");
        };
        assert_eq!(
            call.func.to_token_stream().to_string(),
            quote!(BlockPredicate::AllOf).to_string(),
        );
        assert_eq!(call.args.len(), 1);
        array(&call.args[0])
    }

    #[test]
    fn hoe_rules_use_nested_block_names() {
        let hoe = generated_hoe(BLOCKS);
        let entries = array(field(&hoe.expr, "entries"));
        assert_eq!(entries.len(), 2);
        let first = array(field(entries[0], "rules"));
        let second = array(field(entries[1], "rules"));
        assert_eq!(
            first.len(),
            2,
            "farmland and coarse dirt rules must survive"
        );
        assert_eq!(second.len(), 1, "rooted dirt rule must survive");

        let farmland = all_of(first[0]);
        let coarse_dirt = all_of(first[1]);
        assert_eq!(farmland.len(), 2);
        assert_eq!(coarse_dirt.len(), 2);
        assert_field(
            farmland[0],
            "tag",
            quote!(tag::Block::MINECRAFT_TURNS_INTO_FARMLAND),
        );
        assert_field(farmland[0], "offset", quote!((0i8, 0i8, 0i8)));
        assert_field(coarse_dirt[0], "blocks", quote!(&[BlockId::COARSE_DIRT]));
        assert_field(coarse_dirt[0], "offset", quote!((0i8, 0i8, 0i8)));
        for air in [farmland[1], coarse_dirt[1]] {
            assert_field(air, "tag", quote!(tag::Block::MINECRAFT_AIR));
            assert_field(air, "offset", quote!((0i8, 1i8, 0i8)));
        }
        let rooted_dirt = field(second[0], "predicate");
        assert_field(rooted_dirt, "blocks", quote!(&[BlockId::ROOTED_DIRT]));
        assert_field(rooted_dirt, "offset", quote!((0i8, 0i8, 0i8)));

        // Explicit state properties remain unsupported; this checks target IDs only.
        assert_field(
            first[0],
            "provider",
            quote!(BlockTransformerStateProvider::SimpleState(
                BlockId::FARMLAND
            )),
        );
        for rule in [first[1], second[0]] {
            assert_field(
                rule,
                "provider",
                quote!(BlockTransformerStateProvider::SimpleState(BlockId::DIRT)),
            );
        }
        assert_field(
            entries[0],
            "disallowed_faces",
            quote!(&[BlockDirection::Down]),
        );
        assert_field(entries[1], "disallowed_faces", quote!(&[]));
        for entry in &entries {
            assert_field(entry, "item_damage_per_use", quote!(1u16));
            assert_field(
                entry,
                "sound",
                quote!(Some(crate::sound::Sound::ItemHoeTill)),
            );
            assert_field(entry, "particle", quote!(None));
            assert_field(entry, "transform_type", quote!(None));
            assert_field(entry, "update_from_neighbors", quote!(true));
        }
        assert_field(entries[0], "loot", quote!(None));
        assert_field(entries[0], "drop_strategy", quote!(None));
        assert_field(
            entries[1],
            "loot",
            quote!(Some("minecraft:till/rooted_dirt")),
        );
        assert_field(
            entries[1],
            "drop_strategy",
            quote!(Some(DropStrategy::ClickedFace)),
        );
    }

    #[test]
    fn rules_with_unknown_predicate_or_target_blocks_are_filtered() {
        for (removed, expected) in [
            ("farmland", [1, 1]),
            ("coarse_dirt", [1, 1]),
            ("rooted_dirt", [2, 0]),
            ("dirt", [1, 0]),
        ] {
            let mut blocks: serde_json::Value = serde_json::from_str(BLOCKS).unwrap();
            blocks["blocks"]
                .as_array_mut()
                .unwrap()
                .retain(|block| block["name"] != removed);
            let hoe = generated_hoe(&blocks.to_string());
            let entries = array(field(&hoe.expr, "entries"));
            let counts: Vec<_> = entries
                .iter()
                .map(|entry| array(field(entry, "rules")).len())
                .collect();
            assert_eq!(counts, expected, "removed {removed}");
        }
    }

    #[test]
    fn malformed_block_inventory_is_rejected() {
        let blocks: serde_json::Value = serde_json::from_str(BLOCKS).unwrap();
        let mut missing_blocks = blocks.clone();
        missing_blocks.as_object_mut().unwrap().remove("blocks");
        let mut non_array_blocks = blocks.clone();
        non_array_blocks["blocks"] = serde_json::json!({});
        let mut missing_name = blocks.clone();
        missing_name["blocks"][0]
            .as_object_mut()
            .unwrap()
            .remove("name");
        let mut non_string_name = blocks;
        non_string_name["blocks"][0]["name"] = serde_json::json!(1);
        for malformed in [
            missing_blocks,
            non_array_blocks,
            missing_name,
            non_string_name,
        ] {
            assert!(std::panic::catch_unwind(|| generated_hoe(&malformed.to_string())).is_err());
        }
    }
}
