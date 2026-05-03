use std::{fs, path::Path};

use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::LitStr;

/// `rolls` can be a bare float or an object with `type/min/max`.
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum RollsStruct {
    Constant(f32),
    Provider {
        #[serde(rename = "type")]
        provider_type: String,
        #[allow(dead_code)]
        #[serde(default)]
        min: f32,
        #[serde(default)]
        max: f32,
    },
}

impl RollsStruct {
    fn min(&self) -> i32 {
        match self {
            Self::Constant(v) => v.round() as i32,
            Self::Provider { min, .. } => min.round() as i32,
        }
    }
    fn max(&self) -> i32 {
        match self {
            Self::Constant(v) => v.round() as i32,
            Self::Provider { max, .. } => max.round() as i32,
        }
    }
}

/// A `set_count` count provider (uniform or constant).
#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
enum CountStruct {
    Constant(f32),
    Provider {
        #[serde(rename = "type")]
        #[allow(dead_code)]
        provider_type: String,
        #[serde(default)]
        min: f32,
        #[serde(default)]
        max: f32,
    },
}

impl CountStruct {
    fn min(&self) -> i32 {
        match self {
            Self::Constant(v) => v.round() as i32,
            Self::Provider { min, .. } => min.round() as i32,
        }
    }
    fn max(&self) -> i32 {
        match self {
            Self::Constant(v) => v.round() as i32,
            Self::Provider { max, .. } => max.round() as i32,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
struct EntryFunctionStruct {
    function: String,
    count: Option<CountStruct>,
}

/// A single entry inside a pool.
#[derive(Deserialize, Clone, Debug)]
struct PoolEntryStruct {
    #[serde(rename = "type")]
    entry_type: String,
    /// Item name (only for `minecraft:item`).
    name: Option<String>,
    /// Weight (default 1 if absent).
    #[serde(default = "default_weight")]
    weight: i32,
    /// Optional list of functions.
    #[serde(default)]
    functions: Vec<EntryFunctionStruct>,
}

fn default_weight() -> i32 {
    1
}

#[derive(Deserialize, Clone, Debug)]
struct PoolStruct {
    entries: Vec<PoolEntryStruct>,
    rolls: RollsStruct,
}

/// Top-level chest loot table JSON.
#[derive(Deserialize, Clone, Debug)]
struct ChestLootTableJson {
    pools: Vec<PoolStruct>,
}

/// Convert a relative path (e.g. `"trial_chambers/entrance"`) to a Minecraft
/// namespaced key (e.g. `"minecraft:chests/trial_chambers/entrance"`).
fn path_to_key(relative: &str) -> String {
    format!("minecraft:chests/{relative}")
}

/// Convert a file stem path to a valid Rust SCREAMING_SNAKE_CASE identifier prefix.
/// e.g. `"trial_chambers/entrance"` -> `"TRIAL_CHAMBERS_ENTRANCE"`
fn path_to_ident(relative: &str) -> String {
    relative.replace('/', "_").to_shouty_snake_case()
}

/// Read every chest loot JSON from `../assets/loot_table/chests/` (recursively)
/// and emit a `pumpkin-data/src/generated/chest_loot.rs` with static constants
/// and a `get_chest_loot_table(key) -> Option<&'static ChestLootTable>` function.
pub fn build() -> TokenStream {
    let base = Path::new("../assets/loot_table/chests");

    // Collect all JSON files recursively, sorted for deterministic output.
    let mut files: Vec<(String, ChestLootTableJson)> = collect_json_files(base, base);
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut all_tokens = TokenStream::new();

    // Emit one set of statics per file
    let mut table_idents = Vec::new();
    let mut table_keys = Vec::new();

    for (relative_path, table) in &files {
        let prefix = path_to_ident(relative_path);
        let key = path_to_key(relative_path);
        let table_ident = format_ident!("{}", prefix);

        let pool_tokens = emit_table(&prefix, table, &mut all_tokens);

        let pools_ident = format_ident!("{}_POOLS", prefix);
        all_tokens.extend(quote! {
            static #pools_ident: &[ChestLootPool] = &[#(#pool_tokens),*];
            pub static #table_ident: ChestLootTable = ChestLootTable { pools: #pools_ident };
        });

        table_idents.push(table_ident);
        table_keys.push(LitStr::new(&key, Span::call_site()));
    }

    // Emit get_chest_loot_table
    all_tokens.extend(quote! {
        pub fn get_chest_loot_table(key: &str) -> Option<&'static ChestLootTable> {
            match key {
                #(#table_keys => Some(&#table_idents),)*
                _ => None,
            }
        }
    });

    quote! {
        pub use pumpkin_util::chest_loot_table::*;
        #all_tokens
    }
}

/// Emit static entry arrays and pool literals for one table.
/// Returns the list of `ChestLootPool` literals (one per pool).
fn emit_table(
    prefix: &str,
    table: &ChestLootTableJson,
    tokens: &mut TokenStream,
) -> Vec<TokenStream> {
    let mut pool_literals = Vec::new();

    for (pool_idx, pool) in table.pools.iter().enumerate() {
        let min_rolls = pool.rolls.min();
        let max_rolls = pool.rolls.max();

        let mut entry_literals = Vec::new();
        let mut empty_weight: i32 = 0;

        for entry in &pool.entries {
            match entry.entry_type.as_str() {
                "minecraft:empty" => {
                    empty_weight += entry.weight;
                }
                "minecraft:item" => {
                    let name = match &entry.name {
                        Some(n) => n.clone(),
                        None => continue, // malformed, skip
                    };
                    let weight = entry.weight;

                    // Extract min/max count from the first `set_count` function.
                    let (min_count, max_count) = entry
                        .functions
                        .iter()
                        .find(|f| f.function == "minecraft:set_count")
                        .and_then(|f| f.count.as_ref())
                        .map(|c| (c.min(), c.max()))
                        .unwrap_or((1, 1));

                    let name_lit = LitStr::new(&name, Span::call_site());
                    entry_literals.push(quote! {
                        ChestLootEntry {
                            item: #name_lit,
                            weight: #weight,
                            min_count: #min_count,
                            max_count: #max_count,
                        }
                    });
                }
                _ => {} // skip loot_table refs, dynamic, etc.
            }
        }

        // Emit the entries static array.
        let entries_ident = format_ident!("{}_POOL{}_ENTRIES", prefix, pool_idx);
        tokens.extend(quote! {
            static #entries_ident: &[ChestLootEntry] = &[#(#entry_literals),*];
        });

        pool_literals.push(quote! {
            ChestLootPool {
                entries: #entries_ident,
                min_rolls: #min_rolls,
                max_rolls: #max_rolls,
                empty_weight: #empty_weight,
            }
        });
    }

    pool_literals
}

/// Recursively collect all `*.json` files under `dir`, returning a vec of
/// `(relative_stem_path, parsed_table)`.
fn collect_json_files(base: &Path, dir: &Path) -> Vec<(String, ChestLootTableJson)> {
    let mut result = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return result,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            result.extend(collect_json_files(base, &path));
        } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
            // Build relative stem: e.g. "trial_chambers/entrance"
            let rel = path.strip_prefix(base).unwrap();
            let stem = rel.with_extension("");
            // Use forward slashes on all platforms.
            let relative = stem
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");

            let raw = match fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Warning: could not read {}: {e}", path.display());
                    continue;
                }
            };

            match serde_json::from_str::<ChestLootTableJson>(&raw) {
                Ok(table) => result.push((relative, table)),
                Err(e) => eprintln!("Warning: could not parse {}: {e}", path.display()),
            }
        }
    }

    result
}
