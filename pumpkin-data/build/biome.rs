use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
struct BiomeData {
    name: String,
    id: u16,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/biome.json");

    let biomes: Vec<BiomeData> = serde_json::from_str(include_str!("../../assets/biome.json"))
        .expect("Failed to parse biome.json");
    let mut variants = TokenStream::new();
    let mut to_id = TokenStream::new();
    let mut from_id = TokenStream::new();

    for data in biomes.into_iter() {
        let name = data.name;
        let id = data.id;
        let full_name = format!("minecraft:{name}");
        let name = format_ident!("{}", name.to_pascal_case());
        variants.extend([quote! {
            #[serde(rename = #full_name)]
            #name,
        }]);
        to_id.extend([quote! {
            Self::#name => #id,
        }]);
        from_id.extend([quote! {
            #id => Self::#name,
        }]);
    }
    quote! {
        #[derive(Clone, Deserialize, Copy, Hash, PartialEq, Eq, Debug)]
        pub enum Biome {
            #variants
        }

        impl Biome {
            pub fn to_id(&self) -> u16 {
                match self {
                    #to_id
                }
            }

            pub fn from_id(id: u16) -> Biome {
                match id {
                    #from_id
                    _ => panic!("Unknown biome id: {}", id)
                }
            }
        }
    }
}
