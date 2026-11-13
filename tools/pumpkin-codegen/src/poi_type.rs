use std::fs;

use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

#[derive(serde::Deserialize)]
pub(crate) struct PoiTypeDefinition {
    id: u8,
    max_tickets: i32,
    valid_range: i32,
    state_ids: Vec<u16>,
}

impl PoiTypeDefinition {
    pub(crate) const fn id(&self) -> u8 {
        self.id
    }
}

pub(crate) fn read_poi_types() -> Vec<(String, PoiTypeDefinition)> {
    let poi_types: std::collections::BTreeMap<String, PoiTypeDefinition> =
        serde_json::from_str(&fs::read_to_string("../../assets/poi_types.json").unwrap())
            .expect("Failed to parse poi_types.json");
    let mut poi_types: Vec<_> = poi_types.into_iter().collect();
    poi_types.sort_by_key(|(_, definition)| definition.id);
    poi_types
}

/// Generates the point of interest types with their ticket counts, ranges and block states.
pub fn build() -> TokenStream {
    let poi_types = read_poi_types();

    let mut variants = TokenStream::new();
    let mut all = TokenStream::new();
    let mut name_match = TokenStream::new();
    let mut from_name_match = TokenStream::new();
    let mut max_tickets_match = TokenStream::new();
    let mut valid_range_match = TokenStream::new();
    let mut state_entries = TokenStream::new();

    for (name, definition) in &poi_types {
        let ident = format_ident!("{}", name.to_pascal_case());
        let namespaced = format!("minecraft:{name}");
        let id = definition.id;
        let max_tickets = definition.max_tickets;
        let valid_range = definition.valid_range;
        variants.extend(quote! { #ident = #id, });
        all.extend(quote! { Self::#ident, });
        name_match.extend(quote! { Self::#ident => #name, });
        from_name_match.extend(quote! { #name | #namespaced => Some(Self::#ident), });
        max_tickets_match.extend(quote! { Self::#ident => #max_tickets, });
        valid_range_match.extend(quote! { Self::#ident => #valid_range, });
        for state_id in &definition.state_ids {
            state_entries.extend(quote! { #state_id => PoiType::#ident, });
        }
    }

    quote! {
        use crate::tag::{RegistryKey, Taggable};

        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        #[repr(u8)]
        pub enum PoiType {
            #variants
        }

        static POI_TYPE_BY_STATE: phf::Map<u16, PoiType> = phf::phf_map! {
            #state_entries
        };

        impl PoiType {
            pub const ALL: &'static [Self] = &[#all];

            #[must_use]
            pub const fn id(self) -> u8 {
                self as u8
            }

            #[must_use]
            pub const fn name(self) -> &'static str {
                match self {
                    #name_match
                }
            }

            #[must_use]
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #from_name_match
                    _ => None,
                }
            }

            #[must_use]
            pub const fn max_tickets(self) -> i32 {
                match self {
                    #max_tickets_match
                }
            }

            #[must_use]
            pub const fn valid_range(self) -> i32 {
                match self {
                    #valid_range_match
                }
            }

            #[must_use]
            pub fn from_state_id(state_id: u16) -> Option<Self> {
                POI_TYPE_BY_STATE.get(&state_id).copied()
            }
        }

        impl Taggable for PoiType {
            #[inline]
            fn tag_key() -> RegistryKey {
                RegistryKey::PointOfInterestType
            }

            #[inline]
            fn registry_key(&self) -> &str {
                self.name()
            }

            #[inline]
            fn registry_id(&self) -> u16 {
                u16::from(self.id())
            }
        }
    }
}
