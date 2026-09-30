use pumpkin_data::{Block, BlockStateId, translation};
use pumpkin_util::text::TextComponent;

use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::argument_types::block::BlockArgumentType;
use crate::argument_types::block_predicate::{ERROR_NO_VALUE, ERROR_UNCLOSED_PROPERTIES};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

pub const ERROR_UNKNOWN_PROPERTY: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNKNOWN,
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNKNOWN,
);

pub const ERROR_DUPLICATE_PROPERTY: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_DUPLICATE,
    translation::java::ARGUMENT_BLOCK_PROPERTY_DUPLICATE,
);

pub const ERROR_INVALID_VALUE: CommandErrorType<3> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_INVALID,
    translation::java::ARGUMENT_BLOCK_PROPERTY_INVALID,
);

/// A block with optional `[property=value,...]` state, like vanilla's `BlockStateArgument`.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockStateArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for BlockStateArgumentType {
    type Item = BlockStateId;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let block: &'static Block =
            ArgumentType::<crate::source::DummySource>::parse(&BlockArgumentType, reader)?;
        if reader.peek() != Some('[') {
            return Ok(block.default_state.id);
        }
        read_properties(block, reader)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::BlockState
    }

    fn examples(&self) -> Vec<String> {
        examples!("stone", "minecraft:stone", "stone[foo=bar]")
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.build()
    }
}

/// Mirrors vanilla `BlockStateParser.readProperties`, applying each property to the default state.
fn read_properties(
    block: &'static Block,
    reader: &mut StringReader,
) -> Result<BlockStateId, CommandSyntaxError> {
    let block_name = || TextComponent::text(format!("minecraft:{}", block.name));
    let valid_properties: Vec<(&'static str, &'static str)> = block
        .states
        .iter()
        .filter_map(|state| block.properties(state.id))
        .flat_map(|props| props.to_props())
        .collect();
    let mut properties = block
        .properties(block.default_state.id)
        .map(|props| props.to_props())
        .unwrap_or_default();
    let mut seen: Vec<&'static str> = Vec::new();

    reader.skip();
    reader.skip_whitespace();
    while reader.can_read_char() && reader.peek() != Some(']') {
        reader.skip_whitespace();
        let key_start = reader.cursor();
        let key = reader.read_string()?;
        let Some(&(name, _)) = valid_properties.iter().find(|(name, _)| *name == key) else {
            reader.set_cursor(key_start);
            return Err(ERROR_UNKNOWN_PROPERTY.create(
                reader,
                block_name(),
                TextComponent::text(key),
            ));
        };
        if seen.contains(&name) {
            reader.set_cursor(key_start);
            return Err(ERROR_DUPLICATE_PROPERTY.create(
                reader,
                TextComponent::text(key),
                block_name(),
            ));
        }
        seen.push(name);

        reader.skip_whitespace();
        if reader.peek() != Some('=') {
            return Err(ERROR_NO_VALUE.create(reader, TextComponent::text(key)));
        }
        reader.skip();
        reader.skip_whitespace();

        let value_start = reader.cursor();
        let raw = reader.read_string()?;
        let Some(&(_, value)) = valid_properties
            .iter()
            .find(|(n, v)| *n == name && *v == raw)
        else {
            reader.set_cursor(value_start);
            return Err(ERROR_INVALID_VALUE.create(
                reader,
                block_name(),
                TextComponent::text(raw),
                TextComponent::text(name),
            ));
        };
        if let Some(entry) = properties.iter_mut().find(|(n, _)| *n == name) {
            entry.1 = value;
        }

        reader.skip_whitespace();
        if reader.can_read_char() {
            if reader.peek() != Some(',') {
                if reader.peek() != Some(']') {
                    return Err(ERROR_UNCLOSED_PROPERTIES.create(reader));
                }
                break;
            }
            reader.skip();
        }
    }

    if reader.can_read_char() {
        reader.skip();
    } else {
        return Err(ERROR_UNCLOSED_PROPERTIES.create(reader));
    }

    Ok(block.from_properties(&properties).to_state_id(block))
}

impl_copy_get!(BlockStateArgumentType, BlockStateId);

#[cfg(test)]
mod test {
    use pumpkin_data::BlockStateId;

    use crate::argument_types::block_predicate::ERROR_UNCLOSED_PROPERTIES;
    use crate::argument_types::block_state::{
        BlockStateArgumentType, ERROR_DUPLICATE_PROPERTY, ERROR_INVALID_VALUE,
        ERROR_UNKNOWN_PROPERTY,
    };
    use crate::string_reader::StringReader;

    // State ids are taken from the extracted vanilla `assets/blocks.json`.
    #[test]
    fn parse_properties() {
        let mut reader = StringReader::new("birch_stairs");
        assert_parse_ok_reset!(
            &mut reader,
            BlockStateArgumentType,
            BlockStateId::new(11561).unwrap()
        );

        let mut reader = StringReader::new("minecraft:birch_stairs[facing=west]");
        assert_parse_ok_reset!(
            &mut reader,
            BlockStateArgumentType,
            BlockStateId::new(11601).unwrap()
        );

        let mut reader = StringReader::new("birch_stairs[ half = top , facing=west ]");
        assert_parse_ok_reset!(
            &mut reader,
            BlockStateArgumentType,
            BlockStateId::new(11591).unwrap()
        );
    }

    #[test]
    fn reject_bad_properties() {
        let mut reader = StringReader::new("birch_stairs[color=red]");
        assert_parse_err_reset!(&mut reader, BlockStateArgumentType, &ERROR_UNKNOWN_PROPERTY);

        let mut reader = StringReader::new("birch_stairs[facing=up]");
        assert_parse_err_reset!(&mut reader, BlockStateArgumentType, &ERROR_INVALID_VALUE);

        let mut reader = StringReader::new("birch_stairs[facing=west,facing=east]");
        assert_parse_err_reset!(
            &mut reader,
            BlockStateArgumentType,
            &ERROR_DUPLICATE_PROPERTY
        );

        let mut reader = StringReader::new("birch_stairs[facing=west");
        assert_parse_err_reset!(
            &mut reader,
            BlockStateArgumentType,
            &ERROR_UNCLOSED_PROPERTIES
        );
    }
}
