use std::fs;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::array_to_tokenstream;

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/particles.json");

    let particle: Vec<String> =
        serde_json::from_str(&fs::read_to_string("../assets/particles.json").unwrap())
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
    quote! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Particle {
            #variants
        }

        impl Particle {
            #[doc = r" Try to parse a `Particle` from a resource location string."]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #type_from_name
                    _ => None
                }
            }
            pub const fn to_name(&self) -> &'static str {
                match self {
                    #type_to_name
                }
            }
        }
    }
}
