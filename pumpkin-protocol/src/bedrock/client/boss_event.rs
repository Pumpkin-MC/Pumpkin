use std::io::{Error, Write};

use crate::{
    codec::{var_int::VarInt, var_long::VarLong},
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[packet(74)]
pub struct CBossEvent {
    pub boss_entity_id: VarLong,
    pub action: BossEventAction,
}

pub enum BossEventAction {
    Add {
        title: String,
        health_percent: f32,
        screen_darken: u16,
        color: VarInt,
        overlay: VarInt,
    },
    Remove,
    UpdateHealth(f32),
    UpdateTitle(String),
    UpdateProperties {
        screen_darken: u16,
        color: VarInt,
        overlay: VarInt,
    },
}

impl PacketWrite for CBossEvent {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.boss_entity_id.write(writer)?;
        match &self.action {
            BossEventAction::Add {
                title,
                health_percent,
                screen_darken,
                color,
                overlay,
            } => {
                VarInt(0).write(writer)?;
                title.write(writer)?;
                health_percent.write(writer)?;
                screen_darken.write(writer)?;
                color.write(writer)?;
                overlay.write(writer)?;
            }
            BossEventAction::Remove => {
                VarInt(2).write(writer)?;
            }
            BossEventAction::UpdateHealth(health) => {
                VarInt(3).write(writer)?;
                health.write(writer)?;
            }
            BossEventAction::UpdateTitle(title) => {
                VarInt(4).write(writer)?;
                title.write(writer)?;
            }
            BossEventAction::UpdateProperties {
                screen_darken,
                color,
                overlay,
            } => {
                VarInt(5).write(writer)?;
                screen_darken.write(writer)?;
                color.write(writer)?;
                overlay.write(writer)?;
            }
        }
        Ok(())
    }
}
