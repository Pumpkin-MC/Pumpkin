#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_inventory::stonecutter_screen_handler::StonecutterScreenHandler;
use pumpkin_protocol::bedrock::server::stonecutter_set_recipe::SStonecutterSetRecipe;

impl BedrockClient {
    pub fn handle_stonecutter_set_recipe(
        &self,
        player: &Arc<Player>,
        packet: &SStonecutterSetRecipe,
    ) {
        let current_screen_handler = player
            .current_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let mut screen_handler = current_screen_handler
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if screen_handler.window_type() != Some(pumpkin_data::screen::WindowType::Stonecutter) {
            return;
        }

        let Some(stonecutter) = screen_handler
            .as_any_mut()
            .downcast_mut::<StonecutterScreenHandler>()
        else {
            return;
        };

        if packet.recipe_index.0 >= 0 {
            stonecutter.set_selected_recipe(packet.recipe_index.0 as u8);
        } else {
            stonecutter.set_selected_recipe(u8::MAX);
        }
    }
}
