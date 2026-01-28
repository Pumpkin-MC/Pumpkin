use std::io::Write;

use crate::java::client::play::BosseventAction;
use crate::ser::NetworkWriteExt;
use crate::{ClientPacket, WritingError};
use pumpkin_data::packet::clientbound::PLAY_BOSS_EVENT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::MinecraftVersion;

/// Updates the "Boss Bar" displayed at the top of the player's screen.
///
/// This packet is used to manage health bars for entities like the Ender Dragon
/// or Wither, as well as custom progress bars for server events or raids.
#[java_packet(PLAY_BOSS_EVENT)]
pub struct CBossEvent<'a> {
    /// A unique identifier for this specific boss bar instance.
    pub uuid: &'a uuid::Uuid,
    /// The action to perform (Add, Remove, Update Health, etc.).
    pub action: BosseventAction,
}

impl<'a> CBossEvent<'a> {
    #[must_use]
    pub fn new(uuid: &'a uuid::Uuid, action: BosseventAction) -> Self {
        Self { uuid, action }
    }
}

impl ClientPacket for CBossEvent<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        _version: &MinecraftVersion,
    ) -> Result<(), WritingError> {
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
                write.write_u8(*flags)
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
                write.write_u8(*flags)
            }
        }
    }
}
