//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/player_info_update.rs`.

use crate::java::client::play::CPlayerInfoUpdate;
use crate::java::client::play::Player;
use crate::java::client::play::PlayerAction;
use crate::java::client::play::PlayerInfoFlags;
use crate::java::legacy::{LegacyWrite, LegacyWriteExt};
use crate::{Property, ser::NetworkWriteExt, ser::WritingError};
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Write;

impl LegacyWrite for CPlayerInfoUpdate<'_> {
    fn write_legacy(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        // UPDATE_LIST_PRIORITY was added in 1.21.2 and UPDATE_HAT in 1.21.4.
        // Mask unsupported bits and omit their data so older clients can parse the packet.
        let mut effective_actions = self.actions;
        if *version < JavaMinecraftVersion::V_1_21_2 {
            effective_actions &= !PlayerInfoFlags::UPDATE_LIST_PRIORITY.bits();
        }
        if *version < JavaMinecraftVersion::V_1_21_4 {
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
                            w.write_component_legacy(text_component, version)
                        })?;
                    }
                    PlayerAction::UpdateListOrder(order) => {
                        // Added in 1.21.2
                        if effective_actions & PlayerInfoFlags::UPDATE_LIST_PRIORITY.bits() != 0 {
                            p.write_var_int(order)?;
                        }
                    }
                    PlayerAction::UpdateHat(show_hat) => {
                        // Added in 1.21.4
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
