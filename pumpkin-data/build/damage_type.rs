use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use heck::{ToShoutySnakeCase, ToPascalCase};

#[derive(Deserialize)]
pub struct DamageTypeData {
    death_message_type: Option<String>,
    exhaustion: f32,
    message_id: String,
    scaling: String,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/damage_type");

    let damage_type_dir = Path::new("../assets/damage_type");
    let mut constants = Vec::new();
    let mut enum_variants = Vec::new();

    for entry in fs::read_dir(damage_type_dir).expect("Failed to read damage_type directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            let content = fs::read_to_string(&path)
                .expect("Failed to read damage type file");
            
            let data: DamageTypeData = serde_json::from_str(&content)
                .expect("Failed to parse damage type JSON");
                
            let name = path.file_stem()
                .expect("Invalid filename")
                .to_str()
                .expect("Invalid UTF-8 in filename");
                
            let const_ident = crate::ident(&name.to_shouty_snake_case());
            let enum_ident = crate::ident(&name.to_pascal_case());
            
            enum_variants.push(enum_ident.clone());
            
            let death_message_type = match &data.death_message_type {
                Some(msg) => quote! { Some(#msg) },
                None => quote! { None },
            };
            
            let exhaustion = data.exhaustion;
            let message_id = &data.message_id;
            let scaling = &data.scaling;
            
            constants.push(quote! {
                pub const #const_ident: DamageTypeData = DamageTypeData {
                    death_message_type: #death_message_type,
                    exhaustion: #exhaustion,
                    message_id: #message_id,
                    scaling: #scaling,
                };
            });
        }
    }

    let enum_arms = enum_variants.iter().map(|variant| {
        let const_name = variant.to_string().to_shouty_snake_case();
        let const_ident = crate::ident(&const_name);
        quote! {
            DamageType::#variant => &#const_ident,
        }
    });

    quote! {
        #[derive(Clone, Debug)]
        pub struct DamageTypeData {
            pub death_message_type: Option<&'static str>,
            pub exhaustion: f32,
            pub message_id: &'static str,
            pub scaling: &'static str,
        }

        #(#constants)*

        #[derive(Clone, Copy, Debug)]
        #[repr(u8)]
        pub enum DamageType {
            #(#enum_variants,)*
        }

        impl DamageType {
            pub const fn data(&self) -> &'static DamageTypeData {
                match self {
                    #(#enum_arms)*
                }
            }
        }
    }
}
