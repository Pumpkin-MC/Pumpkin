use std::io::Write;

use crate::java::client::play::BosseventAction;
use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, WritingError};
use pumpkin_data::packet::clientbound::PLAY_BOSS_EVENT;
use pumpkin_macros::packet;

#[packet(PLAY_BOSS_EVENT)]
pub struct CBossEvent<'a> {
    pub uuid: &'a uuid::Uuid,
    pub action: BosseventAction,
}

impl<'a> CBossEvent<'a> {
    pub fn new(uuid: &'a uuid::Uuid, action: BosseventAction) -> Self {
        Self { uuid, action }
    }
}

impl ClientPacket for CBossEvent<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;

        write.write_uuid(self.uuid)?;
        let action = &self.action;
        match action {
            BosseventAction::Add {
                title,
                health,
                color,
                division,
                flags,
            } => {
                write.write_var_int(&0.into())?;
                write.write_slice(&title.encode())?;
                write.write_f32_be(*health)?;
                write.write_var_int(color)?;
                write.write_var_int(division)?;
                write.write_u8_be(*flags)
            }
            BosseventAction::Remove => write.write_var_int(&1.into()),
            BosseventAction::UpdateHealth(health) => {
                write.write_var_int(&2.into())?;
                write.write_f32_be(*health)
            }
            BosseventAction::UpdateTile(title) => {
                write.write_var_int(&3.into())?;
                write.write_slice(&title.encode())
            }
            BosseventAction::UpdateStyle { color, dividers } => {
                write.write_var_int(&4.into())?;
                write.write_var_int(color)?;
                write.write_var_int(dividers)
            }
            BosseventAction::UpdateFlags(flags) => {
                write.write_var_int(&5.into())?;
                write.write_u8_be(*flags)
            }
        }
    }
}
