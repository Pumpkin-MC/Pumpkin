use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::fs;

/// Raw deserialization shape for the brewing recipe data from `potion_brewing.json`.
#[derive(Deserialize)]
struct PotionBrewing {
    //potion_types: Vec<Vec<String>>,
    /// Recipes that transform one potion into another using a brewing ingredient.
    potion_recipes: Vec<Recipes>,
    /// Recipes that transform one item into another using a brewing ingredient.
    item_recipes: Vec<Recipes>,
}

/// A single brewing recipe entry mapping a source item/potion and ingredient to an output.
#[derive(Deserialize)]
pub struct Recipes {
    /// Namespaced resource location of the input potion or item.
    from: String,
    /// List of namespaced resource locations for valid brewing ingredients.
    ingredient: Vec<String>,
    /// Namespaced resource location of the output potion or item.
    to: String,
}

impl Recipes {
    /// Converts this recipe into a `TokenStream` for a `PotionRecipe` struct literal.
    pub fn get_tokens_potion(self) -> TokenStream {
        let from = format_ident!(
            "{}",
            self.from.strip_prefix("minecraft:").unwrap().to_uppercase()
        );
        let to = format_ident!(
            "{}",
            self.to.strip_prefix("minecraft:").unwrap().to_uppercase()
        );

        let slots = self.ingredient.iter().map(|slot| {
            format_ident!(
                "{}",
                slot.strip_prefix("minecraft:").unwrap().to_uppercase()
            )
        });

        quote! {
            PotionRecipe {
                from: &Potion::#from,
                ingredient: &[#(&Item::#slots),*],
                to: &Potion::#to,
            },
        }
    }

    /// Converts this recipe into a `TokenStream` for an `ItemRecipe` struct literal.
    pub fn get_tokens_item(self) -> TokenStream {
        let from = format_ident!(
            "{}",
            self.from.strip_prefix("minecraft:").unwrap().to_uppercase()
        );
        let to = format_ident!(
            "{}",
            self.to.strip_prefix("minecraft:").unwrap().to_uppercase()
        );

        let slots = self.ingredient.iter().map(|slot| {
            format_ident!(
                "{}",
                slot.strip_prefix("minecraft:").unwrap().to_uppercase()
            )
        });

        quote! {
            ItemRecipe {
                from: &Item::#from,
                ingredient: &[#(&Item::#slots),*],
                to: &Item::#to,
            },
        }
    }
}

/// Generates the `TokenStream` for `POTION_RECIPES` and `ITEM_RECIPES` constant arrays.
pub fn build() -> TokenStream {
    let json: PotionBrewing =
        serde_json::from_str(&fs::read_to_string("../assets/potion_brewing.json").unwrap())
            .expect("Failed to parse potion_brewing.json");

    let item_recipes_tokens: Vec<TokenStream> = json
        .item_recipes
        .into_iter()
        .map(Recipes::get_tokens_item)
        .collect();
    let item_len = item_recipes_tokens.len();

    // 3. Generate Potion Recipes
    let potion_recipes_tokens: Vec<TokenStream> = json
        .potion_recipes
        .into_iter()
        .map(Recipes::get_tokens_potion)
        .collect();
    let potion_len = potion_recipes_tokens.len();

    quote! {
        #![allow(dead_code)]
        use crate::potion::Potion;
        use crate::item::Item;

        pub struct PotionRecipe {
            from: &'static Potion,
            ingredient: &'static [&'static Item],
            to: &'static Potion,
        }

        pub struct ItemRecipe {
            from: &'static Item,
            ingredient: &'static [&'static Item],
            to: &'static Item,
        }

        pub const ITEM_RECIPES: [ItemRecipe; #item_len] = [#(#item_recipes_tokens)*];
        pub const POTION_RECIPES: [PotionRecipe; #potion_len] = [#(#potion_recipes_tokens)*];
    }
}
