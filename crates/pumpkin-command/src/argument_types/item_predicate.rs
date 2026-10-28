use pumpkin_data::data_component_impl::CustomDataImpl;
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_data::translation;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::text::TextComponent;

use crate::argument_types::nbt_path::compare_nbt;
use crate::snbt::SnbtParser;

use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

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
    /// A base predicate narrowed by `[component=value]` tests.
    Filtered(Box<Self>, Vec<ComponentTest>),
}

/// One `[...]` entry of an item predicate. Only `custom_data` is checked; other
/// components are accepted so unsupported filters don't reject every item.
#[derive(Clone)]
pub struct ComponentTest {
    negated: bool,
    key: String,
    /// `None` checks for presence, `Some((partial, value))` compares the value.
    value: Option<(bool, NbtTag)>,
}

impl ComponentTest {
    fn test(&self, stack: &ItemStack) -> bool {
        let key = self.key.strip_prefix("minecraft:").unwrap_or(&self.key);
        let result = match key {
            "custom_data" => {
                let have = stack
                    .get_data_component::<CustomDataImpl>()
                    .map(|data| NbtTag::Compound(data.data.clone()));
                match (&self.value, have) {
                    (None, have) => have.is_some(),
                    (Some((true, want)), have) => compare_nbt(
                        want,
                        &have.unwrap_or_else(|| NbtTag::Compound(NbtCompound::new())),
                    ),
                    (Some((false, want)), Some(have)) => *want == have,
                    (Some((false, _)), None) => false,
                }
            }
            _ => return true,
        };
        result != self.negated
    }
}

impl ItemPredicate {
    #[must_use]
    pub fn test(&self, stack: &ItemStack) -> bool {
        match self {
            Self::Any => true,
            Self::Item(item) => stack.get_item().id == item.id,
            Self::Tag(tag) => tag.contains(&stack.get_item().id),
            Self::Filtered(base, tests) => {
                base.test(stack) && tests.iter().all(|test| test.test(stack))
            }
        }
    }
}

pub struct ItemPredicateArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for ItemPredicateArgumentType {
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

            Ok(with_components(ItemPredicate::Tag(items), reader))
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
                return Ok(with_components(ItemPredicate::Any, reader));
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

            Ok(with_components(ItemPredicate::Item(item), reader))
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
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.build()
    }
}

/// Parses an optional `[...]` component filter after the base predicate. Falls
/// back to ignoring the filter (the old behaviour) if it can't be understood.
fn with_components(base: ItemPredicate, reader: &mut StringReader) -> ItemPredicate {
    if reader.peek() != Some('[') {
        skip_components_and_nbt(reader);
        return base;
    }
    let start = reader.cursor();
    if let Some(tests) = parse_component_tests(reader) {
        skip_components_and_nbt(reader);
        return if tests.is_empty() {
            base
        } else {
            ItemPredicate::Filtered(Box::new(base), tests)
        };
    }
    reader.set_cursor(start);
    skip_components_and_nbt(reader);
    base
}

fn parse_component_tests(reader: &mut StringReader) -> Option<Vec<ComponentTest>> {
    reader.skip();
    let mut tests = Vec::new();
    loop {
        reader.skip_whitespace();
        if reader.peek() == Some(']') {
            reader.skip();
            return Some(tests);
        }
        let negated = reader.peek() == Some('!');
        if negated {
            reader.skip();
        }
        let key_start = reader.cursor();
        while let Some(c) = reader.peek() {
            if c.is_alphanumeric() || matches!(c, '_' | ':' | '/' | '.' | '-') {
                reader.skip();
            } else {
                break;
            }
        }
        let key = reader.string()[key_start..reader.cursor()].to_string();
        if key.is_empty() {
            return None;
        }
        reader.skip_whitespace();
        let value = match reader.peek() {
            Some(op @ ('=' | '~')) => {
                reader.skip();
                reader.skip_whitespace();
                let tag = SnbtParser::parse_for_commands(reader).ok()?;
                Some((op == '~', tag))
            }
            _ => None,
        };
        tests.push(ComponentTest {
            negated,
            key,
            value,
        });
        reader.skip_whitespace();
        match reader.read() {
            Some(',') => {}
            Some(']') => return Some(tests),
            // `|` alternatives are not supported.
            _ => return None,
        }
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
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<ItemPredicate, CommandSyntaxError> {
        Ok(context.get_argument::<ItemPredicate>(name)?.clone())
    }
}
