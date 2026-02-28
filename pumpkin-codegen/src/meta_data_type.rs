use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::version::MinecraftVersion;

pub fn build() -> TokenStream {
    let assets = [
        (MinecraftVersion::V_1_21, "1_21_meta_data_type.json"),
        (MinecraftVersion::V_1_21_2, "1_21_2_meta_data_type.json"),
        (MinecraftVersion::V_1_21_4, "1_21_4_meta_data_type.json"),
        (MinecraftVersion::V_1_21_5, "1_21_5_meta_data_type.json"),
        (MinecraftVersion::V_1_21_6, "1_21_6_meta_data_type.json"),
        (MinecraftVersion::V_1_21_7, "1_21_7_meta_data_type.json"),
        (MinecraftVersion::V_1_21_9, "1_21_9_meta_data_type.json"),
        (MinecraftVersion::V_1_21_11, "1_21_11_meta_data_type.json"),
    ];

    let mut handlers_map: BTreeMap<String, BTreeMap<MinecraftVersion, i32>> = BTreeMap::new();

    for &(ver, file) in &assets {
        let path = format!("../assets/meta_data_type/{file}");
        let parsed: BTreeMap<String, i32> =
            serde_json::from_str(&fs::read_to_string(&path).unwrap())
                .unwrap_or_else(|_| panic!("Failed to parse {file}"));
        for (name, id) in parsed {
            handlers_map.entry(name).or_default().insert(ver, id);
        }
    }

    let mut structure = TokenStream::new();
    let mut to_id_arms = TokenStream::new();
    for (ver, _) in &assets {
        let field_ident = ver.to_field_ident();
        structure.extend(quote! {
            #field_ident: i32,
        });
        to_id_arms.extend(quote! {
            #ver => self.#field_ident,
        });
    }

    let mut variants = TokenStream::new();
    for (name, ids) in &handlers_map {
        let mut fields = TokenStream::new();
        for (ver, _) in &assets {
            let field_ident = ver.to_field_ident();
            let id = ids.get(ver).unwrap_or(&-1);
            fields.extend(quote! {
                #field_ident: #id,
            });
        }
        let ident = format_ident!("{}", name.to_uppercase());
        variants.extend(quote! {
            pub const #ident: MetaDataType = MetaDataType {
                #fields
            };
        });
    }

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct MetaDataType {
            #structure
        }

        impl MetaDataType {
            #variants

            pub const fn id(&self, version: pumpkin_util::version::MinecraftVersion) -> i32 {
                match version {
                    #to_id_arms
                    _ => -1i32,
                }
            }
        }
    }
}
