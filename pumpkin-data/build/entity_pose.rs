use proc_macro2::TokenStream;
use quote::quote;

use crate::array_to_tokenstream;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/entity_pose.json");

    let poses: Vec<String> = serde_json::from_str(include_str!("../../assets/entity_pose.json"))
        .expect("Failed to parse entity_pose.json");
    let variants = array_to_tokenstream(&poses);

    quote! {
        #[derive(Clone, Copy)]
        pub enum EntityPose {
            #variants
        }
    }
}
