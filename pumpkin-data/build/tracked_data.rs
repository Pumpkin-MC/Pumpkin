use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::{collections::BTreeMap, fs};

pub(crate) fn build() -> TokenStream {
    // 1. Track files
    println!("cargo:rerun-if-changed=../assets/tracked_data/1_21_6_tracked_data.json");
    println!("cargo:rerun-if-changed=../assets/tracked_data/1_21_7_tracked_data.json");
    println!("cargo:rerun-if-changed=../assets/tracked_data/1_21_11_tracked_data.json");

    let data_6: BTreeMap<String, u8> = serde_json::from_str(
        &fs::read_to_string("../assets/tracked_data/1_21_6_tracked_data.json")
            .expect("1.21.6 data missing"),
    )
    .unwrap();

    let data_7: BTreeMap<String, u8> = serde_json::from_str(
        &fs::read_to_string("../assets/tracked_data/1_21_7_tracked_data.json")
            .expect("1.21.7 data missing"),
    )
    .unwrap();

    let data_11: BTreeMap<String, u8> = serde_json::from_str(
        &fs::read_to_string("../assets/tracked_data/1_21_11_tracked_data.json")
            .expect("1.21.11 data missing"),
    )
    .unwrap();

    let mut constants = TokenStream::new();

    for (name, id_11) in &data_11 {
        let ident = format_ident!("DATA_{}", name.to_uppercase());

        let id_6 = data_6.get(name).copied().unwrap_or(255); // 255 as an 'Invalid' marker
        let id_7 = data_7.get(name).copied().unwrap_or(255); // 255 as an 'Invalid' marker

        constants.extend(quote! {
            pub const #ident: TrackedId = TrackedId {
                latest: #id_11,
                v1_21_6: #id_6,
                v1_21_7: #id_7,
            };
        });
    }

    quote! {
        use pumpkin_util::version::MinecraftVersion;

        #[derive(Copy, Clone, Debug)]
        pub struct TrackedId {
            pub latest: u8,
            pub v1_21_6: u8,
            pub v1_21_7: u8,
        }

        impl TrackedId {
            pub fn get(&self, version: &MinecraftVersion) -> u8 {
                match version {
                    MinecraftVersion::V_1_21_6 => self.v1_21_6,
                    MinecraftVersion::V_1_21_7 => self.v1_21_7,
                    _ => self.latest,
                }
            }
        }

        impl From<TrackedId> for u8 {
            fn from(id: TrackedId) -> u8 {
                id.latest
            }
        }

        pub struct TrackedData;

        impl TrackedData {
            #constants
        }
    }
}
