use crate::{codec::var_uint::VarUInt, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
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

// Represents a subcommand that can chain commands, e.g. /execute.
// Written as a flat list in section 3 of the packet; Commands reference
// entries by index via ChainedSubcommandOffsets.
#[derive(PacketWrite)]
pub struct ChainedSubcommand {
    pub name: String,
    pub values: Vec<ChainedSubcommandValue>,
}

#[derive(PacketWrite)]
pub struct ChainedSubcommandValue {
    /// Index into the `ChainedSubcommandValues` flat list — `VarUInt`
    pub index: VarUInt,
    /// Argument type flags (basic types only, no `ARG_FLAG`_* modifiers) — `VarUInt`
    pub value: VarUInt,
}

#[derive(PacketWrite)]
pub struct CommandEnum {
    pub name: String,
    pub value_indices: Vec<u32>,
}

#[derive(PacketWrite)]
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

#[derive(PacketWrite)]
pub struct CommandOverload {
    /// Written as a single byte before parameter count ← MISSING in original
    /// true = this overload uses chained subcommands instead of regular params
    pub chaining: bool,
    pub parameters: Vec<CommandParameter>,
}

#[derive(Clone, PacketWrite)]
pub struct CommandParameter {
    pub name: String,
    /// LE u32 — encodes type flags (`ARG_FLAG_VALID` | `ARG_FLAG_ENUM` | index, or raw type)
    pub type_info: u32,
    pub optional: bool,
    /// Options byte (`ARG_FLAG`_* options) — putByte
    pub options: u8,
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

#[derive(Clone, PacketWrite)]
pub struct SoftEnum {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Clone, PacketWrite)]
pub struct CommandEnumConstraint {
    pub affected_value_index: i32,
    pub enum_index: i32,
    pub constraints: Vec<u8>,
}
