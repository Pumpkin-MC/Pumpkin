use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;
use std::io::{Error, Write};

#[packet(76)]
pub struct CAvailableCommands {
    pub enum_values: Vec<String>,
    pub chained_subcommand_values: Vec<String>,
    pub suffixes: Vec<String>,
    pub enums: Vec<CommandEnum>,
    pub chained_subcommands: Vec<ChainedSubcommand>,
    pub commands: Vec<Command>,
    pub dynamic_enums: Vec<DynamicEnum>,
    pub constraints: Vec<CommandEnumConstraint>,
}

impl PacketWrite for CAvailableCommands {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        VarInt(self.enum_values.len() as i32).write(writer)?;
        for value in &self.enum_values {
            value.write(writer)?;
        }

        VarInt(self.chained_subcommand_values.len() as i32).write(writer)?;
        for value in &self.chained_subcommand_values {
            value.write(writer)?;
        }

        VarInt(self.suffixes.len() as i32).write(writer)?;
        for value in &self.suffixes {
            value.write(writer)?;
        }

        VarInt(self.enums.len() as i32).write(writer)?;
        for value in &self.enums {
            value.write(writer)?;
        }

        VarInt(self.chained_subcommands.len() as i32).write(writer)?;
        for value in &self.chained_subcommands {
            value.write(writer)?;
        }

        VarInt(self.commands.len() as i32).write(writer)?;
        for value in &self.commands {
            value.write(writer)?;
        }

        VarInt(self.dynamic_enums.len() as i32).write(writer)?;
        for value in &self.dynamic_enums {
            value.write(writer)?;
        }

        VarInt(self.constraints.len() as i32).write(writer)?;
        for value in &self.constraints {
            value.write(writer)?;
        }

        Ok(())
    }
}

// --- Command Enum ---
pub struct CommandEnum {
    pub name: String,
    pub value_indices: Vec<u32>, // Fixed: LE u32 instead of VarInt
}

impl PacketWrite for CommandEnum {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name.write(writer)?;
        VarInt(self.value_indices.len() as i32).write(writer)?;
        for &index in &self.value_indices {
            writer.write_all(&index.to_le_bytes())?;
        }
        Ok(())
    }
}

// --- Chained Subcommand ---
pub struct ChainedSubcommand {
    pub name: String,
    pub values: Vec<ChainedSubcommandValue>,
}

impl PacketWrite for ChainedSubcommand {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name.write(writer)?;
        VarInt(self.values.len() as i32).write(writer)?;
        for value in &self.values {
            value.write(writer)?;
        }
        Ok(())
    }
}

// --- Chained Subcommand Value ---
pub struct ChainedSubcommandValue {
    pub name_index: VarInt,
    pub type_index: VarInt,
}

impl PacketWrite for ChainedSubcommandValue {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name_index.write(writer)?;
        self.type_index.write(writer)?;
        Ok(())
    }
}

// --- Command ---
pub struct Command {
    pub name: String,
    pub description: String,
    pub flags: u16,
    pub permission: String,
    pub aliases_enum_index: i32,
    pub chained_subcommand_data_indices: Vec<u32>,
    pub overloads: Vec<CommandOverload>,
}

impl PacketWrite for Command {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name.write(writer)?;
        self.description.write(writer)?;
        writer.write_all(&self.flags.to_le_bytes())?;
        self.permission.write(writer)?;
        writer.write_all(&self.aliases_enum_index.to_le_bytes())?;

        VarInt(self.chained_subcommand_data_indices.len() as i32).write(writer)?;
        for &index in &self.chained_subcommand_data_indices {
            writer.write_all(&index.to_le_bytes())?;
        }

        VarInt(self.overloads.len() as i32).write(writer)?;
        for overload in &self.overloads {
            overload.write(writer)?;
        }
        Ok(())
    }
}

// --- Command Overload ---
pub struct CommandOverload {
    pub chaining: bool, // Fixed: Added missing bool
    pub parameters: Vec<CommandParameter>,
}

impl PacketWrite for CommandOverload {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&[if self.chaining { 1 } else { 0 }])?;
        VarInt(self.parameters.len() as i32).write(writer)?;
        for param in &self.parameters {
            param.write(writer)?;
        }
        Ok(())
    }
}

// --- Command Parameter ---
#[derive(Clone)]
pub struct CommandParameter {
    pub name: String,
    pub type_info: u32, // Fixed: LE u32 instead of VarInt
    pub optional: bool,
    pub flags: u8, // Fixed: Added flags
}

impl PacketWrite for CommandParameter {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name.write(writer)?;
        writer.write_all(&self.type_info.to_le_bytes())?;
        writer.write_all(&[if self.optional { 1 } else { 0 }])?;
        writer.write_all(&[self.flags])?;
        Ok(())
    }
}

// --- Dynamic Enum ---
#[derive(PacketWrite)]
pub struct DynamicEnum {
    pub name: String,
    pub values: Vec<String>,
}

// --- Command Enum Constraint ---
pub struct CommandEnumConstraint {
    pub affected_value_index: u32, // Fixed: LE u32 instead of VarInt
    pub enum_index: u32,           // Fixed: LE u32 instead of VarInt
    pub constraints: Vec<u8>,
}

impl PacketWrite for CommandEnumConstraint {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.affected_value_index.to_le_bytes())?;
        writer.write_all(&self.enum_index.to_le_bytes())?;
        VarInt(self.constraints.len() as i32).write(writer)?;
        writer.write_all(&self.constraints)?;
        Ok(())
    }
}
