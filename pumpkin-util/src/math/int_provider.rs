use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use serde::Deserialize;
use syn::LitInt;

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum NormalIntProvider {
    #[serde(rename = "minecraft:uniform")]
    Uniform(UniformIntProvider),
    // TODO: Add more...
}

impl ToTokens for NormalIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            NormalIntProvider::Uniform(uniform) => {
                tokens.extend(quote! {
                    NormalIntProvider::Uniform(#uniform)
                });
            }
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum IntProvider {
    Object(NormalIntProvider),
    Constant(i32),
}

impl ToTokens for IntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            IntProvider::Object(int_provider) => {
                tokens.extend(quote! {
                    IntProvider::Object(#int_provider)
                });
            }
            IntProvider::Constant(i) => tokens.extend(quote! {
                IntProvider::Constant(#i)
            }),
        }
    }
}

impl IntProvider {
    pub fn get_min(&self) -> i32 {
        match self {
            IntProvider::Object(int_provider) => match int_provider {
                NormalIntProvider::Uniform(uniform) => uniform.get_min(),
            },
            IntProvider::Constant(i) => *i,
        }
    }

    pub fn get(&self) -> i32 {
        match self {
            IntProvider::Object(int_provider) => match int_provider {
                NormalIntProvider::Uniform(uniform) => uniform.get(),
            },
            IntProvider::Constant(i) => *i,
        }
    }

    pub fn get_max(&self) -> i32 {
        match self {
            IntProvider::Object(int_provider) => match int_provider {
                NormalIntProvider::Uniform(uniform) => uniform.get_max(),
            },
            IntProvider::Constant(i) => *i,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct UniformIntProvider {
    pub min_inclusive: i32,
    pub max_inclusive: i32,
}

impl ToTokens for UniformIntProvider {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let min_inclusive = LitInt::new(&self.min_inclusive.to_string(), Span::call_site());
        let max_inclusive = LitInt::new(&self.max_inclusive.to_string(), Span::call_site());

        tokens.extend(quote! {
            UniformIntProvider { min_inclusive: #min_inclusive, max_inclusive: #max_inclusive }
        });
    }
}

impl UniformIntProvider {
    pub fn get_min(&self) -> i32 {
        self.min_inclusive
    }
    pub fn get(&self) -> i32 {
        rand::random_range(self.min_inclusive..self.max_inclusive)
    }
    pub fn get_max(&self) -> i32 {
        self.max_inclusive
    }
}
