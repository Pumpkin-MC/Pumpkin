use crate::{
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    errors::error_types::CommandErrorType,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};
use pumpkin_data::{Block, BlockStateId, translation};
use pumpkin_util::text::TextComponent;

pub const INVALID_BLOCK_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_ID_INVALID,
    translation::java::ARGUMENT_BLOCK_ID_INVALID,
);

pub const ERROR_NO_VALUE: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_NOVALUE,
    translation::java::ARGUMENT_BLOCK_PROPERTY_NOVALUE,
);

pub const ERROR_UNCLOSED_PROPERTIES: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNCLOSED,
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNCLOSED,
);

pub const ERROR_UNKNOWN_PROPERTY: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNKNOWN,
    translation::java::ARGUMENT_BLOCK_PROPERTY_UNKNOWN,
);

pub const ERROR_INVALID_PROPERTY: CommandErrorType<3> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_INVALID,
    translation::java::ARGUMENT_BLOCK_PROPERTY_INVALID,
);

pub const ERROR_DUPLICATE_PROPERTY: CommandErrorType<2> = CommandErrorType::new(
    translation::java::ARGUMENT_BLOCK_PROPERTY_DUPLICATE,
    translation::java::ARGUMENT_BLOCK_PROPERTY_DUPLICATE,
);

/// A block name together with the state properties that were written after it.
///
/// This is vanilla's `BlockInput`: `state_id` is what a placing command such as `setblock`
/// writes, while `properties` keeps only what the command actually spelled out so a matching
/// command can leave everything else unspecified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockInput {
    pub block: &'static Block,
    /// The block's default state with every written property applied.
    pub state_id: BlockStateId,
    /// Only the properties that were written, in the order they were written.
    pub properties: Vec<(String, String)>,
}

impl BlockInput {
    /// Whether the given state satisfies this input.
    ///
    /// Properties that were not written are wildcards, the way vanilla's
    /// `BlockPredicateArgument` treats them.
    #[must_use]
    pub fn test(&self, block: &Block, state_id: BlockStateId) -> bool {
        self.block.id == block.id && properties_match(block, state_id, &self.properties)
    }
}

/// Whether `state_id` of `block` carries every one of `expected`.
///
/// Properties absent from `expected` are not looked at.
#[must_use]
pub fn properties_match(
    block: &Block,
    state_id: BlockStateId,
    expected: &[(String, String)],
) -> bool {
    if expected.is_empty() {
        return true;
    }
    let Some(props) = block.properties(state_id) else {
        return false;
    };
    let actual = props.to_props();
    expected.iter().all(|(name, value)| {
        actual
            .iter()
            .any(|(n, v)| *n == name.as_str() && *v == value.as_str())
    })
}

/// The property names a block accepts, each with the values it accepts for it.
///
/// Vanilla reads this off `StateDefinition.getProperties()`. Pumpkin's generated block data
/// carries no such table, so the same information is recovered by walking the block's states:
/// every state lists every property, so the union over all states is exactly the definition.
fn property_table(block: &Block) -> Vec<(&'static str, Vec<&'static str>)> {
    let mut table: Vec<(&'static str, Vec<&'static str>)> = Vec::new();
    for state in block.states {
        let Some(props) = block.properties(state.id) else {
            break;
        };
        for (name, value) in props.to_props() {
            if let Some((_, values)) = table.iter_mut().find(|(n, _)| *n == name) {
                if !values.contains(&value) {
                    values.push(value);
                }
            } else {
                table.push((name, vec![value]));
            }
        }
    }
    table
}

/// Reads a `[key=value, ...]` state suffix if one follows, returning an empty list otherwise.
///
/// This is vanilla's `BlockStateParser.readProperties`. When `block` is given, every name is
/// checked against that block's property definition and every value against the ones that
/// property accepts, and a name may only appear once. Tag predicates pass [`None`], because a
/// tag names many blocks and vanilla only resolves its properties against each candidate at
/// test time.
pub fn parse_properties(
    reader: &mut StringReader,
    block: Option<&Block>,
) -> Result<Vec<(String, String)>, CommandSyntaxError> {
    let mut properties = Vec::new();
    if reader.peek() != Some('[') {
        return Ok(properties);
    }
    let table = block.map(property_table);
    let block_name = block.map_or("", |block| block.name);

    reader.skip();
    reader.skip_whitespace();
    while reader.can_read_char() && reader.peek() != Some(']') {
        let key_start = reader.cursor();
        while let Some(c) = reader.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                reader.skip();
            } else {
                break;
            }
        }
        let key = reader.string()[key_start..reader.cursor()].to_string();

        if let Some(table) = &table
            && !table.iter().any(|(name, _)| *name == key)
        {
            return Err(ERROR_UNKNOWN_PROPERTY.create(
                reader,
                TextComponent::text(block_name),
                TextComponent::text(key),
            ));
        }
        if properties.iter().any(|(name, _)| *name == key) {
            return Err(ERROR_DUPLICATE_PROPERTY.create(
                reader,
                TextComponent::text(key),
                TextComponent::text(block_name),
            ));
        }

        reader.skip_whitespace();
        if reader.peek() != Some('=') {
            return Err(ERROR_NO_VALUE.create(
                reader,
                TextComponent::text(key),
                TextComponent::text(block_name),
            ));
        }
        reader.skip();
        reader.skip_whitespace();

        let value_start = reader.cursor();
        while let Some(c) = reader.peek() {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                reader.skip();
            } else {
                break;
            }
        }
        let value = reader.string()[value_start..reader.cursor()].to_string();

        if let Some(table) = &table
            && let Some((_, values)) = table.iter().find(|(name, _)| *name == key)
            && !values.contains(&value.as_str())
        {
            return Err(ERROR_INVALID_PROPERTY.create(
                reader,
                TextComponent::text(block_name),
                TextComponent::text(value),
                TextComponent::text(key),
            ));
        }

        properties.push((key, value));
        reader.skip_whitespace();
        if reader.peek() == Some(',') {
            reader.skip();
            reader.skip_whitespace();
        } else if reader.peek() == Some(']') {
            break;
        } else {
            return Err(ERROR_UNCLOSED_PROPERTIES.create_without_context());
        }
    }
    if reader.peek() == Some(']') {
        reader.skip();
    } else {
        return Err(ERROR_UNCLOSED_PROPERTIES.create_without_context());
    }
    Ok(properties)
}

/// Reads a block name, resolving it to a registered block.
pub fn parse_block_name(reader: &mut StringReader) -> Result<&'static Block, CommandSyntaxError> {
    let start = reader.cursor();
    while let Some(c) = reader.peek() {
        if c.is_alphanumeric() || c == '_' || c == ':' || c == '/' || c == '.' || c == '-' {
            reader.skip();
        } else {
            break;
        }
    }
    let block_name = &reader.string()[start..reader.cursor()];
    let normalized = if block_name.contains(':') {
        block_name.to_string()
    } else {
        format!("minecraft:{block_name}")
    };

    Block::from_name(&normalized)
        .ok_or_else(|| INVALID_BLOCK_ERROR_TYPE.create(reader, TextComponent::text(normalized)))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BlockArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for BlockArgumentType {
    type Item = BlockInput;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let block = parse_block_name(reader)?;
        let properties = parse_properties(reader, Some(block))?;
        let state_id = if properties.is_empty() {
            block.default_state.id
        } else {
            let props: Vec<(&str, &str)> = properties
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str()))
                .collect();
            block.from_properties(&props).to_state_id(block)
        };

        Ok(BlockInput {
            block,
            state_id,
            properties,
        })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::BlockState
    }

    fn examples(&self) -> Vec<String> {
        examples!["stone", "minecraft:stone", "oak_stairs[facing=east]"]
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
mod tests {
    use super::{
        BlockArgumentType, ERROR_DUPLICATE_PROPERTY, ERROR_INVALID_PROPERTY, ERROR_NO_VALUE,
        ERROR_UNCLOSED_PROPERTIES, ERROR_UNKNOWN_PROPERTY, INVALID_BLOCK_ERROR_TYPE,
    };
    use crate::argument_types::argument_type::ArgumentType;
    use crate::source::DummySource;
    use crate::string_reader::StringReader;
    use pumpkin_data::Block;

    fn parse(
        input: &str,
    ) -> Result<super::BlockInput, crate::errors::command_syntax_error::CommandSyntaxError> {
        ArgumentType::<DummySource>::parse(&BlockArgumentType, &mut StringReader::new(input))
    }

    /// A bare name still resolves, and it still means the default state.
    #[test]
    fn plain_name_is_the_default_state() {
        let parsed = parse("minecraft:stone").expect("stone is a block");
        assert_eq!(parsed.block.id, Block::STONE.id);
        assert_eq!(parsed.state_id, Block::STONE.default_state.id);
        assert!(parsed.properties.is_empty());
    }

    /// The whole point of the fix: a written property has to reach the placed state.
    #[test]
    fn written_properties_select_a_state() {
        let parsed = parse("minecraft:wheat[age=3]").expect("wheat accepts age");
        assert_ne!(parsed.state_id, Block::WHEAT.default_state.id);
        let props = Block::WHEAT
            .properties(parsed.state_id)
            .expect("wheat has properties")
            .to_props();
        assert!(props.contains(&("age", "3")));
    }

    /// Several properties at once, with the whitespace vanilla tolerates.
    #[test]
    fn multiple_properties_are_all_applied() {
        let parsed = parse("oak_stairs[ facing = east , half = top ]").expect("stairs parse");
        let props = parsed
            .block
            .properties(parsed.state_id)
            .expect("stairs have properties")
            .to_props();
        assert!(props.contains(&("facing", "east")));
        assert!(props.contains(&("half", "top")));
    }

    /// Only what was written is kept, so matching can treat the rest as wildcards.
    #[test]
    fn unwritten_properties_are_wildcards_when_matching() {
        let parsed = parse("oak_stairs[facing=east]").expect("stairs parse");
        assert_eq!(parsed.properties, vec![("facing".into(), "east".into())]);

        let mut matched = 0;
        for state in Block::OAK_STAIRS.states {
            if parsed.test(&Block::OAK_STAIRS, state.id) {
                matched += 1;
                let props = Block::OAK_STAIRS
                    .properties(state.id)
                    .expect("stairs have properties")
                    .to_props();
                assert!(props.contains(&("facing", "east")));
            }
        }
        assert!(matched > 1, "other properties should have stayed free");
        assert!(
            matched < Block::OAK_STAIRS.states.len(),
            "facing was ignored"
        );
        assert!(!parsed.test(&Block::STONE, Block::STONE.default_state.id));
    }

    #[test]
    fn bad_input_is_rejected() {
        for (input, expected) in [
            (
                "minecraft:not_a_block",
                &INVALID_BLOCK_ERROR_TYPE as &dyn crate::errors::error_types::AnyCommandErrorType,
            ),
            ("minecraft:stone[facing=east]", &ERROR_UNKNOWN_PROPERTY),
            ("minecraft:wheat[age=99]", &ERROR_INVALID_PROPERTY),
            ("minecraft:wheat[age=1,age=2]", &ERROR_DUPLICATE_PROPERTY),
            ("minecraft:wheat[age]", &ERROR_NO_VALUE),
            ("minecraft:wheat[age=1", &ERROR_UNCLOSED_PROPERTIES),
        ] {
            let error = parse(input).expect_err(input).error_type;
            assert_eq!(error, expected, "{input}");
        }
    }
}
