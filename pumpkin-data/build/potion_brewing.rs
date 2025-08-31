use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Clone, Hash, Eq, PartialEq)]
struct PotionBrewing {
    potion_types: Vec<Vec<String>>,
    potion_recipes: Vec<Recipes>,
    item_recipes: Vec<Recipes>,
}

#[derive(Deserialize, Clone, Hash, Eq, PartialEq)]
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

    let mut item = TokenStream::new();
    let mut potion = TokenStream::new();

    let item_len = json.item_recipes.len();
    let potion_len = json.potion_recipes.len();

    for j in json.item_recipes {
        item.extend(j.get_tokens_item());
    }
    for j in json.potion_recipes {
        potion.extend(j.get_tokens_potion());
    }

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

        pub const ITEM_RECIPES: [ItemRecipe; #item_len] = [#item];
        pub const POTION_RECIPES: [PotionRecipe; #potion_len] = [#potion];
    }
}
