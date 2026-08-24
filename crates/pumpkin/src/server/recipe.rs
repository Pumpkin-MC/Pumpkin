use pumpkin_inventory::crafting::recipe_provider::RecipeProvider;
use pumpkin_inventory::slot::BoxFuture;
pub use pumpkin_protocol::codec::recipe::DynamicRecipe;
use tokio::sync::RwLock;

pub struct RecipeManager {
    dynamic_recipes: RwLock<Vec<DynamicRecipe>>,
}

impl Default for RecipeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RecipeManager {
    #[must_use]
    pub fn new() -> Self {
        Self {
            dynamic_recipes: RwLock::new(Vec::new()),
        }
    }

    pub async fn add_recipe(&self, recipe: DynamicRecipe) {
        let mut recipes = self.dynamic_recipes.write().await;
        recipes.push(recipe);
    }

    pub async fn add_recipes(&self, new_recipes: impl IntoIterator<Item = DynamicRecipe>) {
        let mut recipes = self.dynamic_recipes.write().await;
        recipes.extend(new_recipes);
    }

    pub async fn set_recipes(&self, new_recipes: Vec<DynamicRecipe>) {
        let mut recipes = self.dynamic_recipes.write().await;
        *recipes = new_recipes;
    }

    pub async fn clear(&self) {
        let mut recipes = self.dynamic_recipes.write().await;
        recipes.clear();
    }

    pub async fn get_dynamic_recipes_internal(&self) -> Vec<DynamicRecipe> {
        self.dynamic_recipes.read().await.clone()
    }
}

impl RecipeProvider for RecipeManager {
    fn get_dynamic_recipes(&self) -> BoxFuture<'_, Vec<DynamicRecipe>> {
        Box::pin(async move { self.dynamic_recipes.read().await.clone() })
    }
}
