use std::io::Write;

use bitflags::bitflags;
use pumpkin_data::packet::clientbound::play::PLAYER_INFO_UPDATE;
use pumpkin_macros::java_packet;

use pumpkin_util::text::TextComponent;

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
        /// Sets the sorting order in the Tab list (Added in 1.21.2).
        const UPDATE_LIST_PRIORITY  = 0x40;
        /// Toggles the visibility of the player's hat layer (Added in 1.21.4).
        const UPDATE_HAT            = 0x80;
    }
}

/// Updates one or more players' information on the client.
///
/// This packet replaces the legacy "Player Info" packet with a more efficient
/// bitmask-driven approach. Instead of sending full data every time, the
/// server only sends the fields specified in the `actions` bitmask.
#[java_packet(PLAYER_INFO_UPDATE)]
pub struct CPlayerInfoUpdate<'a> {
    /// The bitmask (`PlayerInfoFlags`) determining which data follows.
    pub actions: u8,
    /// The list of players being updated. Their actions are written in bitmask order, whatever
    /// order they are listed in.
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

impl CPlayerInfoUpdate<'_> {
    /// Writes `flags`, then each player's matching actions in flag order, the order the client
    /// reads them. Actions without their flag are skipped; a flag without its action is an error.
    pub(crate) fn write_flagged<W: Write>(
        &self,
        mut write: W,
        flags: PlayerInfoFlags,
        display_name: impl Fn(&mut W, &TextComponent) -> Result<(), WritingError>,
    ) -> Result<(), WritingError> {
        write.write_u8(flags.bits())?;
        write.write_list::<Player>(self.players, |p, player| {
            p.write_uuid(&player.uuid)?;
            for flag in flags.iter() {
                let action = player
                    .actions
                    .iter()
                    .find(|action| action.flag() == flag)
                    .ok_or_else(|| {
                        WritingError::Message(format!("player info {flag:?} has no action"))
                    })?;
                write_action(p, action, &display_name)?;
            }
            Ok(())
        })
    }
}

fn write_action<W: Write>(
    p: &mut W,
    action: &PlayerAction<'_>,
    display_name: &impl Fn(&mut W, &TextComponent) -> Result<(), WritingError>,
) -> Result<(), WritingError> {
    match action {
        PlayerAction::AddPlayer { name, properties } => {
            p.write_string(name)?;
            p.write_list::<Property>(properties, |p, v| {
                p.write_string(&v.name)?;
                p.write_string(&v.value)?;
                p.write_option(&v.signature, |p, v| p.write_string(v))
            })
        }
        PlayerAction::InitializeChat(init_chat) => p.write_option(init_chat, |p, v| {
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
        }),
        PlayerAction::UpdateGameMode(gamemode) => p.write_var_int(gamemode),
        PlayerAction::UpdateListed(listed) => p.write_bool(*listed),
        PlayerAction::UpdateLatency(latency) => p.write_var_int(latency),
        PlayerAction::UpdateDisplayName(name) => {
            p.write_option(name, |p, name| display_name(p, name))
        }
        PlayerAction::UpdateListOrder(order) => p.write_var_int(order),
        PlayerAction::UpdateHat(show_hat) => p.write_bool(*show_hat),
    }
}

impl ClientPacket for CPlayerInfoUpdate<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        self.write_flagged(
            write,
            PlayerInfoFlags::from_bits_truncate(self.actions),
            NetworkWriteExt::write_component,
        )
    }
}
