#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    pub async fn handle_recipe_book_seen_recipe(
        &self,
        server: &Arc<Server>,
        player: &Arc<Player>,
        packet: SRecipeBookSeenRecipe,
    ) {
        let mut event = crate::plugin::api::events::player::player_recipe_discover::PlayerRecipeDiscoverEvent::new(
            player.clone(),
            format!("display_{}", packet.recipe_display_id.0),
        );
        server.plugin_manager.fire(server, &mut event).await;
    }
}
