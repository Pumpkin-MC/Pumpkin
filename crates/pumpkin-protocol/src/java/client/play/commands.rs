use std::io::Write;

use pumpkin_data::packet::clientbound::play::COMMANDS;
use pumpkin_macros::java_packet;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{ClientPacket, VarInt, WritingError, ser::NetworkWriteExt};

/// Sends the entire command tree to the client for client-side parsing and tab-completion.
///
/// Minecraft uses the "Brigadier" command system. This packet informs the client
/// which commands exist, their arguments, and how they branch, allowing the
/// client to highlight syntax errors in red before the command is even sent.
#[java_packet(COMMANDS)]
pub struct CCommands<'a> {
    /// A flat list of all nodes in the command graph.
    /// Nodes reference each other by their index in this array.
    pub nodes: Box<[ProtoNode<'a>]>,
    /// The index of the "root" node in the `nodes` array.
    /// This is the entry point for all commands (the '/' symbol).
    pub root_node_index: VarInt,
}

impl<'a> CCommands<'a> {
    #[must_use]
    pub const fn new(nodes: Box<[ProtoNode<'a>]>, root_node_index: VarInt) -> Self {
        Self {
            nodes,
            root_node_index,
        }
    }
}

impl ClientPacket for CCommands<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;
        write.write_list(&self.nodes, |bytebuf, node: &ProtoNode| {
            node.write_to(bytebuf, version)
        })?;
        write.write_var_int(&self.root_node_index)
    }
}

pub struct ProtoNode<'a> {
    pub children: Box<[VarInt]>,
    pub node_type: ProtoNodeType<'a>,
}

#[derive(Debug)]
pub enum ProtoNodeType<'a> {
    Root,
    Literal {
        name: &'a str,
        is_executable: bool,
        redirect_target: Option<i32>,
        /// Added in 1.21.6. Indicates that the command node is restricted.
        restricted: bool,
    },
    Argument {
        name: &'a str,
        is_executable: bool,
        redirect_target: Option<i32>,
        parser: ArgumentType,
        override_suggestion_type: Option<SuggestionProviders>,
        /// Added in 1.21.6. Indicates that the command node is restricted.
        restricted: bool,
    },
}

impl ProtoNode<'_> {
    const FLAG_IS_EXECUTABLE: u8 = 4;
    const FLAG_HAS_REDIRECT: u8 = 8;
    const FLAG_HAS_SUGGESTION_TYPE: u8 = 16;
    /// Added in 1.21.6 (bit 0x20). Indicates that the command node is restricted.
    const FLAG_IS_RESTRICTED: u8 = 32;

    pub fn write_to(
        &self,
        write: &mut impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let v1_21_6 = *version >= JavaMinecraftVersion::V_1_21_6;

        // flags
        let mut redirect_target_on_flag = 0i32;

        let flags = match self.node_type {
            ProtoNodeType::Root => 0,
            ProtoNodeType::Literal {
                name: _,
                is_executable,
                redirect_target,
                restricted,
            } => {
                let mut n = 1;
                if restricted && v1_21_6 {
                    n |= Self::FLAG_IS_RESTRICTED;
                }
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE;
                }
                if let Some(target) = redirect_target {
                    n |= Self::FLAG_HAS_REDIRECT;
                    redirect_target_on_flag = target;
                }
                n
            }
            ProtoNodeType::Argument {
                name: _,
                is_executable,
                parser: _,
                override_suggestion_type,
                redirect_target,
                restricted,
            } => {
                let mut n = 2;
                if restricted && v1_21_6 {
                    n |= Self::FLAG_IS_RESTRICTED;
                }
                if override_suggestion_type.is_some() {
                    n |= Self::FLAG_HAS_SUGGESTION_TYPE;
                }
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE;
                }
                if let Some(target) = redirect_target {
                    n |= Self::FLAG_HAS_REDIRECT;
                    redirect_target_on_flag = target;
                }
                n
            }
        };
        write.write_u8(flags)?;

        // child count + children
        write.write_list(&self.children, |bytebuf, child| {
            bytebuf.write_var_int(child)
        })?;

        // redirect node
        if flags & Self::FLAG_HAS_REDIRECT != 0 {
            write.write_var_int(&redirect_target_on_flag.into())?;
        }

        // name
        match self.node_type {
            ProtoNodeType::Argument { name, .. } | ProtoNodeType::Literal { name, .. } => {
                write.write_string(name)?;
            }
            ProtoNodeType::Root => {}
        }

        // parser id + properties
        if let ProtoNodeType::Argument { parser, .. } = &self.node_type {
            parser.write_to_buffer(write, version)?;
        }

        if flags & Self::FLAG_HAS_SUGGESTION_TYPE != 0 {
            match &self.node_type {
                ProtoNodeType::Argument {
                    override_suggestion_type,
                    ..
                } => {
                    // suggestion type
                    let suggestion_type = override_suggestion_type.as_ref().ok_or_else(|| {
                        WritingError::Message("ProtoNode::FLAG_HAS_SUGGESTION_TYPE set but override_suggestion_type is None".into())
                    })?;
                    write.write_string(suggestion_type.resource_location())?;
                }
                _ => return Err(WritingError::Message(
                    "`ProtoNode::FLAG_HAS_SUGGESTION_TYPE` is only implemented for `ProtoNodeType::Argument`".into()
                )),
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
#[repr(u32)]
pub enum ArgumentType {
    Bool,
    Float { min: Option<f32>, max: Option<f32> },
    Double { min: Option<f64>, max: Option<f64> },
    Integer { min: Option<i32>, max: Option<i32> },
    Long { min: Option<i64>, max: Option<i64> },
    String(StringProtoArgBehavior),
    Entity { flags: u8 },
    GameProfile,
    BlockPos,
    ColumnPos,
    Vec3,
    Vec2,
    BlockState,
    BlockPredicate,
    ItemStack,
    ItemPredicate,
    Color,
    HexColor,
    Component,
    Style,
    Message,
    NbtCompound,
    NbtTag,
    NbtPath,
    Objective,
    ObjectiveCriteria,
    Operation,
    Particle,
    Angle,
    Rotation,
    ScoreboardSlot,
    ScoreHolder { flags: u8 },
    Swizzle,
    Team,
    ItemSlot,
    ItemSlots,
    ResourceLocation,
    Function,
    EntityAnchor,
    IntRange,
    FloatRange,
    Dimension,
    Gamemode,
    Time { min: i32 },
    ResourceOrTag { identifier: Identifier },
    ResourceOrTagKey { identifier: Identifier },
    Resource { identifier: Identifier },
    ResourceKey { identifier: Identifier },
    ResourceSelector,
    TemplateMirror,
    TemplateRotation,
    Heightmap,
    LootTable,
    LootPredicate,
    LootModifier,
    Dialog,
    Uuid,
}

impl ArgumentType {
    pub const ENTITY_FLAG_ONLY_SINGLE: u8 = 1;
    pub const ENTITY_FLAG_PLAYERS_ONLY: u8 = 2;

    pub const SCORE_HOLDER_FLAG_ALLOW_MULTIPLE: u8 = 1;

    #[must_use]
    pub fn to_id(&self, version: &JavaMinecraftVersion) -> i32 {
        // SAFETY: Since Self is repr(u32), it is guaranteed to hold the discriminant in the first 4 bytes
        // See https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        let id = unsafe { *std::ptr::from_ref::<Self>(self).cast::<u32>() };

        pumpkin_data::argument_type_id_remap::remap_argument_type_id_for_version(id, *version)
            as i32
    }

    #[must_use]
    #[expect(clippy::match_same_arms)]
    pub fn legacy_identifier_name_for_version(
        &self,
        version: &JavaMinecraftVersion,
    ) -> (&'static str, bool) {
        match self {
            Self::Bool => ("brigadier:bool", false),
            Self::Float { .. } => ("brigadier:float", false),
            Self::Double { .. } => ("brigadier:double", false),
            Self::Integer { .. } => ("brigadier:integer", false),
            Self::Long { .. } => {
                if *version >= JavaMinecraftVersion::V_1_14 {
                    ("brigadier:long", false)
                } else {
                    ("brigadier:integer", false)
                }
            }
            Self::String(_) => ("brigadier:string", true),
            Self::Entity { .. } => ("minecraft:entity", false),
            Self::GameProfile => ("minecraft:game_profile", false),
            Self::BlockPos => ("minecraft:block_pos", false),
            Self::ColumnPos => ("minecraft:column_pos", false),
            Self::Vec3 => ("minecraft:vec3", false),
            Self::Vec2 => ("minecraft:vec2", false),
            Self::BlockState => ("minecraft:block_state", false),
            Self::BlockPredicate => ("minecraft:block_predicate", false),
            Self::ItemStack => ("minecraft:item_stack", false),
            Self::ItemPredicate => ("minecraft:item_predicate", false),
            Self::Color => ("minecraft:color", false),
            Self::Component => ("minecraft:component", false),
            Self::Message => ("minecraft:message", false),
            Self::NbtCompound => {
                if *version >= JavaMinecraftVersion::V_1_14 {
                    ("minecraft:nbt_compound_tag", false)
                } else {
                    ("minecraft:nbt", false)
                }
            }
            Self::NbtTag => {
                if *version >= JavaMinecraftVersion::V_1_14 {
                    ("minecraft:nbt_tag", false)
                } else {
                    ("minecraft:nbt", false)
                }
            }
            Self::NbtPath => ("minecraft:nbt_path", false),
            Self::Objective => ("minecraft:objective", false),
            Self::ObjectiveCriteria => ("minecraft:objective_criteria", false),
            Self::Operation => ("minecraft:operation", false),
            Self::Particle => ("minecraft:particle", false),
            Self::Angle => {
                if *version >= JavaMinecraftVersion::V_1_16 {
                    ("minecraft:angle", false)
                } else {
                    ("brigadier:string", true)
                }
            }
            Self::Rotation => ("minecraft:rotation", false),
            Self::ScoreboardSlot => ("minecraft:scoreboard_slot", false),
            Self::ScoreHolder { .. } => ("minecraft:score_holder", false),
            Self::Swizzle => ("minecraft:swizzle", false),
            Self::Team => ("minecraft:team", false),
            Self::ItemSlot | Self::ItemSlots => ("minecraft:item_slot", false),
            Self::ResourceLocation => ("minecraft:resource_location", false),
            Self::Function => ("minecraft:function", false),
            Self::EntityAnchor => ("minecraft:entity_anchor", false),
            Self::IntRange => ("minecraft:int_range", false),
            Self::FloatRange => {
                if *version >= JavaMinecraftVersion::V_1_14 {
                    ("minecraft:float_range", false)
                } else {
                    ("brigadier:string", true)
                }
            }
            Self::Dimension => ("minecraft:dimension", false),
            Self::Gamemode => ("brigadier:string", true),
            Self::Time { .. } => {
                if *version >= JavaMinecraftVersion::V_1_14 {
                    ("minecraft:time", false)
                } else {
                    ("brigadier:string", true)
                }
            }
            Self::TemplateMirror => {
                if *version >= JavaMinecraftVersion::V_1_19 {
                    ("minecraft:template_mirror", false)
                } else {
                    ("brigadier:string", true)
                }
            }
            Self::TemplateRotation => {
                if *version >= JavaMinecraftVersion::V_1_19 {
                    ("minecraft:template_rotation", false)
                } else {
                    ("brigadier:string", true)
                }
            }
            Self::Uuid if *version >= JavaMinecraftVersion::V_1_16 => ("minecraft:uuid", false),
            _ => ("brigadier:string", true),
        }
    }

    #[must_use]
    pub fn legacy_identifier_name(&self) -> (&'static str, bool) {
        self.legacy_identifier_name_for_version(&JavaMinecraftVersion::V_1_18_2)
    }

    #[expect(clippy::match_same_arms)]
    pub fn write_to_buffer(
        &self,
        write: &mut impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version >= JavaMinecraftVersion::V_1_19 {
            let id = self.to_id(version);
            write.write_var_int(&(id).into())?;
            if id == 5 {
                let behavior_val = match self {
                    Self::String(StringProtoArgBehavior::SingleWord) => 0,
                    Self::String(StringProtoArgBehavior::QuotablePhrase) => 1,
                    Self::String(StringProtoArgBehavior::GreedyPhrase) => 2,
                    _ => 0,
                };
                return write.write_var_int(&behavior_val.into());
            }

            match self {
                Self::Float { min, max } => Self::write_number_arg(*min, *max, write),
                Self::Double { min, max } => Self::write_number_arg(*min, *max, write),
                Self::Integer { min, max } => Self::write_number_arg(*min, *max, write),
                Self::Long { min, max } => Self::write_number_arg(*min, *max, write),
                Self::Entity { flags } => Self::write_with_flags(*flags, write),
                Self::ScoreHolder { flags } => Self::write_with_flags(*flags, write),
                Self::Time { min } => {
                    if *version >= JavaMinecraftVersion::V_1_19_4 {
                        write.write_i32_be(*min)
                    } else {
                        Ok(())
                    }
                }
                Self::ResourceOrTag { identifier } => {
                    Self::write_with_identifier(identifier, write)
                }
                Self::ResourceOrTagKey { identifier } => {
                    Self::write_with_identifier(identifier, write)
                }
                Self::Resource { identifier } => Self::write_with_identifier(identifier, write),
                Self::ResourceKey { identifier } => Self::write_with_identifier(identifier, write),
                _ => Ok(()),
            }
        } else {
            let (identifier, is_remapped_to_string) =
                self.legacy_identifier_name_for_version(version);
            write.write_string(identifier)?;
            if is_remapped_to_string {
                let behavior_val = match self {
                    Self::String(StringProtoArgBehavior::SingleWord) => 0,
                    Self::String(StringProtoArgBehavior::QuotablePhrase) => 1,
                    Self::String(StringProtoArgBehavior::GreedyPhrase) => 2,
                    _ => 0,
                };
                return write.write_var_int(&behavior_val.into());
            }

            match self {
                Self::Float { min, max } => Self::write_number_arg(*min, *max, write),
                Self::Double { min, max } => Self::write_number_arg(*min, *max, write),
                Self::Integer { min, max } => Self::write_number_arg(*min, *max, write),
                Self::Long { min, max } => {
                    if *version >= JavaMinecraftVersion::V_1_14 {
                        Self::write_number_arg(*min, *max, write)
                    } else {
                        let min_i32 = min.map(|v| v.clamp(i32::MIN as i64, i32::MAX as i64) as i32);
                        let max_i32 = max.map(|v| v.clamp(i32::MIN as i64, i32::MAX as i64) as i32);
                        Self::write_number_arg(min_i32, max_i32, write)
                    }
                }
                Self::Entity { flags } => Self::write_with_flags(*flags, write),
                Self::ScoreHolder { flags } => Self::write_with_flags(*flags, write),
                Self::Time { .. } => Ok(()),
                _ => Ok(()),
            }
        }
    }

    fn write_number_arg<T: NumberCmdArg>(
        min: Option<T>,
        max: Option<T>,
        write: &mut impl Write,
    ) -> Result<(), WritingError> {
        let mut flags: u8 = 0;
        if min.is_some() {
            flags |= 1;
        }
        if max.is_some() {
            flags |= 2;
        }

        write.write_u8(flags)?;
        if let Some(min) = min {
            min.write(write)?;
        }
        if let Some(max) = max {
            max.write(write)?;
        }

        Ok(())
    }

    fn write_with_flags(flags: u8, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_u8(flags)
    }

    fn write_with_identifier(
        extra_identifier: &Identifier,
        write: &mut impl Write,
    ) -> Result<(), WritingError> {
        write.write_string(extra_identifier.to_string().as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum StringProtoArgBehavior {
    SingleWord,
    QuotablePhrase,
    /// does not stop after a space
    GreedyPhrase,
}

trait NumberCmdArg {
    fn write(self, write: &mut impl Write) -> std::result::Result<(), WritingError>;
}

impl NumberCmdArg for f32 {
    fn write(self, write: &mut impl Write) -> std::result::Result<(), WritingError> {
        write.write_f32_be(self)
    }
}

impl NumberCmdArg for f64 {
    fn write(self, write: &mut impl Write) -> std::result::Result<(), WritingError> {
        write.write_f64_be(self)
    }
}

impl NumberCmdArg for i32 {
    fn write(self, write: &mut impl Write) -> std::result::Result<(), WritingError> {
        write.write_i32_be(self)
    }
}

impl NumberCmdArg for i64 {
    fn write(self, write: &mut impl Write) -> std::result::Result<(), WritingError> {
        write.write_i64_be(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SuggestionProviders {
    AskServer,
    AllRecipes,
    AvailableSounds,
    SummonableEntities,
}

impl SuggestionProviders {
    const fn resource_location(self) -> &'static str {
        match self {
            Self::AskServer => "minecraft:ask_server",
            Self::AllRecipes => "minecraft:all_recipes",
            Self::AvailableSounds => "minecraft:available_sounds",
            Self::SummonableEntities => "minecraft:summonable_entities",
        }
    }
}
