#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    #[allow(clippy::unused_async)]
    pub async fn handle_recipe_book_seen_recipe(
        &self,
        _player: &Arc<Player>,
        _packet: SRecipeBookSeenRecipe,
    ) {
        // Client acknowledged a recipe display; no server action needed.
    }
}
