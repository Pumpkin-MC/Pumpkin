use std::{collections::HashMap, fs};

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

pub struct EnumCreator {
    pub name: String,
    pub value: Vec<String>,
}

impl ToTokens for EnumCreator {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name.to_pascal_case());
        let values = self
            .value
            .iter()
            .map(|value| {
                let name = format_ident!("{}", value.to_pascal_case());
                name
            })
            .collect::<Vec<_>>();
        tokens.extend(quote! {
            pub enum #name {
                #(#values),*
            }
        });
    }
}
pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/tags.json");

    let tags: HashMap<String, HashMap<String, Vec<String>>> =
        serde_json::from_str(&fs::read_to_string("../assets/tags.json").unwrap())
            .expect("Failed to parse tags.json");
    let registry_key_enum = EnumCreator {
        name: "RegistryKey".to_string(),
        value: tags.keys().map(|key| key.to_string()).collect(),
    }
    .to_token_stream();

    // Generate tag arrays for each registry key
    let mut tag_dicts = Vec::new();
    let mut match_arms = Vec::new();
    let mut match_arms_tags_all = Vec::new();
    let mut tag_identifiers = Vec::new();

    for (key, tag_map) in &tags {
        let key_pascal = format_ident!("{}", key.to_pascal_case());
        let dict_name = format_ident!("{}_TAGS", key.to_pascal_case().to_uppercase());

        // Create a HashMap to store tag name -> index mapping
        let mut tag_values = Vec::new();

        // Collect all unique tags
        for (tag_name, values) in tag_map {
            tag_values.push((tag_name.clone(), values.clone()));
        }

        // Generate the static array of tag values
        let tag_array_entries = tag_values
            .iter()
            .map(|(tag_name, values)| {
                let tag_values_array = values.iter().map(|v| quote! { #v }).collect::<Vec<_>>();
                quote! {
                    #tag_name => &[#(#tag_values_array),*]
                }
            })
            .collect::<Vec<_>>();
        // Add the static array declaration
        tag_dicts.push(quote! {
            static #dict_name: phf::Map<&str, &[&str]> = phf::phf_map! {
                #(#tag_array_entries),*
            };
        });

        // Add match arm for this registry key
        match_arms.push(quote! {
            RegistryKey::#key_pascal => {
                #dict_name.get(tag).copied()
            }
        });

        match_arms_tags_all.push(quote! {
            RegistryKey::#key_pascal => {
                &#dict_name
            }
        });

        tag_identifiers.push(quote! {
            Self::#key_pascal => #key
        });
    }

    quote! {
        #[derive(Eq, PartialEq, Hash, Debug)]
        #registry_key_enum

        impl RegistryKey {
            // IDK why the linter is saying this isn't used
            #[allow(dead_code)]
            pub fn identifier_string(&self) -> &str {
                match self {
                    #(#tag_identifiers),*
                }
            }
        }

        #(#tag_dicts)*

        pub fn get_tag_values(tag_category: RegistryKey, tag: &str) -> Option<&'static [&'static str]> {
            match tag_category {
                #(#match_arms),*
            }
        }

        pub fn get_registry_key_tags(tag_category: &RegistryKey) -> &phf::Map<&'static str, &'static [&'static str]> {
            match tag_category {
                #(#match_arms_tags_all),*
            }
        }

        pub trait Tagable {
            fn tag_key() -> RegistryKey;
            fn registry_key(&self) -> &str;

            /// Returns `None` if the tag does not exist.
            fn is_tagged_with(&self, tag: &str) -> Option<bool> {
                let tag = tag.strip_prefix("#").unwrap_or(tag);
                let items = get_tag_values(Self::tag_key(), tag)?;
                Some(items.iter().any(|elem| *elem == self.registry_key()))
            }

            fn get_tag_values(tag: &str) -> Option<&'static [&'static str]> {
                let tag = tag.strip_prefix("#").unwrap_or(tag);
                get_tag_values(Self::tag_key(), tag)
            }
        }
    }
}
