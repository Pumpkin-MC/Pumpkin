use pumpkin_data::packet::clientbound::PLAY_PLAYER_COMBAT_KILL;
use pumpkin_macros::packet;
use pumpkin_util::text::TextComponent;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_PLAYER_COMBAT_KILL)]
pub struct CCombatDeath<'a> {
    player_id: VarInt,
    message: &'a TextComponent,
}

impl<'a> CCombatDeath<'a> {
    pub fn new(player_id: VarInt, message: &'a TextComponent) -> Self {
        Self { player_id, message }
    }
}
