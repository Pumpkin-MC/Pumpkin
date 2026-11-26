use std::io::Write;

use pumpkin_data::packet::clientbound::play::SET_PLAYER_TEAM;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, translation::Locale, version::JavaMinecraftVersion};

use crate::{
    ClientPacket,
    codec::var_int::VarInt,
    ser::{NetworkWriteExt, WritingError},
};

#[repr(i8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamMethod {
    Create = 0,
    Remove = 1,
    Update = 2,
    AddPlayers = 3,
    RemovePlayers = 4,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamParameters<'a> {
    pub display_name: &'a TextComponent,
    pub options: i8,
    pub nametag_visibility: &'a str,
    pub collision_rule: &'a str,
    pub color: i32,
    pub player_prefix: &'a TextComponent,
    pub player_suffix: &'a TextComponent,
}

#[java_packet(SET_PLAYER_TEAM)]
pub struct CSetPlayerTeam<'a> {
    pub team_name: String,
    pub method: TeamMethod,
    pub parameters: Option<TeamParameters<'a>>,
    pub players: Box<[String]>,
}

fn nametag_visibility_to_id(s: &str) -> i32 {
    match s {
        "never" => 1,
        "hideForOtherTeams" => 2,
        "hideForOwnTeam" => 3,
        _ => 0,
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

impl ClientPacket for CSetPlayerTeam<'_> {
    fn write_packet_data(
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
                write.write_component(params.display_name, version)?;
                write.write_component(params.player_prefix, version)?;
                write.write_component(params.player_suffix, version)?;
                write
                    .write_var_int(&VarInt(nametag_visibility_to_id(params.nametag_visibility)))?;
                write.write_var_int(&VarInt(collision_rule_to_id(params.collision_rule)))?;
                // Team color is a plain VarInt enum (21 = RESET, i.e. no color), not an optional.
                let color_id = if params.color < 0 { 21 } else { params.color };
                write.write_var_int(&VarInt(color_id))?;
                write.write_i8(params.options)?;
            } else if *version >= JavaMinecraftVersion::V_1_13 {
                write.write_component(params.display_name, version)?;
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
                write.write_component(params.player_prefix, version)?;
                write.write_component(params.player_suffix, version)?;
            } else {
                let display_name_legacy = params
                    .display_name
                    .to_legacy_string_for_version(version, Locale::EnUs);
                let player_prefix_legacy = params
                    .player_prefix
                    .to_legacy_string_for_version(version, Locale::EnUs);
                let player_suffix_legacy = params
                    .player_suffix
                    .to_legacy_string_for_version(version, Locale::EnUs);

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

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(
        method: TeamMethod,
        color: i32,
        options: i8,
        players: &[&str],
        version: &JavaMinecraftVersion,
    ) -> Vec<u8> {
        let component = TextComponent::text("");
        let packet = CSetPlayerTeam {
            team_name: "T".to_string(),
            method,
            parameters: Some(TeamParameters {
                display_name: &component,
                options,
                nametag_visibility: "always",
                collision_rule: "always",
                color,
                player_prefix: &component,
                player_suffix: &component,
            }),
            players: players.iter().map(|p| (*p).to_string()).collect(),
        };
        let mut out = Vec::new();
        packet.write_packet_data(&mut out, version).unwrap();
        out
    }

    /// Bytes after the (empty) prefix/suffix components, as the 26.2+ client reads them:
    /// nametag visibility, collision rule, color (VarInt), friendly flags.
    fn tail_after_rules(bytes: &[u8]) -> &[u8] {
        // name(1+1) method(1) then three components; locate by the fixed trailing layout instead.
        &bytes[bytes.len() - 4..]
    }

    #[test]
    fn create_writes_color_as_plain_varint_26_2() {
        let v = JavaMinecraftVersion::V_26_2;
        let bytes = encode(TeamMethod::Create, 10, 0x01, &[], &v);
        // ntv=0, collision=0, color=10, flags=1, then player count (VarInt 0) => 6 bytes tail
        assert_eq!(&bytes[bytes.len() - 5..], &[0x00, 0x00, 0x0a, 0x01, 0x00]);
    }

    #[test]
    fn update_ends_after_flags_without_extra_byte() {
        let v = JavaMinecraftVersion::V_26_2;
        let bytes = encode(TeamMethod::Update, 10, 0x01, &[], &v);
        assert_eq!(tail_after_rules(&bytes), &[0x00, 0x00, 0x0a, 0x01]);
    }

    #[test]
    fn negative_color_is_reset() {
        let v = JavaMinecraftVersion::V_26_2;
        let bytes = encode(TeamMethod::Update, -1, 0x00, &[], &v);
        assert_eq!(tail_after_rules(&bytes), &[0x00, 0x00, 21, 0x00]);
    }
}
