use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};

pub const ERROR_INVALID_JSON: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_COMPONENT_INVALID,
    translation::java::ARGUMENT_COMPONENT_INVALID,
);

#[derive(Clone, Copy)]
pub struct ComponentArgumentType;

impl ArgumentType for ComponentArgumentType {
    type Item = TextComponent;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let remaining = reader.remaining_part();
        if let Ok(comp) = serde_json::from_str::<TextComponent>(remaining) {
            reader.set_cursor(reader.string().len());
            return Ok(comp);
        }

        let s = reader.read_string()?;
        if let Ok(comp) = serde_json::from_str::<TextComponent>(&s) {
            return Ok(comp);
        }

        Ok(TextComponent::text(s))
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Component
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder.build()
    }
}

impl ComponentArgumentType {
    pub fn get(context: &CommandContext, name: &str) -> Result<TextComponent, CommandSyntaxError> {
        context.get_argument::<TextComponent>(name).cloned()
    }
}
