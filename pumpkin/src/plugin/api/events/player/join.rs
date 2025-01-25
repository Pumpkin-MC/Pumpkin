use std::sync::Arc;

use pumpkin_util::text::TextComponent;

use crate::{
    entity::player::Player,
    plugin::{CancellableEvent, Event},
};

use super::{PlayerEvent, PlayerJoinEvent};

pub struct PlayerJoinEventImpl {
    player: Arc<Player>,
    join_message: TextComponent,
    is_cancelled: bool,
}

impl PlayerJoinEventImpl {
    pub fn new(player: Arc<Player>, join_message: TextComponent) -> Self {
        Self {
            player,
            join_message,
            is_cancelled: false,
        }
    }
}

impl PlayerJoinEvent for PlayerJoinEventImpl {
    fn get_join_message(&self) -> &TextComponent {
        &self.join_message
    }

    fn set_join_message(&mut self, message: TextComponent) {
        self.join_message = message;
    }
}

impl PlayerEvent for PlayerJoinEventImpl {
    fn get_player(&self) -> Arc<Player> {
        self.player.clone()
    }
}

impl CancellableEvent for PlayerJoinEventImpl {
    fn is_cancelled(&self) -> bool {
        self.is_cancelled
    }

    fn set_cancelled(&mut self, cancelled: bool) {
        self.is_cancelled = cancelled;
    }
}

impl Event for PlayerJoinEventImpl {
    fn get_name_static() -> &'static str {
        "PlayerJoinEvent"
    }

    fn get_name(&self) -> &'static str {
        "PlayerJoinEvent"
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
