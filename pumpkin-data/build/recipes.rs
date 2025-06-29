use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum RecipeTypes {
    #[serde(rename = "minecraft:blasting")]
    Blasting,
    #[serde(rename = "minecraft:campfire_cooking")]
    CampfireCooking,
    #[serde(rename = "minecraft:crafting_shaped")]
    CraftingShaped(CraftingShapedRecipeStruct),
    #[serde(rename = "minecraft:crafting_shapeless")]
    CraftingShapeless(CraftingShapelessRecipeStruct),
    #[serde(rename = "minecraft:crafting_transmute")]
    CraftingTransmute(CraftingTransmuteRecipeStruct),
    #[serde(rename = "minecraft:crafting_decorated_pot")]
    CraftingDecoratedPot(CraftingDecoratedPotStruct),
    #[serde(rename = "minecraft:smelting")]
    Smelting,
    #[serde(rename = "minecraft:smithing_transform")]
    SmithingTransform,
    #[serde(rename = "minecraft:smithing_trim")]
    SmithingTrim,
    #[serde(rename = "minecraft:smoking")]
    Smoking,
    #[serde(rename = "minecraft:stonecutting")]
    Stonecutting,
    #[serde(other)]
    #[serde(rename = "minecraft:crafting_special_*")]
    CraftingSpecial,
}

#[derive(Deserialize, Clone, Debug)]
pub struct CraftingShapedRecipeStruct {
    category: Option<RecipeCategoryTypes>,
    group: Option<String>,
    show_notification: Option<bool>,
    key: HashMap<String, RecipeIngredientTypes>,
    pattern: Vec<String>,
    result: RecipeResultStruct,
}

impl ToTokens for CraftingShapedRecipeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let category = match &self.category {
            Some(category) => category.to_token_stream(),
            None => RecipeCategoryTypes::Misc.to_token_stream(),
        };
        let group = match &self.group {
            Some(group) => quote! { Some(#group) },
            None => quote! { None },
        };
        let show_notification = self.show_notification.unwrap_or(true);
        let key = self
            .key
            .iter()
            .map(|(key, ingredient)| {
                let key = key.chars().next().unwrap();
                quote! { (#key, #ingredient) }
            })
            .collect::<Vec<_>>();
        let pattern = self
            .pattern
            .iter()
            .map(|c| c.to_token_stream())
            .collect::<Vec<_>>();
        let result = self.result.to_token_stream();

        tokens.extend(quote! {
            CraftingRecipeTypes::CraftingShaped {
                category: #category,
                group: #group,
                show_notification: #show_notification,
                key: &[#(#key),*],
                pattern: &[#(#pattern),*],
                result: #result,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct CraftingShapelessRecipeStruct {
    category: Option<RecipeCategoryTypes>,
    group: Option<String>,
    ingredients: Vec<RecipeIngredientTypes>,
    result: RecipeResultStruct,
}

impl ToTokens for CraftingShapelessRecipeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let category = match &self.category {
            Some(category) => category.to_token_stream(),
            None => RecipeCategoryTypes::Misc.to_token_stream(),
        };
        let group = match &self.group {
            Some(group) => quote! { Some(#group) },
            None => quote! { None },
        };
        let ingredients = self
            .ingredients
            .iter()
            .map(|c| c.to_token_stream())
            .collect::<Vec<_>>();
        let result = self.result.to_token_stream();

        tokens.extend(quote! {
            CraftingRecipeTypes::CraftingShapeless {
                category: #category,
                group: #group,
                ingredients: &[#(#ingredients),*],
                result: #result,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct CraftingTransmuteRecipeStruct {
    category: Option<RecipeCategoryTypes>,
    group: Option<String>,
    input: RecipeIngredientTypes,
    material: RecipeIngredientTypes,
    result: RecipeResultStruct,
}

impl ToTokens for CraftingTransmuteRecipeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let category = match &self.category {
            Some(category) => category.to_token_stream(),
            None => RecipeCategoryTypes::Misc.to_token_stream(),
        };
        let group = match &self.group {
            Some(group) => quote! { Some(#group) },
            None => quote! { None },
        };
        let input = self.input.to_token_stream();
        let material = self.material.to_token_stream();
        let result = self.result.to_token_stream();

        tokens.extend(quote! {
            CraftingRecipeTypes::CraftingTransmute {
                category: #category,
                group: #group,
                input: #input,
                material: #material,
                result: #result,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct CraftingDecoratedPotStruct {
    category: Option<RecipeCategoryTypes>,
}

impl ToTokens for CraftingDecoratedPotStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let category = match &self.category {
            Some(category) => category.to_token_stream(),
            None => RecipeCategoryTypes::Misc.to_token_stream(),
        };

        tokens.extend(quote! {
            CraftingRecipeTypes::CraftingDecoratedPot {
                category: #category,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct RecipeResultStruct {
    id: String,
    count: Option<u8>,
    // TODO: components: Option<RecipeResultComponentsStruct>,
}

impl ToTokens for RecipeResultStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let id = self.id.to_token_stream();
        let count = self.count.unwrap_or(1).to_token_stream();

        tokens.extend(quote! {
            RecipeResultStruct {
                id: #id,
                count: #count,
            }
        });
    }
}

#[derive(Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum RecipeIngredientTypes {
    Simple(String),
    OneOf(Vec<String>),
}

impl ToTokens for RecipeIngredientTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            RecipeIngredientTypes::Simple(ingredient) => {
                if ingredient.starts_with("#") {
                    quote! { RecipeIngredientTypes::Tagged(#ingredient) }
                } else {
                    quote! { RecipeIngredientTypes::Simple(#ingredient) }
                }
            }
            RecipeIngredientTypes::OneOf(ingredients) => {
                let ingredients = ingredients
                    .iter()
                    .map(|c| c.to_token_stream())
                    .collect::<Vec<_>>();
                quote! { RecipeIngredientTypes::OneOf(&[#(#ingredients),*]) }
            }
        };

        tokens.extend(name);
    }
}

#[derive(Deserialize, Clone, Debug)]
pub enum RecipeCategoryTypes {
    #[serde(rename = "equipment")]
    Equipment,
    #[serde(rename = "building")]
    Building,
    #[serde(rename = "redstone")]
    Restone,
    #[serde(rename = "misc")]
    Misc,
}

impl ToTokens for RecipeCategoryTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            RecipeCategoryTypes::Equipment => {
                quote! { RecipeCategoryTypes::Equipment }
            }
            RecipeCategoryTypes::Building => {
                quote! { RecipeCategoryTypes::Building }
            }
            RecipeCategoryTypes::Restone => {
                quote! { RecipeCategoryTypes::Restone }
            }
            RecipeCategoryTypes::Misc => {
                quote! { RecipeCategoryTypes::Misc }
            }
        };

        tokens.extend(name);
    }
}

pub(crate) fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../assets/recipes.json");

    let recipes_assets: Vec<RecipeTypes> =
        serde_json::from_str(include_str!("../../assets/recipes.json"))
            .expect("Failed to parse recipes.json");

    let mut crafting_recipes = Vec::new();

    for recipe in recipes_assets {
        match recipe {
            RecipeTypes::Blasting => {}
            RecipeTypes::CampfireCooking => {}
            RecipeTypes::CraftingShaped(recipe) => {
                crafting_recipes.push(recipe.to_token_stream());
            }
            RecipeTypes::CraftingShapeless(recipe) => {
                crafting_recipes.push(recipe.to_token_stream());
            }
            RecipeTypes::CraftingTransmute(recipe) => {
                crafting_recipes.push(recipe.to_token_stream());
            }
            RecipeTypes::CraftingDecoratedPot(recipe) => {
                crafting_recipes.push(recipe.to_token_stream());
            }
            RecipeTypes::Smelting => {}
            RecipeTypes::SmithingTransform => {}
            RecipeTypes::SmithingTrim => {}
            RecipeTypes::Smoking => {}
            RecipeTypes::Stonecutting => {}
            RecipeTypes::CraftingSpecial => {}
        }
    }

    quote! {
        use crate::tag::Tagable;
        use crate::item::Item;

        #[derive(Clone, Debug)]
        pub enum CraftingRecipeTypes {
            CraftingShaped {
                category: RecipeCategoryTypes,
                group: Option<&'static str>,
                show_notification: bool,
                key: &'static [(char, RecipeIngredientTypes)],
                pattern: &'static [&'static str],
                result: RecipeResultStruct,
            },
            CraftingShapeless {
                category: RecipeCategoryTypes,
                group: Option<&'static str>,
                ingredients: &'static [RecipeIngredientTypes],
                result: RecipeResultStruct,
            },
            CraftingTransmute {
                category: RecipeCategoryTypes,
                group: Option<&'static str>,
                input: RecipeIngredientTypes,
                material: RecipeIngredientTypes,
                result: RecipeResultStruct,
            },
            CraftingDecoratedPot {
                category: RecipeCategoryTypes,
            },
            CraftingSpecial,
        }

        #[derive(Clone, Debug)]
        pub struct RecipeResultStruct {
            pub id: &'static str,
            pub count: u8,
        }

        #[derive(Clone, Debug)]
        pub enum RecipeIngredientTypes {
            Simple(&'static str),
            Tagged(&'static str),
            OneOf(&'static [&'static str]),
        }

        impl RecipeIngredientTypes {
            pub fn match_item(&self, item: &Item) -> bool {
                match self {
                    RecipeIngredientTypes::Simple(ingredient) => {
                        let name = format!("minecraft:{}", item.registry_key);
                        name == *ingredient
                    }
                    RecipeIngredientTypes::Tagged(tag) => item
                        .is_tagged_with(tag)
                        .expect("Crafting recipe used invalid tag"),
                    RecipeIngredientTypes::OneOf(ingredients) => {
                        let name = format!("minecraft:{}", item.registry_key);
                        ingredients.contains(&name.as_str())
                    }
                }
            }
        }

        #[derive(Clone, Debug)]
        pub enum RecipeCategoryTypes {
            Equipment,
            Building,
            Restone,
            Misc,
        }

        pub static RECIPES_CRAFTING: &[CraftingRecipeTypes] = &[
            #(#crafting_recipes),*
        ];
    }
}
