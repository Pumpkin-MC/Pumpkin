use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::remap::{MappingNode, ParsedMappings, Remapper};
use crate::remap_nodes;
use crate::version::JavaMinecraftVersion;

/// Generates the `TokenStream` for per-version data component type ID remap tables and the
/// `remap_data_component_type_id_for_version`/`remap_data_component_type_id_from_version` functions.
pub fn build() -> TokenStream {
    let remapper: Remapper<_, Option<Vec<u32>>> = Remapper {
        version: JavaMinecraftVersion::V_26_2,
        remapper: |first, second| match (first, second) {
            (Some(first), Some(second)) => Some(
                first
                    .iter()
                    .map(|&id| second.get(id as usize).copied().unwrap_or(id))
                    .collect(),
            ),
            (None, Some(second)) => Some(second.clone()),
            (Some(first), None) => Some(first.clone()),
            (None, None) => None,
        },
        serializer: |&file| {
            ParsedMappings::parse_mapping_file(file, "data_component_type")
                .map(|mappings| mappings.to_u32(file))
        },
    };

    let all_mappings = remap_nodes!(remapper);
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
        let Some(mapping) = mapping else {
            continue;
        };
        let versions = crate::remap::version_patterns(*ver);

        // Forward: 26.2 → old version
        {
            let ident = format_ident!(
                "{}",
                format!(
                    "DATA_COMPONENT_TYPE_ID_REMAP_{:?}_TO_{:?}",
                    remapper.version, ver
                )
                .to_uppercase()
            );
            let mapping_tokens: Vec<_> = mapping
                .iter()
                .copied()
                .map(Literal::u32_unsuffixed)
                .collect();
            static_values.extend(quote! {
                pub static #ident: &[u32] = &[#(#mapping_tokens),*];
            });
            match_arms_id_for_ver.extend(quote! {
                #(#versions)|* => #ident
                    .get(data_component_type_id as usize)
                    .copied()
                    .unwrap_or(data_component_type_id),
            });
        }
        // Reverse: old version → 26.2
        {
            let reversed = reverse_mapping(mapping, mapping_size);
            let ident = format_ident!(
                "{}",
                format!(
                    "DATA_COMPONENT_TYPE_ID_REMAP_{:?}_TO_{:?}",
                    ver, remapper.version
                )
                .to_uppercase()
            );
            let mapping_tokens: Vec<_> =
                reversed.into_iter().map(Literal::u32_unsuffixed).collect();
            static_values.extend(quote! {
                pub static #ident: &[u32] = &[#(#mapping_tokens),*];
            });
            match_arms_id_from_ver.extend(quote! {
                #(#versions)|* => #ident
                    .get(data_component_type_id as usize)
                    .copied()
                    .unwrap_or(data_component_type_id),
            });
        }
    }

    quote! {
        use pumpkin_util::version::JavaMinecraftVersion;

        #static_values

        #[must_use]
        pub fn remap_data_component_type_id_for_version(
            data_component_type_id: u32,
            version: JavaMinecraftVersion,
        ) -> u32 {
            match version {
                #match_arms_id_for_ver
                _ => data_component_type_id,
            }
        }

        #[must_use]
        pub fn remap_data_component_type_id_from_version(
            data_component_type_id: u32,
            version: JavaMinecraftVersion,
        ) -> u32 {
            match version {
                #match_arms_id_from_ver
                _ => data_component_type_id,
            }
        }
    }
}

fn reverse_mapping(mapping: &[u32], mapped_size: usize) -> Vec<u32> {
    let mut result = vec![0u32; mapped_size];
    for (new_id, old_id) in mapping.iter().enumerate() {
        let old_id = *old_id as usize;
        if old_id != 0 && old_id < mapped_size {
            result[old_id] = new_id as u32;
        }
    }
    result
}
