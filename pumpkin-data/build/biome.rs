use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/biome.json");

    let biomes: Vec<String> = serde_json::from_str(include_str!("../../assets/biome.json"))
        .expect("Failed to parse biome.json");
    let mut variants = TokenStream::new();

    for status in biomes.iter() {
        let full_name = format!("minecraft:{status}");
        let name = format_ident!("{}", status.to_pascal_case());
        variants.extend([quote! {
            #[serde(rename = #full_name)]
            #name,
        }]);
    }
    quote! {
        #[derive(Clone, Deserialize, Copy, Hash, PartialEq, Eq)]
        pub enum Biome {
            #variants
        }
    }
}
