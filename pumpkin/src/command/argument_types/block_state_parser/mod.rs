use pumpkin_data::{Block, block_properties::BlockProperties, tag::{RegistryKey, get_latest_map, get_tag_ids}, translation::java::{ARGUMENT_BLOCK_ID_INVALID, ARGUMENT_BLOCK_PROPERTY_DUPLICATE, ARGUMENT_BLOCK_PROPERTY_INVALID, ARGUMENT_BLOCK_PROPERTY_NOVALUE, ARGUMENT_BLOCK_PROPERTY_UNCLOSED, ARGUMENT_BLOCK_PROPERTY_UNKNOWN, ARGUMENT_BLOCK_TAG_DISALLOWED, ARGUMENTS_BLOCK_TAG_UNKNOWN}};
use pumpkin_util::{identifier::Identifier, text::TextComponent};

use crate::command::{argument_types::FromStringReader, errors::{command_syntax_error::CommandSyntaxError, error_types::CommandErrorType}, string_reader::StringReader};

pub const NO_TAGS_ALLOWED_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    ARGUMENT_BLOCK_TAG_DISALLOWED,
    ARGUMENT_BLOCK_TAG_DISALLOWED
);

pub const UNKNOWN_BLOCK_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    ARGUMENT_BLOCK_ID_INVALID,
    ARGUMENT_BLOCK_ID_INVALID
);

pub const UNKNOWN_PROPERTY_ERROR_TYPE: CommandErrorType<2> = CommandErrorType::new(
    ARGUMENT_BLOCK_PROPERTY_UNKNOWN,
    ARGUMENT_BLOCK_PROPERTY_UNKNOWN
);

pub const DUPLICATE_PROPERTY_ERROR_TYPE: CommandErrorType<2> = CommandErrorType::new(
    ARGUMENT_BLOCK_PROPERTY_DUPLICATE,
    ARGUMENT_BLOCK_PROPERTY_DUPLICATE
);

pub const INVALID_VALUE_ERROR_TYPE: CommandErrorType<3> = CommandErrorType::new(
    ARGUMENT_BLOCK_PROPERTY_INVALID,
    ARGUMENT_BLOCK_PROPERTY_INVALID
);

pub const EXPECTED_VALUE_ERROR_TYPE: CommandErrorType<2> = CommandErrorType::new(
    ARGUMENT_BLOCK_PROPERTY_NOVALUE,
    ARGUMENT_BLOCK_PROPERTY_NOVALUE
);

pub const EXPECTED_END_OF_PROPERTIES_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    ARGUMENT_BLOCK_PROPERTY_UNCLOSED,
    ARGUMENT_BLOCK_PROPERTY_UNCLOSED
);

pub const UNKNOWN_TAG_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    ARGUMENTS_BLOCK_TAG_UNKNOWN,
    ARGUMENTS_BLOCK_TAG_UNKNOWN
);

pub struct BlockStateParser<'b, 'a> {
    reader: &'b mut StringReader<'a>,
    for_testing: bool,
    allow_nbt: bool,
    suggestions: BlockStateParserSuggestions,
    id: Option<Identifier>,
    tag: Option<&'static [u16]>,
    block_properties: Option<Box<dyn BlockProperties>>,
    state: Option<u16>
}

impl<'b, 'a> BlockStateParser<'b, 'a> {
    fn parse(&mut self) -> Result<(), CommandSyntaxError> {
        if self.for_testing {
            self.suggestions = BlockStateParserSuggestions::BlockIdOrTag;
        } else {
            self.suggestions = BlockStateParserSuggestions::Item;
        }

        if self.reader.peek() == Some('#') {
            self.read_tag();
            self.suggestions = BlockStateParserSuggestions::OpenVaguePropertiesOrNbt;
            if self.reader.peek() == Some('[') {
                self.read_vague_properties();
                self.suggestions = BlockStateParserSuggestions::OpenNbt;
            }
        } else {
            self.read_block();
            self.suggestions = BlockStateParserSuggestions::OpenPropertiesOrNbt;
            if self.reader.peek() == Some('[') {
                self.read_properties();
                self.suggestions = BlockStateParserSuggestions::OpenNbt;
            }
        }

        if self.allow_nbt && self.reader.peek() == Some('{') {
            self.suggestions = BlockStateParserSuggestions::None;
            self.read_nbt();
        }

        Ok(())
    }

    fn read_block(&mut self) -> Result<(), CommandSyntaxError> {
        let start = self.reader.cursor();
        let id = Identifier::from_reader(self.reader)?;
        if let Some(block) = Block::from_name(&id.to_string()) {
            let state_id = block.default_state.id;
            self.block_properties = block.properties(state_id);
            self.state = Some(state_id);
            Ok(())
        } else {
            self.reader.set_cursor(start);
            Err(UNKNOWN_TAG_ERROR_TYPE.create(self.reader, TextComponent::text(id.to_string())))
        }
    }

    fn read_tag(&mut self) -> Result<(), CommandSyntaxError> {
        if self.for_testing {
            let start = self.reader.cursor();
            self.reader.expect('#')?;
            self.suggestions = BlockStateParserSuggestions::Tag;
            let id = Identifier::from_reader(self.reader)?;
            if let Some(tag) = get_tag_ids(RegistryKey::Block, &id.to_string()) {
                self.tag = Some(tag);
                Ok(())
            } else {
                self.reader.set_cursor(start);
                Err(UNKNOWN_TAG_ERROR_TYPE.create(self.reader, TextComponent::text(id.to_string())))
            }
        } else {
            Err(NO_TAGS_ALLOWED_ERROR_TYPE.create(self.reader))
        }
    }

    fn read_properties(&mut self) -> Result<(), CommandSyntaxError> {
        self.reader.skip();
        self.suggestions = BlockStateParserSuggestions::PropertyNameOrEnd;
        self.reader.skip_whitespace();

        while !matches!(self.reader.peek(), None | Some(']')) {
            self.reader.skip_whitespace();
            let key_start = self.reader.cursor();
            let key = self.reader.read_string();

            self.block_properties.is_some_and(|props| props.)
        }
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub(crate) enum BlockStateParserSuggestions {
    #[default]
    None,

    BlockIdOrTag,
    Item,
    OpenVaguePropertiesOrNbt,
    OpenPropertiesOrNbt,
    OpenNbt,
    Tag,
    PropertyNameOrEnd
}