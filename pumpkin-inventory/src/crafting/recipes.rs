use async_trait::async_trait;

use pumpkin_data::recipes::RecipeResultStruct;
use pumpkin_world::inventory::Inventory;

#[async_trait]
pub trait RecipeInputInventory: Inventory {
    fn get_width(&self) -> usize;
    fn get_height(&self) -> usize;

    async fn match_recipe(&self) -> Option<&RecipeResultStruct>;
}
