use std::io::Write;

use pumpkin_data::packet::clientbound::PLAY_COMMANDS;
use pumpkin_macros::packet;

use crate::{
    ClientPacket, VarInt,
    ser::{NetworkWriteExt, WritingError},
};

#[packet(PLAY_COMMANDS)]
pub struct CCommands<'a> {
    pub nodes: Box<[ProtoNode<'a>]>,
    pub root_node_index: VarInt,
}

impl<'a> CCommands<'a> {
    pub fn new(nodes: Box<[ProtoNode<'a>]>, root_node_index: VarInt) -> Self {
        Self {
            nodes,
            root_node_index,
        }
    }
}

impl ClientPacket for CCommands<'_> {
    fn write_packet_data(&self, write: impl Write) -> Result<(), WritingError> {
        let mut write = write;
        write.write_list(&self.nodes, |bytebuf, node: &ProtoNode| {
            node.write_to(bytebuf)
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
    },
    Argument {
        name: &'a str,
        is_executable: bool,
        parser: ArgumentType<'a>,
        override_suggestion_type: Option<SuggestionProviders>,
    },
}

impl ProtoNode<'_> {
    const FLAG_IS_EXECUTABLE: u8 = 4;
    const FLAG_HAS_REDIRECT: u8 = 8;
    const FLAG_HAS_SUGGESTION_TYPE: u8 = 16;

    pub fn write_to(&self, write: &mut impl Write) -> Result<(), WritingError> {
        // flags
        let flags = match self.node_type {
            ProtoNodeType::Root => 0,
            ProtoNodeType::Literal {
                name: _,
                is_executable,
            } => {
                let mut n = 1;
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE
                }
                n
            }
            ProtoNodeType::Argument {
                name: _,
                is_executable,
                parser: _,
                override_suggestion_type,
            } => {
                let mut n = 2;
                if override_suggestion_type.is_some() {
                    n |= Self::FLAG_HAS_SUGGESTION_TYPE
                }
                if is_executable {
                    n |= Self::FLAG_IS_EXECUTABLE
                }
                n
            }
        };
        write.write_u8_be(flags)?;

        // child count + children
        write.write_list(&self.children, |bytebuf, child| {
            bytebuf.write_var_int(child)
        })?;

        // redirect node
        if flags & Self::FLAG_HAS_REDIRECT != 0 {
            write.write_var_int(&1.into())?;
        }

        // name
        match self.node_type {
            ProtoNodeType::Argument { name, .. } | ProtoNodeType::Literal { name, .. } => {
                write.write_string(name)?;
            }
            ProtoNodeType::Root => {}
        }

        // parser id + properties
        if let ProtoNodeType::Argument {
            name: _,
            is_executable: _,
            parser,
            override_suggestion_type: _,
        } = &self.node_type
        {
            parser.write_to_buffer(write)?;
        }

        if flags & Self::FLAG_HAS_SUGGESTION_TYPE != 0 {
            match &self.node_type {
                ProtoNodeType::Argument {
                    name: _,
                    is_executable: _,
                    parser: _,
                    override_suggestion_type,
                } => {
                    // suggestion type
                    let suggestion_type = &override_suggestion_type.expect("ProtoNode::FLAG_HAS_SUGGESTION_TYPE should only be set if override_suggestion_type is not `None`.");
                    write.write_string(suggestion_type.identifier())?;
                }
                _ => unimplemented!(
                    "`ProtoNode::FLAG_HAS_SUGGESTION_TYPE` is only implemented for `ProtoNodeType::Argument`"
                ),
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
#[repr(u32)]
pub enum ArgumentType<'a> {
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
    Component,
    Style,
    Message,
    Nbt,
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
    ResourceOrTag { identifier: &'a str },
    ResourceOrTagKey { identifier: &'a str },
    Resource { identifier: &'a str },
    ResourceKey { identifier: &'a str },
    TemplateMirror,
    TemplateRotation,
    Heightmap,
    LootTable,
    LootPredicate,
    LootModifier,
    Uuid,
}

impl ArgumentType<'_> {
    pub const ENTITY_FLAG_ONLY_SINGLE: u8 = 1;
    pub const ENTITY_FLAG_PLAYERS_ONLY: u8 = 2;

    pub const SCORE_HOLDER_FLAG_ALLOW_MULTIPLE: u8 = 1;

    pub fn write_to_buffer(&self, write: &mut impl Write) -> Result<(), WritingError> {
        // Safety: Since Self is repr(u32), it is guaranteed to hold the discriminant in the first 4 bytes
        // See https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        let id = unsafe { *(self as *const Self as *const i32) };
        write.write_var_int(&(id).into())?;
        match self {
            Self::Float { min, max } => Self::write_number_arg(*min, *max, write),
            Self::Double { min, max } => Self::write_number_arg(*min, *max, write),
            Self::Integer { min, max } => Self::write_number_arg(*min, *max, write),
            Self::Long { min, max } => Self::write_number_arg(*min, *max, write),
            Self::String(behavior) => {
                let i = match behavior {
                    StringProtoArgBehavior::SingleWord => 0,
                    StringProtoArgBehavior::QuotablePhrase => 1,
                    StringProtoArgBehavior::GreedyPhrase => 2,
                };
                write.write_var_int(&i.into())
            }
            Self::Entity { flags } => Self::write_with_flags(*flags, write),
            Self::ScoreHolder { flags } => Self::write_with_flags(*flags, write),
            Self::Time { min } => write.write_i32_be(*min),
            Self::ResourceOrTag { identifier } => Self::write_with_identifier(identifier, write),
            Self::ResourceOrTagKey { identifier } => Self::write_with_identifier(identifier, write),
            Self::Resource { identifier } => Self::write_with_identifier(identifier, write),
            Self::ResourceKey { identifier } => Self::write_with_identifier(identifier, write),
            _ => Ok(()),
        }
    }

    fn write_number_arg<T: NumberCmdArg>(
        min: Option<T>,
        max: Option<T>,
        write: &mut impl Write,
    ) -> Result<(), WritingError> {
        let mut flags: u8 = 0;
        if min.is_some() {
            flags |= 1
        }
        if max.is_some() {
            flags |= 2
        }

        write.write_u8_be(flags)?;
        if let Some(min) = min {
            min.write(write)?;
        }
        if let Some(max) = max {
            max.write(write)?;
        }

        Ok(())
    }

    fn write_with_flags(flags: u8, write: &mut impl Write) -> Result<(), WritingError> {
        write.write_u8_be(flags)
    }

    fn write_with_identifier(
        extra_identifier: &str,
        write: &mut impl Write,
    ) -> Result<(), WritingError> {
        write.write_string(extra_identifier)
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
    fn identifier(&self) -> &'static str {
        match self {
            Self::AskServer => "minecraft:ask_server",
            Self::AllRecipes => "minecraft:all_recipes",
            Self::AvailableSounds => "minecraft:available_sounds",
            Self::SummonableEntities => "minecraft:summonable_entities",
        }
    }
}
