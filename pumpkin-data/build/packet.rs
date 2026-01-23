use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Packets {
    version: u32,
    serverbound: BTreeMap<String, BTreeMap<String, i32>>,
    clientbound: BTreeMap<String, BTreeMap<String, i32>>,
}

pub fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/packet/1_21_11_packets.json");

    let packets: Packets =
        serde_json::from_str(&fs::read_to_string("../assets/packet/1_21_11_packets.json").unwrap())
            .expect("Failed to parse packets.json");

    let version = packets.version;
    let serverbound_consts = parse_packets(packets.serverbound);
    let clientbound_consts = parse_packets(packets.clientbound);

    quote!(
        /// The current Minecraft protocol version.
        pub const CURRENT_MC_PROTOCOL: u32 = #version;

        pub mod serverbound {
            #serverbound_consts
        }

        pub mod clientbound {
            #clientbound_consts
        }
    )
}

pub fn parse_packets(phases: BTreeMap<String, BTreeMap<String, i32>>) -> TokenStream {
    let mut consts = TokenStream::new();

    for (phase_name, packets) in phases {
        for (packet_name, packet_id) in packets {
            let sanitized_name = packet_name
                .replace('/', "_")
                .replace('-', "_")
                .to_uppercase();

            let const_name = format_ident!("{}_{}", phase_name.to_uppercase(), sanitized_name);

            consts.extend(quote! {
                pub const #const_name: i32 = #packet_id;
            });
        }
    }
    consts
}
