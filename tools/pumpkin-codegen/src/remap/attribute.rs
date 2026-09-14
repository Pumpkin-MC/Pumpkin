use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::remap::{MappingNode, ParsedMappings, Remapper};
use crate::remap_nodes;
use crate::version::JavaMinecraftVersion;

/// Generates the `TokenStream` for per-version attribute ID remap tables and the
/// `remap_attribute_id_for_version` function.
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
            ParsedMappings::parse_mapping_file(file, "attributes")
                .map(|mappings| mappings.to_u32(file))
        },
    };

    let all_mappings = remap_nodes!(remapper);
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
            .map(Literal::u32_unsuffixed)
            .collect();
        static_values.extend(quote! {
            pub static #ident: &[u32] = &[#(#mapping_tokens),*];
        });
        let versions = crate::remap::version_patterns(*ver);
        match_arms.extend(quote! {
            #(#versions)|* => #ident
                .get(attribute_id as usize)
                .copied()
                .unwrap_or(attribute_id),
        });
    }

    quote! {
        use pumpkin_util::version::JavaMinecraftVersion;

        #static_values

        #[must_use]
        pub fn remap_attribute_id_for_version(
            attribute_id: u32,
            version: JavaMinecraftVersion,
        ) -> u32 {
            match version {
                #match_arms
                _ => attribute_id,
            }
        }
    }
}
