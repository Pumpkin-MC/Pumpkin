use pumpkin_data::data_component::DataComponent;
use pumpkin_data::data_component_impl::{DataComponentImpl, read_data};
use pumpkin_data::item::Item;
use pumpkin_data::item_stack::ItemStack;
use pumpkin_data::translation;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

use crate::argument_types::FromStringReader;
use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::snbt::SnbtParser;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

pub const ERROR_UNKNOWN_ITEM: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ITEM_ID_INVALID,
    translation::java::ARGUMENT_ITEM_ID_INVALID,
);
pub const ERROR_UNKNOWN_COMPONENT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_COMPONENT_UNKNOWN,
    translation::java::ARGUMENTS_ITEM_COMPONENT_UNKNOWN,
);
pub const ERROR_MALFORMED_COMPONENT: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_COMPONENT_MALFORMED,
    translation::java::ARGUMENTS_ITEM_COMPONENT_MALFORMED,
);
pub const ERROR_EXPECTED_COMPONENT: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_COMPONENT_EXPECTED,
    translation::java::ARGUMENTS_ITEM_COMPONENT_EXPECTED,
);
pub const ERROR_REPEATED_COMPONENT: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_ITEM_COMPONENT_REPEATED,
    translation::java::ARGUMENTS_ITEM_COMPONENT_REPEATED,
);

type ComponentPatch = Vec<(DataComponent, Option<Box<dyn DataComponentImpl>>)>;

fn read_components(reader: &mut StringReader) -> Result<ComponentPatch, CommandSyntaxError> {
    reader.expect('[')?;
    let mut patch: ComponentPatch = Vec::new();
    while reader.can_read_char() && reader.peek() != Some(']') {
        reader.skip_whitespace();
        let removal = reader.peek() == Some('!');
        if removal {
            reader.skip();
        }
        let component = read_component_type(reader)?;
        if patch.iter().any(|(visited, _)| *visited == component) {
            return Err(ERROR_REPEATED_COMPONENT
                .create_without_context(TextComponent::text(component.to_name())));
        }
        if removal {
            patch.push((component, None));
        } else {
            reader.skip_whitespace();
            reader.expect('=')?;
            reader.skip_whitespace();
            let start = reader.cursor();
            let tag = SnbtParser::parse_for_commands(reader)?;
            let end = reader.cursor();
            let value = read_data(component, &tag).ok_or_else(|| {
                reader.set_cursor(start);
                ERROR_MALFORMED_COMPONENT.create(
                    reader,
                    TextComponent::text(component.to_name()),
                    TextComponent::text(reader.string()[start..end].to_string()),
                )
            })?;
            patch.push((component, Some(value)));
        }
        reader.skip_whitespace();
        if reader.peek() != Some(',') {
            break;
        }
        reader.skip();
        reader.skip_whitespace();
        if !reader.can_read_char() {
            return Err(ERROR_EXPECTED_COMPONENT.create(reader));
        }
    }
    reader.expect(']')?;
    Ok(patch)
}

fn read_component_type(reader: &mut StringReader) -> Result<DataComponent, CommandSyntaxError> {
    if !reader.can_read_char() {
        return Err(ERROR_EXPECTED_COMPONENT.create(reader));
    }
    let start = reader.cursor();
    let id = Identifier::from_reader(reader)?;
    DataComponent::try_from_name(&id.to_string()).ok_or_else(|| {
        reader.set_cursor(start);
        ERROR_UNKNOWN_COMPONENT.create(reader, TextComponent::text(id.to_string()))
    })
}

#[derive(Clone, Copy)]
pub struct ItemStackArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for ItemStackArgumentType {
    type Item = ItemStack;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        while let Some(c) = reader.peek() {
            if c.is_alphanumeric() || c == '_' || c == ':' || c == '/' || c == '.' || c == '-' {
                reader.skip();
            } else {
                break;
            }
        }
        let raw_id = &reader.string()[start..reader.cursor()];
        let item = Item::from_registry_key(raw_id).ok_or_else(|| {
            let full_name = if raw_id.contains(':') {
                raw_id.to_string()
            } else {
                format!("minecraft:{raw_id}")
            };
            ERROR_UNKNOWN_ITEM.create(reader, TextComponent::text(full_name))
        })?;

        let mut stack = ItemStack::new(1, item);

        if reader.peek() == Some('[') {
            stack.patch = read_components(reader)?;
        }

        Ok(stack)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ItemStack
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.build()
    }
}

impl ItemStackArgumentType {
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<ItemStack, CommandSyntaxError> {
        context.get_argument::<ItemStack>(name).cloned()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::source::DummySource;

    fn parse(input: &str) -> Result<ItemStack, CommandSyntaxError> {
        ArgumentType::<DummySource>::parse(&ItemStackArgumentType, &mut StringReader::new(input))
    }

    #[test]
    fn components_with_slashes_parse() {
        let stack = parse("salmon_spawn_egg[salmon/size=\"large\",!max_stack_size]").unwrap();
        assert_eq!(stack.patch.len(), 2);
        assert_eq!(stack.patch[0].0, DataComponent::SalmonSize);
        assert!(stack.patch[1].1.is_none());
    }

    #[test]
    fn bad_component_input_errors_instead_of_looping() {
        assert!(parse("salmon_spawn_egg[salmon/size]").is_err());
        assert!(parse("salmon_spawn_egg[not/a_component=1]").is_err());
        assert!(parse("salmon_spawn_egg[salmon/size=\"large\",").is_err());
        assert!(parse("salmon_spawn_egg[%]").is_err());
    }
}
