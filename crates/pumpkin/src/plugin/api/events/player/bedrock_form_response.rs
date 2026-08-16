use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player responds to a Bedrock custom form.
#[derive(Event, Clone)]
pub struct BedrockFormResponseEvent {
    /// The player who responded to the form.
    pub player: Arc<Player>,

    /// The ID of the form.
    pub form_id: u32,

    /// The response data as a JSON string. None if the form was closed.
    pub response_data: Option<String>,
}

impl BedrockFormResponseEvent {
    /// Creates a new instance of `BedrockFormResponseEvent`.
    #[must_use]
    pub const fn new(player: Arc<Player>, form_id: u32, response_data: Option<String>) -> Self {
        Self {
            player,
            form_id,
            response_data,
        }
    }
}

impl PlayerEvent for BedrockFormResponseEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
