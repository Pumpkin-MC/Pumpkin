use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/status_effects.json");

    let chunk_status: Vec<String> =
        serde_json::from_str(include_str!("../../assets/status_effects.json"))
            .expect("Failed to parse status_effects.json");
    let mut variants = TokenStream::new();
    let mut type_from_name = TokenStream::new();
    let mut type_to_name = TokenStream::new();

    for status in chunk_status.iter() {
        let const_ident = format_ident!("{}", status.to_pascal_case());
        let resource_name = status.to_lowercase();

        variants.extend([quote! {
            #const_ident,
        }]);
        type_from_name.extend(quote! {
            #resource_name => Some(Self::#const_ident),
        });
        type_to_name.extend(quote! {
            Self::#const_ident => #resource_name,
        });
    }
    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum EffectType {
            #variants
        }

        impl EffectType {
            #[doc = r" Try to parse a Effect Type from a resource location string"]
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
