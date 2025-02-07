use proc_macro2::TokenStream;
use quote::quote;

use crate::array_to_tokenstream;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/particles.json");

    let particle: Vec<String> = serde_json::from_str(include_str!("../../assets/particles.json"))
        .expect("Failed to parse particles.json");
    let variants = array_to_tokenstream(particle);

    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Particle {
            #variants
        }
    }
}
