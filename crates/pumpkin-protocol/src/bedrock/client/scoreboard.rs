use std::io::{Error, Write};

use crate::{
    codec::{var_int::VarInt, var_long::VarLong, var_uint::VarUInt},
    serial::PacketWrite,
};
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

#[packet(108)]
pub struct CSetScore {
    pub action: VarInt, // 0 = change, 1 = remove
    pub entries: Vec<ScoreEntry>,
}

impl PacketWrite for CSetScore {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarUInt(self.entries.len() as u32).write(writer)?;
        for entry in &self.entries {
            let entry_type = if self.action.0 == 1 {
                0
            } else {
                entry.entry_type.0
            };
            VarUInt(entry_type as u32).write(writer)?;
            match entry_type {
                0 => "remove".write(writer)?,
                1 => "changeplayer".write(writer)?,
                2 => "changeentity".write(writer)?,
                3 => "changefakeplayer".write(writer)?,
                _ => return Err(Error::other("Invalid scoreboard entry type")),
            }
            entry.write_for_type(writer, entry_type)?;
        }
        Ok(())
    }
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
        self.write_for_type(writer, self.entry_type.0)
    }
}

impl ScoreEntry {
    fn write_for_type<W: Write>(&self, writer: &mut W, entry_type: i32) -> Result<(), Error> {
        VarLong(self.scoreboard_id).write(writer)?;
        if entry_type == 0 {
            (!self.objective_name.is_empty()).write(writer)?;
            if !self.objective_name.is_empty() {
                self.objective_name.write(writer)?;
            }
            return Ok(());
        }
        if self.objective_name.is_empty() {
            " ".write(writer)?;
        } else {
            self.objective_name.write(writer)?;
        }
        self.score.0.write(writer)?;
        match entry_type {
            1 | 2 => {
                VarLong(self.entity_unique_id).write(writer)?;
            }
            3 => {
                if self.custom_name.is_empty() {
                    " ".write(writer)?;
                } else {
                    self.custom_name.write(writer)?;
                }
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

impl CRemoveObjective {
    pub fn new(objective_name: impl Into<String>) -> Self {
        Self {
            objective_name: objective_name.into(),
        }
    }
}
