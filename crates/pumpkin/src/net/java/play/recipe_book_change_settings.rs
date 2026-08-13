#[allow(clippy::wildcard_imports)]
use super::*;

impl JavaClient {
    #[allow(clippy::unused_async)]
    pub async fn handle_recipe_book_change_settings(
        &self,
        _player: &Arc<Player>,
        _packet: SRecipeBookChangeSettings,
    ) {
        // Client is updating its recipe book filter/open state; no server action needed.
    }
}
