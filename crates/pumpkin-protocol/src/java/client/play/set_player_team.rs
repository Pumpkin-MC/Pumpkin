use std::io::Write;

use pumpkin_data::packet::clientbound::play::SET_PLAYER_TEAM;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, version::JavaMinecraftVersion};

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
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_string(&self.team_name)?;

        write.write_i8(self.method as i8)?;

        if self.method == TeamMethod::Create || self.method == TeamMethod::Update {
            let Some(params) = &self.parameters else {
                return Err(WritingError::Message(
                    "Parameters missing for Create/Update".into(),
                ));
            };

            write.write_component(params.display_name)?;
            write.write_component(params.player_prefix)?;
            write.write_component(params.player_suffix)?;
            write.write_var_int(&VarInt(nametag_visibility_to_id(params.nametag_visibility)))?;
            write.write_var_int(&VarInt(collision_rule_to_id(params.collision_rule)))?;
            if params.color >= 0 && params.color != 21 {
                write.write_bool(true)?;
                write.write_var_int(&VarInt(params.color))?;
            } else {
                write.write_bool(false)?;
            }
            write.write_i8(params.options)?;
        }

        if self.method == TeamMethod::Create
            || self.method == TeamMethod::AddPlayers
            || self.method == TeamMethod::RemovePlayers
        {
            write.write_var_int(&VarInt(self.players.len() as i32))?;
            for player in &self.players {
                write.write_string_bounded(player, 40)?;
            }
        }

        Ok(())
    }
}
