use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct DataDrivenBrewingRecipe {
    input: BrewingInput,
    reagent: BrewingReagent,
    output: BrewingOutput,
}

#[derive(Deserialize)]
struct BrewingInput {
    item: String,
    potion_contents: BrewingPotionContents,
}

#[derive(Deserialize)]
struct BrewingPotionContents {
    #[serde(alias = "potion")]
    potions: String,
}

#[derive(Deserialize)]
struct BrewingReagent {
    item: String,
}

#[derive(Deserialize)]
struct BrewingOutput {
    id: String,
    components: BrewingComponents,
}

#[derive(Deserialize)]
struct BrewingComponents {
    #[serde(rename = "minecraft:potion_contents")]
    potion_contents: BrewingOutputPotionContents,
}

#[derive(Deserialize)]
struct BrewingOutputPotionContents {
    potion: String,
}

/// Generates the `TokenStream` for `BREWING_RECIPES`, `POTION_RECIPES` and `ITEM_RECIPES`.
pub fn build() -> TokenStream {
    // 2. Load 26.3 data-driven brewing recipes from datapack
    let brewing_dir = Path::new("../../assets/datapacks/26_2/data/minecraft/recipe/brewing");
    let mut entries: Vec<_> = fs::read_dir(brewing_dir)
        .expect("Missing brewing recipe directory")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect();
    entries.sort_by_key(|e| e.path());

    let mut brewing_tokens = Vec::new();
    for entry in entries {
        let content = fs::read_to_string(entry.path()).expect("Failed to read brewing recipe");
        let recipe: DataDrivenBrewingRecipe = serde_json::from_str(&content).unwrap_or_else(|e| {
            panic!(
                "Failed to parse brewing recipe {}: {e}",
                entry.path().display()
            )
        });

        let from_item = format_ident!(
            "{}",
            recipe
                .input
                .item
                .strip_prefix("minecraft:")
                .unwrap_or(&recipe.input.item)
                .to_uppercase()
        );
        let from_potion = format_ident!(
            "{}",
            recipe
                .input
                .potion_contents
                .potions
                .strip_prefix("minecraft:")
                .unwrap_or(&recipe.input.potion_contents.potions)
                .to_uppercase()
        );
        let ingredient = format_ident!(
            "{}",
            recipe
                .reagent
                .item
                .strip_prefix("minecraft:")
                .unwrap_or(&recipe.reagent.item)
                .to_uppercase()
        );
        let to_item = format_ident!(
            "{}",
            recipe
                .output
                .id
                .strip_prefix("minecraft:")
                .unwrap_or(&recipe.output.id)
                .to_uppercase()
        );
        let to_potion = format_ident!(
            "{}",
            recipe
                .output
                .components
                .potion_contents
                .potion
                .strip_prefix("minecraft:")
                .unwrap_or(&recipe.output.components.potion_contents.potion)
                .to_uppercase()
        );

        brewing_tokens.push(quote! {
            BrewingRecipe {
                from_item: &Item::#from_item,
                from_potion: &Potion::#from_potion,
                ingredient: &Item::#ingredient,
                to_item: &Item::#to_item,
                to_potion: &Potion::#to_potion,
            }
        });
    }
    let brewing_len = brewing_tokens.len();

    quote! {
        #![allow(dead_code)]
        use crate::potion::Potion;
        use crate::item::Item;

        pub struct BrewingRecipe {
            pub from_item: &'static Item,
            pub from_potion: &'static Potion,
            pub ingredient: &'static Item,
            pub to_item: &'static Item,
            pub to_potion: &'static Potion,
        }

        impl BrewingRecipe {
            #[must_use]
            pub const fn from_item(&self) -> &'static Item { self.from_item }
            #[must_use]
            pub const fn from_potion(&self) -> &'static Potion { self.from_potion }
            #[must_use]
            pub const fn ingredient(&self) -> &'static Item { self.ingredient }
            #[must_use]
            pub const fn reagent(&self) -> &'static Item { self.ingredient }
            #[must_use]
            pub const fn to_item(&self) -> &'static Item { self.to_item }
            #[must_use]
            pub const fn to_potion(&self) -> &'static Potion { self.to_potion }
        }

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

        impl PotionRecipe {
            #[must_use]
            pub const fn from(&self) -> &'static Potion { self.from }
            #[must_use]
            pub const fn ingredient(&self) -> &'static [&'static Item] { self.ingredient }
            #[must_use]
            pub const fn to(&self) -> &'static Potion { self.to }
        }

        impl ItemRecipe {
            #[must_use]
            pub const fn from(&self) -> &'static Item { self.from }
            #[must_use]
            pub const fn ingredient(&self) -> &'static [&'static Item] { self.ingredient }
            #[must_use]
            pub const fn to(&self) -> &'static Item { self.to }
        }

        pub const BREWING_RECIPES: [BrewingRecipe; #brewing_len] = [#(#brewing_tokens),*];
        pub const ITEM_RECIPES: [ItemRecipe; 0] = [];
        pub const POTION_RECIPES: [PotionRecipe; 0] = [];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_potion_brewing() {
        let _ = build();
    }
}
