use pumpkin_data::block::Block;

use quote::quote;

pub(crate) fn block_impl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_string = item.to_string();
    let registry_id = input_string.trim_matches('"');

    if std::env::var("CARGO_PKG_NAME").unwrap() == "pumpkin-data" {
        quote! {
            crate::block::Block::from_registry_key(#registry_id).expect("Invalid registry id")
        }
        .into()
    } else {
        quote! {
            pumpkin_data::block::Block::from_registry_key(#registry_id).expect("Invalid registry id")
        }
        .into()
    }
}

pub(crate) fn block_state_impl(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_string = item.to_string();
    let registry_id = input_string.trim_matches('"');

    let block = Block::from_registry_key(registry_id).expect("Invalid registry id");

    let default_state_id = block.default_state_id;

    if std::env::var("CARGO_PKG_NAME").unwrap() == "pumpkin-data" {
        quote! {
            crate::block::get_state_by_state_id(#default_state_id).expect("Invalid state id")
        }
        .into()
    } else {
        quote! {
            pumpkin_data::block::get_state_by_state_id(#default_state_id).expect("Invalid state id")
        }
        .into()
    }
}
