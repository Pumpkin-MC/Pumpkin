use pumpkin_data::Block;

use quote::quote;

pub(crate) fn default_block_state_impl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_string = item.to_string();
    let registry_id = input_string.trim_matches('"');

    let state = Block::from_registry_key(registry_id).expect("Invalid registry id");
    let default_state_id = state.default_state_id;

    if std::env::var("CARGO_PKG_NAME").unwrap() == "pumpkin-world" {
        quote! {
            crate::block::RawBlockState {
                state_id: #default_state_id,
          }
        }
        .into()
    } else {
        quote! {
            pumpkin_world::block::RawBlockState {
                state_id: #default_state_id,
            }
        }
        .into()
    }
}
