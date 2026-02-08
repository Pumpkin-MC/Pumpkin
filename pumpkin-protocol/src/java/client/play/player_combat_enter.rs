use pumpkin_data::packet::clientbound::PLAY_PLAYER_COMBAT_ENTER;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// Notifies the client that they have entered combat.
///
/// Used by the client for the death screen and combat logging detection.
#[derive(Serialize)]
#[java_packet(PLAY_PLAYER_COMBAT_ENTER)]
pub struct CPlayerCombatEnter;
