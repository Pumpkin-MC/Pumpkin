use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;

use crate::ident;

#[derive(Deserialize)]
pub struct Packets {
    serverbound: HashMap<String, Vec<String>>,
    clientbound: HashMap<String, Vec<String>>,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/packets.json");

    let packets: Packets = serde_json::from_str(include_str!("../../assets/packets.json"))
        .expect("Failed to parse packets.json");
    let serverbound_consts = parse_packets(packets.serverbound);
    let clientbound_consts = parse_packets(packets.clientbound);

    quote!(
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
            let name = ident(name);
            consts.extend([quote! {
                pub const #name: i32 = #packet_id;
            }]);
        }
    }
    consts
}
