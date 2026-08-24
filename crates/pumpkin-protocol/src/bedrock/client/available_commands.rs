// Last verified for v2169

use crate::{
    bedrock::{client::CommandPermissionLevel, enum_as_str::EnumAsStr},
    codec::var_uint::VarUInt,
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(76)]
pub struct CAvailableCommands {
    pub enum_values: Vec<String>,
    pub chained_subcommand_values: Vec<String>,
    pub post_fixes: Vec<String>,
    pub enum_data: Vec<EnumData>,
    pub chained_subcommand_data: Vec<ChainedSubcommandData>,
    pub commands: Vec<CommandData>,
    pub soft_enums: Vec<SoftEnumData>,
    pub constraints: Vec<ConstrainedValueData>,
}

#[derive(PacketWrite)]
pub struct EnumData {
    pub name: String,
    pub values: Vec<u32>,
}

// Represents a subcommand that can chain commands, e.g. /execute.
// Written as a flat list in section 3 of the packet; Commands reference
// entries by index via ChainedSubcommandOffsets.
#[derive(PacketWrite)]
pub struct ChainedSubcommandData {
    pub name: String,
    pub subcommand_values: Vec<ChainedSubcommandRelationship>,
}

#[derive(PacketWrite)]
pub struct ChainedSubcommandRelationship {
    /// Index into the `ChainedSubcommandValues` flat list
    pub index: VarUInt,
    /// Argument type flags (basic types only, no `ARG_FLAG`_* modifiers)
    pub value: VarUInt,
}
#[derive(PacketWrite)]
pub struct CommandData {
    pub name: String,
    pub description: String,
    pub flags: u16,
    pub permission_level: EnumAsStr<CommandPermissionLevel>,
    /// -1 means no aliases
    pub alias_enum: i32,
    pub command_data_chained_subcommand_indexes: Vec<u32>,
    pub overloads: Vec<OverloadData>,
}

#[derive(PacketWrite)]
pub struct OverloadData {
    pub is_chaining: bool,
    pub parameter_data: Vec<ParamData>,
}

#[derive(Clone, PacketWrite)]
pub struct ParamData {
    pub name: String,
    /// encodes type flags (`ARG_FLAG_VALID` | `ARG_FLAG_ENUM` | index, or raw type)
    pub parse_symbol: u32,
    pub is_optional: bool,
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

#[derive(Clone, PacketWrite)]
pub struct SoftEnumData {
    pub enum_name: String,
    pub enum_options: Vec<String>,
}

#[derive(Clone, PacketWrite)]
pub struct ConstrainedValueData {
    pub enum_value_symbol: u32,
    pub enum_symbol: u32,
    pub constraint_indices: Vec<u8>,
}
