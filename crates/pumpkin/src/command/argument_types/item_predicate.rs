use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

pub const ERROR_UNKNOWN_ITEM: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ITEM_ID_INVALID,
    translation::java::ARGUMENT_ITEM_ID_INVALID,
);

pub const ERROR_UNKNOWN_TAG: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_TAG_UNKNOWN,
    translation::java::ARGUMENTS_ITEM_TAG_UNKNOWN,
);

#[derive(Clone)]
pub enum ItemPredicate {
    Item(&'static Item),
    Tag(Vec<u16>),
    Any,
}

impl ItemPredicate {
    #[must_use]
    pub fn test(&self, stack: &ItemStack) -> bool {
        match self {
            Self::Any => true,
            Self::Item(item) => stack.get_item().id == item.id,
            Self::Tag(tag) => tag.contains(&stack.get_item().id),
        }
    }
}

pub struct ItemPredicateArgumentType;

impl ArgumentType for ItemPredicateArgumentType {
    type Item = ItemPredicate;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if reader.peek() == Some('#') {
            reader.skip();
            let start = reader.cursor();
            while let Some(c) = reader.peek() {
                if c.is_alphanumeric() || c == '_' || c == ':' || c == '/' || c == '.' || c == '-' {
                    reader.skip();
                } else {
                    break;
                }
            }
            let tag_str = &reader.string()[start..reader.cursor()];
            let normalized = if tag_str.contains(':') {
                tag_str.to_string()
            } else {
                format!("minecraft:{tag_str}")
            };
            let stripped = normalized.strip_prefix("minecraft:").unwrap_or(&normalized);

            let tag_ids = get_tag_ids(RegistryKey::Item, &normalized)
                .or_else(|| get_tag_ids(RegistryKey::Item, stripped));

            let items = match tag_ids {
                Some(ids) => ids.to_vec(),
                None => {
                    return Err(ERROR_UNKNOWN_TAG.create(reader, TextComponent::text(normalized)));
                }
            };

            skip_components_and_nbt(reader);

            Ok(ItemPredicate::Tag(items))
        } else {
            let start = reader.cursor();
            while let Some(c) = reader.peek() {
                if c.is_alphanumeric()
                    || c == '_'
                    || c == ':'
                    || c == '/'
                    || c == '.'
                    || c == '-'
                    || c == '*'
                {
                    reader.skip();
                } else {
                    break;
                }
            }
            let item_str = &reader.string()[start..reader.cursor()];
            if item_str == "*" {
                return Ok(ItemPredicate::Any);
            }

            let normalized = if item_str.contains(':') {
                item_str.to_string()
            } else {
                format!("minecraft:{item_str}")
            };
            let stripped = normalized.strip_prefix("minecraft:").unwrap_or(&normalized);

            let item = Item::from_registry_key(&normalized)
                .or_else(|| Item::from_registry_key(stripped))
                .ok_or_else(|| {
                    ERROR_UNKNOWN_ITEM.create(reader, TextComponent::text(normalized))
                })?;

            skip_components_and_nbt(reader);

            Ok(ItemPredicate::Item(item))
        }
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::ItemPredicate
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "stick".to_string(),
            "minecraft:stick".to_string(),
            "#stick".to_string(),
            "#stick{foo:'bar'}".to_string(),
        ]
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.build()
    }
}

fn skip_components_and_nbt(reader: &mut StringReader) {
    if reader.peek() == Some('[') {
        reader.skip();
        let mut depth = 1;
        while reader.can_read_char() && depth > 0 {
            match reader.read() {
                Some('[') => depth += 1,
                Some(']') => depth -= 1,
                _ => {}
            }
        }
    }
    if reader.peek() == Some('{') {
        reader.skip();
        let mut depth = 1;
        while reader.can_read_char() && depth > 0 {
            match reader.read() {
                Some('{') => depth += 1,
                Some('}') => depth -= 1,
                _ => {}
            }
        }
    }
}

impl ItemPredicateArgumentType {
    pub fn get(context: &CommandContext, name: &str) -> Result<ItemPredicate, CommandSyntaxError> {
        Ok(context.get_argument::<ItemPredicate>(name)?.clone())
    }
}
