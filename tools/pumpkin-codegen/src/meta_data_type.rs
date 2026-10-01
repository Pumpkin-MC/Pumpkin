use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

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

/// Generates the `TokenStream` for the `MetaDataType` struct for Minecraft 26.3.
pub fn build() -> TokenStream {
    let path = "../../assets/meta_data_type.json";
    let parsed: BTreeMap<String, i32> = serde_json::from_str(
        &fs::read_to_string(path).expect("Failed to read meta_data_type.json"),
    )
    .expect("Failed to parse meta_data_type.json");

    let mut handlers_map: BTreeMap<String, i32> = BTreeMap::new();
    for (name, id) in parsed {
        let canonical = canonicalize_type_name(&name);
        handlers_map.insert(canonical, id);
    }

    let mut variants = TokenStream::new();
    for (name, id) in &handlers_map {
        let ident = format_ident!("{}", name.to_uppercase());
        variants.extend(quote! {
            pub const #ident: MetaDataType = MetaDataType { id: #id };
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
            pub id: i32,
        }

        impl MetaDataType {
            #variants

            #aliases

            #[must_use]
            pub const fn id(&self, _version: pumpkin_util::version::JavaMinecraftVersion) -> i32 {
                self.id
            }
        }
    }
}
