use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::version::JavaMinecraftVersion;

fn canonicalize_type_name(name: &str) -> String {
    match name {
        "integer" | "int" => "int".to_string(),
        "entity_pose" | "pose" => "pose".to_string(),
        "facing" | "direction" => "direction".to_string(),
        "text_component" | "component" => "component".to_string(),
        "optional_text_component" | "optional_component" => "optional_component".to_string(),
        "optional_int" | "optional_unsigned_int" => "optional_unsigned_int".to_string(),
        "vector_3f" | "vector3" => "vector3".to_string(),
        "quaternion_f" | "quaternion" => "quaternion".to_string(),
        "rotation" | "rotations" => "rotations".to_string(),
        "particle_list" | "particles" => "particles".to_string(),
        "copper_golem_state" | "weathering_copper_state" => "weathering_copper_state".to_string(),
        "profile" | "resolvable_profile" => "resolvable_profile".to_string(),
        "arm" | "humanoid_arm" => "humanoid_arm".to_string(),
        other => other.to_string(),
    }
}

/// Generates the `TokenStream` for the `MetaDataType` struct with per-version ID fields and constants.
pub fn build() -> TokenStream {
    let assets = [
        (JavaMinecraftVersion::V_1_21, "1_21_meta_data_type.json"),
        (JavaMinecraftVersion::V_1_21_2, "1_21_2_meta_data_type.json"),
        (JavaMinecraftVersion::V_1_21_4, "1_21_4_meta_data_type.json"),
        (JavaMinecraftVersion::V_1_21_5, "1_21_5_meta_data_type.json"),
        (JavaMinecraftVersion::V_1_21_6, "1_21_6_meta_data_type.json"),
        (JavaMinecraftVersion::V_1_21_7, "1_21_7_meta_data_type.json"),
        (JavaMinecraftVersion::V_1_21_9, "1_21_9_meta_data_type.json"),
        (
            JavaMinecraftVersion::V_1_21_11,
            "1_21_11_meta_data_type.json",
        ),
        (JavaMinecraftVersion::V_26_1, "26_1_meta_data_type.json"),
        (JavaMinecraftVersion::V_26_2, "26_2_meta_data_type.json"),
    ];

    let mut handlers_map: BTreeMap<String, BTreeMap<JavaMinecraftVersion, i32>> = BTreeMap::new();

    for &(ver, file) in &assets {
        let path = format!("../../assets/meta_data_type/{file}");
        let parsed: BTreeMap<String, i32> =
            serde_json::from_str(&fs::read_to_string(&path).unwrap())
                .unwrap_or_else(|_| panic!("Failed to parse {file}"));
        for (name, id) in parsed {
            let canonical = canonicalize_type_name(&name);
            handlers_map.entry(canonical).or_default().insert(ver, id);
        }
    }

    let mut structure = TokenStream::new();
    let mut to_id_arms = TokenStream::new();
    for (ver, _) in &assets {
        let field_ident = ver.to_field_ident();
        structure.extend(quote! {
            #field_ident: i32,
        });
        to_id_arms.extend(quote! {
            #ver => self.#field_ident,
        });
    }

    let mut variants = TokenStream::new();
    for (name, ids) in &handlers_map {
        let mut fields = TokenStream::new();
        for (ver, _) in &assets {
            let field_ident = ver.to_field_ident();
            let id = ids.get(ver).unwrap_or(&-1);
            fields.extend(quote! {
                #field_ident: #id,
            });
        }
        let ident = format_ident!("{}", name.to_uppercase());
        variants.extend(quote! {
            pub const #ident: MetaDataType = MetaDataType {
                #fields
            };
        });
    }

    let aliases = quote! {
        pub const INTEGER: MetaDataType = Self::INT;
        pub const ENTITY_POSE: MetaDataType = Self::POSE;
        pub const FACING: MetaDataType = Self::DIRECTION;
        pub const TEXT_COMPONENT: MetaDataType = Self::COMPONENT;
        pub const OPTIONAL_TEXT_COMPONENT: MetaDataType = Self::OPTIONAL_COMPONENT;
        pub const OPTIONAL_INT: MetaDataType = Self::OPTIONAL_UNSIGNED_INT;
        pub const VECTOR_3F: MetaDataType = Self::VECTOR3;
        pub const QUATERNION_F: MetaDataType = Self::QUATERNION;
        pub const ROTATION: MetaDataType = Self::ROTATIONS;
        pub const PARTICLE_LIST: MetaDataType = Self::PARTICLES;
        pub const COPPER_GOLEM_STATE: MetaDataType = Self::WEATHERING_COPPER_STATE;
        pub const PROFILE: MetaDataType = Self::RESOLVABLE_PROFILE;
        pub const ARM: MetaDataType = Self::HUMANOID_ARM;
    };

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct MetaDataType {
            #structure
        }

        impl MetaDataType {
            #variants

            #aliases

            pub const fn id(&self, version: pumpkin_util::version::JavaMinecraftVersion) -> i32 {
                match version {
                    #to_id_arms
                    _ => -1i32,
                }
            }
        }
    }
}
