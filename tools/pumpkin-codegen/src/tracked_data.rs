use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, fs};

use crate::version::JavaMinecraftVersion;

/// The newest protocol version used as the fallback for unknown versions in `TrackedId::get`.
const LATEST_VERSION: JavaMinecraftVersion = JavaMinecraftVersion::V_26_2;

/// Generates the `TokenStream` for `TrackedId`, `TrackedData`, and all per-entity tracking constants.
pub(crate) fn build() -> TokenStream {
    let assets = [
        (JavaMinecraftVersion::V_1_21, "1_21_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_2, "1_21_2_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_4, "1_21_4_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_5, "1_21_5_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_6, "1_21_6_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_7, "1_21_7_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_9, "1_21_9_tracked_data.json"),
        (JavaMinecraftVersion::V_1_21_11, "1_21_11_tracked_data.json"),
        (JavaMinecraftVersion::V_26_1, "26_1_tracked_data.json"),
        (JavaMinecraftVersion::V_26_2, "26_2_tracked_data.json"),
    ];

    let mut versions = BTreeMap::new();
    for (ver, file) in assets {
        let path = format!("../../assets/tracked_data/{file}");

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read JSON file: {path} {e}"));
        let mut parsed: BTreeMap<String, u8> = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));

        // The upstream flattened asset loses duplicate Mojang field names.
        // Before 26.1 the wolf tracker was named VARIANT; newer mappings use
        // DATA_VARIANT_ID, which collides with unrelated entity fields in the
        // flattened table. Preserve the legacy value and set the entity-scoped
        // ID for the new mappings explicitly.
        let wolf_variant_id = match ver {
            JavaMinecraftVersion::V_26_1 | JavaMinecraftVersion::V_26_2 => Some(23),
            _ => parsed.get("VARIANT").copied(),
        };
        if let Some(id) = wolf_variant_id {
            parsed.insert("WOLF_VARIANT_ID".to_owned(), id);
        }

        let cat_variant_id = match ver {
            JavaMinecraftVersion::V_26_1 | JavaMinecraftVersion::V_26_2 => Some(20),
            _ => parsed.get("CAT_VARIANT").copied().or(Some(19)),
        };
        if let Some(id) = cat_variant_id {
            parsed.insert("CAT_VARIANT".to_owned(), id);
            parsed.insert("CAT_VARIANT_ID".to_owned(), id);
        }

        let cat_collar_color_id = match ver {
            JavaMinecraftVersion::V_26_1 | JavaMinecraftVersion::V_26_2 => Some(23),
            _ => Some(22),
        };
        if let Some(id) = cat_collar_color_id {
            parsed.insert("CAT_COLLAR_COLOR".to_owned(), id);
            parsed.insert("CAT_COLLAR_COLOR_ID".to_owned(), id);
        }

        let wolf_collar_color_id = match ver {
            JavaMinecraftVersion::V_26_1 | JavaMinecraftVersion::V_26_2 => Some(21),
            _ => Some(20),
        };
        if let Some(id) = wolf_collar_color_id {
            parsed.insert("WOLF_COLLAR_COLOR".to_owned(), id);
            parsed.insert("WOLF_COLLAR_COLOR_ID".to_owned(), id);
        }

        let sound_variant_id = match ver {
            JavaMinecraftVersion::V_26_1 | JavaMinecraftVersion::V_26_2 => Some(24),
            JavaMinecraftVersion::V_1_21_5
            | JavaMinecraftVersion::V_1_21_6
            | JavaMinecraftVersion::V_1_21_7
            | JavaMinecraftVersion::V_1_21_9
            | JavaMinecraftVersion::V_1_21_11 => Some(23),
            _ => None,
        };
        if let Some(id) = sound_variant_id {
            parsed.insert("SOUND_VARIANT".to_owned(), id);
            parsed.insert("SOUND_VARIANT_ID".to_owned(), id);
            parsed.insert("CAT_SOUND_VARIANT_ID".to_owned(), id);
            parsed.insert("WOLF_SOUND_VARIANT_ID".to_owned(), id);
        }

        let is_lying_id = match ver {
            JavaMinecraftVersion::V_26_1 | JavaMinecraftVersion::V_26_2 => Some(21),
            _ => Some(20),
        };
        if let Some(id) = is_lying_id {
            parsed.insert("IS_LYING".to_owned(), id);
            parsed.insert("IN_SLEEPING_POSE".to_owned(), id);
        }

        let relax_state_one_id = match ver {
            JavaMinecraftVersion::V_26_1 | JavaMinecraftVersion::V_26_2 => Some(22),
            _ => Some(21),
        };
        if let Some(id) = relax_state_one_id {
            parsed.insert("RELAX_STATE_ONE".to_owned(), id);
            parsed.insert("HEAD_DOWN".to_owned(), id);
        }

        versions.insert(ver, parsed);
    }

    let tracked_data_struct = generate_struct(&versions);
    let constants = generate_consts(&versions);

    quote! {
        use pumpkin_util::version::JavaMinecraftVersion;

        #tracked_data_struct

        pub struct TrackedData;

        impl TrackedData {
            #constants
        }
    }
}

/// Generates the `TrackedId` struct definition with one `u8` field per supported version.
fn generate_struct<T>(versions: &BTreeMap<JavaMinecraftVersion, T>) -> TokenStream {
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
            pub fn get(&self, version: &JavaMinecraftVersion) -> u8 {
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
fn generate_consts(versions: &BTreeMap<JavaMinecraftVersion, BTreeMap<String, u8>>) -> TokenStream {
    let mut constants = TokenStream::new();

    // Union of all normalized names across every version
    let all_names: std::collections::BTreeSet<String> = versions
        .values()
        .flat_map(|data| data.keys().map(|k| normalize_name(k)))
        .collect();

    for final_name in &all_names {
        let ident = format_ident!("{}", final_name);
        // Some versions prefix keys with DATA_ (Bedrock), others don't (Java)
        // Try both forms so every version resolves correctly
        let prefixed = format!("DATA_{final_name}");
        let aliases: &[&str] = match final_name.as_str() {
            "CUSTOM_NAME_VISIBLE" => &["NAME_VISIBLE"],
            "IS_LYING" => &["IN_SLEEPING_POSE"],
            "IN_SLEEPING_POSE" => &["IS_LYING"],
            "RELAX_STATE_ONE" => &["HEAD_DOWN"],
            "HEAD_DOWN" => &["RELAX_STATE_ONE"],
            "SOUND_VARIANT" => &["SOUND_VARIANT_ID", "DATA_SOUND_VARIANT_ID"],
            "SOUND_VARIANT_ID" => &["SOUND_VARIANT", "DATA_SOUND_VARIANT_ID"],
            _ => &[],
        };

        let mut fields = TokenStream::new();
        for (ver, data) in versions.iter() {
            let field_ident = ver.to_field_ident();
            let id = data
                .get(final_name.as_str())
                .or_else(|| data.get(prefixed.as_str()))
                .or_else(|| aliases.iter().find_map(|alias| data.get(*alias)))
                .copied()
                .unwrap_or(255);
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

fn normalize_name(name: &str) -> String {
    let upper = name.to_uppercase();
    let normalized = upper
        .strip_prefix("DATA_")
        .map_or(upper.clone(), str::to_string);

    // Mojang renamed the shared entity custom-name visibility tracker from
    // NAME_VISIBLE to DATA_CUSTOM_NAME_VISIBLE in newer mappings. Keep one
    // generated constant usable across both naming schemes.
    if normalized == "NAME_VISIBLE" {
        "CUSTOM_NAME_VISIBLE".to_string()
    } else {
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::build;
    use quote::quote;

    #[test]
    fn wolf_variant_keeps_its_entity_specific_v26_2_tracker_id() {
        let generated = build().to_string();

        assert!(generated.contains("WOLF_VARIANT_ID"));
        let wolf = generated
            .split("WOLF_VARIANT_ID")
            .nth(1)
            .expect("wolf tracker constant");
        assert!(wolf.contains("v1_21_11 : 20u8"));
        assert!(wolf.contains("v26_2 : 23u8"));
    }

    #[test]
    fn cat_trackers_keep_their_entity_specific_tracker_ids() {
        let generated = build().to_string();

        assert!(generated.contains("CAT_VARIANT"));
        let cat_variant = generated
            .split("CAT_VARIANT")
            .nth(1)
            .expect("cat variant constant");
        assert!(cat_variant.contains("v1_21_11 : 19u8"));
        assert!(cat_variant.contains("v26_2 : 20u8"));

        assert!(generated.contains("IS_LYING"));
        let is_lying = generated
            .split("IS_LYING")
            .nth(1)
            .expect("is lying constant");
        assert!(is_lying.contains("v1_21_11 : 20u8"));
        assert!(is_lying.contains("v26_2 : 21u8"));

        assert!(generated.contains("RELAX_STATE_ONE"));
        let relax = generated
            .split("RELAX_STATE_ONE")
            .nth(1)
            .expect("relax state one constant");
        assert!(relax.contains("v1_21_11 : 21u8"));
        assert!(relax.contains("v26_2 : 22u8"));

        assert!(generated.contains("CAT_COLLAR_COLOR"));
        let collar = generated
            .split("CAT_COLLAR_COLOR")
            .nth(1)
            .expect("cat collar color constant");
        assert!(collar.contains("v1_21_11 : 22u8"));
        assert!(collar.contains("v26_2 : 23u8"));

        assert!(generated.contains("SOUND_VARIANT"));
        let sound = generated
            .split("SOUND_VARIANT")
            .nth(1)
            .expect("sound variant constant");
        assert!(sound.contains("v1_21_11 : 23u8"));
        assert!(sound.contains("v26_2 : 24u8"));
    }

    #[test]
    fn checked_in_tracker_table_matches_codegen() {
        let checked_in =
            std::fs::read_to_string("../../crates/pumpkin-data/src/generated/tracked_data.rs")
                .expect("checked-in tracked data");
        let parsed = syn::parse_file(&checked_in).expect("valid generated Rust");

        assert_eq!(quote!(#parsed).to_string(), build().to_string());
    }
}
