use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::remap::{MappingNode, ParsedMappings, Remapper};
use crate::version::JavaMinecraftVersion;

const UNMAPPED: u16 = u16::MAX;

/// Generates per-version attribute ID remap tables and the remapping function.
pub fn build() -> TokenStream {
    let node_1_20_5 = MappingNode {
        version: JavaMinecraftVersion::V_1_20_5,
        value: "../assets/viabackwards/data/mappings-1.21to1.20.5.nbt",
        child: None,
    };
    let node_1_21 = MappingNode {
        version: JavaMinecraftVersion::V_1_21,
        value: "../assets/viabackwards/data/mappings-1.21.2to1.21.nbt",
        child: Some(&node_1_20_5),
    };
    let node_1_21_2 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_2,
        value: "../assets/viabackwards/data/mappings-1.21.4to1.21.2.nbt",
        child: Some(&node_1_21),
    };
    let node_1_21_4 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_4,
        value: "../assets/viabackwards/data/mappings-1.21.5to1.21.4.nbt",
        child: Some(&node_1_21_2),
    };
    let node_1_21_5 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_5,
        value: "../assets/viabackwards/data/mappings-1.21.6to1.21.5.nbt",
        child: Some(&node_1_21_4),
    };
    let node_1_21_6 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_6,
        value: "../assets/viabackwards/data/mappings-1.21.7to1.21.6.nbt",
        child: Some(&node_1_21_5),
    };
    let node_1_21_7 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_7,
        value: "../assets/viabackwards/data/mappings-1.21.9to1.21.7.nbt",
        child: Some(&node_1_21_6),
    };
    let node_1_21_9 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_9,
        value: "../assets/viabackwards/data/mappings-1.21.11to1.21.9.nbt",
        child: Some(&node_1_21_7),
    };
    let node_1_21_11 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_11,
        value: "../assets/viabackwards/data/mappings-26.1to1.21.11.nbt",
        child: Some(&node_1_21_9),
    };
    let node_26_1 = MappingNode {
        version: JavaMinecraftVersion::V_26_1,
        value: "../assets/viabackwards/data/mappings-26.2to26.1.nbt",
        child: Some(&node_1_21_11),
    };
    let remapper: Remapper<_, Option<Vec<u16>>> = Remapper {
        version: JavaMinecraftVersion::V_26_2,
        remapper: |first, second| match (first, second) {
            (Some(first), Some(second)) => Some(
                first
                    .iter()
                    .map(|id| {
                        if *id == UNMAPPED {
                            UNMAPPED
                        } else {
                            second.get(usize::from(*id)).copied().unwrap_or(UNMAPPED)
                        }
                    })
                    .collect(),
            ),
            (None, Some(second)) => Some(second.clone()),
            (Some(first), None) => Some(first.clone()),
            _ => None,
        },
        serializer: |&file| {
            ParsedMappings::parse_mapping_file(file, "attributes").map(|mappings| {
                mappings
                    .forward
                    .iter()
                    .map(|id| u16::try_from(*id).unwrap_or(UNMAPPED))
                    .collect()
            })
        },
    };

    let all_mappings = remapper.process(&node_26_1);
    let mut static_values = TokenStream::new();
    let mut match_arms = TokenStream::new();

    for (ver, mapping) in &all_mappings {
        let Some(mapping) = mapping else {
            continue;
        };

        let ident = format_ident!(
            "{}",
            format!("ATTRIBUTE_ID_REMAP_{:?}_TO_{:?}", remapper.version, ver).to_uppercase()
        );
        let mapping_tokens: Vec<_> = mapping
            .iter()
            .copied()
            .map(Literal::u16_unsuffixed)
            .collect();
        static_values.extend(quote! {
            pub static #ident: &[u16] = &[#(#mapping_tokens),*];
        });
        match_arms.extend(quote! {
            #ver => #ident
                .get(usize::from(attribute_id))
                .copied()
                .filter(|id| *id != u16::MAX),
        });
    }

    quote! {
        #static_values

        #[must_use]
        pub fn remap_attribute_id_for_version(
            attribute_id: u16,
            version: pumpkin_util::version::JavaMinecraftVersion,
        ) -> Option<u16> {
            match version {
                #match_arms
                _ => Some(attribute_id),
            }
        }
    }
}
