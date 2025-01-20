use proc_macro::TokenStream;
use quote::quote;

extern crate proc_macro;

#[proc_macro_attribute]
pub fn client_packet(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input: proc_macro2::TokenStream = input.into();
    let item: proc_macro2::TokenStream = item.into();

    let gen = quote! {
        #item
        impl #impl_generics crate::bytebuf::packet_id::Packet for #name #ty_generics {
            const PACKET_ID: i32 = #input;
        }
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn server_packet(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input: proc_macro2::TokenStream = input.into();
    let item: proc_macro2::TokenStream = item.into();

    let gen = quote! {
        #item
        impl #impl_generics crate::bytebuf::packet_id::Packet for #name #ty_generics {
            const PACKET_ID: i32 = #input;
        }
    };

    gen.into()
}

#[proc_macro_attribute]
pub fn pumpkin_block(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input_string = input.to_string();
    let packet_name = input_string.trim_matches('"');
    let packet_name_split: Vec<&str> = packet_name.split(":").collect();

    let namespace = packet_name_split[0];
    let id = packet_name_split[1];

    let item: proc_macro2::TokenStream = item.into();

    let gen = quote! {
        #item
        impl #impl_generics crate::block::pumpkin_block::BlockMetadata for #name #ty_generics {
            const NAMESPACE: &'static str = #namespace;
            const ID: &'static str = #id;
        }
    };

    gen.into()
}

mod block_state;
#[proc_macro]
pub fn block_state(item: TokenStream) -> TokenStream {
    block_state::block_state_impl(item)
}
