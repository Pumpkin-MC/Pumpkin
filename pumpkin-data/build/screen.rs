use std::fs;

use proc_macro2::TokenStream;
use quote::quote;

use crate::array_to_tokenstream;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/screens.json");

    let screens: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../assets/screens.json").unwrap())
            .expect("Failed to parse screens.json");
    let variants = array_to_tokenstream(&screens);

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum WindowType {
            #variants
        }
    }
}
