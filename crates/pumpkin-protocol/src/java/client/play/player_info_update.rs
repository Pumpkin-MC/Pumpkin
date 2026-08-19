use std::io::Write;

use bitflags::bitflags;
use pumpkin_data::packet::clientbound::PLAY_PLAYER_INFO_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{ClientPacket, Property, WritingError, ser::NetworkWriteExt};

use super::PlayerAction;

bitflags! {
    /// Defines which fields are present in the Player Info Update packet.
    ///
    /// This bitmask allows the server to update multiple aspects of a player's
    /// presence in the Tab List (and global state) in a single packet.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PlayerInfoFlags: u8 {
        /// Adds the player to the client's internal player list (Registry).
        const ADD_PLAYER            = 0x01;
        /// Initializes the chat signature session for secure chat.
        const INITIALIZE_CHAT       = 0x02;
        /// Changes the player's displayed gamemode in the Tab list.
        const UPDATE_GAME_MODE      = 0x04;
        /// Determines if the player is visible in the Tab list.
        const UPDATE_LISTED         = 0x08;
        /// Updates the ping/latency bars.
        const UPDATE_LATENCY        = 0x10;
        /// Changes the name shown in the Tab list (supports formatting).
        const UPDATE_DISPLAY_NAME   = 0x20;
        /// Sets the sorting order in the Tab list (introduced in 1.21.2).
        const UPDATE_LIST_PRIORITY  = 0x40;
        /// Toggles the visibility of the player's hat layer.
        const UPDATE_HAT            = 0x80;
    }
}

/// Updates one or more players' information on the client.
///
/// This packet replaces the legacy "Player Info" packet with a more efficient
/// bitmask-driven approach. Instead of sending full data every time, the
/// server only sends the fields specified in the `actions` bitmask.
#[java_packet(PLAY_PLAYER_INFO_UPDATE)]
pub struct CPlayerInfoUpdate<'a> {
    /// The bitmask (`PlayerInfoFlags`) determining which data follows.
    pub actions: u8,
    /// The list of players being updated. Each player entry contains
    /// data fields in the order they appear in the bitmask.
    pub players: &'a [Player<'a>],
}

pub struct Player<'a> {
    pub uuid: uuid::Uuid,
    pub actions: &'a [PlayerAction<'a>],
}

impl<'a> CPlayerInfoUpdate<'a> {
    #[must_use]
    pub const fn new(actions: u8, players: &'a [Player<'a>]) -> Self {
        Self { actions, players }
    }
}

// TODO: Check if we need this custom impl
impl ClientPacket for CPlayerInfoUpdate<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        // UPDATE_LIST_PRIORITY was added in 1.21.2 and UPDATE_HAT in 26.1.
        // Mask unsupported bits and omit their data so older clients can parse the packet.
        let mut effective_actions = self.actions;
        if *version < JavaMinecraftVersion::V_1_21_2 {
            effective_actions &= !PlayerInfoFlags::UPDATE_LIST_PRIORITY.bits();
        }
        if *version < JavaMinecraftVersion::V_26_1 {
            effective_actions &= !PlayerInfoFlags::UPDATE_HAT.bits();
        }

        write.write_u8(effective_actions)?;
        write.write_list::<Player>(self.players, |p, v| {
            p.write_uuid(&v.uuid)?;
            for action in v.actions {
                match action {
                    PlayerAction::AddPlayer { name, properties } => {
                        p.write_string(name)?;
                        p.write_list::<Property>(properties, |p, v| {
                            p.write_string(&v.name)?;
                            p.write_string(&v.value)?;
                            p.write_option(&v.signature, |p, v| p.write_string(v))
                        })?;
                    }
                    PlayerAction::InitializeChat(init_chat) => {
                        p.write_option(init_chat, |p, v| {
                            p.write_uuid(&v.session_id)?;
                            p.write_i64_be(v.expires_at)?;
                            p.write_var_int(&v.public_key.len().try_into().map_err(|_| {
                                WritingError::Message(format!(
                                    "{} isn't representable as a VarInt",
                                    v.public_key.len()
                                ))
                            })?)?;
                            p.write_slice(&v.public_key)?;
                            p.write_var_int(&v.signature.len().try_into().map_err(|_| {
                                WritingError::Message(format!(
                                    "{} isn't representable as a VarInt",
                                    v.signature.len()
                                ))
                            })?)?;
                            p.write_slice(&v.signature)
                        })?;
                    }
                    PlayerAction::UpdateGameMode(gamemode) => p.write_var_int(gamemode)?,
                    PlayerAction::UpdateListed(listed) => p.write_bool(*listed)?,
                    PlayerAction::UpdateLatency(latency) => p.write_var_int(latency)?,
                    PlayerAction::UpdateDisplayName(display_name) => {
                        p.write_option(display_name, |w, text_component| {
                            if *version < JavaMinecraftVersion::V_1_20_5 {
                                let json =
                                    serde_json::to_string(&text_component.0).unwrap_or_default();
                                w.write_string(&json)
                            } else {
                                w.write_slice(&text_component.encode())
                            }
                        })?;
                    }
                    PlayerAction::UpdateListOrder(order) => {
                        if effective_actions & PlayerInfoFlags::UPDATE_LIST_PRIORITY.bits() != 0 {
                            p.write_var_int(order)?;
                        }
                    }
                    PlayerAction::UpdateHat(show_hat) => {
                        if effective_actions & PlayerInfoFlags::UPDATE_HAT.bits() != 0 {
                            p.write_bool(*show_hat)?;
                        }
                    }
                }
            }

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CPlayerInfoUpdate, Player, PlayerInfoFlags};
    use crate::{ClientPacket, VarInt, java::client::play::PlayerAction};
    use pumpkin_util::version::JavaMinecraftVersion;

    fn write_player_info(version: JavaMinecraftVersion) -> Vec<u8> {
        let actions = [
            PlayerAction::UpdateListOrder(VarInt(42)),
            PlayerAction::UpdateHat(true),
        ];
        let players = [Player {
            uuid: uuid::Uuid::nil(),
            actions: &actions,
        }];
        let packet = CPlayerInfoUpdate::new(
            (PlayerInfoFlags::UPDATE_LIST_PRIORITY | PlayerInfoFlags::UPDATE_HAT).bits(),
            &players,
        );
        let mut bytes = Vec::new();
        packet.write_packet_data(&mut bytes, &version).unwrap();
        bytes
    }

    #[test]
    fn player_info_omits_new_actions_before_1_21_2() {
        let mut expected = vec![0, 1];
        expected.extend_from_slice(&[0; 16]);

        assert_eq!(write_player_info(JavaMinecraftVersion::V_1_21), expected);
    }

    #[test]
    fn player_info_keeps_list_order_before_26_1() {
        let mut expected = vec![PlayerInfoFlags::UPDATE_LIST_PRIORITY.bits(), 1];
        expected.extend_from_slice(&[0; 16]);
        expected.push(42);

        assert_eq!(write_player_info(JavaMinecraftVersion::V_1_21_11), expected);
    }

    #[test]
    fn player_info_writes_hat_from_26_1() {
        let mut expected = vec![
            (PlayerInfoFlags::UPDATE_LIST_PRIORITY | PlayerInfoFlags::UPDATE_HAT).bits(),
            1,
        ];
        expected.extend_from_slice(&[0; 16]);
        expected.extend_from_slice(&[42, 1]);

        assert_eq!(write_player_info(JavaMinecraftVersion::V_26_1), expected);
    }
}
