use proc_macro::TokenStream;
use quote::quote;

pub(crate) fn block_entity_impl(item: TokenStream) -> TokenStream {
    let input_string = item.to_string();
    let block_entity_name = input_string.trim_matches('"');

    quote! {
        pumpkin_world::block::registry::BLOCKS
            .block_entity_types
            .iter()
            .position(|block_type| *block_type == #block_entity_name)
            .unwrap() as u32

    }
    .into()
}
