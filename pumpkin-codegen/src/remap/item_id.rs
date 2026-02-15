use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::remap::{MappingNode, ParsedMappings, Remapper};
use crate::version::MinecraftVersion;

fn reverse_mapping(mapping: &[u16], mapped_size: usize) -> Vec<u16> {
    let mut result = vec![0; mapped_size];
    for i in 0..mapped_size {
        result[mapping.get(i).copied().unwrap_or(i as u16) as usize] = i as u16;
    }
    result
}

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
                    .map(|id| second.get(usize::from(*id)).copied().unwrap_or(*id))
                    .collect(),
            ),
            (None, Some(second)) => Some(
                (0..second.len())
                    .map(|id| second.get(id).copied().unwrap_or(id as u16))
                    .collect(),
            ),
            (Some(first), None) => Some(first.clone()),
            _ => None,
        },
        serializer: |&file| {
            ParsedMappings::parse_mapping_file(file, "items")
                .map(|mappings| mappings.invert_with_default_to_u16(file))
        },
    };

    let all_mappings = remapper.process(&node_1_21_9);
    let mapping_size = all_mappings
        .iter()
        .map(|(_, mapping)| mapping.as_ref().map(|x| x.len()).unwrap_or(0))
        .max()
        .unwrap_or(0);
    let mut static_values = TokenStream::new();
    let mut match_arms_id_for_ver = TokenStream::new();
    let mut match_arms_id_from_ver = TokenStream::new();
    for (ver, mapping) in &all_mappings {
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
        use pumpkin_util::version::MinecraftVersion;

        #static_values

        #[must_use]
        pub fn remap_item_id_for_version(item_id: u16, version: MinecraftVersion) -> u16 {
            match version {
                #match_arms_id_for_ver
                _ => item_id,
            }
        }

        #[must_use]
        pub fn remap_item_id_from_version(item_id: u16, version: MinecraftVersion) -> u16 {
            match version {
                #match_arms_id_from_ver
                _ => item_id,
            }
        }
    }
}
