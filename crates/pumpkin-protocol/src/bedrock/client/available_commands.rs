// Last verified for v2169

use crate::{
    bedrock::{client::CommandPermissionLevel, enum_as_str::EnumAsStr},
    codec::var_uint::VarUInt,
    serial::PacketWrite,
};
use pumpkin_macros::packet;

/// Advertises available console and chat commands, arguments, overloads, and auto-completion structures.
#[derive(PacketWrite)]
#[packet(76)]
pub struct CAvailableCommands {
    /// Global table of all unique string arguments and options referenced by command enums.
    pub enum_values: Vec<String>,
    /// Subcommand names supporting chained execution (e.g. `/execute`).
    pub chained_subcommand_values: Vec<String>,
    /// Suffix strings appended to command parameters.
    pub post_fixes: Vec<String>,
    /// Fixed enumeration definitions mapping enum names to indices in `enum_values`.
    pub enum_data: Vec<EnumData>,
    /// Chained subcommand definitions and argument relationships.
    pub chained_subcommand_data: Vec<ChainedSubcommandData>,
    /// Command specifications and overloads registered for the player.
    pub commands: Vec<CommandData>,
    /// Dynamic enumerations that can be updated at runtime without resending all commands.
    pub soft_enums: Vec<SoftEnumData>,
    /// Value constraints and restrictions applied to specific enum options.
    pub constraints: Vec<ConstrainedValueData>,
}

/// A fixed command enum and its allowed option indices.
#[derive(PacketWrite)]
pub struct EnumData {
    /// Name of the enumeration.
    pub name: String,
    /// Indices of valid values pointing into `enum_values`.
    pub values: Vec<u32>,
}

/// Represents a subcommand that can chain commands, e.g. `/execute`.
#[derive(PacketWrite)]
pub struct ChainedSubcommandData {
    /// Name of the chained subcommand.
    pub name: String,
    /// Value and type relationships for this subcommand.
    pub subcommand_values: Vec<ChainedSubcommandRelationship>,
}

/// Relationship between a chained subcommand value and argument flags.
#[derive(PacketWrite)]
pub struct ChainedSubcommandRelationship {
    /// Index into the `ChainedSubcommandValues` flat list.
    pub index: VarUInt,
    /// Argument type flags (basic types only, no `ARG_FLAG`_* modifiers).
    pub value: VarUInt,
}

/// Metadata, permissions, aliases, and argument overloads for a single registered command.
#[derive(PacketWrite)]
pub struct CommandData {
    /// Canonical command name (without leading slash).
    pub name: String,
    /// Human-readable description displayed in command help.
    pub description: String,
    /// Command behavior flags.
    pub flags: u16,
    /// Minimum permission level required to execute the command.
    pub permission_level: EnumAsStr<CommandPermissionLevel>,
    /// Index into enum table defining aliases for this command (-1 if none).
    pub alias_enum: i32,
    /// Indices of chained subcommands referenced by this command.
    pub command_data_chained_subcommand_indexes: Vec<u32>,
    /// Overload variants specifying acceptable argument sequences.
    pub overloads: Vec<OverloadData>,
}

/// An overload variant describing a sequence of expected arguments.
#[derive(PacketWrite)]
pub struct OverloadData {
    /// Whether this overload is part of a chained command invocation.
    pub is_chaining: bool,
    /// Parameter specifications comprising this overload.
    pub parameter_data: Vec<ParamData>,
}

/// An individual parameter definition within a command overload.
#[derive(Clone, PacketWrite)]
pub struct ParamData {
    /// Parameter name shown in command hints.
    pub name: String,
    /// Encodes type flags (`ARG_FLAG_VALID` | `ARG_FLAG_ENUM` | index, or raw type).
    pub parse_symbol: u32,
    /// Whether this parameter is optional.
    pub is_optional: bool,
    /// Parameter option byte bitflags.
    pub options: u8,
}

/// Bit flags for command argument parsing and symbol types.
pub mod arg_flags {
    /// Flag indicating the argument symbol is valid.
    pub const ARG_FLAG_VALID: u32 = 0x100000;
    /// Flag indicating the argument symbol references a fixed enum index.
    pub const ARG_FLAG_ENUM: u32 = 0x200000;
    /// Flag indicating the argument symbol references a postfix index.
    pub const ARG_FLAG_POSTFIX: u32 = 0x1000000;
    /// Flag indicating the argument symbol references a soft (dynamic) enum index.
    pub const ARG_FLAG_SOFT_ENUM: u32 = 0x4000000;
}

/// Identifiers for built-in Bedrock command argument types.
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

/// A dynamic command enumeration whose choices can update without resending all commands.
#[derive(Clone, PacketWrite)]
pub struct SoftEnumData {
    /// Name of the dynamic enumeration.
    pub enum_name: String,
    /// Available options within the enumeration.
    pub enum_options: Vec<String>,
}

/// Restricts specific enum values based on constraints.
#[derive(Clone, PacketWrite)]
pub struct ConstrainedValueData {
    /// Symbol of the enum value being constrained.
    pub enum_value_symbol: u32,
    /// Symbol of the enum containing the value.
    pub enum_symbol: u32,
    /// Indices of constraints applied.
    pub constraint_indices: Vec<u8>,
}
