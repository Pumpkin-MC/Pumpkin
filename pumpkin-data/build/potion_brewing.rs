use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct PotionBrewing {
    //potion_types: Vec<Vec<String>>,
    potion_recipes: Vec<Recipes>,
    item_recipes: Vec<Recipes>,
}

#[derive(Deserialize)]
pub struct Recipes {
    from: String,
    ingredient: Vec<String>,
    to: String,
}

impl Recipes {
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

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/potion_brewing.json");

    let json: PotionBrewing =
        serde_json::from_str(&fs::read_to_string("../assets/potion_brewing.json").unwrap())
            .expect("Failed to parse potion_brewing.json");

    let item_recipes_tokens: Vec<TokenStream> = json
        .item_recipes
        .into_iter()
        .map(|recipe| recipe.get_tokens_item())
        .collect();
    let item_len = item_recipes_tokens.len();

    // 3. Generate Potion Recipes
    let potion_recipes_tokens: Vec<TokenStream> = json
        .potion_recipes
        .into_iter()
        .map(|recipe| recipe.get_tokens_potion())
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
