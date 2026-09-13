use crate::argument_types::FromStringReader;
use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_protocol::java::client::play::SuggestionProviders;
use pumpkin_util::identifier::Identifier;

pub struct StructureNameArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for StructureNameArgumentType {
    type Item = Identifier;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        Identifier::from_reader(reader)
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        let names = pumpkin_data::structures::StructureKeys::all_names();
        builder
            .filter_and_suggest_iter(names.iter().copied())
            .build()
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ResourceLocation
    }

    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "minecraft:village_plains".to_string(),
            "minecraft:ancient_city".to_string(),
        ]
    }
}
