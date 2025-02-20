use proc_macro2::TokenStream;
use quote::quote;

use crate::array_to_tokenstream;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/game_event.json");

    let game_events: Vec<String> =
        serde_json::from_str(include_str!("../../assets/game_event.json"))
            .expect("Failed to parse game_event.json");
    let variants = array_to_tokenstream(&game_events);

    quote! {
        pub enum GameEvent {
            #variants
        }
    }
}
