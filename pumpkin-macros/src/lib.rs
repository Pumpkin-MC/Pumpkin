use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Nothing, Parser},
    parse_macro_input, Field, Fields, ItemStruct,
};

extern crate proc_macro;

#[proc_macro_attribute]
pub fn event(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;
    let _ = parse_macro_input!(args as Nothing);

    quote! {
        #input

        impl crate::plugin::Event for #name {
            fn get_name_static() -> &'static str {
                stringify!(#name)
            }

            fn get_name(&self) -> &'static str {
                stringify!(#name)
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn cancellable(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);
    let name = item_struct.ident.clone();
    let _ = parse_macro_input!(args as Nothing);

    if let Fields::Named(ref mut fields) = item_struct.fields {
        fields.named.push(
            Field::parse_named
                .parse2(quote! { pub cancelled: bool })
                .unwrap(),
        );
    }

    quote! {
        #item_struct

        impl crate::plugin::Cancellable for #name {
            fn cancelled(&self) -> bool {
                self.cancelled
            }

            fn set_cancelled(&mut self, cancelled: bool) {
                self.cancelled = cancelled;
            }
        }
    }
    .into()
}

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
