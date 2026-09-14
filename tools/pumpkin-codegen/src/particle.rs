use std::fs;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::array_to_tokenstream;

/// Generates the `TokenStream` for the `Particle` enum and its `from_name`/`to_name` methods.
pub fn build() -> TokenStream {
    let particle: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../../assets/particles.json").unwrap())
            .expect("Failed to parse particles.json");
    let variants = array_to_tokenstream(&particle);
    let type_from_name = &particle
        .iter()
        .map(|particle| {
            let id = &particle;
            let name = format_ident!("{}", particle.to_pascal_case());

            quote! {
                #id => Some(Self::#name),
            }
        })
        .collect::<TokenStream>();
    let type_to_name = &particle
        .iter()
        .map(|particle| {
            let id = &particle;
            let name = format_ident!("{}", particle.to_pascal_case());

            quote! {
                Self::#name => #id,
            }
        })
        .collect::<TokenStream>();
    let type_from_id = &particle
        .iter()
        .enumerate()
        .map(|(idx, particle)| {
            let idx = idx as u16;
            let name = format_ident!("{}", particle.to_pascal_case());

            quote! {
                #idx => Some(Self::#name),
            }
        })
        .collect::<TokenStream>();
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Particle {
            #variants
        }

        impl Particle {
            #[doc = r" Try to parse a `Particle` from a resource location string."]
            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }

            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #type_to_name
                }
            }

            #[doc = r" Try to parse a `Particle` from an ID."]
            #[must_use]
            #[allow(clippy::too_many_lines)]
            pub const fn from_id(id: u16) -> Option<Self> {
                match id {
                    #type_from_id
                    _ => None,
                }
            }

            #[must_use]
            pub const fn to_id(&self) -> u16 {
                *self as u16
            }
        }
    }
}
