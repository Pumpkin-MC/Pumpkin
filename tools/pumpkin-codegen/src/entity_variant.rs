use std::{collections::BTreeMap, fs};

use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct VariantEnum {
    entries: Vec<BTreeMap<String, Value>>,
    out_of_bounds: String,
    fallback: Option<String>,
    default: Option<String>,
    #[serde(default)]
    common_variants: Vec<CommonVariant>,
}

#[derive(Deserialize)]
struct CommonVariant {
    pattern: String,
    base_color: String,
    pattern_color: String,
}

fn variant_ident(name: &str) -> proc_macro2::Ident {
    format_ident!("{}", name.to_pascal_case())
}

pub fn build() -> TokenStream {
    let enums: BTreeMap<String, VariantEnum> =
        serde_json::from_str(&fs::read_to_string("../../assets/entity_variants.json").unwrap())
            .expect("Failed to parse entity_variants.json");

    let mut output = TokenStream::new();
    for (key, variant_enum) in &enums {
        output.extend(build_enum(key, variant_enum));
    }
    output
}

fn build_enum(key: &str, variant_enum: &VariantEnum) -> TokenStream {
    let enum_ident = format_ident!("{}", key.to_pascal_case());
    let entries = &variant_enum.entries;
    let names: Vec<&str> = entries
        .iter()
        .map(|entry| entry["name"].as_str().expect("variant name"))
        .collect();
    let ids: Vec<i32> = entries
        .iter()
        .map(|entry| {
            i32::try_from(entry["id"].as_i64().expect("variant id")).expect("variant id range")
        })
        .collect();
    let idents: Vec<_> = names.iter().map(|name| variant_ident(name)).collect();
    let id_literals: Vec<_> = ids.iter().map(|id| Literal::i32_unsuffixed(*id)).collect();

    let first = &idents[0];
    let last = &idents[idents.len() - 1];
    let out_of_bounds = match variant_enum.out_of_bounds.as_str() {
        "fallback" => {
            let fallback = variant_ident(variant_enum.fallback.as_deref().expect("fallback name"));
            quote! { Self::#fallback }
        }
        "clamp" => quote! {
            if id < 0 { Self::#first } else { Self::#last }
        },
        "wrap" => {
            let len = Literal::i32_unsuffixed(i32::try_from(idents.len()).expect("variant count"));
            quote! { Self::ALL[id.rem_euclid(#len) as usize] }
        }
        other => panic!("unknown out of bounds strategy {other}"),
    };

    let default = variant_enum.default.as_deref().map(|name| {
        let ident = variant_ident(name);
        quote! {
            pub const DEFAULT: Self = Self::#ident;
        }
    });

    let mut extras = TokenStream::new();
    let extra_keys: Vec<&String> = entries[0]
        .keys()
        .filter(|key| *key != "id" && *key != "name")
        .collect();
    for extra in extra_keys {
        let fn_ident = format_ident!("{}", extra.to_snake_case());
        let values: Vec<&Value> = entries.iter().map(|entry| &entry[extra.as_str()]).collect();
        let (ty, arms): (TokenStream, Vec<TokenStream>) = match values[0] {
            Value::Bool(_) => (
                quote! { bool },
                values
                    .iter()
                    .map(|value| {
                        let value = value.as_bool().expect("bool extra");
                        quote! { #value }
                    })
                    .collect(),
            ),
            Value::Number(_) => (
                quote! { f32 },
                values
                    .iter()
                    .map(|value| {
                        let value =
                            Literal::f32_unsuffixed(value.as_f64().expect("number extra") as f32);
                        quote! { #value }
                    })
                    .collect(),
            ),
            Value::String(_) => (
                quote! { &'static str },
                values
                    .iter()
                    .map(|value| {
                        let value = value.as_str().expect("string extra");
                        quote! { #value }
                    })
                    .collect(),
            ),
            other => panic!("unsupported extra {extra}: {other}"),
        };
        extras.extend(quote! {
            #[must_use]
            pub const fn #fn_ident(&self) -> #ty {
                match self {
                    #(Self::#idents => #arms,)*
                }
            }
        });
    }

    let common_variants = (!variant_enum.common_variants.is_empty()).then(|| {
        let const_ident = format_ident!("{}_COMMON_VARIANTS", key.trim_end_matches("_pattern").to_uppercase());
        let rows = variant_enum.common_variants.iter().map(|variant| {
            let pattern = variant_ident(&variant.pattern);
            let base = variant_ident(&variant.base_color);
            let color = variant_ident(&variant.pattern_color);
            quote! {
                (#enum_ident::#pattern, crate::dye_color::DyeColor::#base, crate::dye_color::DyeColor::#color)
            }
        });
        quote! {
            pub const #const_ident: &[(#enum_ident, crate::dye_color::DyeColor, crate::dye_color::DyeColor)] = &[
                #(#rows,)*
            ];
        }
    });

    quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum #enum_ident {
            #(#idents,)*
        }

        impl #enum_ident {
            pub const ALL: &'static [Self] = &[#(Self::#idents,)*];

            #default

            #[must_use]
            pub const fn id(&self) -> i32 {
                match self {
                    #(Self::#idents => #id_literals,)*
                }
            }

            #[must_use]
            pub const fn name(&self) -> &'static str {
                match self {
                    #(Self::#idents => #names,)*
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #(#names => Some(Self::#idents),)*
                    _ => None,
                }
            }

            #[must_use]
            pub const fn by_id(id: i32) -> Self {
                match id {
                    #(#id_literals => Self::#idents,)*
                    _ => #out_of_bounds,
                }
            }

            #extras
        }

        #common_variants
    }
}
