use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use proc_macro2::TokenStream;
use quote::quote;

// Builder that generates embedded vanilla datapack resources.
pub(crate) fn build() -> TokenStream {
    let mut entries: Vec<(String, String, Vec<u8>)> = Vec::new();

    load_tags(&mut entries);
    load_recipes(&mut entries);
    load_advancements(&mut entries);
    load_loot_tables(&mut entries);

    // Sort for deterministic output
    entries.sort_by(|a, b| (a.1.as_str(), a.0.as_str()).cmp(&(b.1.as_str(), b.0.as_str())));

    // Build the binary blob (concatenated data) and index
    let mut blob: Vec<u8> = Vec::new();
    let mut index: Vec<(String, String, usize, usize)> = Vec::new();

    for (ns, path, data) in &entries {
        let offset = blob.len();
        blob.extend_from_slice(data);
        let len = blob.len() - offset;
        index.push((ns.clone(), path.clone(), offset, len));
    }

    // Write the blob file
    let blob_path = Path::new(crate::OUT_DIR).join("embedded_data.bin");
    if let Err(e) = fs::write(&blob_path, &blob) {
        eprintln!("Warning: Could not write embedded data blob: {e}");
    }

    // Generate the index array: sorted by (ns, path), with (offset, len) into data
    let index_entries: Vec<_> = index
        .iter()
        .map(|(ns, path, offset, len)| {
            quote! {
                Entry { ns: #ns, path: #path, offset: #offset, len: #len }
            }
        })
        .collect();

    // Collect namespaces
    let ns_set: BTreeSet<String> = entries.iter().map(|(ns, _, _)| ns.clone()).collect();
    let ns_literals: Vec<_> = ns_set.iter().map(|ns| quote! { #ns }).collect();

    // Build prefix map for listing: (ns, top_dir) -> sorted paths
    let mut prefix_entries: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for (ns, path, _) in &entries {
        if let Some(slash_pos) = path.find('/') {
            let top_dir = path[..slash_pos].to_string();
            prefix_entries
                .entry((ns.clone(), top_dir))
                .or_default()
                .push(path.clone());
        }
    }

    let prefix_arms: Vec<_> = prefix_entries
        .iter()
        .map(|((ns, prefix), paths)| {
            let path_literals: Vec<_> = paths.iter().map(|p| quote! { #p }).collect();
            quote! {
                (#ns, #prefix) => &[#(#path_literals),*],
            }
        })
        .collect();

    quote! {
        /// Raw embedded data blob.
        static DATA: &[u8] = include_bytes!("embedded_data.bin");

        /// A single entry in the sorted resource index.
        struct Entry {
            ns: &'static str,
            path: &'static str,
            offset: usize,
            len: usize,
        }

        /// Sorted index of all embedded resources.
        static INDEX: &[Entry] = &[#(#index_entries),*];

        /// Look up a vanilla datapack resource by namespace and path.
        #[must_use]
        pub fn get_vanilla_resource(namespace: &str, path: &str) -> Option<&'static [u8]> {
            let idx = INDEX.binary_search_by(|entry| {
                match entry.ns.cmp(namespace) {
                    std::cmp::Ordering::Equal => entry.path.cmp(path),
                    other => other,
                }
            }).ok()?;
            let e = &INDEX[idx];
            Some(&DATA[e.offset..e.offset + e.len])
        }

        /// Returns all namespaces with embedded vanilla data.
        #[must_use]
        pub fn get_vanilla_namespaces() -> &'static [&'static str] {
            &[#(#ns_literals),*]
        }

        /// List resource paths under a given namespace and top-level directory.
        #[must_use]
        pub fn list_vanilla_resources(namespace: &str, prefix: &str) -> &'static [&'static str] {
            let top_dir = prefix.split('/').next().unwrap_or(prefix);
            if top_dir.is_empty() {
                return &[];
            }
            match (namespace, top_dir) {
                #(#prefix_arms)*
                _ => &[],
            }
        }
    }
}

fn load_tags(entries: &mut Vec<(String, String, Vec<u8>)>) {
    let tag_file = Path::new("../assets/tags/26_1_tags.json");
    let Ok(data) = fs::read_to_string(tag_file) else {
        eprintln!("Warning: Could not read tags file, skipping tag embedding");
        return;
    };
    let Ok(tags): Result<BTreeMap<String, BTreeMap<String, Vec<String>>>, _> =
        serde_json::from_str(&data)
    else {
        eprintln!("Warning: Could not parse tags file");
        return;
    };

    for (registry, tag_map) in &tags {
        for (tag_name, values) in tag_map {
            let json_values: Vec<serde_json::Value> = values
                .iter()
                .map(|v| {
                    if v.contains(':') {
                        serde_json::Value::String(v.clone())
                    } else {
                        serde_json::Value::String(format!("minecraft:{v}"))
                    }
                })
                .collect();

            let tag_json = serde_json::json!({ "values": json_values });
            let bytes = serde_json::to_vec(&tag_json).unwrap_or_default();

            // Strip namespace prefix from tag name
            let tag_stem = if let Some((_, stem)) = tag_name.split_once(':') {
                stem
            } else {
                tag_name.as_str()
            };
            entries.push((
                "minecraft".to_string(),
                format!("tags/{registry}/{tag_stem}.json"),
                bytes,
            ));
        }
    }
}

fn load_recipes(entries: &mut Vec<(String, String, Vec<u8>)>) {
    let recipe_file = Path::new("../assets/recipes.json");
    let Ok(data) = fs::read_to_string(recipe_file) else {
        eprintln!("Warning: Could not read recipes file");
        return;
    };
    let Ok(recipes): Result<BTreeMap<String, serde_json::Value>, _> = serde_json::from_str(&data)
    else {
        eprintln!("Warning: Could not parse recipes file");
        return;
    };

    for (recipe_id, recipe_value) in &recipes {
        let path = recipe_id.strip_prefix("minecraft:").unwrap_or(recipe_id);
        let bytes = serde_json::to_vec(recipe_value).unwrap_or_default();
        entries.push((
            "minecraft".to_string(),
            format!("recipe/{path}.json"),
            bytes,
        ));
    }
}

fn load_advancements(entries: &mut Vec<(String, String, Vec<u8>)>) {
    let adv_file = Path::new("../assets/advancements.json");
    let Ok(data) = fs::read_to_string(adv_file) else {
        eprintln!("Warning: Could not read advancements file");
        return;
    };
    let Ok(advancements): Result<BTreeMap<String, serde_json::Value>, _> =
        serde_json::from_str(&data)
    else {
        eprintln!("Warning: Could not parse advancements file");
        return;
    };

    for (adv_id, adv_value) in &advancements {
        let path = adv_id.strip_prefix("minecraft:").unwrap_or(adv_id);
        let bytes = serde_json::to_vec(adv_value).unwrap_or_default();
        entries.push((
            "minecraft".to_string(),
            format!("advancement/{path}.json"),
            bytes,
        ));
    }
}

fn load_loot_tables(entries: &mut Vec<(String, String, Vec<u8>)>) {
    let loot_dir = Path::new("../assets/loot_table/chests");
    collect_loot_files(loot_dir, loot_dir, entries);
}

fn collect_loot_files(base: &Path, dir: &Path, entries: &mut Vec<(String, String, Vec<u8>)>) {
    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_loot_files(base, &path, entries);
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(bytes) = fs::read(&path) {
                if let Ok(rel) = path.strip_prefix(base) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    entries.push((
                        "minecraft".to_string(),
                        format!("loot_table/chests/{rel_str}"),
                        bytes,
                    ));
                }
            }
        }
    }
}
