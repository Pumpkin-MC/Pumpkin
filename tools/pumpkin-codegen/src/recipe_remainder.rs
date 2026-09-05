use proc_macro2::{Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use std::{collections::BTreeMap, fs};
use syn::LitInt;

#[derive(Deserialize)]
struct ItemEntry {
    id: u16,
    components: ItemComponents,
}

#[derive(Deserialize)]
struct ItemComponents {
    #[serde(rename = "minecraft:use_remainder")]
    use_remainder: Option<UseRemainderComponent>,
}

#[derive(Deserialize)]
struct UseRemainderComponent {
    id: String,
}

/// Generates the `TokenStream` for the `get_recipe_remainder_id` function.
pub fn build() -> TokenStream {
    let items: BTreeMap<String, ItemEntry> =
        serde_json::from_str(&fs::read_to_string("../../assets/items.json").unwrap())
            .expect("Failed to parse items.json");

    let name_to_id: BTreeMap<&str, u16> = items
        .iter()
        .map(|(name, item)| (name.as_str(), item.id))
        .collect();

    let mut remainders: BTreeMap<u16, u16> = BTreeMap::new();
    for (_, item) in &items {
        if let Some(use_remainder) = &item.components.use_remainder {
            let remainder_name = use_remainder
                .id
                .strip_prefix("minecraft:")
                .unwrap_or(&use_remainder.id);
            let remainder_id = name_to_id
                .get(remainder_name)
                .copied()
                .unwrap_or_else(|| panic!("Unknown remainder item: {remainder_name}"));
            remainders.insert(item.id, remainder_id);
        }
    }

    let match_arms: Vec<TokenStream> = remainders
        .into_iter()
        .map(|(item_id, remainder_id)| {
            let item_id_lit = LitInt::new(&item_id.to_string(), Span::call_site());
            let remainder_id_lit = LitInt::new(&remainder_id.to_string(), Span::call_site());

            quote! {
                #item_id_lit => Some(#remainder_id_lit),
            }
        })
        .collect();
    quote! {
        #[must_use]
        #[allow(clippy::match_same_arms)]
        pub const fn get_recipe_remainder_id(item_id: u16) -> Option<u16> {
            match item_id {
                #(#match_arms)*
                _ => None,
            }
        }
    }
}
