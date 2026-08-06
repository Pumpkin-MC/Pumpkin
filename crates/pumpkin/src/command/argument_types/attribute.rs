use std::pin::Pin;

use crate::command::{
    argument_types::FromStringReader,
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    errors::error_types::CommandErrorType,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};
use pumpkin_data::attributes::Attributes;
use pumpkin_data::translation;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::text::TextComponent;

pub const INVALID_ATTRIBUTE_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_ID_INVALID,
    translation::java::ARGUMENT_ID_INVALID,
);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct AttributeArgumentType;

impl ArgumentType for AttributeArgumentType {
    type Item = Attributes;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let identifier = Identifier::from_reader(reader)?;
        let ident_str = identifier.to_string();

        let attr = Attributes::ALL
            .iter()
            .find(|a| a.name == ident_str)
            .cloned()
            .ok_or_else(|| {
                INVALID_ATTRIBUTE_ERROR_TYPE.create(reader, TextComponent::text(ident_str))
            })?;

        Ok(attr)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Resource {
            identifier: Identifier::vanilla("attribute").unwrap(),
        }
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move {
            builder
                .filter_and_suggest_iter(Attributes::ALL.iter().map(|a| a.name))
                .build()
        })
    }
}

impl AttributeArgumentType {
    pub fn get(context: &CommandContext, name: &str) -> Result<Attributes, CommandSyntaxError> {
        context.get_argument::<Attributes>(name).cloned()
    }
}
