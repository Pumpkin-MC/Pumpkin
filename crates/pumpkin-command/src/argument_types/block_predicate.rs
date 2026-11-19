use std::collections::HashMap;

use pumpkin_data::tag::{RegistryKey, get_tag_ids};
use pumpkin_data::{Block, BlockStateId, Property, translation};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

use crate::argument_types::FromStringReader;
use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::argument_types::block::{
    ERROR_DUPLICATE_PROPERTY, ERROR_EXPECTED_VALUE, ERROR_UNCLOSED_PROPERTIES, parse_block_input,
    parse_nbt_compound,
};
use crate::argument_types::nbt_path::compare_nbt;
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

pub const ERROR_UNKNOWN_TAG: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_BLOCK_TAG_UNKNOWN,
    translation::java::ARGUMENTS_BLOCK_TAG_UNKNOWN,
);

#[derive(Clone, Debug, PartialEq)]
pub enum BlockPredicate {
    Block {
        block: &'static Block,
        defined_properties: Vec<Property>,
        nbt: Option<NbtCompound>,
    },
    Tag {
        tag_name: String,
        block_ids: Vec<u16>,
        properties: HashMap<String, String>,
        nbt: Option<NbtCompound>,
    },
}

impl BlockPredicate {
    /// Tests a block state; `block_entity_nbt` is only called when the predicate has NBT.
    #[must_use]
    pub fn test(
        &self,
        state_id: BlockStateId,
        block_entity_nbt: impl FnOnce() -> Option<NbtCompound>,
    ) -> bool {
        let block = Block::from_state_id(state_id);
        let nbt = match self {
            Self::Block {
                block: expected,
                defined_properties,
                nbt,
            } => {
                if block.id != expected.id {
                    return false;
                }
                if !defined_properties.is_empty() {
                    let Some(state_properties) = block.properties(state_id) else {
                        return false;
                    };
                    let state_properties = state_properties.to_props();
                    if !defined_properties
                        .iter()
                        .all(|prop| state_properties.contains(prop))
                    {
                        return false;
                    }
                }
                nbt
            }
            Self::Tag {
                block_ids,
                properties,
                nbt,
                ..
            } => {
                if !block_ids.contains(&block.id.as_u16()) {
                    return false;
                }
                if !properties.is_empty() {
                    let Some(state_properties) = block.properties(state_id) else {
                        return false;
                    };
                    let state_properties = state_properties.to_props();
                    let all_match = properties.iter().all(|(key, value)| {
                        block
                            .parse_property(key, value)
                            .is_ok_and(|prop| state_properties.contains(&prop))
                    });
                    if !all_match {
                        return false;
                    }
                }
                nbt
            }
        };

        nbt.as_ref().is_none_or(|expected| {
            block_entity_nbt().is_some_and(|actual| {
                compare_nbt(
                    &NbtTag::Compound(expected.clone()),
                    &NbtTag::Compound(actual),
                )
            })
        })
    }

    #[must_use]
    pub const fn requires_nbt(&self) -> bool {
        match self {
            Self::Block { nbt, .. } | Self::Tag { nbt, .. } => nbt.is_some(),
        }
    }
}

fn read_vague_properties(
    reader: &mut StringReader,
    tag_id: &str,
) -> Result<HashMap<String, String>, CommandSyntaxError> {
    reader.skip();
    let mut value_start: Option<usize> = None;
    reader.skip_whitespace();
    let mut properties = HashMap::new();

    while reader.can_read_char() && reader.peek() != Some(']') {
        reader.skip_whitespace();
        let key_start = reader.cursor();
        let key = reader.read_string()?;
        if properties.contains_key(&key) {
            reader.set_cursor(key_start);
            return Err(ERROR_DUPLICATE_PROPERTY.create(
                reader,
                TextComponent::text(key),
                TextComponent::text(tag_id.to_string()),
            ));
        }

        reader.skip_whitespace();
        if !reader.can_read_char() || reader.peek() != Some('=') {
            reader.set_cursor(key_start);
            return Err(ERROR_EXPECTED_VALUE.create(
                reader,
                TextComponent::text(key),
                TextComponent::text(tag_id.to_string()),
            ));
        }

        reader.skip();
        reader.skip_whitespace();
        value_start = Some(reader.cursor());
        let value = reader.read_string()?;
        properties.insert(key, value);
        reader.skip_whitespace();
        if reader.can_read_char() {
            value_start = None;
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
        Ok(properties)
    } else {
        if let Some(pos) = value_start {
            reader.set_cursor(pos);
        }
        Err(ERROR_UNCLOSED_PROPERTIES.create(reader))
    }
}

fn parse_tag_predicate(reader: &mut StringReader) -> Result<BlockPredicate, CommandSyntaxError> {
    let initial_cursor = reader.cursor();
    parse_tag_predicate_inner(reader).inspect_err(|_| reader.set_cursor(initial_cursor))
}

fn parse_tag_predicate_inner(
    reader: &mut StringReader,
) -> Result<BlockPredicate, CommandSyntaxError> {
    let tag_start = reader.cursor();
    reader.skip();
    let id = Identifier::from_reader(reader)?;
    let normalized = id.to_string();
    let stripped = normalized.strip_prefix("minecraft:").unwrap_or(&normalized);

    let tag_ids = get_tag_ids(RegistryKey::Block, &normalized)
        .or_else(|| get_tag_ids(RegistryKey::Block, stripped));
    let Some(ids) = tag_ids else {
        reader.set_cursor(tag_start);
        return Err(ERROR_UNKNOWN_TAG.create(reader, TextComponent::text(normalized)));
    };

    let properties = if reader.peek() == Some('[') {
        read_vague_properties(reader, &normalized)?
    } else {
        HashMap::new()
    };
    let nbt = parse_nbt_compound(reader)?;

    Ok(BlockPredicate::Tag {
        tag_name: normalized,
        block_ids: ids.to_vec(),
        properties,
        nbt,
    })
}

pub struct BlockPredicateArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for BlockPredicateArgumentType {
    type Item = BlockPredicate;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if reader.peek() == Some('#') {
            parse_tag_predicate(reader)
        } else {
            let input = parse_block_input(reader, true)?;
            Ok(BlockPredicate::Block {
                block: input.block,
                defined_properties: input.defined_properties,
                nbt: input.nbt,
            })
        }
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::BlockPredicate
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "stone".to_string(),
            "minecraft:stone".to_string(),
            "stone[foo=bar]".to_string(),
            "#stone".to_string(),
            "#stone[foo=bar]{baz=nbt}".to_string(),
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

impl BlockPredicateArgumentType {
    pub fn get<S: crate::source::CommandSource>(
        context: &CommandContext<S>,
        name: &str,
    ) -> Result<BlockPredicate, CommandSyntaxError> {
        Ok(context.get_argument::<BlockPredicate>(name)?.clone())
    }
}

#[cfg(test)]
mod test {
    use super::{BlockPredicateArgumentType, ERROR_UNKNOWN_TAG};
    use crate::{
        argument_types::{
            argument_type::ArgumentType,
            block::{ERROR_DUPLICATE_PROPERTY, ERROR_INVALID_VALUE, ERROR_UNKNOWN_PROPERTY},
        },
        source::DummySource,
        string_reader::StringReader,
    };
    use pumpkin_data::{Block, Property};
    use pumpkin_nbt::compound::NbtCompound;

    #[test]
    fn concrete_predicate_validates_properties_and_matches_subset() {
        let mut reader = StringReader::new("minecraft:oak_stairs[half=top]{CustomName:\"A\"}");
        let pred =
            ArgumentType::<DummySource>::parse(&BlockPredicateArgumentType, &mut reader).unwrap();
        assert!(pred.requires_nbt());

        let top_north = Block::OAK_STAIRS.apply_properties(
            Block::OAK_STAIRS.default_state.id,
            &[Property::Half(pumpkin_data::block_properties::Half::Top)],
        );
        let mut be_nbt = NbtCompound::new();
        be_nbt.put_string("CustomName", "A".to_string());
        be_nbt.put_int("Extra", 42);

        assert!(pred.test(top_north, || Some(be_nbt.clone())));
        assert!(!pred.test(Block::OAK_STAIRS.default_state.id, || Some(be_nbt)));

        let mut bad_prop = StringReader::new("minecraft:stone[half=top]");
        let err = ArgumentType::<DummySource>::parse(&BlockPredicateArgumentType, &mut bad_prop)
            .unwrap_err();
        assert!(err.is(&ERROR_UNKNOWN_PROPERTY));

        let mut bad_val = StringReader::new("minecraft:oak_stairs[half=sideways]");
        let err = ArgumentType::<DummySource>::parse(&BlockPredicateArgumentType, &mut bad_val)
            .unwrap_err();
        assert!(err.is(&ERROR_INVALID_VALUE));
    }

    #[test]
    fn tag_predicate_parses_vague_properties_and_rejects_duplicates() {
        let mut reader = StringReader::new("#minecraft:stairs[half=top]");
        let pred =
            ArgumentType::<DummySource>::parse(&BlockPredicateArgumentType, &mut reader).unwrap();

        let top_stairs = Block::STONE_STAIRS.apply_properties(
            Block::STONE_STAIRS.default_state.id,
            &[Property::Half(pumpkin_data::block_properties::Half::Top)],
        );
        assert!(pred.test(top_stairs, || None));
        assert!(!pred.test(Block::STONE_STAIRS.default_state.id, || None));
        assert!(!pred.test(Block::STONE.default_state.id, || None));

        let mut dup = StringReader::new("#minecraft:stairs[half=top,half=bottom]");
        let err =
            ArgumentType::<DummySource>::parse(&BlockPredicateArgumentType, &mut dup).unwrap_err();
        assert!(err.is(&ERROR_DUPLICATE_PROPERTY));

        let mut unknown = StringReader::new("#minecraft:definitely_not_a_real_tag");
        let err = ArgumentType::<DummySource>::parse(&BlockPredicateArgumentType, &mut unknown)
            .unwrap_err();
        assert!(err.is(&ERROR_UNKNOWN_TAG));
        assert_eq!(unknown.cursor(), 0);
    }
}
