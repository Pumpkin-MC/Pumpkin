use pumpkin_config::BASIC_CONFIG;

mod config;
mod handshake;
mod login;
mod play;
mod status;

fn is_valid_player_name(name: &str) -> bool {
    if name.len() > 16 {
        return false;
    }

    let characters_valid = if BASIC_CONFIG.allow_impossible_actions {
        // Mojang's approach to verify nicknames
        |c: char| c > 32u8 as char && c < 127u8 as char
    } else {
        // What a non-modded client can actually send
        |c: char| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
    };

    name.chars().all(characters_valid)
}
