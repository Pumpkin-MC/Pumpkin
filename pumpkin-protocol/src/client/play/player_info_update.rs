use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_PLAYER_INFO_UPDATE;
use pumpkin_macros::packet;

use crate::{
    ClientPacket, Property,
    ser::{NetworkWriteExt, WritingError},
};

use super::PlayerAction;

#[packet(PLAY_PLAYER_INFO_UPDATE)]
pub struct CPlayerInfoUpdate<'a> {
    pub actions: i8,
    pub players: &'a [Player<'a>],
}

pub struct Player<'a> {
    pub uuid: uuid::Uuid,
    pub actions: &'a [PlayerAction<'a>],
}

impl<'a> CPlayerInfoUpdate<'a> {
    pub fn new(actions: i8, players: &'a [Player<'a>]) -> Self {
        Self { actions, players }
    }
}

impl ClientPacket for CPlayerInfoUpdate<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        write.write_i8_be(self.actions)?;
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
                    PlayerAction::InitializeChat(_) => todo!(),
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
