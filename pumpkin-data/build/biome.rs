use proc_macro2::TokenStream;
use quote::quote;

use crate::array_to_tokenstream;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/biome.json");

    let biomes: Vec<String> = serde_json::from_str(include_str!("../../assets/biome.json"))
        .expect("Failed to parse entity_pose.json");
    let variants = array_to_tokenstream(biomes);

    quote! {
        #[derive(Clone, Copy)]
        pub enum Biome {
            #variants
        }
    }
}
