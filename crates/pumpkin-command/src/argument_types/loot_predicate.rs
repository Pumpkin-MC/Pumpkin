use pumpkin_nbt::tag::NbtTag;
use pumpkin_util::identifier::Identifier;

use crate::{
    argument_types::{
        FromStringReader,
        argument_type::{ArgumentType, JavaClientArgumentType},
    },
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    snbt::SnbtParser,
    string_reader::StringReader,
};

/// A predicate given either by its registry id or inline, like vanilla's `ResourceOrInlineArgument`.
#[derive(Clone, Debug)]
pub enum LootPredicate {
    Reference(Identifier),
    Inline(NbtTag),
}

/// Parses a [`LootPredicate`].
pub struct LootPredicateArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for LootPredicateArgumentType {
    type Item = LootPredicate;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if matches!(reader.peek(), Some('{' | '[')) {
            SnbtParser::parse_for_commands(reader).map(LootPredicate::Inline)
        } else {
            Identifier::from_reader(reader).map(LootPredicate::Reference)
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::LootPredicate
    }

    fn examples(&self) -> Vec<String> {
        examples!(
            "foo:bar",
            "{condition:\"minecraft:random_chance\",chance:0.5}"
        )
    }
}

impl LootPredicateArgumentType {
    pub fn get<'a, S: crate::source::CommandSource>(
        context: &'a CommandContext<S>,
        name: &str,
    ) -> Result<&'a LootPredicate, CommandSyntaxError> {
        context.get_argument(name)
    }
}
