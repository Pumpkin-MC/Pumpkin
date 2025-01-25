use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

use super::CancellableEvent;

pub mod join;
pub mod leave;

pub trait PlayerEvent: CancellableEvent {
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
