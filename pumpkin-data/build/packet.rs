use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

use crate::version::MinecraftVersion;

const LATEST_VERSION: MinecraftVersion = MinecraftVersion::V_1_21_11;

#[derive(Deserialize)]
pub struct Packets {
    #[allow(dead_code)]
    version: u32,
    serverbound: BTreeMap<String, BTreeMap<String, i32>>,
    clientbound: BTreeMap<String, BTreeMap<String, i32>>,
}

pub(crate) fn build() -> TokenStream {
    let assets = [
        (MinecraftVersion::V_1_21_4, "1_21_4_packets.json"),
        (MinecraftVersion::V_1_21_5, "1_21_5_packets.json"),
        (MinecraftVersion::V_1_21_6, "1_21_6_packets.json"),
        (MinecraftVersion::V_1_21_7, "1_21_7_packets.json"),
        (MinecraftVersion::V_1_21_9, "1_21_9_packets.json"),
        (MinecraftVersion::V_1_21_11, "1_21_11_packets.json"),
    ];

    // Parse available packet files into a BTreeMap keyed by MinecraftVersion
    let mut versions = BTreeMap::new();
    for (ver, file) in assets {
        let path = format!("../assets/packet/{file}");
        println!("cargo:rerun-if-changed={path}");

        let content = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Failed to read packet JSON file: {path}"));
        let parsed: Packets = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));

        versions.insert(ver, parsed);
    }

    // Generate PacketId struct definition and impl blocks dynamically based on versions
    let packet_id_struct = generate_struct(&versions);
    let serverbound_consts = generate_mapped_consts(&versions, true);
    let clientbound_consts = generate_mapped_consts(&versions, false);

    quote!(
        use pumpkin_util::version::MinecraftVersion;

        pub const CURRENT_MC_VERSION: MinecraftVersion = #LATEST_VERSION;
        pub const LOWEST_SUPPORTED_MC_VERSION: MinecraftVersion = MinecraftVersion::V_1_21_4;

        #packet_id_struct

        // We place the constants directly into these modules
        pub mod serverbound {
            #serverbound_consts
        }

        pub mod clientbound {
            #clientbound_consts
        }
    )
}

/// Generate the `PacketId` struct and impls (including `to_id`) dynamically based on available versions.
fn generate_struct<T>(versions: &BTreeMap<MinecraftVersion, T>) -> TokenStream {
    // Build struct fields
    let mut struct_fields = TokenStream::new();
    for ver in versions.keys() {
        let ident = ver.to_field_ident();
        struct_fields.extend(quote! {
            pub #ident: i32,
        });
    }

    let latest_field_ident = LATEST_VERSION.to_field_ident();

    // Build match arms
    let mut match_arms = TokenStream::new();
    for ver in versions.keys() {
        let ident = ver.to_field_ident();
        match_arms.extend(quote! {
            #ver => self.#ident,
        });
    }

    quote! {
        #[derive(Clone, Copy, Debug)]
        pub struct PacketId {
            #struct_fields
        }

        impl PacketId {
            /// Converts the requested protocol version into the corresponding packet ID.
            /// Returns -1 if the packet does not exist in that version.
            pub fn to_id(&self, version: MinecraftVersion) -> i32 {
                match version {
                    #match_arms
                    _ => self.#latest_field_ident,
                }
            }
        }

        impl PartialEq<i32> for PacketId {
            fn eq(&self, other: &i32) -> bool {
                self.#latest_field_ident == *other
            }
        }

        impl PartialEq<PacketId> for i32 {
            fn eq(&self, other: &PacketId) -> bool {
                *self == other.#latest_field_ident
            }
        }
    }
}

/// Generate mapped constants where `latest_version` is considered the baseline.
/// `versions` must include the `latest_version` entry.
fn generate_mapped_consts(
    versions: &BTreeMap<MinecraftVersion, Packets>,
    is_serverbound: bool,
) -> TokenStream {
    let mut output = TokenStream::new();

    // Latest = baseline (use provided latest_version)
    let latest = versions
        .get(&LATEST_VERSION)
        .expect("Latest version not found in versions map");

    let latest_phases = if is_serverbound {
        &latest.serverbound
    } else {
        &latest.clientbound
    };

    for (phase, packets) in latest_phases {
        for name in packets.keys() {
            let sanitized_name = name.replace(['/', '-'], "_").to_uppercase();
            let const_name = format_ident!("{}_{}", phase.to_uppercase(), sanitized_name);

            // Build initialization values for each version field (in the same order as version_field_idents)
            let mut init_pairs = TokenStream::new();
            for (ver, map) in versions {
                let phase_map = if is_serverbound {
                    &map.serverbound
                } else {
                    &map.clientbound
                };
                let field_ident = ver.to_field_ident();
                let id = phase_map
                    .get(phase)
                    .and_then(|p| p.get(name))
                    .copied()
                    .unwrap_or(-1);
                init_pairs.extend(quote! {
                    #field_ident: #id,
                });
            }

            output.extend(quote! {
                pub const #const_name: super::PacketId = super::PacketId {
                    #init_pairs
                };
            });
        }
    }

    output
}
