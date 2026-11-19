use heck::ToShoutySnakeCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use std::{collections::BTreeMap, fs};
use syn::LitInt;

#[derive(Deserialize)]
struct Entry {
    id: String,
    duration: i32,
}

#[derive(Deserialize)]
struct ItemId {
    id: u16,
}

/// Generates `get_suspicious_effects`, vanilla `SuspiciousEffectHolder.tryGet(item).getSuspiciousEffects()`.
pub fn build() -> TokenStream {
    let holders: BTreeMap<String, Vec<Entry>> =
        serde_json::from_str(&fs::read_to_string("../../assets/suspicious_effects.json").unwrap())
            .expect("Failed to parse suspicious_effects.json");
    let items: BTreeMap<String, ItemId> =
        serde_json::from_str(&fs::read_to_string("../../assets/items.json").unwrap())
            .expect("Failed to parse items.json");

    let match_arms: Vec<TokenStream> = holders
        .into_iter()
        .map(|(item, entries)| {
            let item_id = items
                .get(&item)
                .unwrap_or_else(|| panic!("Unknown suspicious effect holder {item}"))
                .id;
            let item_id = LitInt::new(&item_id.to_string(), Span::call_site());
            let entries = entries.into_iter().map(|entry| {
                let effect = format_ident!(
                    "{}",
                    entry
                        .id
                        .strip_prefix("minecraft:")
                        .unwrap_or(&entry.id)
                        .to_shouty_snake_case()
                );
                let duration = entry.duration;
                quote! {
                    SuspiciousEffect { effect: &StatusEffect::#effect, duration: #duration }
                }
            });
            quote! {
                #item_id => Some(&[#(#entries),*]),
            }
        })
        .collect();

    quote! {
        use crate::effect::StatusEffect;

        pub struct SuspiciousEffect {
            pub effect: &'static StatusEffect,
            pub duration: i32,
        }

        #[must_use]
        pub const fn get_suspicious_effects(item_id: u16) -> Option<&'static [SuspiciousEffect]> {
            match item_id {
                #(#match_arms)*
                _ => None,
            }
        }
    }
}
