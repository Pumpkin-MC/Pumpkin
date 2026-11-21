use pumpkin_data::{Block, BlockStateId, Property, PropertyLookupError, translation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

use crate::{
    argument_types::{
        FromStringReader,
        argument_type::{ArgumentType, JavaClientArgumentType},
        nbt::NbtCompoundArgumentType,
    },
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    errors::error_types::CommandErrorType,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};

pub const ERROR_NO_TAGS_ALLOWED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_TAG_DISALLOWED,
    translation::java::ARGUMENT_BLOCK_TAG_DISALLOWED,
);

pub const INVALID_BLOCK_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_ID_INVALID,
    translation::java::ARGUMENT_BLOCK_ID_INVALID,
);

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

pub const ERROR_EXPECTED_VALUE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_NOVALUE,
    translation::java::ARGUMENT_BLOCK_PROPERTY_NOVALUE,
);

pub const ERROR_UNCLOSED_PROPERTIES: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNCLOSED,
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNCLOSED,
);

/// Parsed block state argument matching vanilla `BlockInput`.
#[derive(Clone, Debug, PartialEq)]
pub struct BlockInput {
    pub block: &'static Block,
    pub state_id: BlockStateId,
    pub defined_properties: Vec<Property>,
    pub nbt: Option<NbtCompound>,
}

impl BlockInput {
    #[must_use]
    pub const fn new(
        block: &'static Block,
        state_id: BlockStateId,
        defined_properties: Vec<Property>,
        nbt: Option<NbtCompound>,
    ) -> Self {
        Self {
            block,
            state_id,
            defined_properties,
            nbt,
        }
    }

    #[must_use]
    pub fn overwrite_defined_properties(&self, state_id: BlockStateId) -> BlockStateId {
        if state_id == self.state_id {
            return state_id;
        }
        self.block
            .apply_properties(state_id, &self.defined_properties)
    }
}

pub(crate) fn parse_nbt_compound(
    reader: &mut StringReader,
) -> Result<Option<NbtCompound>, CommandSyntaxError> {
    if reader.peek() == Some('{') {
        let tag =
            ArgumentType::<crate::source::DummySource>::parse(&NbtCompoundArgumentType, reader)?;
        Ok(Some(tag))
    } else {
        Ok(None)
    }
}

fn read_block_properties(
    reader: &mut StringReader,
    block: &'static Block,
    block_id: &str,
) -> Result<Vec<Property>, CommandSyntaxError> {
    reader.skip();
    reader.skip_whitespace();
    let mut defined: Vec<Property> = Vec::new();

    while reader.can_read_char() && reader.peek() != Some(']') {
        reader.skip_whitespace();
        let key_start = reader.cursor();
        let key = reader.read_string()?;

        if !block.has_property(&key) {
            reader.set_cursor(key_start);
            return Err(ERROR_UNKNOWN_PROPERTY.create(
                reader,
                TextComponent::text(block_id.to_string()),
                TextComponent::text(key),
            ));
        }

        if defined.iter().any(|p| p.key().to_str() == key) {
            reader.set_cursor(key_start);
            return Err(ERROR_DUPLICATE_PROPERTY.create(
                reader,
                TextComponent::text(key),
                TextComponent::text(block_id.to_string()),
            ));
        }

        reader.skip_whitespace();
        if reader.peek() != Some('=') {
            return Err(ERROR_EXPECTED_VALUE.create(
                reader,
                TextComponent::text(key),
                TextComponent::text(block_id.to_string()),
            ));
        }
        reader.skip();
        reader.skip_whitespace();

        let val_start = reader.cursor();
        let raw_value = reader.read_string()?;
        let prop = match block.parse_property(&key, &raw_value) {
            Ok(prop) => prop,
            Err(PropertyLookupError::UnknownProperty) => {
                reader.set_cursor(key_start);
                return Err(ERROR_UNKNOWN_PROPERTY.create(
                    reader,
                    TextComponent::text(block_id.to_string()),
                    TextComponent::text(key),
                ));
            }
            Err(PropertyLookupError::InvalidValue) => {
                reader.set_cursor(val_start);
                return Err(ERROR_INVALID_VALUE.create(
                    reader,
                    TextComponent::text(block_id.to_string()),
                    TextComponent::text(raw_value),
                    TextComponent::text(key),
                ));
            }
        };
        defined.push(prop);

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
        Ok(defined)
    } else {
        Err(ERROR_UNCLOSED_PROPERTIES.create(reader))
    }
}

pub(crate) fn parse_block_input(
    reader: &mut StringReader,
    allow_nbt: bool,
) -> Result<BlockInput, CommandSyntaxError> {
    let initial_cursor = reader.cursor();
    parse_block_input_inner(reader, allow_nbt).inspect_err(|_| reader.set_cursor(initial_cursor))
}

fn parse_block_input_inner(
    reader: &mut StringReader,
    allow_nbt: bool,
) -> Result<BlockInput, CommandSyntaxError> {
    if reader.peek() == Some('#') {
        return Err(ERROR_NO_TAGS_ALLOWED.create(reader));
    }

    let id_start = reader.cursor();
    let id = Identifier::from_reader(reader)?;
    let id_str = id.to_string();
    let Some(block) = Block::from_name(&id_str) else {
        reader.set_cursor(id_start);
        return Err(INVALID_BLOCK_ERROR_TYPE.create(reader, TextComponent::text(id_str)));
    };

    let defined_properties = if reader.peek() == Some('[') {
        read_block_properties(reader, block, &id_str)?
    } else {
        Vec::new()
    };

    let state_id = block.apply_properties(block.default_state.id, &defined_properties);
    let nbt = if allow_nbt {
        parse_nbt_compound(reader)?
    } else {
        None
    };

    Ok(BlockInput::new(block, state_id, defined_properties, nbt))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for BlockArgumentType {
    type Item = BlockInput;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        parse_block_input(reader, true)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::BlockState
    }

    fn examples(&self) -> Vec<String> {
        examples!("stone", "minecraft:stone", "stone[foo=bar]", "foo{bar=baz}")
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.build()
    }
}

impl BlockArgumentType {
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<BlockInput, CommandSyntaxError> {
        Ok(context.get_argument::<BlockInput>(name)?.clone())
    }
}

#[cfg(test)]
mod test {
    use super::{
        BlockArgumentType, ERROR_DUPLICATE_PROPERTY, ERROR_EXPECTED_VALUE, ERROR_INVALID_VALUE,
        ERROR_NO_TAGS_ALLOWED, ERROR_UNCLOSED_PROPERTIES, ERROR_UNKNOWN_PROPERTY,
        INVALID_BLOCK_ERROR_TYPE,
    };
    use crate::{
        argument_types::argument_type::ArgumentType, source::DummySource,
        string_reader::StringReader,
    };
    use pumpkin_data::{Block, Property};

    #[test]
    fn parses_block_state_properties_and_nbt() {
        let mut reader = StringReader::new(
            "minecraft:chest[facing=south,waterlogged=true]{CustomName:\"Loot\"} rest",
        );
        let input = ArgumentType::<DummySource>::parse(&BlockArgumentType, &mut reader).unwrap();
        assert_eq!(input.block.id, Block::CHEST.id);
        assert_ne!(input.state_id, Block::CHEST.default_state.id);
        assert!(Block::CHEST.is_waterlogged(input.state_id));
        assert_eq!(input.defined_properties.len(), 2);
        assert!(
            input
                .defined_properties
                .contains(&Property::Waterlogged(true))
        );
        assert_eq!(
            input
                .nbt
                .as_ref()
                .and_then(|nbt| nbt.get_string("CustomName")),
            Some("Loot")
        );
        assert_eq!(reader.remaining_part(), " rest");
    }

    #[test]
    fn validates_block_specific_property_domains_and_errors() {
        let mut hopper_ok = StringReader::new("hopper[facing=north]");
        assert!(ArgumentType::<DummySource>::parse(&BlockArgumentType, &mut hopper_ok).is_ok());

        let mut hopper_up = StringReader::new("hopper[facing=up]");
        assert_parse_err_reset!(hopper_up, BlockArgumentType, &ERROR_INVALID_VALUE);

        let mut fence_invalid = StringReader::new("oak_fence[west=none]");
        assert_parse_err_reset!(fence_invalid, BlockArgumentType, &ERROR_INVALID_VALUE);

        let mut wall_valid = StringReader::new("cobblestone_wall[west=none]");
        assert!(ArgumentType::<DummySource>::parse(&BlockArgumentType, &mut wall_valid).is_ok());

        let mut unknown_prop = StringReader::new("stone[facing=north]");
        assert_parse_err_reset!(unknown_prop, BlockArgumentType, &ERROR_UNKNOWN_PROPERTY);

        let mut duplicate_prop = StringReader::new("oak_fence[west=true,west=false]");
        assert_parse_err_reset!(duplicate_prop, BlockArgumentType, &ERROR_DUPLICATE_PROPERTY);

        let mut missing_val = StringReader::new("oak_fence[west]");
        assert_parse_err_reset!(missing_val, BlockArgumentType, &ERROR_EXPECTED_VALUE);

        let mut unclosed = StringReader::new("oak_fence[west=true");
        assert_parse_err_reset!(unclosed, BlockArgumentType, &ERROR_UNCLOSED_PROPERTIES);

        let mut disallowed_tag = StringReader::new("#minecraft:planks");
        assert_parse_err_reset!(disallowed_tag, BlockArgumentType, &ERROR_NO_TAGS_ALLOWED);

        let mut invalid_id = StringReader::new("minecraft:not_a_real_block");
        assert_parse_err_reset!(invalid_id, BlockArgumentType, &INVALID_BLOCK_ERROR_TYPE);
    }
}
