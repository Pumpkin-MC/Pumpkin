use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};

use crate::remap::{MappingNode, ParsedMappings, Remapper};
use crate::version::JavaMinecraftVersion;

/// Generates the `TokenStream` for per-version particle ID remap tables and the
/// `remap_particle_id_for_version` function.
pub fn build() -> TokenStream {
    let node_1_7_6 = MappingNode {
        version: JavaMinecraftVersion::V_1_7_6,
        value: "../../assets/viarewind/data/mappings-1.8to1.7.10.nbt",
        child: None,
    };
    let node_1_8 = MappingNode {
        version: JavaMinecraftVersion::V_1_8,
        value: "../../assets/viarewind/data/mappings-1.9.4to1.8.nbt",
        child: Some(&node_1_7_6),
    };
    let node_1_9 = MappingNode {
        version: JavaMinecraftVersion::V_1_9,
        value: "../../assets/viabackwards/data/mappings-1.10to1.9.4.nbt",
        child: Some(&node_1_8),
    };
    let node_1_10 = MappingNode {
        version: JavaMinecraftVersion::V_1_10,
        value: "../../assets/viabackwards/data/mappings-1.11to1.10.nbt",
        child: Some(&node_1_9),
    };
    let node_1_11 = MappingNode {
        version: JavaMinecraftVersion::V_1_11,
        value: "../../assets/viabackwards/data/mappings-1.12to1.11.nbt",
        child: Some(&node_1_10),
    };
    let node_1_12 = MappingNode {
        version: JavaMinecraftVersion::V_1_12,
        value: "../../assets/viabackwards/data/mappings-1.13to1.12.nbt",
        child: Some(&node_1_11),
    };
    let node_1_13 = MappingNode {
        version: JavaMinecraftVersion::V_1_13,
        value: "../../assets/viabackwards/data/mappings-1.13.2to1.13.nbt",
        child: Some(&node_1_12),
    };
    let node_1_13_2 = MappingNode {
        version: JavaMinecraftVersion::V_1_13_2,
        value: "../../assets/viabackwards/data/mappings-1.14to1.13.2.nbt",
        child: Some(&node_1_13),
    };
    let node_1_14 = MappingNode {
        version: JavaMinecraftVersion::V_1_14,
        value: "../../assets/viabackwards/data/mappings-1.15to1.14.nbt",
        child: Some(&node_1_13_2),
    };
    let node_1_15 = MappingNode {
        version: JavaMinecraftVersion::V_1_15,
        value: "../../assets/viabackwards/data/mappings-1.16to1.15.nbt",
        child: Some(&node_1_14),
    };
    let node_1_16 = MappingNode {
        version: JavaMinecraftVersion::V_1_16,
        value: "../../assets/viabackwards/data/mappings-1.16.2to1.16.nbt",
        child: Some(&node_1_15),
    };
    let node_1_16_2 = MappingNode {
        version: JavaMinecraftVersion::V_1_16_2,
        value: "../../assets/viabackwards/data/mappings-1.17to1.16.2.nbt",
        child: Some(&node_1_16),
    };
    let node_1_17 = MappingNode {
        version: JavaMinecraftVersion::V_1_17,
        value: "../../assets/viabackwards/data/mappings-1.18to1.17.nbt",
        child: Some(&node_1_16_2),
    };
    let node_1_18 = MappingNode {
        version: JavaMinecraftVersion::V_1_18,
        value: "../../assets/viabackwards/data/mappings-1.19to1.18.nbt",
        child: Some(&node_1_17),
    };
    let node_1_19 = MappingNode {
        version: JavaMinecraftVersion::V_1_19,
        value: "../../assets/viabackwards/data/mappings-1.19.3to1.19.nbt",
        child: Some(&node_1_18),
    };
    let node_1_19_3 = MappingNode {
        version: JavaMinecraftVersion::V_1_19_3,
        value: "../../assets/viabackwards/data/mappings-1.19.4to1.19.3.nbt",
        child: Some(&node_1_19),
    };
    let node_1_19_4 = MappingNode {
        version: JavaMinecraftVersion::V_1_19_4,
        value: "../../assets/viabackwards/data/mappings-1.20to1.19.4.nbt",
        child: Some(&node_1_19_3),
    };
    let node_1_20 = MappingNode {
        version: JavaMinecraftVersion::V_1_20,
        value: "../../assets/viabackwards/data/mappings-1.20.2to1.20.nbt",
        child: Some(&node_1_19_4),
    };
    let node_1_20_2 = MappingNode {
        version: JavaMinecraftVersion::V_1_20_2,
        value: "../../assets/viabackwards/data/mappings-1.20.3to1.20.2.nbt",
        child: Some(&node_1_20),
    };
    let node_1_20_3 = MappingNode {
        version: JavaMinecraftVersion::V_1_20_3,
        value: "../../assets/viabackwards/data/mappings-1.20.5to1.20.3.nbt",
        child: Some(&node_1_20_2),
    };
    let node_1_20_5 = MappingNode {
        version: JavaMinecraftVersion::V_1_20_5,
        value: "../../assets/viabackwards/data/mappings-1.21to1.20.5.nbt",
        child: Some(&node_1_20_3),
    };
    let node_1_21 = MappingNode {
        version: JavaMinecraftVersion::V_1_21,
        value: "../../assets/viabackwards/data/mappings-1.21.2to1.21.nbt",
        child: Some(&node_1_20_5),
    };
    let node_1_21_2 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_2,
        value: "../../assets/viabackwards/data/mappings-1.21.4to1.21.2.nbt",
        child: Some(&node_1_21),
    };
    let node_1_21_4 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_4,
        value: "../../assets/viabackwards/data/mappings-1.21.5to1.21.4.nbt",
        child: Some(&node_1_21_2),
    };
    let node_1_21_5 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_5,
        value: "../../assets/viabackwards/data/mappings-1.21.6to1.21.5.nbt",
        child: Some(&node_1_21_4),
    };
    let node_1_21_6 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_6,
        value: "../../assets/viabackwards/data/mappings-1.21.7to1.21.6.nbt",
        child: Some(&node_1_21_5),
    };
    let node_1_21_7 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_7,
        value: "../../assets/viabackwards/data/mappings-1.21.9to1.21.7.nbt",
        child: Some(&node_1_21_6),
    };
    let node_1_21_9 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_9,
        value: "../../assets/viabackwards/data/mappings-1.21.11to1.21.9.nbt",
        child: Some(&node_1_21_7),
    };
    let node_1_21_11 = MappingNode {
        version: JavaMinecraftVersion::V_1_21_11,
        value: "../../assets/viabackwards/data/mappings-26.1to1.21.11.nbt",
        child: Some(&node_1_21_9),
    };
    let node_26_1 = MappingNode {
        version: JavaMinecraftVersion::V_26_1,
        value: "../../assets/viabackwards/data/mappings-26.2to26.1.nbt",
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
            ParsedMappings::parse_mapping_file(file, "particles")
                .map(|mappings| mappings.to_u16(file))
        },
    };

    let all_mappings = remapper.process(&node_26_1);
    let mut static_values = TokenStream::new();
    let mut match_arms = TokenStream::new();
    for (ver, mapping) in &all_mappings {
        let ident = format_ident!(
            "{}",
            format!("PARTICLE_ID_REMAP_{:?}_TO_{:?}", remapper.version, ver).to_uppercase()
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
        let versions = crate::remap::version_patterns(*ver);
        match_arms.extend(quote! {
            #(#versions)|* => #ident
                .get(usize::from(particle_id))
                .copied()
                .unwrap_or(particle_id),
        });
    }

    quote! {
        use pumpkin_util::version::JavaMinecraftVersion;

        #static_values

        #[must_use]
        pub fn remap_particle_id_for_version(
            particle_id: u16,
            version: JavaMinecraftVersion,
        ) -> u16 {
            match version {
                #match_arms
                _ => particle_id,
            }
        }
    }
}
