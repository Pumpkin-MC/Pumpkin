use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_SET_PLAYER_TEAM;
use pumpkin_macros::java_packet;
use pumpkin_util::{text::TextComponent, version::MinecraftVersion};

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

pub struct TeamParameters<'a> {
    pub display_name: &'a TextComponent,
    pub options: i8,
    pub nametag_visibility: &'a str,
    pub collision_rule: &'a str,
    pub color: i32,
    pub player_prefix: &'a TextComponent,
    pub player_suffix: &'a TextComponent,
}

#[java_packet(PLAY_SET_PLAYER_TEAM)]
pub struct CSetPlayerTeam<'a> {
    pub team_name: String,
    pub method: TeamMethod,
    pub parameters: Option<TeamParameters<'a>>,
    pub players: Vec<String>,
}

impl ClientPacket for CSetPlayerTeam<'_> {
    fn write_packet_data(
        &self,
        mut write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_string(&self.team_name)?;
        write.write_i8(self.method as i8)?;

        match self.method {
            TeamMethod::Create | TeamMethod::Update => {
                if let Some(params) = &self.parameters {
                    write.write_slice(&params.display_name.encode())?;
                    write.write_i8(params.options)?;
                    write.write_string(params.nametag_visibility)?;
                    write.write_string(params.collision_rule)?;
                    write.write_var_int(&VarInt(params.color))?;
                    write.write_slice(&params.player_prefix.encode())?;
                    write.write_slice(&params.player_suffix.encode())?;
                } else {
                    return Err(WritingError::Message(
                        "Parameters missing for Create/Update".into(),
                    ));
                }
            }
            _ => {}
        }

        match self.method {
            TeamMethod::Create | TeamMethod::AddPlayers | TeamMethod::RemovePlayers => {
                write.write_var_int(&VarInt(self.players.len() as i32))?;
                for player in &self.players {
                    write.write_string(player)?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
