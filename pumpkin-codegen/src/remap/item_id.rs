use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::remap::{MappingNode, ParsedMappings, Remapper};
use crate::version::JavaMinecraftVersion;

/// Generates the `TokenStream` for per-version item ID remap tables and the
/// `remap_item_id_for_version`/`remap_item_id_from_version` functions.
pub fn build() -> TokenStream {
    // ViaBackwards mappings go new → old (26.1 → 1.21.11 → ... → 1.20.5)
    // This is the correct direction for a new server sending to old clients.
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
            ParsedMappings::parse_mapping_file(file, "items").map(|mappings| mappings.to_u16(file))
        },
    };

    let all_mappings = remapper.process(&node_26_1);
    let mapping_size = all_mappings
        .iter()
        .flat_map(|(_, mapping)| {
            mapping
                .as_ref()
                .map(|m| m.iter().copied().max().unwrap_or(0))
        })
        .max()
        .unwrap_or(0) as usize
        + 1;

    let mut static_values = TokenStream::new();
    let mut match_arms_id_for_ver = TokenStream::new();
    let mut match_arms_id_from_ver = TokenStream::new();

    for (ver, mapping) in &all_mappings {
        // Forward: 26.1 → old version (for sending to old clients)
        {
            let ident = format_ident!(
                "{}",
                format!("ITEM_ID_REMAP_{:?}_TO_{:?}", remapper.version, ver).to_uppercase()
            );
            let mapping_tokens: Vec<_> = mapping
                .as_ref()
                .unwrap()
                .iter()
                .copied()
                .map(Literal::u16_unsuffixed)
                .collect();
            static_values.extend(quote! {
                const #ident: &[u16] = &[#(#mapping_tokens),*];
            });
            match_arms_id_for_ver.extend(quote! {
                #ver => #ident
                    .get(usize::from(item_id))
                    .copied()
                    .unwrap_or(item_id),
            });
        }
        // Reverse: old version → 26.1 (for receiving from old clients)
        {
            let reversed = reverse_mapping(mapping.as_ref().unwrap(), mapping_size);
            let ident = format_ident!(
                "{}",
                format!("ITEM_ID_REMAP_{:?}_TO_{:?}", ver, remapper.version).to_uppercase()
            );
            let mapping_tokens: Vec<_> =
                reversed.into_iter().map(Literal::u16_unsuffixed).collect();
            static_values.extend(quote! {
                const #ident: &[u16] = &[#(#mapping_tokens),*];
            });
            match_arms_id_from_ver.extend(quote! {
                #ver => #ident
                    .get(usize::from(item_id))
                    .copied()
                    .unwrap_or(item_id),
            });
        }
    }

    quote! {
        use pumpkin_util::version::JavaMinecraftVersion;

        #static_values

        #[must_use]
        pub fn remap_item_id_for_version(item_id: u16, version: JavaMinecraftVersion) -> u16 {
            match version {
                #match_arms_id_for_ver
                _ => item_id,
            }
        }

        #[must_use]
        pub fn remap_item_id_from_version(item_id: u16, version: JavaMinecraftVersion) -> u16 {
            match version {
                #match_arms_id_from_ver
                _ => item_id,
            }
        }
    }
}

fn reverse_mapping(mapping: &[u16], mapped_size: usize) -> Vec<u16> {
    let mut result = vec![0u16; mapped_size];
    for (new_id, old_id) in mapping.iter().enumerate() {
        let old_id = *old_id as usize;
        if old_id != 0 && old_id < mapped_size {
            result[old_id] = new_id as u16;
        }
    }
    result
}
