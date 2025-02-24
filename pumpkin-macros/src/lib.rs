use heck::{ToPascalCase, ToSnakeCase};
use proc_macro::TokenStream;
use pumpkin_data::item::Item;
use quote::quote;
use syn::{
    Block, Expr, Field, Fields, ItemStruct, Stmt,
    parse::{Nothing, Parser},
    parse_macro_input,
};

extern crate proc_macro;

#[proc_macro_derive(Event)]
pub fn event(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    quote! {
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
                .parse2(quote! {
                    /// A boolean indicating cancel state of the event.
                    pub cancelled: bool
                })
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

#[proc_macro]
pub fn send_cancellable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Block);

    let mut event = None;
    let mut after_block = None;
    let mut cancelled_block = None;

    for stmt in input.stmts {
        if let Stmt::Expr(expr, _) = stmt {
            if event.is_none() {
                event = Some(expr);
            } else if let Expr::Block(b) = expr {
                if let Some(ref label) = b.label {
                    if label.name.ident == "after" {
                        after_block = Some(b);
                    } else if label.name.ident == "cancelled" {
                        cancelled_block = Some(b);
                    }
                }
            }
        }
    }

    if let Some(event) = event {
        if let Some(after_block) = after_block {
            if let Some(cancelled_block) = cancelled_block {
                quote! {
                    let event = crate::PLUGIN_MANAGER
                        .lock()
                        .await
                        .fire(#event)
                        .await;

                    if !event.cancelled {
                        #after_block
                    } else {
                        #cancelled_block
                    }
                }
                .into()
            } else {
                quote! {
                    let event = crate::PLUGIN_MANAGER
                        .lock()
                        .await
                        .fire(#event)
                        .await;

                    if !event.cancelled {
                        #after_block
                    }
                }
                .into()
            }
        } else if let Some(cancelled_block) = cancelled_block {
            quote! {
                let event = crate::PLUGIN_MANAGER
                    .lock()
                    .await
                    .fire(#event)
                    .await;

                if event.cancelled {
                    #cancelled_block
                }
            }
            .into()
        } else {
            quote! {
                let event = crate::PLUGIN_MANAGER
                    .lock()
                    .await
                    .fire(#event)
                    .await;
            }
            .into()
        }
    } else {
        panic!("Event must be specified");
    }
}

#[proc_macro_attribute]
pub fn packet(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input: proc_macro2::TokenStream = input.into();
    let item: proc_macro2::TokenStream = item.into();

    let code = quote! {
        #item
        impl #impl_generics crate::bytebuf::packet::Packet for #name #ty_generics {
            const PACKET_ID: i32 = #input;
        }
    };

    code.into()
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

    let code = quote! {
        #item
        impl #impl_generics crate::block::pumpkin_block::BlockMetadata for #name #ty_generics {
            const NAMESPACE: &'static str = #namespace;
            const ID: &'static str = #id;
        }
    };

    code.into()
}

#[proc_macro_attribute]
pub fn pumpkin_item(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input_string = input.to_string();
    let packet_name = input_string.trim_matches('"');
    let item_id = Item::from_name(packet_name).unwrap();
    let id = item_id.id;

    let item: proc_macro2::TokenStream = item.into();

    let code = quote! {
        #item
        impl #impl_generics crate::item::pumpkin_item::ItemMetadata for #name #ty_generics {
            const ID: u16 = #id;
        }
    };

    code.into()
}

#[proc_macro_attribute]
pub fn block_property(input: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(item.clone()).unwrap();
    let name = &ast.ident;
    let (impl_generics, ty_generics, _) = ast.generics.split_for_impl();

    let input_string = input.to_string();
    let input_parts: Vec<&str> = input_string.split("[").collect();
    let property_name = input_parts[0].trim_ascii().trim_matches(&['"', ','][..]);
    let mut property_values: Vec<&str> = Vec::new();
    if input_parts.len() > 1 {
        property_values = input_parts[1]
            .trim_matches(']')
            .split(", ")
            .map(|p| p.trim_ascii().trim_matches(&['"', ','][..]))
            .collect::<Vec<&str>>();
    }

    let item: proc_macro2::TokenStream = item.into();

    let (variants, is_enum): (Vec<proc_macro2::Ident>, bool) = match ast.data {
        syn::Data::Enum(enum_item) => (
            enum_item.variants.into_iter().map(|v| v.ident).collect(),
            true,
        ),
        syn::Data::Struct(struct_type) => {
            let fields = match struct_type.fields {
                Fields::Named(_) => panic!("Block properties can't have named fields"),
                Fields::Unnamed(fields) => fields.unnamed,
                Fields::Unit => panic!("Block properties must have fields"),
            };
            if fields.len() != 1 {
                panic!("Block properties structs must have exactly one field");
            }
            let struct_type = match fields.first().unwrap().ty {
                syn::Type::Path(ref type_path) => {
                    type_path.path.segments.first().unwrap().ident.to_string()
                }
                _ => panic!("Block properties can only have primitive types"),
            };
            match struct_type.as_str() {
                "bool" => (
                    vec![
                        proc_macro2::Ident::new("true", proc_macro2::Span::call_site()),
                        proc_macro2::Ident::new("false", proc_macro2::Span::call_site()),
                    ],
                    false,
                ),
                _ => panic!("This type is not supported (Why not implement it yourself?)"),
            }
        }
        _ => panic!("Block properties can only be enums or structs"),
    };

    let values = variants.iter().enumerate().map(|(i, v)| match is_enum {
        true => {
            let mut value = v.to_string().to_snake_case();
            if !property_values.is_empty() && i < property_values.len() {
                value = property_values[i].to_string();
            }
            quote! {
                Self::#v => #value.to_string(),
            }
        }
        false => {
            let value = v.to_string();
            quote! {
                Self(#v) => #value.to_string(),
            }
        }
    });

    let from_values = variants.iter().enumerate().map(|(i, v)| match is_enum {
        true => {
            let mut value = v.to_string().to_snake_case();
            if !property_values.is_empty() && i < property_values.len() {
                value = property_values[i].to_string();
            }
            quote! {
                #value => Self::#v,
            }
        }
        false => {
            let value = v.to_string();
            quote! {
                #value => Self(#v),
            }
        }
    });

    let extra_fns = variants.iter().map(|v| {
        let title = proc_macro2::Ident::new(
            &v.to_string().to_pascal_case(),
            proc_macro2::Span::call_site(),
        );
        quote! {
            pub fn #title() -> Self {
                Self(#v)
            }
        }
    });

    let extra = if is_enum {
        quote! {}
    } else {
        quote! {
            impl #name {
                #(#extra_fns)*
            }
        }
    };

    let code = quote! {
        #item
        impl #impl_generics crate::block::properties::BlockPropertyMetadata for #name #ty_generics {
            fn name(&self) -> &'static str {
                #property_name
            }
            fn value(&self) -> String {
                match self {
                    #(#values)*
                }
            }
            fn from_value(value: String) -> Self {
                match value.as_str() {
                    #(#from_values)*
                    _ => panic!("Invalid value for block property"),
                }
            }
        }
        #extra
    };

    code.into()
}

mod block_state;
#[proc_macro]
pub fn block_state(item: TokenStream) -> TokenStream {
    block_state::block_state_impl(item)
}
mod block;
#[proc_macro]
pub fn block_entity(item: TokenStream) -> TokenStream {
    block::block_entity_impl(item)
}
