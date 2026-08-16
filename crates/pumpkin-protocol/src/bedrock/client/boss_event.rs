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
        color: VarInt,
        overlay: VarInt,
    },
    Remove,
    UpdateHealth(f32),
    UpdateTitle(String),
    UpdateProperties {
        color: VarInt,
        overlay: VarInt,
    },
}

impl PacketWrite for CBossEvent {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.boss_entity_id.write(writer)?;
        VarLong(0).write(writer)?; // player_entity_id

        let event_type: u8;
        let mut title = String::new();
        let mut health_percent = 0.0f32;
        let mut color: u8 = 0;
        let mut overlay: u8 = 0;

        match &self.action {
            BossEventAction::Add {
                title: t,
                health_percent: hp,
                color: c,
                overlay: o,
            } => {
                event_type = 0;
                title.clone_from(t);
                health_percent = *hp;
                color = c.0 as u8;
                overlay = o.0 as u8;
            }
            BossEventAction::Remove => {
                event_type = 2;
            }
            BossEventAction::UpdateHealth(health) => {
                event_type = 3;
                health_percent = *health;
            }
            BossEventAction::UpdateTitle(t) => {
                event_type = 4;
                title.clone_from(t);
            }
            BossEventAction::UpdateProperties {
                color: c,
                overlay: o,
            } => {
                event_type = 5;
                color = c.0 as u8;
                overlay = o.0 as u8;
            }
        }

        event_type.write(writer)?;
        title.write(writer)?; // title
        title.write(writer)?; // filtered_title
        health_percent.write(writer)?;
        color.write(writer)?;
        overlay.write(writer)?;

        Ok(())
    }
}
