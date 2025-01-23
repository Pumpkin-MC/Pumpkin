use proc_macro2::TokenStream;
use quote::quote;

use crate::array_to_tokenstream;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/scoreboard_display_slot.json");

    let sound_categories: Vec<String> =
        serde_json::from_str(include_str!("../../assets/scoreboard_display_slot.json"))
            .expect("Failed to parse scoreboard_display_slot.json");
    let variants = array_to_tokenstream(sound_categories);

    quote! {
        #[repr(u8)]
        pub enum ScoreboardDisplaySlot {
            #variants
        }
    }
}
