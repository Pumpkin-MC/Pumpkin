use proc_macro2::TokenStream;
use quote::quote;

use crate::array_to_tokenstream;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/sounds.json");

    let sound: Vec<String> = serde_json::from_str(include_str!("../../assets/sounds.json"))
        .expect("Failed to parse sounds.json");
    let variants = array_to_tokenstream(sound);

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[repr(u16)]
        pub enum Sound {
            #variants
        }
    }
}
