use std::io::{Error, Write};

use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(107)]
pub struct CSetDisplayObjective {
    pub display_slot: String,
    pub objective_name: String,
    pub display_name: String,
    pub criteria_name: String,
    pub sort_order: VarInt,
}

#[derive(PacketWrite)]
#[packet(108)]
pub struct CSetScore {
    pub action: VarInt, // 0 = change, 1 = remove
    pub entries: Vec<ScoreEntry>,
}

pub struct ScoreEntry {
    pub scoreboard_id: i64,
    pub objective_name: String,
    pub score: VarInt,
    pub entry_type: VarInt, // 1 = player, 2 = entity, 3 = fake player
    pub entity_unique_id: i64,
    pub custom_name: String,
}

impl PacketWrite for ScoreEntry {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.scoreboard_id.write(writer)?;
        self.objective_name.write(writer)?;
        self.score.write(writer)?;
        self.entry_type.write(writer)?;
        match self.entry_type.0 {
            1 | 2 => {
                self.entity_unique_id.write(writer)?;
            }
            3 => {
                self.custom_name.write(writer)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(PacketWrite)]
#[packet(106)]
pub struct CRemoveObjective {
    pub objective_name: String,
}
