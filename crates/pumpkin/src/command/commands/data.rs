#![allow(
    clippy::too_many_lines,
    clippy::manual_let_else,
    clippy::map_unwrap_or,
    clippy::option_if_let_else,
    clippy::trivially_copy_pass_by_ref,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::missing_const_for_fn,
    clippy::redundant_closure_for_method_calls
)]

use std::sync::Arc;

use pumpkin_data::translation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::PermissionLvl;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::coordinates::block_pos::BlockPosArgumentType;
use crate::command::argument_types::core::double::DoubleArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::argument_types::nbt::{NbtCompoundArgumentType, NbtTagArgumentType};
use crate::command::argument_types::nbt_path::{
    ERROR_DATA_TOO_DEEP, NbtPath, NbtPathArgumentType, is_too_deep,
};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::server::Server;
use crate::world::World;

const DESCRIPTION: &str = "Queries or modifies NBT data of entities, blocks, and storages.";
const PERMISSION: &str = "minecraft:command.data";

pub const ERROR_MERGE_UNCHANGED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_MERGE_FAILED,
    translation::java::COMMANDS_DATA_MERGE_FAILED,
);

pub const ERROR_GET_NOT_NUMBER: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_GET_INVALID,
    translation::java::COMMANDS_DATA_GET_INVALID,
);

pub const ERROR_GET_NON_EXISTENT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_GET_UNKNOWN,
    translation::java::COMMANDS_DATA_GET_UNKNOWN,
);

pub const ERROR_MULTIPLE_TAGS: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_GET_MULTIPLE,
    translation::java::COMMANDS_DATA_GET_MULTIPLE,
);

pub const ERROR_EXPECTED_OBJECT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_MODIFY_EXPECTED_OBJECT,
    translation::java::COMMANDS_DATA_MODIFY_EXPECTED_OBJECT,
);

pub const ERROR_EXPECTED_VALUE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_MODIFY_EXPECTED_VALUE,
    translation::java::COMMANDS_DATA_MODIFY_EXPECTED_VALUE,
);

pub const ERROR_INVALID_SUBSTRING: CommandErrorType<2> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_MODIFY_INVALID_SUBSTRING,
    translation::java::COMMANDS_DATA_MODIFY_INVALID_SUBSTRING,
);

pub const ERROR_BLOCK_INVALID: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_BLOCK_INVALID,
    translation::java::COMMANDS_DATA_BLOCK_INVALID,
);

pub const ERROR_ENTITY_INVALID: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_DATA_ENTITY_INVALID,
    translation::java::COMMANDS_DATA_ENTITY_INVALID,
);

#[must_use]
pub fn snbt_colorful_display(tag: &NbtTag, _depth: usize) -> TextComponent {
    let folded = TextComponent::text("<...>").color_named(NamedColor::Gray);
    match tag {
        NbtTag::End => TextComponent::text(""),
        NbtTag::Byte(value) => TextComponent::text(format!("{value}"))
            .color_named(NamedColor::Gold)
            .add_child(TextComponent::text("b").color_named(NamedColor::Red)),
        NbtTag::Short(value) => TextComponent::text(format!("{value}"))
            .color_named(NamedColor::Gold)
            .add_child(TextComponent::text("s").color_named(NamedColor::Red)),
        NbtTag::Int(value) => TextComponent::text(format!("{value}")).color_named(NamedColor::Gold),
        NbtTag::Long(value) => TextComponent::text(format!("{value}"))
            .color_named(NamedColor::Gold)
            .add_child(TextComponent::text("L").color_named(NamedColor::Red)),
        NbtTag::Float(value) => TextComponent::text(format!("{value}"))
            .color_named(NamedColor::Gold)
            .add_child(TextComponent::text("f").color_named(NamedColor::Red)),
        NbtTag::Double(value) => TextComponent::text(format!("{value}"))
            .color_named(NamedColor::Gold)
            .add_child(TextComponent::text("d").color_named(NamedColor::Red)),
        NbtTag::ByteArray(value) => {
            let mut content = TextComponent::text("[")
                .add_child(TextComponent::text("B").color_named(NamedColor::Red))
                .add_child(TextComponent::text("; "));

            for (index, byte) in value.iter().take(128).enumerate() {
                content = content
                    .add_child(TextComponent::text(format!("{byte}")))
                    .add_child(TextComponent::text("b").color_named(NamedColor::Red));
                if index < value.len() - 1 {
                    content = content.add_child(TextComponent::text(", "));
                }
            }

            if value.len() > 128 {
                content = content.add_child(folded);
            }

            content.add_child(TextComponent::text("]"))
        }
        NbtTag::String(value) => {
            let escaped = value.replace('"', "\\\"");
            TextComponent::text(format!("\"{escaped}\"")).color_named(NamedColor::Green)
        }
        NbtTag::List(value) => {
            let mut content = TextComponent::text("[");
            for (index, tag) in value.iter().take(128).enumerate() {
                content = content.add_child(snbt_colorful_display(tag, 0));
                if index < value.len() - 1 {
                    content = content.add_child(TextComponent::text(", "));
                }
            }
            if value.len() > 128 {
                content = content.add_child(folded);
            }
            content.add_child(TextComponent::text("]"))
        }
        NbtTag::Compound(value) => {
            let mut content = TextComponent::text("{");
            let mut keys: Vec<&Box<str>> = value.child_tags.keys().collect();
            keys.sort();
            for (index, key) in keys.into_iter().take(128).enumerate() {
                let tag = &value.child_tags[key];
                content = content
                    .add_child(
                        TextComponent::text(format!("{key}: ")).color_named(NamedColor::Aqua),
                    )
                    .add_child(snbt_colorful_display(tag, 0));
                if index < value.child_tags.len() - 1 {
                    content = content.add_child(TextComponent::text(", "));
                }
            }
            if value.child_tags.len() > 128 {
                content = content.add_child(folded);
            }
            content.add_child(TextComponent::text("}"))
        }
        NbtTag::IntArray(value) => {
            let mut content = TextComponent::text("[")
                .add_child(TextComponent::text("I").color_named(NamedColor::Red))
                .add_child(TextComponent::text("; "));
            for (index, int) in value.iter().take(128).enumerate() {
                content = content
                    .add_child(TextComponent::text(format!("{int}")).color_named(NamedColor::Gold));
                if index < value.len() - 1 {
                    content = content.add_child(TextComponent::text(", "));
                }
            }
            if value.len() > 128 {
                content = content.add_child(folded);
            }
            content.add_child(TextComponent::text("]"))
        }
        NbtTag::LongArray(value) => {
            let mut content = TextComponent::text("[")
                .add_child(TextComponent::text("L").color_named(NamedColor::Red))
                .add_child(TextComponent::text("; "));
            for (index, long) in value.iter().take(128).enumerate() {
                content = content
                    .add_child(TextComponent::text(format!("{long}")).color_named(NamedColor::Gold))
                    .add_child(TextComponent::text("L").color_named(NamedColor::Red));
                if index < value.len() - 1 {
                    content = content.add_child(TextComponent::text(", "));
                }
            }
            if value.len() > 128 {
                content = content.add_child(folded);
            }
            content.add_child(TextComponent::text("]"))
        }
    }
}

// ---------------- DATA ACCESSOR ABSTRACTION ----------------

pub trait DataAccessor: Send + Sync {
    fn set_data(&self, data: &NbtCompound) -> Result<(), CommandSyntaxError>;
    fn get_data(&self) -> Result<NbtCompound, CommandSyntaxError>;
    fn get_modified_success(&self) -> TextComponent;
    fn get_print_success(&self, data: &NbtTag) -> TextComponent;
    fn get_print_scaled_success(&self, path: &NbtPath, scale: f64, value: i32) -> TextComponent;
}

pub struct EntityDataAccessor {
    entity: Arc<dyn EntityBase>,
}

impl EntityDataAccessor {
    #[must_use]
    pub fn new(entity: Arc<dyn EntityBase>) -> Self {
        Self { entity }
    }
}

impl DataAccessor for EntityDataAccessor {
    fn set_data(&self, tag: &NbtCompound) -> Result<(), CommandSyntaxError> {
        if self.entity.get_player().is_some() {
            return Err(ERROR_ENTITY_INVALID.create_without_context());
        }
        self.entity.read_nbt_non_mut(tag);
        Ok(())
    }

    fn get_data(&self) -> Result<NbtCompound, CommandSyntaxError> {
        let mut nbt = NbtCompound::new();
        self.entity.write_nbt(&mut nbt);
        Ok(nbt)
    }

    fn get_modified_success(&self) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::COMMANDS_DATA_ENTITY_MODIFIED,
            translation::java::COMMANDS_DATA_ENTITY_MODIFIED,
            [self.entity.get_display_name()],
        )
    }

    fn get_print_success(&self, data: &NbtTag) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::COMMANDS_DATA_ENTITY_QUERY,
            translation::java::COMMANDS_DATA_ENTITY_QUERY,
            [
                self.entity.get_display_name(),
                snbt_colorful_display(data, 0),
            ],
        )
    }

    fn get_print_scaled_success(&self, path: &NbtPath, scale: f64, value: i32) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::COMMANDS_DATA_ENTITY_GET,
            translation::java::COMMANDS_DATA_ENTITY_GET,
            [
                TextComponent::text(path.as_str().to_string()),
                self.entity.get_display_name(),
                TextComponent::text(format!("{scale:.2}")),
                TextComponent::text(value.to_string()),
            ],
        )
    }
}

pub struct BlockDataAccessor {
    pos: BlockPos,
    world: Arc<World>,
}

impl BlockDataAccessor {
    pub fn new(pos: BlockPos, world: Arc<World>) -> Result<Self, CommandSyntaxError> {
        if world.get_block_entity(&pos).is_none() {
            return Err(ERROR_BLOCK_INVALID.create_without_context());
        }
        Ok(Self { pos, world })
    }
}

impl DataAccessor for BlockDataAccessor {
    fn set_data(&self, tag: &NbtCompound) -> Result<(), CommandSyntaxError> {
        if self.world.get_block_entity(&self.pos).is_some() {
            self.world.add_block_entity_nbt(self.pos, tag);
            Ok(())
        } else {
            Err(ERROR_BLOCK_INVALID.create_without_context())
        }
    }

    fn get_data(&self) -> Result<NbtCompound, CommandSyntaxError> {
        if let Some(block_entity) = self.world.get_block_entity(&self.pos) {
            let mut nbt = NbtCompound::new();
            block_entity.write_internal(&mut nbt);
            Ok(nbt)
        } else {
            Err(ERROR_BLOCK_INVALID.create_without_context())
        }
    }

    fn get_modified_success(&self) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::COMMANDS_DATA_BLOCK_MODIFIED,
            translation::java::COMMANDS_DATA_BLOCK_MODIFIED,
            [TextComponent::text(format!(
                "{}, {}, {}",
                self.pos.0.x, self.pos.0.y, self.pos.0.z
            ))],
        )
    }

    fn get_print_success(&self, data: &NbtTag) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::COMMANDS_DATA_BLOCK_QUERY,
            translation::java::COMMANDS_DATA_BLOCK_QUERY,
            [
                TextComponent::text(format!(
                    "{}, {}, {}",
                    self.pos.0.x, self.pos.0.y, self.pos.0.z
                )),
                snbt_colorful_display(data, 0),
            ],
        )
    }

    fn get_print_scaled_success(&self, path: &NbtPath, scale: f64, value: i32) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::COMMANDS_DATA_BLOCK_GET,
            translation::java::COMMANDS_DATA_BLOCK_GET,
            [
                TextComponent::text(path.as_str().to_string()),
                TextComponent::text(format!("{}", self.pos.0.x)),
                TextComponent::text(format!("{}", self.pos.0.y)),
                TextComponent::text(format!("{}", self.pos.0.z)),
                TextComponent::text(format!("{scale:.2}")),
                TextComponent::text(value.to_string()),
            ],
        )
    }
}

pub struct StorageDataAccessor {
    id: String,
    server: Arc<Server>,
}

impl StorageDataAccessor {
    #[must_use]
    pub fn new(id: String, server: Arc<Server>) -> Self {
        Self { id, server }
    }
}

impl DataAccessor for StorageDataAccessor {
    fn set_data(&self, tag: &NbtCompound) -> Result<(), CommandSyntaxError> {
        let mut storage = self
            .server
            .command_storage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        storage.insert(self.id.clone(), tag.clone());
        Ok(())
    }

    fn get_data(&self) -> Result<NbtCompound, CommandSyntaxError> {
        let storage = self
            .server
            .command_storage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(storage.get(&self.id).cloned().unwrap_or_default())
    }

    fn get_modified_success(&self) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::COMMANDS_DATA_STORAGE_MODIFIED,
            translation::java::COMMANDS_DATA_STORAGE_MODIFIED,
            [TextComponent::text(self.id.clone())],
        )
    }

    fn get_print_success(&self, data: &NbtTag) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::COMMANDS_DATA_STORAGE_QUERY,
            translation::java::COMMANDS_DATA_STORAGE_QUERY,
            [
                TextComponent::text(self.id.clone()),
                snbt_colorful_display(data, 0),
            ],
        )
    }

    fn get_print_scaled_success(&self, path: &NbtPath, scale: f64, value: i32) -> TextComponent {
        TextComponent::translate_cross(
            translation::java::COMMANDS_DATA_STORAGE_GET,
            translation::java::COMMANDS_DATA_STORAGE_GET,
            [
                TextComponent::text(path.as_str().to_string()),
                TextComponent::text(self.id.clone()),
                TextComponent::text(format!("{scale:.2}")),
                TextComponent::text(value.to_string()),
            ],
        )
    }
}

pub fn get_single_tag(
    path: &NbtPath,
    accessor: &dyn DataAccessor,
) -> Result<NbtTag, CommandSyntaxError> {
    let data = accessor.get_data()?;
    let root = NbtTag::Compound(data);
    let tags = path.get(&root)?;
    match tags.len() {
        0 => Err(ERROR_GET_NON_EXISTENT
            .create_without_context(TextComponent::text(path.as_str().to_string()))),
        1 => Ok(tags.into_iter().next().unwrap()),
        _ => Err(ERROR_MULTIPLE_TAGS.create_without_context()),
    }
}

fn get_as_text(tag: &NbtTag) -> Result<String, CommandSyntaxError> {
    match tag {
        NbtTag::String(s) => Ok(s.to_string()),
        NbtTag::Byte(b) => Ok(b.to_string()),
        NbtTag::Short(s) => Ok(s.to_string()),
        NbtTag::Int(i) => Ok(i.to_string()),
        NbtTag::Long(l) => Ok(l.to_string()),
        NbtTag::Float(f) => Ok(f.to_string()),
        NbtTag::Double(d) => Ok(d.to_string()),
        _ => {
            Err(ERROR_EXPECTED_VALUE
                .create_without_context(TextComponent::text(format!("{tag:?}"))))
        }
    }
}

fn substring(input: &str, start: i32, end: Option<i32>) -> Result<String, CommandSyntaxError> {
    let len = input.chars().count() as i32;
    let abs_start = if start >= 0 { start } else { len + start };
    let abs_end = match end {
        Some(e) => {
            if e >= 0 {
                e
            } else {
                len + e
            }
        }
        None => len,
    };
    if abs_start >= 0 && abs_end <= len && abs_start <= abs_end {
        let chars: Vec<char> = input.chars().collect();
        Ok(chars[(abs_start as usize)..(abs_end as usize)]
            .iter()
            .collect())
    } else {
        Err(ERROR_INVALID_SUBSTRING.create_without_context(
            TextComponent::text(start.to_string()),
            TextComponent::text(end.unwrap_or(len).to_string()),
        ))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TargetKind {
    Entity,
    Block,
    Storage,
}

impl TargetKind {
    pub fn access(
        &self,
        context: &CommandContext,
        prefix: &str,
    ) -> Result<Box<dyn DataAccessor>, CommandSyntaxError> {
        match self {
            Self::Entity => {
                let entity = EntityArgumentType::get_entity(context, &format!("{prefix}_target"))?;
                Ok(Box::new(EntityDataAccessor::new(entity)))
            }
            Self::Block => {
                let pos =
                    BlockPosArgumentType::get_block_pos(context, &format!("{prefix}_target"))?;
                let world = context.world().clone();
                let accessor = BlockDataAccessor::new(pos, world)?;
                Ok(Box::new(accessor))
            }
            Self::Storage => {
                let id = context.get_argument::<pumpkin_util::identifier::Identifier>(&format!(
                    "{prefix}_target"
                ))?;
                let server = context.server().clone();
                Ok(Box::new(StorageDataAccessor::new(id.to_string(), server)))
            }
        }
    }
}

// ---------------- EXECUTION LOGIC ----------------

fn execute_get(
    context: &CommandContext,
    target_kind: TargetKind,
    has_path: bool,
    has_scale: bool,
) -> CommandExecutorResult {
    let accessor = target_kind.access(context, "target")?;
    if !has_path {
        let data = accessor.get_data()?;
        let result = data.child_tags.len() as i32;
        context
            .source
            .send_feedback(accessor.get_print_success(&NbtTag::Compound(data)), false);
        return Ok(result);
    }

    let path = context.get_argument::<NbtPath>("path")?;
    if has_scale {
        let scale = DoubleArgumentType::get(context, "scale")?;
        let tag = get_single_tag(path, accessor.as_ref())?;
        let num = match &tag {
            NbtTag::Byte(b) => *b as f64,
            NbtTag::Short(s) => *s as f64,
            NbtTag::Int(i) => *i as f64,
            NbtTag::Long(l) => *l as f64,
            NbtTag::Float(f) => *f as f64,
            NbtTag::Double(d) => *d,
            _ => {
                return Err(ERROR_GET_NOT_NUMBER
                    .create_without_context(TextComponent::text(path.as_str().to_string())));
            }
        };
        let result = (num * scale).floor() as i32;
        context.source.send_feedback(
            accessor.get_print_scaled_success(path, scale, result),
            false,
        );
        return Ok(result);
    }

    let tag = get_single_tag(path, accessor.as_ref())?;
    let result = match &tag {
        NbtTag::Byte(b) => *b as i32,
        NbtTag::Short(s) => *s as i32,
        NbtTag::Int(i) => *i,
        NbtTag::Long(l) => *l as i32,
        NbtTag::Float(f) => f.floor() as i32,
        NbtTag::Double(d) => d.floor() as i32,
        NbtTag::ByteArray(arr) => arr.len() as i32,
        NbtTag::IntArray(arr) => arr.len() as i32,
        NbtTag::LongArray(arr) => arr.len() as i32,
        NbtTag::String(s) => s.len() as i32,
        NbtTag::List(list) => list.len() as i32,
        NbtTag::Compound(compound) => compound.child_tags.len() as i32,
        NbtTag::End => {
            return Err(ERROR_GET_NON_EXISTENT
                .create_without_context(TextComponent::text(path.as_str().to_string())));
        }
    };
    context
        .source
        .send_feedback(accessor.get_print_success(&tag), false);
    Ok(result)
}

fn execute_merge(context: &CommandContext, target_kind: TargetKind) -> CommandExecutorResult {
    let accessor = target_kind.access(context, "target")?;
    let nbt = NbtCompoundArgumentType::get(context, "nbt")?.clone();
    if is_too_deep(&NbtTag::Compound(nbt.clone()), 0) {
        return Err(ERROR_DATA_TOO_DEEP.create_without_context());
    }
    let old = accessor.get_data()?;
    let mut updated = old.clone();
    for (k, v) in nbt.child_tags {
        updated.child_tags.insert(k, v);
    }
    if old == updated {
        return Err(ERROR_MERGE_UNCHANGED.create_without_context());
    }
    accessor.set_data(&updated)?;
    context
        .source
        .send_feedback(accessor.get_modified_success(), true);
    Ok(1)
}

fn execute_remove(context: &CommandContext, target_kind: TargetKind) -> CommandExecutorResult {
    let accessor = target_kind.access(context, "target")?;
    let path = context.get_argument::<NbtPath>("path")?;
    let mut data = accessor.get_data()?;
    let mut root = NbtTag::Compound(data.clone());
    let count = path.remove(&mut root);
    if count == 0 {
        return Err(ERROR_MERGE_UNCHANGED.create_without_context());
    }
    if let NbtTag::Compound(new_compound) = root {
        data = new_compound;
    }
    accessor.set_data(&data)?;
    context
        .source
        .send_feedback(accessor.get_modified_success(), true);
    Ok(count)
}

#[derive(Clone, Copy)]
pub enum ModifyMode {
    Insert(i32),
    Prepend,
    Append,
    Set,
    Merge,
}

#[derive(Clone, Copy)]
pub enum SourceMode {
    From {
        has_path: bool,
    },
    String {
        has_path: bool,
        has_start: bool,
        has_end: bool,
    },
    Value,
}

fn resolve_source_tags(
    context: &CommandContext,
    source_kind: Option<TargetKind>,
    source_mode: SourceMode,
) -> Result<Vec<NbtTag>, CommandSyntaxError> {
    match source_mode {
        SourceMode::Value => {
            let val = NbtTagArgumentType::get(context, "value")?.clone();
            Ok(vec![val])
        }
        SourceMode::From { has_path } => {
            let source_accessor = source_kind.unwrap().access(context, "source")?;
            let data = source_accessor.get_data()?;
            if has_path {
                let source_path = context.get_argument::<NbtPath>("sourcePath")?;
                source_path.get(&NbtTag::Compound(data))
            } else {
                Ok(vec![NbtTag::Compound(data)])
            }
        }
        SourceMode::String {
            has_path,
            has_start,
            has_end,
        } => {
            let source_accessor = source_kind.unwrap().access(context, "source")?;
            let data = source_accessor.get_data()?;
            let tags = if has_path {
                let source_path = context.get_argument::<NbtPath>("sourcePath")?;
                source_path.get(&NbtTag::Compound(data))?
            } else {
                vec![NbtTag::Compound(data)]
            };

            let start = if has_start {
                IntegerArgumentType::get(context, "start")?
            } else {
                0
            };
            let end = if has_end {
                Some(IntegerArgumentType::get(context, "end")?)
            } else {
                None
            };

            let mut string_tags = Vec::with_capacity(tags.len());
            for tag in tags {
                let text = get_as_text(&tag)?;
                let sub = substring(&text, start, end)?;
                string_tags.push(NbtTag::String(sub.into_boxed_str()));
            }
            Ok(string_tags)
        }
    }
}

fn execute_modify(
    context: &CommandContext,
    target_kind: TargetKind,
    modify_mode: ModifyMode,
    source_kind: Option<TargetKind>,
    source_mode: SourceMode,
) -> CommandExecutorResult {
    let target_accessor = target_kind.access(context, "target")?;
    let target_path = context.get_argument::<NbtPath>("targetPath")?;
    let mut target_data = target_accessor.get_data()?;
    let mut root = NbtTag::Compound(target_data.clone());
    let source_tags = resolve_source_tags(context, source_kind, source_mode)?;

    let result = match modify_mode {
        ModifyMode::Insert(idx) => target_path.insert(idx, &mut root, &source_tags)?,
        ModifyMode::Prepend => target_path.insert(0, &mut root, &source_tags)?,
        ModifyMode::Append => target_path.insert(-1, &mut root, &source_tags)?,
        ModifyMode::Set => {
            let last_tag = source_tags.last().cloned().unwrap_or(NbtTag::End);
            target_path.set(&mut root, last_tag)?
        }
        ModifyMode::Merge => {
            let mut combined = NbtCompound::new();
            for tag in &source_tags {
                if is_too_deep(tag, 0) {
                    return Err(ERROR_DATA_TOO_DEEP.create_without_context());
                }
                if let NbtTag::Compound(c) = tag {
                    for (k, v) in &c.child_tags {
                        combined.child_tags.insert(k.clone(), v.clone());
                    }
                } else {
                    return Err(ERROR_EXPECTED_OBJECT
                        .create_without_context(TextComponent::text(format!("{tag:?}"))));
                }
            }

            target_path.set(&mut root, NbtTag::Compound(combined))?
        }
    };

    if result == 0 {
        return Err(ERROR_MERGE_UNCHANGED.create_without_context());
    }

    if let NbtTag::Compound(new_compound) = root {
        target_data = new_compound;
    }
    target_accessor.set_data(&target_data)?;
    context
        .source
        .send_feedback(target_accessor.get_modified_success(), true);
    Ok(result)
}

// ---------------- EXECUTOR STRUCTS ----------------

struct GetExecutor {
    target_kind: TargetKind,
    has_path: bool,
    has_scale: bool,
}

impl CommandExecutor for GetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        execute_get(context, self.target_kind, self.has_path, self.has_scale)
    }
}

struct MergeExecutor {
    target_kind: TargetKind,
}

impl CommandExecutor for MergeExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        execute_merge(context, self.target_kind)
    }
}

struct RemoveExecutor {
    target_kind: TargetKind,
}

impl CommandExecutor for RemoveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        execute_remove(context, self.target_kind)
    }
}

struct ModifyExecutor {
    target_kind: TargetKind,
    modify_mode: ModifyMode,
    source_kind: Option<TargetKind>,
    source_mode: SourceMode,
}

impl CommandExecutor for ModifyExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let modify_mode = match self.modify_mode {
            ModifyMode::Insert(_) => {
                let idx = IntegerArgumentType::get(context, "index")?;
                ModifyMode::Insert(idx)
            }
            other => other,
        };
        execute_modify(
            context,
            self.target_kind,
            modify_mode,
            self.source_kind,
            self.source_mode,
        )
    }
}

// ---------------- REGISTRATION ----------------

fn make_target_arg(
    kind: TargetKind,
    name: &'static str,
) -> crate::command::argument_builder::RequiredArgumentBuilder {
    match kind {
        TargetKind::Entity => argument(name, EntityArgumentType::Entity),
        TargetKind::Block => argument(name, BlockPosArgumentType),
        TargetKind::Storage => argument(name, IdentifierArgumentType),
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let all_target_kinds = [
        (TargetKind::Entity, "entity"),
        (TargetKind::Block, "block"),
        (TargetKind::Storage, "storage"),
    ];

    let mut data_cmd = command("data", DESCRIPTION).requires(PERMISSION);

    // Merge & Get & Remove
    for &(target_kind, name) in &all_target_kinds {
        // data merge <target> <nbt>
        data_cmd = data_cmd.then(literal("merge").then(literal(name).then(
            make_target_arg(target_kind, "target_target").then(
                argument("nbt", NbtCompoundArgumentType).executes(MergeExecutor { target_kind }),
            ),
        )));

        // data get <target> [<path>] [<scale>]
        data_cmd = data_cmd.then(
            literal("get").then(
                literal(name).then(
                    make_target_arg(target_kind, "target_target")
                        .executes(GetExecutor {
                            target_kind,
                            has_path: false,
                            has_scale: false,
                        })
                        .then(
                            argument("path", NbtPathArgumentType)
                                .executes(GetExecutor {
                                    target_kind,
                                    has_path: true,
                                    has_scale: false,
                                })
                                .then(argument("scale", DoubleArgumentType::any()).executes(
                                    GetExecutor {
                                        target_kind,
                                        has_path: true,
                                        has_scale: true,
                                    },
                                )),
                        ),
                ),
            ),
        );

        // data remove <target> <path>
        data_cmd = data_cmd.then(literal("remove").then(literal(name).then(
            make_target_arg(target_kind, "target_target").then(
                argument("path", NbtPathArgumentType).executes(RemoveExecutor { target_kind }),
            ),
        )));
    }

    // data modify <target> <targetPath> (insert <index> | prepend | append | set | merge) ...
    let modify_modes = [
        ("insert", ModifyMode::Insert(0)),
        ("prepend", ModifyMode::Prepend),
        ("append", ModifyMode::Append),
        ("set", ModifyMode::Set),
        ("merge", ModifyMode::Merge),
    ];

    for &(target_kind, target_name) in &all_target_kinds {
        let mut target_path_arg = argument("targetPath", NbtPathArgumentType);

        for &(mod_name, mod_mode) in &modify_modes {
            let mut mod_node = if mod_name == "insert" {
                literal("insert").then(argument("index", IntegerArgumentType::any()))
            } else {
                literal(mod_name)
            };

            // value <value>
            mod_node = mod_node.then(literal("value").then(
                argument("value", NbtTagArgumentType).executes(ModifyExecutor {
                    target_kind,
                    modify_mode: mod_mode,
                    source_kind: None,
                    source_mode: SourceMode::Value,
                }),
            ));

            // from & string sources
            for &(source_kind, source_name) in &all_target_kinds {
                // from <source> [<sourcePath>]
                mod_node = mod_node.then(
                    literal("from").then(
                        literal(source_name).then(
                            make_target_arg(source_kind, "source_target")
                                .executes(ModifyExecutor {
                                    target_kind,
                                    modify_mode: mod_mode,
                                    source_kind: Some(source_kind),
                                    source_mode: SourceMode::From { has_path: false },
                                })
                                .then(argument("sourcePath", NbtPathArgumentType).executes(
                                    ModifyExecutor {
                                        target_kind,
                                        modify_mode: mod_mode,
                                        source_kind: Some(source_kind),
                                        source_mode: SourceMode::From { has_path: true },
                                    },
                                )),
                        ),
                    ),
                );

                // string <source> [<sourcePath>] [<start>] [<end>]
                mod_node = mod_node.then(
                    literal("string").then(
                        literal(source_name).then(
                            make_target_arg(source_kind, "source_target")
                                .executes(ModifyExecutor {
                                    target_kind,
                                    modify_mode: mod_mode,
                                    source_kind: Some(source_kind),
                                    source_mode: SourceMode::String {
                                        has_path: false,
                                        has_start: false,
                                        has_end: false,
                                    },
                                })
                                .then(
                                    argument("sourcePath", NbtPathArgumentType)
                                        .executes(ModifyExecutor {
                                            target_kind,
                                            modify_mode: mod_mode,
                                            source_kind: Some(source_kind),
                                            source_mode: SourceMode::String {
                                                has_path: true,
                                                has_start: false,
                                                has_end: false,
                                            },
                                        })
                                        .then(
                                            argument("start", IntegerArgumentType::any())
                                                .executes(ModifyExecutor {
                                                    target_kind,
                                                    modify_mode: mod_mode,
                                                    source_kind: Some(source_kind),
                                                    source_mode: SourceMode::String {
                                                        has_path: true,
                                                        has_start: true,
                                                        has_end: false,
                                                    },
                                                })
                                                .then(
                                                    argument("end", IntegerArgumentType::any())
                                                        .executes(ModifyExecutor {
                                                            target_kind,
                                                            modify_mode: mod_mode,
                                                            source_kind: Some(source_kind),
                                                            source_mode: SourceMode::String {
                                                                has_path: true,
                                                                has_start: true,
                                                                has_end: true,
                                                            },
                                                        }),
                                                ),
                                        ),
                                ),
                        ),
                    ),
                );
            }

            target_path_arg = target_path_arg.then(mod_node);
        }

        data_cmd = data_cmd.then(
            literal("modify").then(
                literal(target_name)
                    .then(make_target_arg(target_kind, "target_target").then(target_path_arg)),
            ),
        );
    }

    dispatcher.register(data_cmd);
}
