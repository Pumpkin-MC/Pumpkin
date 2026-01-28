use std::io::Write;

use bitflags::bitflags;
use pumpkin_data::packet::clientbound::PLAY_PLAYER_INFO_UPDATE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

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
        /// Sets the sorting order in the Tab list (Latest 2026 feature).
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
    pub fn new(actions: u8, players: &'a [Player<'a>]) -> Self {
        Self { actions, players }
    }
}

// TODO: Check if we need this custom impl
impl ClientPacket for CPlayerInfoUpdate<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_u8(self.actions)?;
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
                    PlayerAction::UpdateLatency(_) => todo!(),
                    PlayerAction::UpdateDisplayName(_) => todo!(),
                    PlayerAction::UpdateListOrder => todo!(),
                }
            }

            Ok(())
        })
    }
}
