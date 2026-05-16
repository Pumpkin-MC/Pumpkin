use crate::{codec::var_uint::VarUInt, serial::PacketWrite};
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
    pub soft_enums: Vec<SoftEnum>,
    pub constraints: Vec<CommandEnumConstraint>,
}

impl PacketWrite for CAvailableCommands {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        fn write_string_slice<W: Write>(writer: &mut W, slice: &[String]) -> Result<(), Error> {
            VarUInt(slice.len() as u32).write(writer)?;
            for s in slice {
                s.write(writer)?;
            }
            Ok(())
        }

        // 1. Enum values
        write_string_slice(writer, &self.enum_values)?;

        // 2. Chained subcommand values (The flat string list)
        write_string_slice(writer, &self.chained_subcommand_values)?;

        // 3. Suffixes
        write_string_slice(writer, &self.suffixes)?;

        // 4. Enums
        VarUInt(self.enums.len() as u32).write(writer)?;
        for e in &self.enums {
            e.write(writer)?;
        }

        // 5. Chained Subcommands
        VarUInt(self.chained_subcommands.len() as u32).write(writer)?;
        for cs in &self.chained_subcommands {
            cs.write(writer)?;
        }

        // 6. Commands
        VarUInt(self.commands.len() as u32).write(writer)?;
        for cmd in &self.commands {
            cmd.write(writer)?;
        }

        // 7. Dynamic (Soft) Enums
        VarUInt(self.soft_enums.len() as u32).write(writer)?;
        for se in &self.soft_enums {
            se.write(writer)?;
        }

        // 8. Constraints
        VarUInt(self.constraints.len() as u32).write(writer)?;
        for c in &self.constraints {
            c.write(writer)?;
        }

        Ok(())
    }
}

// Represents a subcommand that can chain commands, e.g. /execute.
// Written as a flat list in section 3 of the packet; Commands reference
// entries by index via ChainedSubcommandOffsets.
pub struct ChainedSubcommand {
    pub name: String,
    pub values: Vec<ChainedSubcommandValue>,
}

pub struct ChainedSubcommandValue {
    /// Index into the `ChainedSubcommandValues` flat list — `VarUInt`
    pub index: u32,
    /// Argument type flags (basic types only, no `ARG_FLAG`_* modifiers) — `VarUInt`
    pub value: u32,
}

impl PacketWrite for ChainedSubcommand {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name.write(writer)?;
        VarUInt(self.values.len() as u32).write(writer)?;
        for v in &self.values {
            VarUInt(v.index).write(writer)?;
            VarUInt(v.value).write(writer)?;
        }
        Ok(())
    }
}

pub struct CommandEnum {
    pub name: String,
    pub value_indices: Vec<usize>,
}

impl CommandEnum {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name.write(writer)?;
        VarUInt(self.value_indices.len() as u32).write(writer)?;
        for &index in &self.value_indices {
            writer.write_all(&(index as u32).to_le_bytes())?;
        }
        Ok(())
    }
}

pub struct Command {
    pub name: String,
    pub description: String,
    /// LE u16 — putLShort
    pub flags: u16,
    /// Permission string (e.g. "any", "admin")
    pub permission: String,
    /// LE i32 — putLInt; -1 means no aliases
    pub aliases_enum_index: i32,
    /// LE u32 each — indices into the `chained_subcommands` flat list
    pub chained_subcommand_offsets: Vec<u32>,
    pub overloads: Vec<CommandOverload>,
}

impl PacketWrite for Command {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name.write(writer)?;
        self.description.write(writer)?;
        writer.write_all(&self.flags.to_le_bytes())?;
        self.permission.write(writer)?;
        writer.write_all(&self.aliases_enum_index.to_le_bytes())?;

        // Chained subcommand offsets
        VarUInt(self.chained_subcommand_offsets.len() as u32).write(writer)?;
        for &offset in &self.chained_subcommand_offsets {
            writer.write_all(&offset.to_le_bytes())?;
        }

        VarUInt(self.overloads.len() as u32).write(writer)?;
        for overload in &self.overloads {
            overload.write(writer)?;
        }
        Ok(())
    }
}

pub struct CommandOverload {
    /// Written as a single byte before parameter count ← MISSING in original
    /// true = this overload uses chained subcommands instead of regular params
    pub chaining: bool,
    pub parameters: Vec<CommandParameter>,
}

impl PacketWrite for CommandOverload {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        // Chaining bool ← MISSING in original — client reads this byte first
        writer.write_all(&[u8::from(self.chaining)])?;

        VarUInt(self.parameters.len() as u32).write(writer)?;
        for param in &self.parameters {
            param.write(writer)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct CommandParameter {
    pub name: String,
    /// LE u32 — encodes type flags (`ARG_FLAG_VALID` | `ARG_FLAG_ENUM` | index, or raw type)
    pub type_info: u32,
    pub optional: bool,
    /// Options byte (`ARG_FLAG`_* options) — putByte
    pub options: u8,
}

impl PacketWrite for CommandParameter {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name.write(writer)?;
        writer.write_all(&self.type_info.to_le_bytes())?;
        writer.write_all(&[u8::from(self.optional)])?;
        writer.write_all(&[self.options])?;
        Ok(())
    }
}

// Constants matching PocketMine's ARG_FLAG_* and ARG_TYPE_* values
pub mod arg_flags {
    pub const ARG_FLAG_VALID: u32 = 0x100000;
    pub const ARG_FLAG_ENUM: u32 = 0x200000;
    pub const ARG_FLAG_POSTFIX: u32 = 0x1000000;
    pub const ARG_FLAG_SOFT_ENUM: u32 = 0x4000000;
}

pub mod arg_types {
    pub const ARG_TYPE_INT: u32 = 0x01;
    pub const ARG_TYPE_FLOAT: u32 = 0x03;
    pub const ARG_TYPE_VALUE: u32 = 0x04;
    pub const ARG_TYPE_WILDCARD_INT: u32 = 0x05;
    pub const ARG_TYPE_OPERATOR: u32 = 0x06;
    pub const ARG_TYPE_COMPARE_OPERATOR: u32 = 0x07;
    pub const ARG_TYPE_TARGET: u32 = 0x08;
    pub const ARG_TYPE_WILDCARD_TARGET: u32 = 0x0a;
    pub const ARG_TYPE_FILE_PATH: u32 = 0x0f;
    pub const ARG_TYPE_INT_RANGE: u32 = 0x17;
    pub const ARG_TYPE_EQUIPMENT_SLOT: u32 = 0x26;
    pub const ARG_TYPE_STRING: u32 = 0x27;
    pub const ARG_TYPE_BLOCK_POS: u32 = 0x2d;
    pub const ARG_TYPE_ENTITY_POS: u32 = 0x2e;
    pub const ARG_TYPE_RAW_TEXT: u32 = 0x33;
    pub const ARG_TYPE_JSON: u32 = 0x36;
    pub const ARG_TYPE_MESSAGE: u32 = 0x3c;
    pub const ARG_TYPE_COMMAND: u32 = 0x46;
}

pub mod command_permissions {
    pub const ANY: &str = "any";
    pub const GAME_DIRECTORS: &str = "gamedirectors";
    pub const ADMIN: &str = "admin";
    pub const HOST: &str = "host";
    pub const OWNER: &str = "owner";
    pub const INTERNAL: &str = "internal";
}

pub struct SoftEnum {
    pub name: String,
    pub values: Vec<String>,
}

impl PacketWrite for SoftEnum {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        self.name.write(writer)?;
        VarUInt(self.values.len() as u32).write(writer)?;
        for value in &self.values {
            value.write(writer)?;
        }
        Ok(())
    }
}

pub struct CommandEnumConstraint {
    pub affected_value_index: i32,
    pub enum_index: i32,
    pub constraints: Vec<u8>,
}

impl PacketWrite for CommandEnumConstraint {
    fn write<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        writer.write_all(&self.affected_value_index.to_le_bytes())?;
        writer.write_all(&self.enum_index.to_le_bytes())?;
        VarUInt(self.constraints.len() as u32).write(writer)?;
        writer.write_all(&self.constraints)?;
        Ok(())
    }
}
