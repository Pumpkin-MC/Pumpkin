use proc_macro2::TokenStream;
use quote::quote;

use crate::array_to_tokenstream;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/sound_category.json");

    let sound_categories: Vec<String> =
        serde_json::from_str(include_str!("../../assets/sound_category.json"))
            .expect("Failed to parse sound_category.json");
    let variants = array_to_tokenstream(sound_categories);

    quote! {
        #[derive(Clone, Copy)]
        #[repr(u8)]
        pub enum SoundCategory {
            #variants
        }
    }
}
