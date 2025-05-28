use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Packets {
    version: u32,
    serverbound: HashMap<String, Vec<String>>,
    clientbound: HashMap<String, Vec<String>>,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/packets.json");

    let packets: Packets = serde_json::from_str(include_str!("../../assets/packets.json"))
        .expect("Failed to parse packets.json");
    let version = packets.version;
    let serverbound_consts = parse_packets(packets.serverbound);
    let clientbound_consts = parse_packets(packets.clientbound);

    quote!(
        /// The current Minecraft protocol version. This changes only when the protocol itself is modified.
        pub const CURRENT_MC_PROTOCOL: u32 = #version;

        pub mod serverbound {
            #serverbound_consts
        }

        pub mod clientbound {
            #clientbound_consts
        }
    )
}

pub(crate) fn parse_packets(packets: HashMap<String, Vec<String>>) -> proc_macro2::TokenStream {
    let mut consts = TokenStream::new();

    for packet in packets {
        let phase = packet.0;

        for (id, packet_name) in packet.1.iter().enumerate() {
            let packet_id = id as i32;
            let name = format!("{phase}_{packet_name}").to_uppercase();
            let name = format_ident!("{}", name);
            consts.extend([quote! {
                pub const #name: i32 = #packet_id;
            }]);
        }
    }
    consts
}
