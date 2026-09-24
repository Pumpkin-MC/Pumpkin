use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use rayon::prelude::*;
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

struct ParsedJigsaw {
    pos: [i32; 3],
    name: String,
    target: String,
    pool: String,
    final_state: String,
    joint: String,
    facing: String,
    up: String,
    selection_priority: i32,
    placement_priority: i32,
}

struct ParsedTemplate {
    id: String,
    const_name: String,
    size: [i32; 3],
    jigsaws: Vec<ParsedJigsaw>,
}

fn collect_nbt_files(dir: &Path, prefix: &str, files: &mut Vec<(String, PathBuf)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            let next_prefix = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            collect_nbt_files(&path, &next_prefix, files);
        } else if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("nbt"))
        {
            let stem = path.file_stem().unwrap().to_string_lossy();
            let template_id = if prefix.is_empty() {
                stem.to_string()
            } else {
                format!("{prefix}/{stem}")
            };
            files.push((template_id, path));
        }
    }
}

fn parse_template_nbt(id: String, path: &Path) -> Option<ParsedTemplate> {
    let bytes = fs::read(path).ok()?;
    let mut cursor = Cursor::new(bytes);
    let compound = pumpkin_nbt::nbt_compress::read_gzip_compound_tag(&mut cursor).ok()?;

    // 1. size
    let size_list = compound.get_list("size")?;
    if size_list.len() != 3 {
        return None;
    }
    let size_x = size_list[0].extract_int()?;
    let size_y = size_list[1].extract_int()?;
    let size_z = size_list[2].extract_int()?;
    let size = [size_x, size_y, size_z];

    // 2. palette
    let palette_list = if let Some(p) = compound.get_list("palette") {
        p
    } else if let Some(p_list) = compound.get_list("palettes") {
        let first = p_list.first()?;
        if let pumpkin_nbt::tag::NbtTag::List(list) = first {
            list
        } else {
            return None;
        }
    } else {
        return None;
    };

    struct PaletteEntryInfo {
        is_jigsaw: bool,
        facing: String,
        up: String,
    }

    let mut palette_entries = Vec::new();
    for tag in palette_list {
        let pumpkin_nbt::tag::NbtTag::Compound(entry) = tag else {
            palette_entries.push(PaletteEntryInfo {
                is_jigsaw: false,
                facing: "north".to_string(),
                up: "up".to_string(),
            });
            continue;
        };

        let block_name = entry
            .get_string("id")
            .or_else(|| entry.get_string("Name"))
            .unwrap_or_default();
        let is_jigsaw = block_name == "minecraft:jigsaw";

        let (facing, up) = if is_jigsaw {
            let orientation = entry
                .get_compound("properties")
                .or_else(|| entry.get_compound("Properties"))
                .and_then(|p| p.get_string("orientation"));
            if let Some(orientation) = orientation {
                let mut parts = orientation.split('_');
                let f = parts.next().unwrap_or("north").to_string();
                let u = parts.next().unwrap_or("up").to_string();
                (f, u)
            } else {
                ("north".to_string(), "up".to_string())
            }
        } else {
            ("north".to_string(), "up".to_string())
        };

        palette_entries.push(PaletteEntryInfo {
            is_jigsaw,
            facing,
            up,
        });
    }

    // 3. blocks
    let mut jigsaws = Vec::new();
    if let Some(blocks_list) = compound.get_list("blocks") {
        for tag in blocks_list {
            let pumpkin_nbt::tag::NbtTag::Compound(b) = tag else {
                continue;
            };
            let state_idx = b
                .get_int("state")
                .or_else(|| b.get("state").and_then(|s| s.extract_int()))
                .unwrap_or(-1) as usize;
            if state_idx >= palette_entries.len() || !palette_entries[state_idx].is_jigsaw {
                continue;
            }

            let Some(pos_list) = b.get_list("pos") else {
                continue;
            };
            if pos_list.len() != 3 {
                continue;
            }
            let px = pos_list[0].extract_int().unwrap_or(0);
            let py = pos_list[1].extract_int().unwrap_or(0);
            let pz = pos_list[2].extract_int().unwrap_or(0);

            let nbt = b.get_compound("nbt");
            let name = nbt
                .and_then(|n| n.get_string("name"))
                .unwrap_or_default()
                .to_string();
            let target = nbt
                .and_then(|n| n.get_string("target"))
                .unwrap_or_default()
                .to_string();
            let pool = nbt
                .and_then(|n| n.get_string("pool"))
                .unwrap_or_default()
                .to_string();
            let final_state = nbt
                .and_then(|n| n.get_string("final_state"))
                .unwrap_or_default()
                .to_string();
            let joint = nbt
                .and_then(|n| n.get_string("joint"))
                .unwrap_or_default()
                .to_string();
            let selection_priority = nbt
                .and_then(|n| n.get_int("selection_priority"))
                .unwrap_or(0);
            let placement_priority = nbt
                .and_then(|n| n.get_int("placement_priority"))
                .unwrap_or(0);

            let pal = &palette_entries[state_idx];

            jigsaws.push(ParsedJigsaw {
                pos: [px, py, pz],
                name,
                target,
                pool,
                final_state,
                joint,
                facing: pal.facing.clone(),
                up: pal.up.clone(),
                selection_priority,
                placement_priority,
            });
        }
    }

    let const_name = id.to_uppercase().replace(['/', '-', ':', '.'], "_");

    Some(ParsedTemplate {
        id,
        const_name,
        size,
        jigsaws,
    })
}

fn direction_to_tokens(dir: &str) -> TokenStream {
    match dir {
        "down" => quote!(pumpkin_util::BlockDirection::Down),
        "up" => quote!(pumpkin_util::BlockDirection::Up),
        "north" => quote!(pumpkin_util::BlockDirection::North),
        "south" => quote!(pumpkin_util::BlockDirection::South),
        "west" => quote!(pumpkin_util::BlockDirection::West),
        "east" => quote!(pumpkin_util::BlockDirection::East),
        _ => quote!(pumpkin_util::BlockDirection::North),
    }
}

fn joint_to_tokens(joint: &str) -> TokenStream {
    match joint {
        "aligned" => quote!(StaticJigsawJointType::Aligned),
        _ => quote!(StaticJigsawJointType::Rollable),
    }
}

pub fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../../assets/datapack/data/minecraft/structure");

    let root_dir = Path::new("../../assets/datapack/data/minecraft/structure");
    let mut files = Vec::new();
    collect_nbt_files(root_dir, "", &mut files);

    let mut parsed_templates: Vec<ParsedTemplate> = files
        .par_iter()
        .filter_map(|(id, path)| parse_template_nbt(id.clone(), path))
        .collect();

    parsed_templates.sort_by(|a, b| a.id.cmp(&b.id));

    let mut constants = Vec::new();
    let mut match_arms = Vec::new();
    let mut all_names = Vec::new();

    for tmpl in &parsed_templates {
        let const_ident = format_ident!("{}", tmpl.const_name);
        let id_str = &tmpl.id;
        let full_id = format!("minecraft:{id_str}");
        let [sx, sy, sz] = tmpl.size;

        let jigsaw_tokens: Vec<TokenStream> = tmpl
            .jigsaws
            .iter()
            .map(|j| {
                let [jx, jy, jz] = j.pos;
                let j_name = &j.name;
                let j_target = &j.target;
                let j_pool = &j.pool;
                let j_final_state = &j.final_state;
                let j_joint = joint_to_tokens(&j.joint);
                let j_facing = direction_to_tokens(&j.facing);
                let j_up = direction_to_tokens(&j.up);
                let j_sel = j.selection_priority;
                let j_place = j.placement_priority;

                quote! {
                    StaticJigsawBlock {
                        pos: [#jx, #jy, #jz],
                        name: #j_name,
                        target: #j_target,
                        pool: #j_pool,
                        final_state: #j_final_state,
                        joint: #j_joint,
                        facing: #j_facing,
                        up: #j_up,
                        selection_priority: #j_sel,
                        placement_priority: #j_place,
                    }
                }
            })
            .collect();

        constants.push(quote! {
            pub const #const_ident: StaticStructureMetadata = StaticStructureMetadata {
                size: [#sx, #sy, #sz],
                jigsaws: &[#(#jigsaw_tokens),*],
            };
        });

        match_arms.push(quote! {
            #id_str | #full_id => Some(&Self::#const_ident),
        });

        all_names.push(full_id);
    }

    quote! {
        /* This file is generated. Do not edit manually. */

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
        pub enum StaticJigsawJointType {
            #[default]
            Rollable,
            Aligned,
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct StaticJigsawBlock {
            pub pos: [i32; 3],
            pub name: &'static str,
            pub target: &'static str,
            pub pool: &'static str,
            pub final_state: &'static str,
            pub joint: StaticJigsawJointType,
            pub facing: pumpkin_util::BlockDirection,
            pub up: pumpkin_util::BlockDirection,
            pub selection_priority: i32,
            pub placement_priority: i32,
        }

        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct StaticStructureMetadata {
            pub size: [i32; 3],
            pub jigsaws: &'static [StaticJigsawBlock],
        }

        pub struct StaticStructureMetadataList;

        impl StaticStructureMetadataList {
            #(#constants)*

            #[must_use]
            pub fn get(id: &str) -> Option<&'static StaticStructureMetadata> {
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
