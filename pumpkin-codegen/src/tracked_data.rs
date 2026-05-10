use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, fs};

use crate::version::MinecraftVersion;

/// The newest protocol version used as the fallback for unknown versions in `TrackedId::get`.
const LATEST_VERSION: MinecraftVersion = MinecraftVersion::V_26_1;

/// Generates the `TokenStream` for `TrackedId`, `TrackedData`, and all per-entity tracking constants.
pub(crate) fn build() -> TokenStream {
    let assets = [
        (MinecraftVersion::V_1_21, "1_21_tracked_data.json"),
        (MinecraftVersion::V_1_21_2, "1_21_2_tracked_data.json"),
        (MinecraftVersion::V_1_21_4, "1_21_4_tracked_data.json"),
        (MinecraftVersion::V_1_21_5, "1_21_5_tracked_data.json"),
        (MinecraftVersion::V_1_21_6, "1_21_6_tracked_data.json"),
        (MinecraftVersion::V_1_21_7, "1_21_7_tracked_data.json"),
        (MinecraftVersion::V_1_21_9, "1_21_9_tracked_data.json"),
        (MinecraftVersion::V_1_21_11, "1_21_11_tracked_data.json"),
        (MinecraftVersion::V_26_1, "26_1_tracked_data.json"),
    ];

    let mut versions = BTreeMap::new();
    for (ver, file) in assets {
        let path = format!("../assets/tracked_data/{file}");

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read JSON file: {path} {e}"));
        let parsed: BTreeMap<String, u8> = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));

        versions.insert(ver, parsed);
    }

    let tracked_data_struct = generate_struct(&versions);
    let constants = generate_consts(&versions);

    quote! {
        use pumpkin_util::version::MinecraftVersion;

        #tracked_data_struct

        pub struct TrackedData;

        impl TrackedData {
            #constants
        }
    }
}

/// Generates the `TrackedId` struct definition with one `u8` field per supported version.
fn generate_struct<T>(versions: &BTreeMap<MinecraftVersion, T>) -> TokenStream {
    // Build struct fields
    let mut struct_fields = TokenStream::new();
    for ver in versions.keys() {
        let ident = ver.to_field_ident();
        struct_fields.extend(quote! {
            pub #ident: u8,
        });
    }

    let latest_field_ident = LATEST_VERSION.to_field_ident();

    // Build match arms
    let mut match_arms = TokenStream::new();
    for ver in versions.keys() {
        let ident = ver.to_field_ident();
        match_arms.extend(quote! {
            #ver => self.#ident,
        });
    }

    quote! {
        pub struct TrackedId {
            #struct_fields
        }

        impl TrackedId {
            pub fn get(&self, version: &MinecraftVersion) -> u8 {
                match version {
                    #match_arms
                    _ => self.#latest_field_ident,
                }
            }
        }

        impl From<TrackedId> for u8 {
            fn from(id: TrackedId) -> u8 {
                id.#latest_field_ident
            }
        }
    }
}

/// Generates `TrackedId` constants for every tracked data key present in the latest version.
fn generate_consts(versions: &BTreeMap<MinecraftVersion, BTreeMap<String, u8>>) -> TokenStream {
    let mut constants = TokenStream::new();
    let mut generated_names = std::collections::HashSet::new();

    let latest_data = versions.get(&LATEST_VERSION).unwrap();
    for name in latest_data.keys() {
        let name_upper = name.to_uppercase();
        let final_name = if let Some(stripped) = name_upper.strip_prefix("DATA_") {
            stripped.to_string()
        } else {
            name_upper.to_string()
        };

        if !generated_names.insert(final_name.clone()) {
            continue;
        }

        let ident = format_ident!("{}", final_name);

        let mut fields = TokenStream::new();
        for (ver, data) in versions.iter() {
            let field_ident = ver.to_field_ident();
            // 255 as an 'Invalid' marker
            let id = data.get(name).copied().unwrap_or(255);
            fields.extend(quote! {
                #field_ident: #id,
            });
        }

        constants.extend(quote! {
            pub const #ident: TrackedId = TrackedId { #fields };
        });
    }

    constants
}
