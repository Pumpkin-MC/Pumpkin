use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use rayon::prelude::*;
use serde::Deserialize;

const DEFAULT_NAMESPACE: &str = "minecraft";

#[derive(Deserialize)]
struct MinimalBlockAssets {
    blocks: Vec<MinimalBlock>,
}

#[derive(Deserialize)]
struct MinimalBlock {
    name: String,
    default_state_id: u16,
    properties: Vec<i32>,
    states: Vec<MinimalBlockState>,
}

#[derive(Deserialize)]
struct MinimalBlockState {
    id: u16,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum MinimalPropertyType {
    #[serde(rename = "boolean")]
    Boolean,
    #[serde(rename = "int")]
    Int { min: u8, max: u8 },
    #[serde(rename = "enum")]
    Enum { values: Vec<String> },
}

#[derive(Deserialize)]
struct MinimalProperty {
    hash_key: i32,
    serialized_name: String,
    #[serde(flatten)]
    property_type: MinimalPropertyType,
}

fn resolve_state_id(
    block_map: &BTreeMap<String, MinimalBlock>,
    prop_map: &BTreeMap<i32, MinimalProperty>,
    name: &str,
    properties: &[(String, String)],
) -> u16 {
    let clean_name = name.strip_prefix("minecraft:").unwrap_or(name);
    let Some(block) = block_map.get(clean_name) else {
        panic!("Unknown block in structure template: {name}");
    };
    if block.properties.is_empty() || properties.is_empty() {
        return block.default_state_id;
    }

    let min_state_id = block.states[0].id;
    let mut default_idx = block.default_state_id - min_state_id;
    let mut curr = 0u16;
    let mut mul = 1u16;

    for &prop_hash in block.properties.iter().rev() {
        let prop = &prop_map[&prop_hash];
        let count = match &prop.property_type {
            MinimalPropertyType::Boolean => 2,
            MinimalPropertyType::Int { min, max } => u16::from(*max - *min + 1),
            MinimalPropertyType::Enum { values } => values.len() as u16,
        };
        let default_val_idx = default_idx % count;
        default_idx /= count;

        let val_idx = if let Some((_, v)) = properties
            .iter()
            .find(|(k, _)| k.as_str() == prop.serialized_name.as_str())
        {
            match &prop.property_type {
                MinimalPropertyType::Boolean => match v.as_str() {
                    "true" => 0,
                    "false" => 1,
                    _ => default_val_idx,
                },
                MinimalPropertyType::Int { min, max } => v
                    .parse::<u8>()
                    .ok()
                    .filter(|&n| n >= *min && n <= *max)
                    .map_or(default_val_idx, |n| u16::from(n - *min)),
                MinimalPropertyType::Enum { values } => values
                    .iter()
                    .position(|s| s == v)
                    .map_or(default_val_idx, |idx| idx as u16),
            }
        } else {
            default_val_idx
        };

        curr += val_idx * mul;
        mul *= count;
    }

    block.states[curr as usize].id
}

struct EmbeddedPack {
    id: String,
    path: PathBuf,
}

struct ParsedPaletteEntry {
    name: String,
    properties: Vec<(String, String)>,
}

struct ParsedBlockEntity {
    pos: [i32; 3],
    nbt: Vec<u8>,
}

struct ParsedEntity {
    pos: [f64; 3],
    block_pos: [i32; 3],
    nbt: Vec<u8>,
}

struct ParsedStructureTemplate {
    resource_id: String,
    bare_id: String,
    is_default: bool,
    size: [i32; 3],
    author: String,
    palettes: Vec<Vec<ParsedPaletteEntry>>,
    packed_blocks: Vec<u8>,
    block_entities: Vec<ParsedBlockEntity>,
    entities: Vec<ParsedEntity>,
}

fn parse_palette_entry(entry: &pumpkin_nbt::compound::NbtCompound) -> ParsedPaletteEntry {
    let name = entry
        .get_string("id")
        .or_else(|| entry.get_string("Name"))
        .unwrap_or_default()
        .to_string();

    let mut properties = Vec::new();
    if let Some(props) = entry
        .get_compound("properties")
        .or_else(|| entry.get_compound("Properties"))
    {
        for (k, v) in &props.child_tags {
            if let pumpkin_nbt::tag::NbtTag::String(val) = v {
                properties.push((k.to_string(), val.to_string()));
            }
        }
    }
    properties.sort_by(|a, b| a.0.cmp(&b.0));
    ParsedPaletteEntry { name, properties }
}

fn parse_template_nbt(
    resource_id: String,
    bare_id: String,
    is_default: bool,
    path: &Path,
) -> Option<ParsedStructureTemplate> {
    let bytes = fs::read(path).ok()?;
    let mut cursor = Cursor::new(bytes);
    let compound = pumpkin_nbt::nbt_compress::read_gzip_compound_tag(&mut cursor).ok()?;

    // 1. size
    let size_list = compound.get_list("size")?;
    if size_list.len() != 3 {
        return None;
    }
    let sx = size_list[0].extract_int()?;
    let sy = size_list[1].extract_int()?;
    let sz = size_list[2].extract_int()?;
    let size = [sx, sy, sz];

    // 2. author
    let author = compound.get_string("author").unwrap_or("?").to_string();

    // 3. palettes
    let mut palettes = Vec::new();
    if let Some(palettes_list) = compound.get_list("palettes") {
        for tag in palettes_list {
            if let pumpkin_nbt::tag::NbtTag::List(list) = tag {
                let mut pal = Vec::new();
                for entry_tag in list {
                    if let pumpkin_nbt::tag::NbtTag::Compound(entry) = entry_tag {
                        pal.push(parse_palette_entry(entry));
                    }
                }
                palettes.push(pal);
            }
        }
    } else if let Some(palette_list) = compound.get_list("palette") {
        let mut pal = Vec::new();
        for entry_tag in palette_list {
            if let pumpkin_nbt::tag::NbtTag::Compound(entry) = entry_tag {
                pal.push(parse_palette_entry(entry));
            }
        }
        palettes.push(pal);
    }

    // 4. blocks
    let mut packed_blocks = Vec::new();
    let mut block_entities = Vec::new();
    if let Some(blocks_list) = compound.get_list("blocks") {
        for tag in blocks_list {
            let pumpkin_nbt::tag::NbtTag::Compound(b) = tag else {
                continue;
            };
            let Some(pos_list) = b.get_list("pos") else {
                continue;
            };
            if pos_list.len() != 3 {
                continue;
            }
            let px = pos_list[0].extract_int().unwrap_or(0);
            let py = pos_list[1].extract_int().unwrap_or(0);
            let pz = pos_list[2].extract_int().unwrap_or(0);

            let state = b
                .get_int("state")
                .or_else(|| b.get("state").and_then(|s| s.extract_int()))
                .unwrap_or(0);

            assert!(
                px >= 0 && px < 256,
                "Block coordinate x out of u8 range: {px}"
            );
            assert!(
                py >= 0 && py < 256,
                "Block coordinate y out of u8 range: {py}"
            );
            assert!(
                pz >= 0 && pz < 256,
                "Block coordinate z out of u8 range: {pz}"
            );
            assert!(
                state >= 0 && state < 256,
                "Palette index out of u8 range: {state}"
            );

            packed_blocks.push(px as u8);
            packed_blocks.push(py as u8);
            packed_blocks.push(pz as u8);
            packed_blocks.push(state as u8);

            if let Some(nbt) = b.get_compound("nbt") {
                let nbt_bytes = pumpkin_nbt::Nbt::from(nbt.clone()).write_unnamed();
                block_entities.push(ParsedBlockEntity {
                    pos: [px, py, pz],
                    nbt: nbt_bytes.to_vec(),
                });
            }
        }
    }

    // 5. entities
    let mut entities = Vec::new();
    if let Some(entities_list) = compound.get_list("entities") {
        for tag in entities_list {
            let pumpkin_nbt::tag::NbtTag::Compound(e) = tag else {
                continue;
            };
            let Some(pos_list) = e.get_list("pos") else {
                continue;
            };
            if pos_list.len() != 3 {
                continue;
            }
            let px = pos_list[0].extract_double().unwrap_or(0.0);
            let py = pos_list[1].extract_double().unwrap_or(0.0);
            let pz = pos_list[2].extract_double().unwrap_or(0.0);

            let Some(block_pos_list) = e.get_list("blockPos") else {
                continue;
            };
            if block_pos_list.len() != 3 {
                continue;
            }
            let bx = block_pos_list[0].extract_int().unwrap_or(0);
            let by = block_pos_list[1].extract_int().unwrap_or(0);
            let bz = block_pos_list[2].extract_int().unwrap_or(0);

            let nbt = e.get_compound("nbt").cloned().unwrap_or_default();
            let nbt_bytes = pumpkin_nbt::Nbt::from(nbt).write_unnamed();

            entities.push(ParsedEntity {
                pos: [px, py, pz],
                block_pos: [bx, by, bz],
                nbt: nbt_bytes.to_vec(),
            });
        }
    }

    Some(ParsedStructureTemplate {
        resource_id,
        bare_id,
        is_default,
        size,
        author,
        palettes,
        packed_blocks,
        block_entities,
        entities,
    })
}

fn scan_structure_dir(
    dir: &Path,
    prefix: &str,
    namespace: &str,
    templates: &mut BTreeMap<String, (bool, String, PathBuf)>,
) {
    let mut entries = fs::read_dir(dir)
        .expect("read structure dir")
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if path.is_dir() {
            let new_prefix = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            scan_structure_dir(&path, &new_prefix, namespace, templates);
            continue;
        }

        if !path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("nbt"))
        {
            continue;
        }

        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("file stem");

        let rel_id = if prefix.is_empty() {
            stem.to_string()
        } else {
            format!("{prefix}/{stem}")
        };

        let resource_id = format!("{namespace}:{rel_id}");
        let is_default = namespace == DEFAULT_NAMESPACE;
        templates.insert(resource_id, (is_default, rel_id, path));
    }
}

pub fn build() -> TokenStream {
    let base_packs = [("vanilla", Path::new("../../assets/datapack"))];
    let container_dir = Path::new("../../assets/tests/datapacks");

    let mut packs = Vec::new();
    for (id, path) in base_packs {
        packs.push(EmbeddedPack {
            id: id.to_string(),
            path: path.to_path_buf(),
        });
    }

    if container_dir.is_dir() {
        let mut entries = fs::read_dir(container_dir)
            .expect("read container dir")
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .collect::<Vec<_>>();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let id = entry.file_name().to_string_lossy().to_string();
            packs.push(EmbeddedPack {
                id,
                path: entry.path(),
            });
        }
    }

    let mut file_map: BTreeMap<String, (bool, String, PathBuf)> = BTreeMap::new();
    let mut embedded_pack_names = Vec::new();

    for pack in &packs {
        let data_dir = pack.path.join("data");
        if !data_dir.is_dir() {
            continue;
        }

        if !embedded_pack_names.iter().any(|id| id == &pack.id) {
            embedded_pack_names.push(pack.id.clone());
        }

        let mut namespaces = fs::read_dir(&data_dir)
            .expect("read data dir")
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .map(|e| (e.file_name().to_string_lossy().to_string(), e.path()))
            .collect::<Vec<_>>();
        namespaces.sort_by(|a, b| a.0.cmp(&b.0));

        for (namespace, ns_dir) in namespaces {
            let struct_dir = ns_dir.join("structure");
            if struct_dir.is_dir() {
                scan_structure_dir(&struct_dir, "", &namespace, &mut file_map);
            }
        }
    }

    let files: Vec<(String, (bool, String, PathBuf))> = file_map.into_iter().collect();

    let mut parsed_templates: Vec<ParsedStructureTemplate> = files
        .par_iter()
        .filter_map(|(res_id, (is_default, bare_id, path))| {
            parse_template_nbt(res_id.clone(), bare_id.clone(), *is_default, path)
        })
        .collect();

    parsed_templates.sort_by(|a, b| a.resource_id.cmp(&b.resource_id));

    let blocks_assets: MinimalBlockAssets =
        serde_json::from_str(&fs::read_to_string("../../assets/blocks.json").unwrap())
            .expect("Failed to parse blocks.json");
    let block_map: BTreeMap<String, MinimalBlock> = blocks_assets
        .blocks
        .into_iter()
        .map(|b| (b.name.clone(), b))
        .collect();

    let generated_properties: Vec<MinimalProperty> =
        serde_json::from_str(&fs::read_to_string("../../assets/properties.json").unwrap())
            .expect("Failed to parse properties.json");
    let prop_map: BTreeMap<i32, MinimalProperty> = generated_properties
        .into_iter()
        .map(|p| (p.hash_key, p))
        .collect();

    let mut definitions = Vec::with_capacity(parsed_templates.len());
    let mut const_refs = Vec::with_capacity(parsed_templates.len());
    let mut index: Vec<(&str, usize)> = Vec::with_capacity(parsed_templates.len() * 2);

    for (i, tmpl) in parsed_templates.iter().enumerate() {
        let tmpl_ident = format_ident!("TEMPLATE_{}", i);
        let [sx, sy, sz] = tmpl.size;
        let author = &tmpl.author;
        let packed_blocks_lit = Literal::byte_string(&tmpl.packed_blocks);

        let palette_tokens: Vec<TokenStream> = tmpl
            .palettes
            .iter()
            .map(|pal| {
                let entries: Vec<TokenStream> = pal
                    .iter()
                    .map(|entry| {
                        let name = &entry.name;
                        let state_id =
                            resolve_state_id(&block_map, &prop_map, name, &entry.properties);
                        let props: Vec<TokenStream> = entry
                            .properties
                            .iter()
                            .map(|(k, v)| crate::processor_list::prop_tokens(k, v))
                            .collect();
                        quote! {
                            StaticPaletteEntry {
                                name: #name,
                                properties: &[#(#props),*],
                                state_id: crate::BlockStateId::new_or_air(#state_id),
                            }
                        }
                    })
                    .collect();
                quote! {
                    &[#(#entries),*]
                }
            })
            .collect();

        let block_entities_tokens: Vec<TokenStream> = tmpl
            .block_entities
            .iter()
            .map(|be| {
                let [x, y, z] = be.pos;
                let nbt_lit = Literal::byte_string(&be.nbt);
                quote! {
                    StaticBlockEntity {
                        pos: [#x, #y, #z],
                        nbt: #nbt_lit,
                    }
                }
            })
            .collect();

        let entities_tokens: Vec<TokenStream> = tmpl
            .entities
            .iter()
            .map(|e| {
                let [x, y, z] = e.pos;
                let [bx, by, bz] = e.block_pos;
                let nbt_lit = Literal::byte_string(&e.nbt);
                quote! {
                    StaticEntity {
                        pos: [#x, #y, #z],
                        block_pos: [#bx, #by, #bz],
                        nbt: #nbt_lit,
                    }
                }
            })
            .collect();

        definitions.push(quote! {
            static #tmpl_ident: StaticStructureTemplate = StaticStructureTemplate {
                size: [#sx, #sy, #sz],
                author: #author,
                palettes: &[#(#palette_tokens),*],
                packed_blocks: #packed_blocks_lit,
                block_entities: &[#(#block_entities_tokens),*],
                entities: &[#(#entities_tokens),*],
            };
        });

        const_refs.push(quote! { &#tmpl_ident });

        index.push((&tmpl.resource_id, i));
        if tmpl.is_default {
            index.push((&tmpl.bare_id, i));
        }
    }

    index.sort_by_key(|&(key, _)| key);
    let index_len = index.len();
    let data_len = parsed_templates.len();
    let index_rows = index.iter().map(|(key, i)| quote! { (#key, #i) });

    let names: Vec<&str> = parsed_templates
        .iter()
        .map(|t| t.resource_id.as_str())
        .collect();
    let pack_names: Vec<&str> = embedded_pack_names.iter().map(String::as_str).collect();

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct StaticPaletteEntry {
            pub name: &'static str,
            pub properties: &'static [crate::block_properties::Property],
            pub state_id: crate::BlockStateId,
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct StaticBlockEntity {
            pub pos: [i32; 3],
            pub nbt: &'static [u8],
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct StaticEntity {
            pub pos: [f64; 3],
            pub block_pos: [i32; 3],
            pub nbt: &'static [u8],
        }

        #[derive(Clone, Copy, Debug)]
        pub struct StaticStructureTemplate {
            pub size: [i32; 3],
            pub author: &'static str,
            pub palettes: &'static [&'static [StaticPaletteEntry]],
            pub packed_blocks: &'static [u8],
            pub block_entities: &'static [StaticBlockEntity],
            pub entities: &'static [StaticEntity],
        }

        #(#definitions)*

        static TEMPLATES: [&StaticStructureTemplate; #data_len] = [
            #(#const_refs),*
        ];

        static TEMPLATE_INDEX: [(&str, usize); #index_len] = [
            #(#index_rows),*
        ];

        #[must_use]
        pub fn get_structure_template(path: &str) -> Option<&'static StaticStructureTemplate> {
            TEMPLATE_INDEX
                .binary_search_by_key(&path, |&(key, _)| key)
                .ok()
                .map(|i| TEMPLATES[TEMPLATE_INDEX[i].1])
        }

        #[must_use]
        #[allow(clippy::too_many_lines, clippy::large_stack_arrays)]
        pub const fn all_template_names() -> &'static [&'static str] {
            &[
                #( #names ),*
            ]
        }

        #[must_use]
        #[allow(clippy::too_many_lines, clippy::large_stack_arrays)]
        pub const fn all_embedded_datapack_names() -> &'static [&'static str] {
            &[
                #( #pack_names ),*
            ]
        }
    }
}
