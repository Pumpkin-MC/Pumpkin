use pumpkin_event::Cancellable;
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

pub mod join;
pub mod leave;

pub trait PlayerEvent: Cancellable {
    fn get_player(&self) -> Arc<Player>;
}

pub trait PlayerJoinEvent: PlayerEvent {
    fn get_join_message(&self) -> &TextComponent;
    fn set_join_message(&mut self, message: TextComponent);
}

pub trait PlayerLeaveEvent: PlayerEvent {
    fn get_leave_message(&self) -> &TextComponent;
    fn set_leave_message(&mut self, message: TextComponent);
}
