//! Plugin recipe registration and builder utilities.
//!
//! This module provides a fluent, type-safe API for defining and registering custom
//! crafting recipes (shaped and shapeless) as well as cooking recipes (smelting, blasting,
//! smoking, and campfire).
//!
//! # Examples
//!
//! ## Registering a Shaped Recipe
//! ```rust,ignore
//! use pumpkin_plugin_api::{
//!     recipe::{Ingredient, RecipeCategory, ShapedRecipeBuilder},
//!     ItemStack, Server,
//! };
//!
//! fn register_recipes(server: &Server) {
//!     let manager = server.get_recipe_manager();
//!
//!     manager.register(
//!         ShapedRecipeBuilder::new("my_plugin:super_sword", ItemStack::new("minecraft:diamond_sword", 1))
//!             .pattern([
//!                 " D ",
//!                 " D ",
//!                 " S ",
//!             ])
//!             .key('D', "minecraft:diamond_block")
//!             .key('S', "minecraft:stick")
//!             .category(RecipeCategory::Equipment)
//!             .group("swords")
//!             .show_notification(true)
//!     ).expect("failed to register shaped recipe");
//! }
//! ```
//!
//! ## Registering a Shapeless Recipe
//! ```rust,ignore
//! use pumpkin_plugin_api::{
//!     recipe::{Ingredient, RecipeCategory, ShapelessRecipeBuilder},
//!     ItemStack, Server,
//! };
//!
//! fn register_recipes(server: &Server) {
//!     let manager = server.get_recipe_manager();
//!
//!     manager.register(
//!         ShapelessRecipeBuilder::new("my_plugin:flint_from_gravel", ItemStack::new("minecraft:flint", 1))
//!             .ingredient_count("minecraft:gravel", 3)
//!             .category(RecipeCategory::Misc)
//!     ).expect("failed to register shapeless recipe");
//! }
//! ```
//!
//! ## Registering a Smelting / Cooking Recipe
//! ```rust,ignore
//! use pumpkin_plugin_api::{
//!     recipe::{CookingRecipeBuilder, RecipeCategory},
//!     ItemStack, Server,
//! };
//!
//! fn register_recipes(server: &Server) {
//!     let manager = server.get_recipe_manager();
//!
//!     manager.register(
//!         CookingRecipeBuilder::smelting(
//!             "my_plugin:fast_iron",
//!             "minecraft:raw_iron",
//!             ItemStack::new("minecraft:iron_ingot", 1),
//!         )
//!         .cooking_time(100)
//!         .experience(0.7)
//!         .category(RecipeCategory::Misc)
//!     ).expect("failed to register smelting recipe");
//! }
//! ```

use std::collections::HashMap;

pub use crate::wit::pumpkin::plugin::recipe::{
    CookingRecipe, CookingType, Ingredient as WitIngredient, RecipeCategory, RecipeManager,
    ShapedRecipe, ShapelessRecipe,
};
use crate::{Context, ItemStack, Server};

/// Represents an ingredient in a recipe.
///
/// Ingredients can be:
/// - A specific item ID (e.g. `"minecraft:diamond"` or `"diamond"`).
/// - A tag representing a group of items (e.g. `"#minecraft:logs"` or `"#logs"`).
/// - One of multiple specific items.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ingredient {
    /// A specific item by registry key (e.g., `"minecraft:diamond"`).
    Item(String),
    /// A tag representing multiple items (e.g., `"minecraft:logs"`).
    Tag(String),
    /// One of multiple specific items.
    OneOf(Vec<String>),
}

impl Ingredient {
    /// Creates an ingredient matching a specific item ID.
    ///
    /// If no namespace is provided (e.g., `"diamond"`), `"minecraft:"` is prepended.
    #[must_use]
    pub fn item(id: impl AsRef<str>) -> Self {
        let s = id.as_ref();
        let normalized = if s.contains(':') {
            s.to_string()
        } else {
            format!("minecraft:{s}")
        };
        Self::Item(normalized)
    }

    /// Creates an ingredient matching a tag group of items.
    ///
    /// If no namespace is provided (e.g., `"logs"`), `"minecraft:"` is prepended.
    /// Any leading `#` character is automatically stripped.
    #[must_use]
    pub fn tag(tag: impl AsRef<str>) -> Self {
        let mut s = tag.as_ref();
        if let Some(stripped) = s.strip_prefix('#') {
            s = stripped;
        }
        let normalized = if s.contains(':') {
            s.to_string()
        } else {
            format!("minecraft:{s}")
        };
        Self::Tag(normalized)
    }

    /// Creates an ingredient matching any of the provided item IDs.
    pub fn one_of<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let list: Vec<String> = items
            .into_iter()
            .map(|s| {
                let s = s.as_ref();
                if s.contains(':') {
                    s.to_string()
                } else {
                    format!("minecraft:{s}")
                }
            })
            .collect();
        Self::OneOf(list)
    }

    /// Alias for [`Ingredient::one_of`].
    pub fn items<I, S>(items: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self::one_of(items)
    }
}

impl From<&str> for Ingredient {
    fn from(s: &str) -> Self {
        if s.starts_with('#') {
            Self::tag(&s[1..])
        } else {
            Self::item(s)
        }
    }
}

impl From<String> for Ingredient {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<&String> for Ingredient {
    fn from(s: &String) -> Self {
        s.as_str().into()
    }
}

impl<const N: usize> From<[&str; N]> for Ingredient {
    fn from(arr: [&str; N]) -> Self {
        Self::one_of(arr)
    }
}

impl<const N: usize> From<[String; N]> for Ingredient {
    fn from(arr: [String; N]) -> Self {
        Self::one_of(arr)
    }
}

impl From<Vec<String>> for Ingredient {
    fn from(vec: Vec<String>) -> Self {
        Self::one_of(vec)
    }
}

impl From<Vec<&str>> for Ingredient {
    fn from(vec: Vec<&str>) -> Self {
        Self::one_of(vec)
    }
}

impl From<&[String]> for Ingredient {
    fn from(slice: &[String]) -> Self {
        Self::one_of(slice)
    }
}

impl From<&[&str]> for Ingredient {
    fn from(slice: &[&str]) -> Self {
        Self::one_of(slice)
    }
}

impl From<Ingredient> for WitIngredient {
    fn from(ing: Ingredient) -> Self {
        match ing {
            Ingredient::Item(id) => Self::Item(id),
            Ingredient::Tag(tag) => Self::Tag(tag),
            Ingredient::OneOf(items) => Self::OneOf(items),
        }
    }
}

impl From<WitIngredient> for Ingredient {
    fn from(ing: WitIngredient) -> Self {
        match ing {
            WitIngredient::Item(id) => Self::Item(id),
            WitIngredient::Tag(tag) => Self::Tag(tag),
            WitIngredient::OneOf(items) => Self::OneOf(items),
        }
    }
}

/// Errors that can occur when building or validating a recipe.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipeError {
    /// The recipe identifier is empty.
    EmptyId,
    /// Pattern is empty.
    EmptyPattern,
    /// Pattern dimensions exceed the crafting grid (maximum 3x3).
    PatternTooLarge {
        /// Width of the pattern.
        width: usize,
        /// Height of the pattern.
        height: usize,
    },
    /// Rows in pattern have inconsistent lengths.
    InconsistentRowWidth {
        /// Expected width based on the first row.
        expected: usize,
        /// Width of the offending row.
        found: usize,
    },
    /// A character in the pattern has no matching ingredient key.
    MissingKey(char),
    /// No ingredients provided in a shapeless recipe.
    NoIngredients,
    /// Too many ingredients in a shapeless recipe (maximum 9).
    TooManyIngredients(usize),
    /// No input ingredient provided for cooking recipe.
    MissingCookingInput,
}

impl std::fmt::Display for RecipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyId => write!(f, "recipe identifier cannot be empty"),
            Self::EmptyPattern => write!(f, "shaped recipe pattern cannot be empty"),
            Self::PatternTooLarge { width, height } => {
                write!(
                    f,
                    "shaped recipe pattern is too large ({width}x{height}, max 3x3)"
                )
            }
            Self::InconsistentRowWidth { expected, found } => {
                write!(
                    f,
                    "inconsistent row width in shaped recipe pattern (expected {expected}, found {found})"
                )
            }
            Self::MissingKey(ch) => write!(
                f,
                "pattern contains character '{ch}' with no corresponding ingredient key"
            ),
            Self::NoIngredients => write!(f, "shapeless recipe requires at least one ingredient"),
            Self::TooManyIngredients(count) => {
                write!(
                    f,
                    "shapeless recipe has too many ingredients ({count}, max 9)"
                )
            }
            Self::MissingCookingInput => write!(f, "cooking recipe requires an input ingredient"),
        }
    }
}

impl std::error::Error for RecipeError {}

/// Builder for constructing and registering shaped crafting recipes.
pub struct ShapedRecipeBuilder {
    id: String,
    pattern: Vec<String>,
    keys: HashMap<char, Ingredient>,
    output: ItemStack,
    group: Option<String>,
    category: Option<RecipeCategory>,
    show_notification: Option<bool>,
}

impl ShapedRecipeBuilder {
    /// Creates a new shaped recipe builder with a unique recipe ID and output item stack.
    #[must_use]
    pub fn new(id: impl Into<String>, output: ItemStack) -> Self {
        Self {
            id: id.into(),
            pattern: Vec::new(),
            keys: HashMap::new(),
            output,
            group: None,
            category: None,
            show_notification: None,
        }
    }

    /// Sets the entire pattern for the shaped recipe.
    ///
    /// Each string in the iterator represents a row in the crafting grid (e.g. `["# #", " s ", "# #"]`).
    #[must_use]
    pub fn pattern<I, S>(mut self, pattern: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.pattern = pattern
            .into_iter()
            .map(|s| s.as_ref().to_string())
            .collect();
        self
    }

    /// Alias for [`ShapedRecipeBuilder::pattern`].
    #[must_use]
    pub fn shape<I, S>(self, pattern: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.pattern(pattern)
    }

    /// Adds a single row to the shaped recipe pattern.
    #[must_use]
    pub fn row(mut self, row: impl AsRef<str>) -> Self {
        self.pattern.push(row.as_ref().to_string());
        self
    }

    /// Defines an ingredient for a character symbol used in the pattern.
    #[must_use]
    pub fn key(mut self, symbol: char, ingredient: impl Into<Ingredient>) -> Self {
        self.keys.insert(symbol, ingredient.into());
        self
    }

    /// Sets the recipe group.
    #[must_use]
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Sets the recipe category in the recipe book.
    #[must_use]
    pub const fn category(mut self, category: RecipeCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Sets whether to show a toast notification when the player unlocks this recipe.
    #[must_use]
    pub const fn show_notification(mut self, show: bool) -> Self {
        self.show_notification = Some(show);
        self
    }

    /// Validates the recipe configuration.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if the ID or pattern is empty, rows are mismatched,
    /// dimensions exceed 3x3, or characters lack ingredient keys.
    pub fn validate(&self) -> Result<(), RecipeError> {
        if self.id.is_empty() {
            return Err(RecipeError::EmptyId);
        }
        if self.pattern.is_empty() {
            return Err(RecipeError::EmptyPattern);
        }
        let height = self.pattern.len();
        let width = self.pattern[0].chars().count();
        if height > 3 || width > 3 || width == 0 {
            return Err(RecipeError::PatternTooLarge { width, height });
        }
        for row in &self.pattern {
            let row_width = row.chars().count();
            if row_width != width {
                return Err(RecipeError::InconsistentRowWidth {
                    expected: width,
                    found: row_width,
                });
            }
            for ch in row.chars() {
                if ch != ' ' && !self.keys.contains_key(&ch) {
                    return Err(RecipeError::MissingKey(ch));
                }
            }
        }
        Ok(())
    }

    /// Builds the shaped recipe structure after validating.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn build(self) -> Result<(String, ShapedRecipe), RecipeError> {
        self.validate()?;
        let key_list: Vec<(String, WitIngredient)> = self
            .keys
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.into()))
            .collect();

        let recipe = ShapedRecipe {
            pattern: self.pattern,
            key: key_list,
            output: self.output,
            group: self.group,
            category: self.category,
            show_notification: self.show_notification,
        };
        Ok((self.id, recipe))
    }

    /// Registers this shaped recipe directly with the provided [`RecipeManager`].
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register(self, manager: &RecipeManager) -> Result<(), RecipeError> {
        let (id, recipe) = self.build()?;
        manager.register_shaped(&id, recipe);
        Ok(())
    }

    /// Registers this shaped recipe with the server.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_to_server(self, server: &Server) -> Result<(), RecipeError> {
        let manager = server.get_recipe_manager();
        self.register(&manager)
    }

    /// Registers this shaped recipe with the plugin context.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_to_context(self, context: &Context) -> Result<(), RecipeError> {
        let manager = context.get_recipe_manager();
        self.register(&manager)
    }
}

/// Builder for constructing and registering shapeless crafting recipes.
pub struct ShapelessRecipeBuilder {
    id: String,
    ingredients: Vec<Ingredient>,
    output: ItemStack,
    group: Option<String>,
    category: Option<RecipeCategory>,
}

impl ShapelessRecipeBuilder {
    /// Creates a new shapeless recipe builder with a unique recipe ID and output item stack.
    #[must_use]
    pub fn new(id: impl Into<String>, output: ItemStack) -> Self {
        Self {
            id: id.into(),
            ingredients: Vec::new(),
            output,
            group: None,
            category: None,
        }
    }

    /// Adds an ingredient to the recipe.
    #[must_use]
    pub fn ingredient(mut self, ingredient: impl Into<Ingredient>) -> Self {
        self.ingredients.push(ingredient.into());
        self
    }

    /// Alias for [`ShapelessRecipeBuilder::ingredient`].
    #[must_use]
    pub fn add_ingredient(self, ingredient: impl Into<Ingredient>) -> Self {
        self.ingredient(ingredient)
    }

    /// Adds multiple copies of an ingredient to the recipe.
    #[must_use]
    pub fn ingredient_count(mut self, ingredient: impl Into<Ingredient>, count: usize) -> Self {
        let ing = ingredient.into();
        for _ in 0..count {
            self.ingredients.push(ing.clone());
        }
        self
    }

    /// Adds multiple ingredients to the recipe.
    #[must_use]
    pub fn ingredients<I, T>(mut self, ingredients: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Ingredient>,
    {
        for ing in ingredients {
            self.ingredients.push(ing.into());
        }
        self
    }

    /// Sets the recipe group.
    #[must_use]
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Sets the recipe category in the recipe book.
    #[must_use]
    pub const fn category(mut self, category: RecipeCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Validates the recipe configuration.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if the ID is empty, ingredients list is empty, or exceeds 9 items.
    pub fn validate(&self) -> Result<(), RecipeError> {
        if self.id.is_empty() {
            return Err(RecipeError::EmptyId);
        }
        if self.ingredients.is_empty() {
            return Err(RecipeError::NoIngredients);
        }
        if self.ingredients.len() > 9 {
            return Err(RecipeError::TooManyIngredients(self.ingredients.len()));
        }
        Ok(())
    }

    /// Builds the shapeless recipe structure after validating.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn build(self) -> Result<(String, ShapelessRecipe), RecipeError> {
        self.validate()?;
        let ingredients: Vec<WitIngredient> =
            self.ingredients.into_iter().map(Into::into).collect();

        let recipe = ShapelessRecipe {
            ingredients,
            output: self.output,
            group: self.group,
            category: self.category,
        };
        Ok((self.id, recipe))
    }

    /// Registers this shapeless recipe directly with the provided [`RecipeManager`].
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register(self, manager: &RecipeManager) -> Result<(), RecipeError> {
        let (id, recipe) = self.build()?;
        manager.register_shapeless(&id, recipe);
        Ok(())
    }

    /// Registers this shapeless recipe with the server.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_to_server(self, server: &Server) -> Result<(), RecipeError> {
        let manager = server.get_recipe_manager();
        self.register(&manager)
    }

    /// Registers this shapeless recipe with the plugin context.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_to_context(self, context: &Context) -> Result<(), RecipeError> {
        let manager = context.get_recipe_manager();
        self.register(&manager)
    }
}

/// Builder for constructing and registering cooking recipes (furnace smelting, blast furnace, smoker, campfire).
pub struct CookingRecipeBuilder {
    id: String,
    cooking_type: CookingType,
    ingredient: Option<Ingredient>,
    output: ItemStack,
    cooking_time: u32,
    experience: f32,
    group: Option<String>,
    category: Option<RecipeCategory>,
}

impl CookingRecipeBuilder {
    /// Creates a new generic cooking recipe builder.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        cooking_type: CookingType,
        ingredient: impl Into<Ingredient>,
        output: ItemStack,
    ) -> Self {
        let cooking_time = match cooking_type {
            CookingType::Smelting => 200,
            CookingType::Blasting | CookingType::Smoking => 100,
            CookingType::Campfire => 600,
        };
        Self {
            id: id.into(),
            cooking_type,
            ingredient: Some(ingredient.into()),
            output,
            cooking_time,
            experience: 0.0,
            group: None,
            category: None,
        }
    }

    /// Creates a smelting (furnace) recipe builder.
    ///
    /// Default cooking time is 200 ticks (10 seconds).
    #[must_use]
    pub fn smelting(
        id: impl Into<String>,
        ingredient: impl Into<Ingredient>,
        output: ItemStack,
    ) -> Self {
        Self::new(id, CookingType::Smelting, ingredient, output)
    }

    /// Creates a blasting (blast furnace) recipe builder.
    ///
    /// Default cooking time is 100 ticks (5 seconds).
    #[must_use]
    pub fn blasting(
        id: impl Into<String>,
        ingredient: impl Into<Ingredient>,
        output: ItemStack,
    ) -> Self {
        Self::new(id, CookingType::Blasting, ingredient, output)
    }

    /// Creates a smoking (smoker) recipe builder.
    ///
    /// Default cooking time is 100 ticks (5 seconds).
    #[must_use]
    pub fn smoking(
        id: impl Into<String>,
        ingredient: impl Into<Ingredient>,
        output: ItemStack,
    ) -> Self {
        Self::new(id, CookingType::Smoking, ingredient, output)
    }

    /// Creates a campfire cooking recipe builder.
    ///
    /// Default cooking time is 600 ticks (30 seconds).
    #[must_use]
    pub fn campfire(
        id: impl Into<String>,
        ingredient: impl Into<Ingredient>,
        output: ItemStack,
    ) -> Self {
        Self::new(id, CookingType::Campfire, ingredient, output)
    }

    /// Sets the cooking station type.
    #[must_use]
    pub const fn cooking_type(mut self, cooking_type: CookingType) -> Self {
        self.cooking_type = cooking_type;
        self
    }

    /// Sets the input ingredient.
    #[must_use]
    pub fn ingredient(mut self, ingredient: impl Into<Ingredient>) -> Self {
        self.ingredient = Some(ingredient.into());
        self
    }

    /// Alias for [`CookingRecipeBuilder::ingredient`].
    #[must_use]
    pub fn input(self, ingredient: impl Into<Ingredient>) -> Self {
        self.ingredient(ingredient)
    }

    /// Sets the cooking time in ticks (20 ticks = 1 second).
    #[must_use]
    pub const fn cooking_time(mut self, ticks: u32) -> Self {
        self.cooking_time = ticks;
        self
    }

    /// Sets the experience points awarded when taking the cooked item out.
    #[must_use]
    pub const fn experience(mut self, exp: f32) -> Self {
        self.experience = exp;
        self
    }

    /// Sets the recipe group.
    #[must_use]
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Sets the recipe category in the recipe book.
    #[must_use]
    pub const fn category(mut self, category: RecipeCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Validates the recipe configuration.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if the ID is empty or no input ingredient is set.
    pub fn validate(&self) -> Result<(), RecipeError> {
        if self.id.is_empty() {
            return Err(RecipeError::EmptyId);
        }
        if self.ingredient.is_none() {
            return Err(RecipeError::MissingCookingInput);
        }
        Ok(())
    }

    /// Builds the cooking recipe structure after validating.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn build(self) -> Result<(String, CookingType, CookingRecipe), RecipeError> {
        self.validate()?;
        let ingredient = self
            .ingredient
            .ok_or(RecipeError::MissingCookingInput)?
            .into();
        let recipe = CookingRecipe {
            ingredient,
            output: self.output,
            experience: self.experience,
            cooking_time: self.cooking_time,
            group: self.group,
            category: self.category,
        };
        Ok((self.id, self.cooking_type, recipe))
    }

    /// Registers this cooking recipe directly with the provided [`RecipeManager`].
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register(self, manager: &RecipeManager) -> Result<(), RecipeError> {
        let (id, cooking_type, recipe) = self.build()?;
        manager.register_cooking(&id, cooking_type, recipe);
        Ok(())
    }

    /// Registers this cooking recipe with the server.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_to_server(self, server: &Server) -> Result<(), RecipeError> {
        let manager = server.get_recipe_manager();
        self.register(&manager)
    }

    /// Registers this cooking recipe with the plugin context.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_to_context(self, context: &Context) -> Result<(), RecipeError> {
        let manager = context.get_recipe_manager();
        self.register(&manager)
    }
}

/// Trait for recipe types that can be registered with a [`RecipeManager`].
pub trait RegistrableRecipe {
    /// Registers this recipe with the provided recipe manager.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation or registration fails.
    fn register(self, manager: &RecipeManager) -> Result<(), RecipeError>;
}

impl RegistrableRecipe for ShapedRecipeBuilder {
    fn register(self, manager: &RecipeManager) -> Result<(), RecipeError> {
        let (id, recipe) = self.build()?;
        manager.register_shaped(&id, recipe);
        Ok(())
    }
}

impl RegistrableRecipe for ShapelessRecipeBuilder {
    fn register(self, manager: &RecipeManager) -> Result<(), RecipeError> {
        let (id, recipe) = self.build()?;
        manager.register_shapeless(&id, recipe);
        Ok(())
    }
}

impl RegistrableRecipe for CookingRecipeBuilder {
    fn register(self, manager: &RecipeManager) -> Result<(), RecipeError> {
        let (id, cooking_type, recipe) = self.build()?;
        manager.register_cooking(&id, cooking_type, recipe);
        Ok(())
    }
}

impl Context {
    /// Returns the global recipe manager for registering custom recipes.
    #[must_use]
    pub fn get_recipe_manager(&self) -> RecipeManager {
        self.get_server().get_recipe_manager()
    }

    /// Registers a custom recipe with the server.
    ///
    /// Accepts any recipe builder ([`ShapedRecipeBuilder`], [`ShapelessRecipeBuilder`], [`CookingRecipeBuilder`]).
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_recipe(&self, recipe: impl RegistrableRecipe) -> Result<(), RecipeError> {
        self.get_recipe_manager().register(recipe)
    }

    /// Registers a shaped crafting recipe.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_shaped_recipe(&self, builder: ShapedRecipeBuilder) -> Result<(), RecipeError> {
        self.register_recipe(builder)
    }

    /// Registers a shapeless crafting recipe.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_shapeless_recipe(
        &self,
        builder: ShapelessRecipeBuilder,
    ) -> Result<(), RecipeError> {
        self.register_recipe(builder)
    }

    /// Registers a cooking recipe.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_cooking_recipe(
        &self,
        builder: CookingRecipeBuilder,
    ) -> Result<(), RecipeError> {
        self.register_recipe(builder)
    }
}

impl Server {
    /// Registers a custom recipe with the server.
    ///
    /// Accepts any recipe builder ([`ShapedRecipeBuilder`], [`ShapelessRecipeBuilder`], [`CookingRecipeBuilder`]).
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_recipe(&self, recipe: impl RegistrableRecipe) -> Result<(), RecipeError> {
        self.get_recipe_manager().register(recipe)
    }

    /// Registers a shaped crafting recipe.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_shaped_recipe(&self, builder: ShapedRecipeBuilder) -> Result<(), RecipeError> {
        self.register_recipe(builder)
    }

    /// Registers a shapeless crafting recipe.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_shapeless_recipe(
        &self,
        builder: ShapelessRecipeBuilder,
    ) -> Result<(), RecipeError> {
        self.register_recipe(builder)
    }

    /// Registers a cooking recipe.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_cooking_recipe(
        &self,
        builder: CookingRecipeBuilder,
    ) -> Result<(), RecipeError> {
        self.register_recipe(builder)
    }
}

impl RecipeManager {
    /// Registers a custom recipe with the server.
    ///
    /// Accepts any recipe builder ([`ShapedRecipeBuilder`], [`ShapelessRecipeBuilder`], [`CookingRecipeBuilder`]).
    ///
    /// # Examples
    /// ```rust,ignore
    /// manager.register(
    ///     ShapedRecipeBuilder::new("my:sword", output)
    ///         .pattern([" D ", " D ", " S "])
    ///         .key('D', "diamond")
    ///         .key('S', "stick")
    /// )?;
    /// ```
    ///
    /// # Errors
    /// Returns [`RecipeError`] if recipe validation fails.
    pub fn register(&self, recipe: impl RegistrableRecipe) -> Result<(), RecipeError> {
        recipe.register(self)
    }

    /// Registers a shaped crafting recipe from a builder.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_shaped_recipe(&self, builder: ShapedRecipeBuilder) -> Result<(), RecipeError> {
        self.register(builder)
    }

    /// Registers a shapeless crafting recipe from a builder.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_shapeless_recipe(
        &self,
        builder: ShapelessRecipeBuilder,
    ) -> Result<(), RecipeError> {
        self.register(builder)
    }

    /// Registers a cooking recipe from a builder.
    ///
    /// # Errors
    /// Returns [`RecipeError`] if validation fails.
    pub fn register_cooking_recipe(
        &self,
        builder: CookingRecipeBuilder,
    ) -> Result<(), RecipeError> {
        self.register(builder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ingredient_normalization() {
        assert_eq!(
            Ingredient::from("diamond"),
            Ingredient::Item("minecraft:diamond".into())
        );
        assert_eq!(
            Ingredient::from("custom:ruby"),
            Ingredient::Item("custom:ruby".into())
        );
        assert_eq!(
            Ingredient::from("#logs"),
            Ingredient::Tag("minecraft:logs".into())
        );
        assert_eq!(
            Ingredient::from("#minecraft:planks"),
            Ingredient::Tag("minecraft:planks".into())
        );
        assert_eq!(
            Ingredient::from(["coal", "charcoal"]),
            Ingredient::OneOf(vec!["minecraft:coal".into(), "minecraft:charcoal".into()])
        );
    }

    #[test]
    fn shaped_recipe_validation_errors() {
        let dummy_output = unsafe { std::mem::zeroed() };
        let builder = ShapedRecipeBuilder::new("test:recipe", dummy_output);
        assert_eq!(builder.validate(), Err(RecipeError::EmptyPattern));
        std::mem::forget(builder);

        let dummy_output = unsafe { std::mem::zeroed() };
        let builder = ShapedRecipeBuilder::new("", dummy_output).pattern(["X"]);
        assert_eq!(builder.validate(), Err(RecipeError::EmptyId));
        std::mem::forget(builder);

        let dummy_output = unsafe { std::mem::zeroed() };
        let builder = ShapedRecipeBuilder::new("test:recipe", dummy_output)
            .pattern(["X", "X", "X", "X"])
            .key('X', "diamond");
        assert_eq!(
            builder.validate(),
            Err(RecipeError::PatternTooLarge {
                width: 1,
                height: 4
            })
        );
        std::mem::forget(builder);

        let dummy_output = unsafe { std::mem::zeroed() };
        let builder = ShapedRecipeBuilder::new("test:recipe", dummy_output)
            .pattern(["XX", "X"])
            .key('X', "diamond");
        assert_eq!(
            builder.validate(),
            Err(RecipeError::InconsistentRowWidth {
                expected: 2,
                found: 1
            })
        );
        std::mem::forget(builder);

        let dummy_output = unsafe { std::mem::zeroed() };
        let builder = ShapedRecipeBuilder::new("test:recipe", dummy_output)
            .pattern(["XY"])
            .key('X', "diamond");
        assert_eq!(builder.validate(), Err(RecipeError::MissingKey('Y')));
        std::mem::forget(builder);
    }

    #[test]
    fn shapeless_recipe_validation_errors() {
        let dummy_output = unsafe { std::mem::zeroed() };
        let builder = ShapelessRecipeBuilder::new("test:recipe", dummy_output);
        assert_eq!(builder.validate(), Err(RecipeError::NoIngredients));
        std::mem::forget(builder);

        let dummy_output = unsafe { std::mem::zeroed() };
        let builder =
            ShapelessRecipeBuilder::new("test:recipe", dummy_output).ingredient_count("stick", 10);
        assert_eq!(builder.validate(), Err(RecipeError::TooManyIngredients(10)));
        std::mem::forget(builder);
    }

    #[test]
    fn cooking_recipe_presets() {
        let dummy_output = unsafe { std::mem::zeroed() };
        let smelting = CookingRecipeBuilder::smelting("test:smelt", "raw_iron", dummy_output);
        assert_eq!(smelting.cooking_time, 200);
        assert_eq!(smelting.cooking_type, CookingType::Smelting);
        std::mem::forget(smelting);

        let dummy_output = unsafe { std::mem::zeroed() };
        let blasting = CookingRecipeBuilder::blasting("test:blast", "iron_ore", dummy_output);
        assert_eq!(blasting.cooking_time, 100);
        assert_eq!(blasting.cooking_type, CookingType::Blasting);
        std::mem::forget(blasting);

        let dummy_output = unsafe { std::mem::zeroed() };
        let campfire = CookingRecipeBuilder::campfire("test:camp", "beef", dummy_output);
        assert_eq!(campfire.cooking_time, 600);
        assert_eq!(campfire.cooking_type, CookingType::Campfire);
        std::mem::forget(campfire);
    }
}
