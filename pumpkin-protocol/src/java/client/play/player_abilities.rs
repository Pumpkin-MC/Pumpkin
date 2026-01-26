use pumpkin_data::packet::clientbound::PLAY_PLAYER_ABILITIES;
use pumpkin_macros::java_packet;
use serde::Serialize;

/// Updates the player's movement and interaction abilities.
///
/// This packet informs the client about the player's state (flying, invulnerable)
/// and sets the movement speeds. While the client applies these visuals,
/// the server must still validate these states to prevent cheating.
#[derive(Serialize)]
#[java_packet(PLAY_PLAYER_ABILITIES)]
pub struct CPlayerAbilities {
    /// A bitmask of player states.
    /// Bit 0 (0x01): Invulnerable (Creative mode)
    /// Bit 1 (0x02): Flying
    /// Bit 2 (0x04): Allow Flying
    /// Bit 3 (0x08): Creative Mode (Instant Break)
    pub flags: i8,
    /// The multiplier for flying speed.
    /// Default is 0.05.
    pub flying_speed: f32,
    /// The field of view modifier (Walking speed multiplier).
    /// Default is 0.1.
    pub field_of_view: f32,
}

impl CPlayerAbilities {
    pub fn new(flags: i8, flying_speed: f32, field_of_view: f32) -> Self {
        Self {
            flags,
            flying_speed,
            field_of_view,
        }
    }
}
