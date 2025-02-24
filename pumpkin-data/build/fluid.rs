use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use syn::{LitInt, LitStr};

#[derive(Deserialize)]
struct FluidState {
    height: f32,
    level: i32,
    is_empty: bool,
    blast_resistance: f32,
    block_state_id: u16,
    is_still: bool,
}

#[derive(Deserialize)]
struct Property {
    name: String,
    values: Vec<String>,
}

#[derive(Deserialize)]
struct Fluid {
    name: String,
    id: u8,
    properties: Vec<Property>,
    default_state_index: usize,
    states: Vec<FluidState>,
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/fluids.json");

    let fluids: Vec<Fluid> = serde_json::from_str(include_str!("../../assets/fluids.json"))
        .expect("Failed to parse fluids.json");

    let mut constants = TokenStream::new();
    let mut id_matches = Vec::new();
    for fluid in fluids {
        let id_name = LitStr::new(&fluid.name, proc_macro2::Span::call_site());
        let const_ident = format_ident!("{}", fluid.name.to_shouty_snake_case());

        let id_lit = LitInt::new(&fluid.id.to_string(), proc_macro2::Span::call_site());
        let mut properties = TokenStream::new();
        if fluid.properties.is_empty() {
            properties.extend(quote!(None));
        } else {
            let internal_properties = fluid.properties.into_iter().map(|property| {
                let key = LitStr::new(&property.name, proc_macro2::Span::call_site());
                let values = property
                    .values
                    .into_iter()
                    .map(|value| LitStr::new(&value, proc_macro2::Span::call_site()));

                quote! {
                    (#key, &[
                        #(#values),*
                    ])
                }
            });
            properties.extend(quote! {
                Some(&[
                    #(#internal_properties),*
                ])
            });
        }

        let fluid_states = fluid.states.into_iter().map(|state| {
            let height = state.height;
            let level = state.level;
            let is_empty = state.is_empty;
            let blast_resistance = state.blast_resistance;
            let block_state_id = state.block_state_id;
            let is_still = state.is_still;
            quote! {
                FluidState {
                    height: #height,
                    level: #level,
                    is_empty: #is_empty,
                    blast_resistance: #blast_resistance,
                    block_state_id: #block_state_id,
                    is_still: #is_still,
                }
            }
        });
        let state_id = fluid.default_state_index as u8;

        id_matches.push(quote! {
            #id_name => Some(#id_lit),
        });

        constants.extend(quote! {
            pub const #const_ident: Fluid = Fluid {
                id: #id_lit,
                properties: #properties,
                states: &[#(#fluid_states),*],
                default_state_index: #state_id
            };

        });
    }

    quote! {
        pub struct FluidState {
            pub height: f32,
            pub level: i32,
            pub is_empty: bool,
            pub blast_resistance: f32,
            pub block_state_id: u16,
            pub is_still: bool,
        }

        pub struct Fluid {
            pub id: u8,
            pub properties: Option<&'static [(&'static str, &'static [&'static str])]>,
            pub states: &'static [FluidState],
            pub default_state_index: u8,
        }

        impl Fluid {
            #constants

            pub fn ident_to_fluid_id(name: &str) -> Option<u8> {
                match name {
                    #(#id_matches)*
                    _ => None
                }
            }
        }
    }
}
