//! Pre-26.3 encodings from `pumpkin-protocol`'s `java/client/play/set_player_team.rs`.

use crate::java::client::play::CSetPlayerTeam;
use crate::java::client::play::TeamMethod;
use crate::java::legacy::{LegacyWrite, LegacyWriteExt};
use crate::{
    codec::var_int::VarInt,
    ser::{NetworkWriteExt, WritingError},
};
use pumpkin_util::{translation::Locale, version::JavaMinecraftVersion};
use std::io::Write;

impl LegacyWrite for CSetPlayerTeam<'_> {
    fn write_legacy(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_18 {
            write.write_string(&self.team_name)?;
        } else {
            write.write_string_bounded(&self.team_name, 16)?;
        }

        write.write_i8(self.method as i8)?;

        if self.method == TeamMethod::Create || self.method == TeamMethod::Update {
            let Some(params) = &self.parameters else {
                return Err(WritingError::Message(
                    "Parameters missing for Create/Update".into(),
                ));
            };

            if *version >= JavaMinecraftVersion::V_26_2 {
                write.write_component_legacy(params.display_name, version)?;
                write.write_component_legacy(params.player_prefix, version)?;
                write.write_component_legacy(params.player_suffix, version)?;
                write
                    .write_var_int(&VarInt(nametag_visibility_to_id(params.nametag_visibility)))?;
                write.write_var_int(&VarInt(collision_rule_to_id(params.collision_rule)))?;
                if params.color >= 0 && params.color != 21 {
                    write.write_bool(true)?;
                    write.write_var_int(&VarInt(params.color))?;
                } else {
                    write.write_bool(false)?;
                }
                write.write_i8(params.options)?;
            } else if *version >= JavaMinecraftVersion::V_1_13 {
                write.write_component_legacy(params.display_name, version)?;
                write.write_i8(params.options)?;
                if *version >= JavaMinecraftVersion::V_1_21_5 {
                    write.write_var_int(&VarInt(nametag_visibility_to_id(
                        params.nametag_visibility,
                    )))?;
                    write.write_var_int(&VarInt(collision_rule_to_id(params.collision_rule)))?;
                } else {
                    write.write_string_bounded(params.nametag_visibility, 40)?;
                    write.write_string_bounded(params.collision_rule, 40)?;
                }
                if *version >= JavaMinecraftVersion::V_1_17 {
                    let color_id = if params.color < 0 { 21 } else { params.color };
                    write.write_var_int(&VarInt(color_id))?;
                } else {
                    write.write_i8(params.color as i8)?;
                }
                write.write_component_legacy(params.player_prefix, version)?;
                write.write_component_legacy(params.player_suffix, version)?;
            } else {
                let display_name_legacy = crate::java::legacy::text::to_legacy_string(
                    params.display_name,
                    version,
                    Locale::EnUs,
                );
                let player_prefix_legacy = crate::java::legacy::text::to_legacy_string(
                    params.player_prefix,
                    version,
                    Locale::EnUs,
                );
                let player_suffix_legacy = crate::java::legacy::text::to_legacy_string(
                    params.player_suffix,
                    version,
                    Locale::EnUs,
                );

                write.write_string_bounded(&display_name_legacy, 32)?;
                write.write_string_bounded(&player_prefix_legacy, 32)?;
                write.write_string_bounded(&player_suffix_legacy, 32)?;
                write.write_i8(params.options)?;

                if *version <= JavaMinecraftVersion::V_1_7_6 {
                    // 1.7.x has no nametag visibility, collision rule, or color in team info
                } else {
                    write.write_string_bounded(params.nametag_visibility, 32)?;
                    if *version >= JavaMinecraftVersion::V_1_9 {
                        write.write_string_bounded(params.collision_rule, 32)?;
                    }
                    write.write_i8(params.color as i8)?;
                }
            }
        }

        if self.method == TeamMethod::Create
            || self.method == TeamMethod::AddPlayers
            || self.method == TeamMethod::RemovePlayers
        {
            if *version <= JavaMinecraftVersion::V_1_7_6 {
                write.write_i16_be(self.players.len() as i16)?;
            } else {
                write.write_var_int(&VarInt(self.players.len() as i32))?;
            }
            for player in &self.players {
                write.write_string_bounded(player, 40)?;
            }
        }

        Ok(())
    }
}

fn collision_rule_to_id(s: &str) -> i32 {
    match s {
        "never" => 1,
        "pushOtherTeams" => 2,
        "pushOwnTeam" => 3,
        _ => 0,
    }
}

fn nametag_visibility_to_id(s: &str) -> i32 {
    match s {
        "never" => 1,
        "hideForOtherTeams" => 2,
        "hideForOwnTeam" => 3,
        _ => 0,
    }
}
