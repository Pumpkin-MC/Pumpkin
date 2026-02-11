use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};

#[derive(Deserialize)]
pub struct Packets {
    version: u32,
    serverbound: BTreeMap<String, BTreeMap<String, i32>>,
    clientbound: BTreeMap<String, BTreeMap<String, i32>>,
}

pub fn build() -> TokenStream {
    // 2. Parse the protocol files
    let packets_7: Packets = serde_json::from_str(
        &fs::read_to_string("../assets/packet/1_21_7_packets.json").expect("1.21.7 file missing"),
    )
    .unwrap();

    let packets_11: Packets = serde_json::from_str(
        &fs::read_to_string("../assets/packet/1_21_11_packets.json").expect("1.21.11 file missing"),
    )
    .unwrap();

    let latest_version = packets_11.version;
    // 3. Generate mapped constants for both directions
    let serverbound_consts = generate_mapped_consts(&packets_11, &packets_7, true);
    let clientbound_consts = generate_mapped_consts(&packets_11, &packets_7, false);

    quote!(
        use pumpkin_util::version::MinecraftVersion;

        pub const CURRENT_MC_PROTOCOL: u32 = #latest_version;

        pub struct PacketId {
            pub latest_id: i32,
            pub v1_21_7_id: i32,
        }

        impl PacketId {
            /// Converts the latest packet ID to the ID used in the requested version.
            /// Returns -1 if the packet does not exist in that version.
            pub fn to_id(&self, version: MinecraftVersion) -> i32 {
                match version {
                    MinecraftVersion::V_1_21_11 => self.latest_id,
                    MinecraftVersion::V_1_21_7 => self.v1_21_7_id,
                    // Default to latest for unrecognized/unmapped versions
                    _ => self.latest_id,
                }
            }
        }

        impl PartialEq<i32> for PacketId {
            fn eq(&self, other: &i32) -> bool {
                self.latest_id == *other
            }
        }

        impl PartialEq<PacketId> for i32 {
            fn eq(&self, other: &PacketId) -> bool {
                *self == other.latest_id
            }
        }

        // We place the constants directly into these modules
        pub mod serverbound {
            #serverbound_consts
        }

        pub mod clientbound {
            #clientbound_consts
        }
    )
}

fn generate_mapped_consts(latest: &Packets, v7: &Packets, is_serverbound: bool) -> TokenStream {
    let mut output = TokenStream::new();
    let latest_phases = if is_serverbound {
        &latest.serverbound
    } else {
        &latest.clientbound
    };
    let v7_phases = if is_serverbound {
        &v7.serverbound
    } else {
        &v7.clientbound
    };

    for (phase, packets) in latest_phases {
        for (name, id_11) in packets {
            let sanitized = name.replace(['/', '-'], "_").to_uppercase();
            let const_name = format_ident!("{}_{}", phase.to_uppercase(), sanitized);

            // Check if the packet exists in 1.21.7 to get the alternate ID
            let id_7 = v7_phases
                .get(phase)
                .and_then(|p| p.get(name))
                .copied()
                .unwrap_or(-1);

            // Since serverbound/clientbound are modules, we reference the PacketId in the parent scope
            output.extend(quote! {
                pub const #const_name: super::PacketId = super::PacketId {
                    latest_id: #id_11,
                    v1_21_7_id: #id_7,
                };
            });
        }
    }
    output
}
