use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::remap::{MappingNode, ParsedMappings, Remapper};
use crate::version::MinecraftVersion;

pub fn build() -> TokenStream {
    let node_1_21_2 = MappingNode {
        version: MinecraftVersion::V_1_21_2,
        value: "../assets/viaversion/data/mappings-1.21.2to1.21.4.nbt",
        child: None,
    };
    let node_1_21_4 = MappingNode {
        version: MinecraftVersion::V_1_21_4,
        value: "../assets/viaversion/data/mappings-1.21.4to1.21.5.nbt",
        child: Some(&node_1_21_2),
    };
    let node_1_21_5 = MappingNode {
        version: MinecraftVersion::V_1_21_5,
        value: "../assets/viaversion/data/mappings-1.21.5to1.21.6.nbt",
        child: Some(&node_1_21_4),
    };
    let node_1_21_6 = MappingNode {
        version: MinecraftVersion::V_1_21_6,
        value: "../assets/viaversion/data/mappings-1.21.6to1.21.7.nbt",
        child: Some(&node_1_21_5),
    };
    let node_1_21_7 = MappingNode {
        version: MinecraftVersion::V_1_21_7,
        value: "../assets/viaversion/data/mappings-1.21.7to1.21.9.nbt",
        child: Some(&node_1_21_6),
    };
    let node_1_21_9 = MappingNode {
        version: MinecraftVersion::V_1_21_9,
        value: "../assets/viaversion/data/mappings-1.21.9to1.21.11.nbt",
        child: Some(&node_1_21_7),
    };

    let remapper: Remapper<_, Option<Vec<u16>>> = Remapper {
        version: MinecraftVersion::V_1_21_11,
        remapper: |first, second| match (first, second) {
            (Some(first), Some(second)) => Some(
                first
                    .iter()
                    .map(|id| second.get(usize::from(*id)).copied().unwrap_or(0))
                    .collect(),
            ),
            (None, Some(second)) => Some(
                (0..second.len())
                    .map(|id| second.get(id).copied().unwrap_or(0))
                    .collect(),
            ),
            (Some(first), None) => Some(first.clone()),
            _ => None,
        },
        serializer: |&file| {
            ParsedMappings::parse_mapping_file(file, "entities")
                .map(|mappings| mappings.invert_with_default_to_u16(file))
        },
    };

    let all_mappings = remapper.process(&node_1_21_9);

    let mut static_values = TokenStream::new();
    let mut match_arms = TokenStream::new();

    for (ver, mapping) in &all_mappings {
        let ident = format_ident!(
            "{}",
            format!("ENTITY_ID_REMAP_{:?}_TO_{:?}", remapper.version, ver).to_uppercase()
        );
        let mapping_tokens: Vec<_> = mapping
            .as_ref()
            .unwrap()
            .iter()
            .copied()
            .map(Literal::u16_unsuffixed)
            .collect();
        static_values.extend(quote! {
            pub static #ident: &[u16] = &[#(#mapping_tokens),*];
        });
        match_arms.extend(quote! {
            #ver => #ident
                .get(usize::from(entity_id))
                .copied()
                .unwrap_or(0),
        });
    }

    quote! {
        #static_values

        #[must_use]
        pub fn remap_entity_id_for_version(entity_id: u16, version: pumpkin_util::version::MinecraftVersion) -> u16 {
            match version {
                #match_arms
                _ => entity_id,
            }
        }
    }
}
