use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

use crate::version::JavaMinecraftVersion;

/// The newest protocol version used as the fallback for unknown versions in `TrackedId::get`.
const LATEST_VERSION: JavaMinecraftVersion = JavaMinecraftVersion::V_26_2;

#[derive(Deserialize)]
struct RawTrackedField {
    id: u8,
    r#type: String,
    #[allow(dead_code)]
    type_id: u8,
}

/// Generates the `TokenStream` for `TrackedId`, `TrackedData`, and all per-entity tracking modules.
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
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(parsed) = serde_json::from_str::<
                BTreeMap<String, BTreeMap<String, RawTrackedField>>,
            >(&content)
            {
                versions.insert(ver, parsed);
            }
        }
    }

    if versions.is_empty() {
        panic!("No tracked data asset files found in assets/tracked_data");
    }

    let tracked_id_struct = generate_tracked_id_struct(&versions);
    let tracked_data_struct = generate_tracked_data_struct();
    let entity_modules = generate_entity_modules(&versions);

    quote! {
        use crate::meta_data_type::MetaDataType;
        use pumpkin_util::version::JavaMinecraftVersion;

        #tracked_id_struct

        #tracked_data_struct

        #entity_modules
    }
}

/// Generates the `TrackedId` struct definition with one `u8` field per supported version.
fn generate_tracked_id_struct<T>(versions: &BTreeMap<JavaMinecraftVersion, T>) -> TokenStream {
    let mut struct_fields = TokenStream::new();
    for ver in versions.keys() {
        let ident = ver.to_field_ident();
        struct_fields.extend(quote! {
            pub #ident: u8,
        });
    }

    let latest_field_ident = if versions.contains_key(&LATEST_VERSION) {
        LATEST_VERSION.to_field_ident()
    } else {
        versions.keys().last().unwrap().to_field_ident()
    };

    let mut match_arms = TokenStream::new();
    for ver in versions.keys() {
        let ident = ver.to_field_ident();
        match_arms.extend(quote! {
            #ver => self.#ident,
        });
    }

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct TrackedId {
            #struct_fields
        }

        impl TrackedId {
            #[must_use]
            pub const fn get(&self, version: &JavaMinecraftVersion) -> u8 {
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

/// Generates the `TrackedData` struct with `id` and `type` fields.
fn generate_tracked_data_struct() -> TokenStream {
    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct TrackedData {
            pub id: TrackedId,
            pub r#type: MetaDataType,
        }

        impl TrackedData {
            #[must_use]
            pub const fn new(id: TrackedId, r#type: MetaDataType) -> Self {
                Self { id, r#type }
            }

            #[must_use]
            pub const fn get(&self, version: &JavaMinecraftVersion) -> u8 {
                self.id.get(version)
            }
        }
    }
}

/// Generates entity-specific modules containing constants for all tracked fields.
fn generate_entity_modules(
    versions: &BTreeMap<JavaMinecraftVersion, BTreeMap<String, BTreeMap<String, RawTrackedField>>>,
) -> TokenStream {
    let mut modules = TokenStream::new();

    let all_entities: BTreeSet<String> = versions
        .values()
        .flat_map(|entities| entities.keys().cloned())
        .collect();

    for entity in &all_entities {
        let entity_ident = format_ident!("{}", entity);

        let all_fields: BTreeSet<String> = versions
            .values()
            .filter_map(|entities| entities.get(entity))
            .flat_map(|fields| fields.keys().cloned())
            .collect();

        let mut field_consts = TokenStream::new();
        let mut defined_idents = BTreeSet::new();

        // 1. Generate base field constants
        for field in &all_fields {
            let field_upper = field.to_uppercase();
            let field_ident = format_ident!("{}", field_upper);
            defined_idents.insert(field_upper.clone());

            let mut id_fields = TokenStream::new();
            let mut latest_type = String::new();

            for (ver, entities) in versions {
                let ver_ident = ver.to_field_ident();
                let field_info = entities.get(entity).and_then(|f| f.get(field));
                let id = field_info.map_or(255u8, |info| info.id);
                if let Some(info) = field_info {
                    latest_type = info.r#type.clone();
                }
                id_fields.extend(quote! {
                    #ver_ident: #id,
                });
            }

            let type_const_ident = format_ident!("{}", latest_type.to_uppercase());

            field_consts.extend(quote! {
                pub const #field_ident: TrackedData = TrackedData {
                    id: TrackedId { #id_fields },
                    r#type: MetaDataType::#type_const_ident,
                };
            });
        }

        // 2. Generate normalized and semantic aliases
        for field in &all_fields {
            let field_upper = field.to_uppercase();
            let field_ident = format_ident!("{}", field_upper);

            let mut candidate_aliases = Vec::new();

            // Strip DATA_ prefix
            if let Some(stripped) = field_upper.strip_prefix("DATA_") {
                candidate_aliases.push(stripped.to_string());
            }

            // Strip _ID suffix
            if let Some(stripped_id) = field_upper.strip_suffix("_ID") {
                candidate_aliases.push(stripped_id.to_string());
                if let Some(norm) = stripped_id.strip_prefix("DATA_") {
                    candidate_aliases.push(norm.to_string());
                }
            }

            // Semantic aliases
            add_semantic_aliases(entity, &field_upper, &mut candidate_aliases);

            for alias in candidate_aliases {
                if !defined_idents.contains(&alias) && is_valid_ident(&alias) {
                    defined_idents.insert(alias.clone());
                    let alias_ident = format_ident!("{}", alias);
                    field_consts.extend(quote! {
                        pub const #alias_ident: TrackedData = #field_ident;
                    });
                }
            }
        }

        modules.extend(quote! {
            pub mod #entity_ident {
                use super::*;

                #field_consts
            }
        });
    }

    modules
}

fn is_valid_ident(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !first.is_ascii_alphabetic() && first != '_' {
        return false;
    }
    if !chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return false;
    }
    !matches!(
        name,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

fn add_semantic_aliases(entity: &str, field: &str, aliases: &mut Vec<String>) {
    match (entity, field) {
        (_, "DATA_FLAGS_ID") => {
            aliases.push("TAMEABLE_FLAGS".to_string());
            aliases.push("FLAGS".to_string());
        }
        (_, "DATA_OWNERUUID_ID") => {
            aliases.push("OWNER_UUID".to_string());
        }
        ("creeper", "DATA_IS_POWERED") => {
            aliases.push("CHARGED".to_string());
        }
        ("creeper", "DATA_SWELL_DIR") => {
            aliases.push("FUSE_ID".to_string());
        }
        ("tnt" | "primed_tnt", "DATA_FUSE_ID") => {
            aliases.push("FUSE_ID".to_string());
        }
        ("sheep", "DATA_WOOL_ID") => {
            aliases.push("WOOL_ID".to_string());
        }
        ("cat", "IS_LYING") => {
            aliases.push("IN_SLEEPING_POSE".to_string());
        }
        ("cat", "RELAX_STATE_ONE") => {
            aliases.push("HEAD_DOWN".to_string());
        }
        ("cat", "DATA_SOUND_VARIANT_ID") | ("wolf", "DATA_SOUND_VARIANT_ID") => {
            aliases.push("SOUND_VARIANT".to_string());
            aliases.push("SOUND_VARIANT_ID".to_string());
        }
        ("cat", "DATA_VARIANT_ID") => {
            aliases.push("CAT_VARIANT".to_string());
            aliases.push("CAT_VARIANT_ID".to_string());
            aliases.push("VARIANT".to_string());
        }
        ("wolf", "DATA_VARIANT_ID") => {
            aliases.push("WOLF_VARIANT_ID".to_string());
            aliases.push("VARIANT".to_string());
        }
        ("cat", "DATA_COLLAR_COLOR") => {
            aliases.push("CAT_COLLAR_COLOR".to_string());
            aliases.push("COLLAR_COLOR".to_string());
        }
        ("wolf", "DATA_COLLAR_COLOR") => {
            aliases.push("WOLF_COLLAR_COLOR".to_string());
            aliases.push("COLLAR_COLOR".to_string());
        }
        ("player" | "avatar" | "mannequin", "DATA_PLAYER_MODE_CUSTOMISATION") => {
            aliases.push("PLAYER_MODE_CUSTOMIZATION_ID".to_string());
        }
        ("player" | "avatar" | "mannequin", "DATA_PLAYER_MAIN_HAND") => {
            aliases.push("MAIN_ARM_ID".to_string());
        }
        ("display" | "block_display" | "item_display" | "text_display", _) => match field {
            "DATA_TRANSFORMATION_INTERPOLATION_START_DELTA_TICKS_ID" => {
                aliases.push("START_INTERPOLATION".to_string());
            }
            "DATA_TRANSFORMATION_INTERPOLATION_DURATION_ID" => {
                aliases.push("INTERPOLATION_DURATION".to_string());
            }
            "DATA_POS_ROT_INTERPOLATION_DURATION_ID" => {
                aliases.push("TELEPORT_DURATION".to_string());
            }
            "DATA_TRANSLATION_ID" => {
                aliases.push("TRANSLATION".to_string());
            }
            "DATA_SCALE_ID" => {
                aliases.push("SCALE".to_string());
            }
            "DATA_LEFT_ROTATION_ID" => {
                aliases.push("LEFT_ROTATION".to_string());
            }
            "DATA_RIGHT_ROTATION_ID" => {
                aliases.push("RIGHT_ROTATION".to_string());
            }
            "DATA_BILLBOARD_RENDER_CONSTRAINTS_ID" => {
                aliases.push("BILLBOARD".to_string());
            }
            "DATA_BRIGHTNESS_OVERRIDE_ID" => {
                aliases.push("BRIGHTNESS".to_string());
            }
            "DATA_VIEW_RANGE_ID" => {
                aliases.push("VIEW_RANGE".to_string());
            }
            "DATA_SHADOW_RADIUS_ID" => {
                aliases.push("SHADOW_RADIUS".to_string());
            }
            "DATA_SHADOW_STRENGTH_ID" => {
                aliases.push("SHADOW_STRENGTH".to_string());
            }
            "DATA_WIDTH_ID" => {
                aliases.push("WIDTH".to_string());
            }
            "DATA_HEIGHT_ID" => {
                aliases.push("HEIGHT".to_string());
            }
            "DATA_GLOW_COLOR_OVERRIDE_ID" => {
                aliases.push("GLOW_COLOR_OVERRIDE".to_string());
            }
            "DATA_BLOCK_STATE_ID" => {
                aliases.push("BLOCK_STATE".to_string());
            }
            "DATA_ITEM_STACK_ID" => {
                aliases.push("ITEM".to_string());
                aliases.push("ITEM_STACK".to_string());
            }
            "DATA_ITEM_DISPLAY_ID" => {
                aliases.push("ITEM_DISPLAY".to_string());
            }
            "DATA_TEXT_ID" => {
                aliases.push("TEXT".to_string());
            }
            "DATA_LINE_WIDTH_ID" => {
                aliases.push("LINE_WIDTH".to_string());
            }
            "DATA_BACKGROUND_COLOR_ID" => {
                aliases.push("BACKGROUND".to_string());
            }
            "DATA_TEXT_OPACITY_ID" => {
                aliases.push("TEXT_OPACITY".to_string());
            }
            "DATA_STYLE_FLAGS_ID" => {
                aliases.push("TEXT_DISPLAY_FLAGS".to_string());
            }
            _ => {}
        },
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::build;
    use quote::quote;

    #[test]
    fn wolf_and_cat_have_correct_entity_specific_tracker_constants() {
        let generated = build().to_string();

        assert!(generated.contains("mod wolf"));
        assert!(generated.contains("mod cat"));
        assert!(generated.contains("DATA_COLLAR_COLOR"));
    }
}
