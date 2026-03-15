use std::{collections::BTreeMap, fs};

use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use serde::Deserialize;

/// Deserialized recipe entry from `recipes.json`, tagged by the `"type"` field.
#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum RecipeTypes {
    /// Blast furnace recipe.
    #[serde(rename = "minecraft:blasting")]
    Blasting(CookingRecipeStruct),
    /// Campfire cooking recipe.
    #[serde(rename = "minecraft:campfire_cooking")]
    CampfireCooking(CookingRecipeStruct),
    /// Shaped crafting table recipe.
    #[serde(rename = "minecraft:crafting_shaped")]
    CraftingShaped(CraftingShapedRecipeStruct),
    /// Shapeless crafting table recipe.
    #[serde(rename = "minecraft:crafting_shapeless")]
    CraftingShapeless(CraftingShapelessRecipeStruct),
    /// Transmute crafting recipe (preserves NBT/components from one slot to another).
    #[serde(rename = "minecraft:crafting_transmute")]
    CraftingTransmute(CraftingTransmuteRecipeStruct),
    /// Decorated-pot crafting recipe.
    #[serde(rename = "minecraft:crafting_decorated_pot")]
    CraftingDecoratedPot(CraftingDecoratedPotStruct),
    /// Furnace smelting recipe.
    #[serde(rename = "minecraft:smelting")]
    Smelting(CookingRecipeStruct),
    /// Smithing table transform recipe (not yet codegen'd).
    #[serde(rename = "minecraft:smithing_transform")]
    SmithingTransform,
    /// Smithing table armor-trim recipe (not yet codegen'd).
    #[serde(rename = "minecraft:smithing_trim")]
    SmithingTrim,
    /// Smoker cooking recipe.
    #[serde(rename = "minecraft:smoking")]
    Smoking(CookingRecipeStruct),
    /// Stonecutter recipe (not yet codegen'd).
    #[serde(rename = "minecraft:stonecutting")]
    Stonecutting,
    /// Any special crafting recipe type (not yet codegen'd).
    #[serde(other)]
    #[serde(rename = "minecraft:crafting_special_*")]
    CraftingSpecial,
}

/// Deserialized cooking recipe (furnace, blast furnace, smoker, or campfire).
#[derive(Deserialize)]
pub struct CookingRecipeStruct {
    /// UI category for this recipe.
    category: Option<RecipeCategoryTypes>,
    /// Optional recipe group used for advancement tracking.
    group: Option<String>,
    /// The single ingredient required by this recipe.
    ingredient: RecipeIngredientTypes,
    /// Number of ticks required to cook this recipe.
    cookingtime: i32,
    /// Experience points awarded when the result is extracted.
    experience: f32,
    /// The item produced by this recipe.
    result: RecipeResultStruct,
}

impl CookingRecipeStruct {
    /// Generate a recipe ID based on result, ingredient, and cooking type
    /// Format: minecraft:{result}_from_{`cooking_type`}_{ingredient}
    fn generate_recipe_id(&self, cooking_type: &str) -> String {
        let result_name = self
            .result
            .id
            .strip_prefix("minecraft:")
            .unwrap_or(&self.result.id);
        let ingredient_name = match &self.ingredient {
            RecipeIngredientTypes::Simple(s) => {
                if s.starts_with('#') {
                    // Tagged ingredient - strip # and replace : with _
                    s.strip_prefix('#').unwrap_or(s).replace(':', "_")
                } else {
                    s.strip_prefix("minecraft:").unwrap_or(s).to_string()
                }
            }
            RecipeIngredientTypes::OneOf(items) => {
                // Use first item for ID generation
                items
                    .first()
                    .map_or("unknown", |s| s.strip_prefix("minecraft:").unwrap_or(s))
                    .to_string()
            }
        };
        format!("minecraft:{result_name}_from_{cooking_type}_{ingredient_name}")
    }

    /// Emits the cooking-recipe struct fields including the provided `recipe_id`.
    ///
    /// # Arguments
    /// – `tokens` – the token stream to extend.
    /// – `recipe_id` – the pre-generated vanilla-format recipe ID string.
    fn to_tokens_with_id(&self, tokens: &mut TokenStream, recipe_id: &str) {
        let category = match &self.category {
            Some(category) => category.to_token_stream(),
            None => RecipeCategoryTypes::Misc.to_token_stream(),
        };
        let group = if let Some(group) = &self.group {
            quote! { Some(#group) }
        } else {
            quote! { None }
        };
        let ingredient = self.ingredient.to_token_stream();
        let cookingtime = self.cookingtime.to_token_stream();
        let experience = self.experience.to_token_stream();
        let result = self.result.to_token_stream();

        tokens.extend(quote! {
                recipe_id: #recipe_id,
                category: #category,
                group: #group,
                ingredient: #ingredient,
                cookingtime: #cookingtime,
                experience: #experience,
                result: #result,
        });
    }
}

impl ToTokens for CookingRecipeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let category = match &self.category {
            Some(category) => category.to_token_stream(),
            None => RecipeCategoryTypes::Misc.to_token_stream(),
        };
        let group = if let Some(group) = &self.group {
            quote! { Some(#group) }
        } else {
            quote! { None }
        };
        let ingredient = self.ingredient.to_token_stream();
        let cookingtime = self.cookingtime.to_token_stream();
        let experience = self.experience.to_token_stream();
        let result = self.result.to_token_stream();

        tokens.extend(quote! {
            //CookingRecipeType::Blasting,CampfireCooking,Smelting,Smoking{
                category: #category,
                group: #group,
                ingredient: #ingredient,
                cookingtime: #cookingtime,
                experience: #experience,
                result: #result,
            //}
        });
    }
}

/// Deserialized shaped crafting recipe.
#[derive(Deserialize)]
pub struct CraftingShapedRecipeStruct {
    /// UI category for this recipe.
    category: Option<RecipeCategoryTypes>,
    /// Optional recipe group used for advancement tracking.
    group: Option<String>,
    /// Whether to show a toast notification when unlocked.
    show_notification: Option<bool>,
    /// Map from pattern key characters to their ingredient types.
    key: BTreeMap<String, RecipeIngredientTypes>,
    /// Row strings defining the crafting grid layout.
    pattern: Vec<String>,
    /// The item produced by this recipe.
    result: RecipeResultStruct,
}

impl ToTokens for CraftingShapedRecipeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let category = match &self.category {
            Some(category) => category.to_token_stream(),
            None => RecipeCategoryTypes::Misc.to_token_stream(),
        };
        let group = if let Some(group) = &self.group {
            quote! { Some(#group) }
        } else {
            quote! { None }
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
            .map(quote::ToTokens::to_token_stream)
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

/// Deserialized shapeless crafting recipe.
#[derive(Deserialize)]
pub struct CraftingShapelessRecipeStruct {
    /// UI category for this recipe.
    category: Option<RecipeCategoryTypes>,
    /// Optional recipe group used for advancement tracking.
    group: Option<String>,
    /// The unordered list of ingredients required.
    ingredients: Vec<RecipeIngredientTypes>,
    /// The item produced by this recipe.
    result: RecipeResultStruct,
}

impl ToTokens for CraftingShapelessRecipeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let category = match &self.category {
            Some(category) => category.to_token_stream(),
            None => RecipeCategoryTypes::Misc.to_token_stream(),
        };
        let group = if let Some(group) = &self.group {
            quote! { Some(#group) }
        } else {
            quote! { None }
        };
        let ingredients = self
            .ingredients
            .iter()
            .map(quote::ToTokens::to_token_stream)
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

/// Deserialized transmute crafting recipe (copies components from input to result).
#[derive(Deserialize)]
pub struct CraftingTransmuteRecipeStruct {
    /// UI category for this recipe.
    category: Option<RecipeCategoryTypes>,
    /// Optional recipe group used for advancement tracking.
    group: Option<String>,
    /// The item whose data components are copied to the result.
    input: RecipeIngredientTypes,
    /// The material item consumed alongside `input`.
    material: RecipeIngredientTypes,
    /// The base item type of the result (inherits components from `input`).
    result: RecipeResultStruct,
}

impl ToTokens for CraftingTransmuteRecipeStruct {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let category = match &self.category {
            Some(category) => category.to_token_stream(),
            None => RecipeCategoryTypes::Misc.to_token_stream(),
        };
        let group = if let Some(group) = &self.group {
            quote! { Some(#group) }
        } else {
            quote! { None }
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

/// Deserialized decorated-pot crafting recipe.
#[derive(Deserialize)]
pub struct CraftingDecoratedPotStruct {
    /// UI category for this recipe.
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

/// Deserialized recipe result specifying the output item and count.
#[derive(Deserialize)]
pub struct RecipeResultStruct {
    /// Registry key of the result item.
    id: String,
    /// Number of result items produced (defaults to 1).
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

/// Deserialized recipe ingredient, which is either a single item/tag or a list of alternatives.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum RecipeIngredientTypes {
    /// A single item registry key or tag (prefixed with `#`).
    Simple(String),
    /// A list of acceptable alternative item registry keys.
    OneOf(Vec<String>),
}

impl ToTokens for RecipeIngredientTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::Simple(ingredient) => {
                if ingredient.starts_with('#') {
                    quote! { RecipeIngredientTypes::Tagged(#ingredient) }
                } else {
                    quote! { RecipeIngredientTypes::Simple(#ingredient) }
                }
            }
            Self::OneOf(ingredients) => {
                let ingredients = ingredients
                    .iter()
                    .map(quote::ToTokens::to_token_stream)
                    .collect::<Vec<_>>();
                quote! { RecipeIngredientTypes::OneOf(&[#(#ingredients),*]) }
            }
        };

        tokens.extend(name);
    }
}

/// Deserialized recipe UI category used to group recipes in the recipe book.
#[derive(Deserialize)]
pub enum RecipeCategoryTypes {
    /// Equipment recipes (tools, weapons, armor).
    #[serde(rename = "equipment")]
    Equipment,
    /// Building block recipes.
    #[serde(rename = "building")]
    Building,
    /// Redstone component recipes.
    #[serde(rename = "redstone")]
    Restone,
    /// Miscellaneous recipes that don't fit other categories.
    #[serde(rename = "misc")]
    Misc,
    /// Food item recipes.
    #[serde(rename = "food")]
    Food,
    /// Block recipes.
    #[serde(rename = "blocks")]
    Blocks,
}

impl ToTokens for RecipeCategoryTypes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = match self {
            Self::Equipment => {
                quote! { RecipeCategoryTypes::Equipment }
            }
            Self::Building => {
                quote! { RecipeCategoryTypes::Building }
            }
            Self::Restone => {
                quote! { RecipeCategoryTypes::Restone }
            }
            Self::Misc => {
                quote! { RecipeCategoryTypes::Misc }
            }
            Self::Food => {
                quote! { RecipeCategoryTypes::Food }
            }
            Self::Blocks => {
                quote! { RecipeCategoryTypes::Blocks }
            }
        };

        tokens.extend(name);
    }
}

/// Reads `recipes.json` and emits the complete recipe constants and helpers `TokenStream`.
pub fn build() -> TokenStream {
    let recipes_assets: Vec<RecipeTypes> =
        serde_json::from_str(&fs::read_to_string("../assets/recipes.json").unwrap())
            .expect("Failed to parse recipes.json");

    let mut crafting_recipes = Vec::new();
    let mut cooking_recipes = Vec::new();

    for recipe in recipes_assets {
        match recipe {
            RecipeTypes::Blasting(recipe) => {
                let recipe_id = recipe.generate_recipe_id("blasting");
                let mut common_cooking_token = TokenStream::new();
                recipe.to_tokens_with_id(&mut common_cooking_token, &recipe_id);
                let blasting_token = quote! {
                    CookingRecipeType::Blasting (CookingRecipe {
                        #common_cooking_token
                    })
                };
                cooking_recipes.push(blasting_token);
            }
            RecipeTypes::CampfireCooking(recipe) => {
                let recipe_id = recipe.generate_recipe_id("campfire_cooking");
                let mut common_cooking_token = TokenStream::new();
                recipe.to_tokens_with_id(&mut common_cooking_token, &recipe_id);
                let campfire_token = quote! {
                    CookingRecipeType::CampfireCooking (CookingRecipe {
                        #common_cooking_token
                    })
                };
                cooking_recipes.push(campfire_token);
            }
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
            RecipeTypes::Smelting(recipe) => {
                let recipe_id = recipe.generate_recipe_id("smelting");
                let mut common_cooking_token = TokenStream::new();
                recipe.to_tokens_with_id(&mut common_cooking_token, &recipe_id);
                let smelting_token = quote! {
                    CookingRecipeType::Smelting(CookingRecipe {
                        #common_cooking_token
                    })
                };
                cooking_recipes.push(smelting_token);
            }
            RecipeTypes::SmithingTransform => {}
            RecipeTypes::SmithingTrim => {}
            RecipeTypes::Smoking(recipe) => {
                let recipe_id = recipe.generate_recipe_id("smoking");
                let mut common_cooking_token = TokenStream::new();
                recipe.to_tokens_with_id(&mut common_cooking_token, &recipe_id);
                let smoking_token = quote! {
                    CookingRecipeType::Smoking(CookingRecipe{
                        #common_cooking_token
                    })
                };
                cooking_recipes.push(smoking_token);
            }
            RecipeTypes::Stonecutting => {}
            RecipeTypes::CraftingSpecial => {}
        }
    }

    quote! {
        use crate::tag::Taggable;
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

        #[allow(dead_code)]
        #[derive(Clone, Debug)]
        pub struct CookingRecipe {
            /// Vanilla-compatible recipe ID (e.g., "minecraft:iron_ingot_from_smelting_iron_ore")
            pub recipe_id: &'static str,
            pub category: RecipeCategoryTypes,
            pub group: Option<&'static str>,
            pub ingredient: RecipeIngredientTypes,
            pub cookingtime: i32,
            pub experience: f32,
            pub result: RecipeResultStruct,
        }

        #[derive(Clone, Debug)]
        pub enum CookingRecipeType {
            Blasting(CookingRecipe),
            Smelting(CookingRecipe),
            Smoking(CookingRecipe),
            CampfireCooking(CookingRecipe),
        }
        #[derive(Clone, Debug)]
        pub enum CookingRecipeKind {
            Blasting,
            Smelting,
            Smoking,
            CampfireCooking,
        }

        impl From<&CookingRecipeType> for CookingRecipeKind {
            fn from(recipe_type: &CookingRecipeType) -> Self {
                match recipe_type {
                    CookingRecipeType::Blasting(_) => CookingRecipeKind::Blasting,
                    CookingRecipeType::Smelting(_) => CookingRecipeKind::Smelting,
                    CookingRecipeType::Smoking(_) => CookingRecipeKind::Smoking,
                    CookingRecipeType::CampfireCooking(_) => CookingRecipeKind::CampfireCooking,
                }
            }
        }

        impl From<CookingRecipeType> for CookingRecipeKind {
            fn from(recipe_type: CookingRecipeType) -> Self {
                match recipe_type {
                    CookingRecipeType::Blasting(_) => CookingRecipeKind::Blasting,
                    CookingRecipeType::Smelting(_) => CookingRecipeKind::Smelting,
                    CookingRecipeType::Smoking(_) => CookingRecipeKind::Smoking,
                    CookingRecipeType::CampfireCooking(_) => CookingRecipeKind::CampfireCooking,
                }
            }
        }

        impl CookingRecipeKind {
            pub fn to_type(self, recipe: CookingRecipe) -> CookingRecipeType {
                match self {
                    CookingRecipeKind::Blasting => CookingRecipeType::Blasting(recipe),
                    CookingRecipeKind::Smelting => CookingRecipeType::Smelting(recipe),
                    CookingRecipeKind::Smoking => CookingRecipeType::Smoking(recipe),
                    CookingRecipeKind::CampfireCooking => CookingRecipeType::CampfireCooking(recipe),
                }
            }
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
            Food,
            Blocks,
        }

        pub static RECIPES_CRAFTING: &[CraftingRecipeTypes] = &[
            #(#crafting_recipes),*
        ];
        pub static RECIPES_COOKING: &[CookingRecipeType] = &[
            #(#cooking_recipes ),*
        ];

        pub fn get_cooking_recipe_with_ingredient(ingredient: &Item, recipe_type: CookingRecipeKind) -> Option<&'static CookingRecipe> {
            RECIPES_COOKING
                .iter()
                .find_map(|recipe| match (recipe, &recipe_type) {
                    (CookingRecipeType::Blasting(cooking_recipe), CookingRecipeKind::Blasting)
                    | (CookingRecipeType::Smelting(cooking_recipe), CookingRecipeKind::Smelting)
                    | (CookingRecipeType::Smoking(cooking_recipe), CookingRecipeKind::Smoking)
                    | (
                        CookingRecipeType::CampfireCooking(cooking_recipe),
                        CookingRecipeKind::CampfireCooking,
                    ) => {
                        cooking_recipe.ingredient.match_item(ingredient).then_some(cooking_recipe)
                    }
                    _ => None,
                })
        }

        /// Get the experience value for a recipe by its recipe ID.
        /// Used for calculating XP when extracting from furnace.
        /// Recipe IDs are in vanilla format like "minecraft:iron_ingot_from_smelting_iron_ore"
        pub fn get_recipe_experience(recipe_id: &str) -> Option<f32> {
            RECIPES_COOKING.iter().find_map(|recipe| {
                let cooking_recipe = match recipe {
                    CookingRecipeType::Blasting(r)
                    | CookingRecipeType::Smelting(r)
                    | CookingRecipeType::Smoking(r)
                    | CookingRecipeType::CampfireCooking(r) => r,
                };
                (cooking_recipe.recipe_id == recipe_id).then_some(cooking_recipe.experience)
            })
        }
    }
}
