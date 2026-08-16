use std::pin::Pin;

use crate::command::argument_types::FromStringReader;
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_protocol::java::client::play::SuggestionProviders;
use pumpkin_util::identifier::Identifier;

pub struct PoolNameArgumentType;

impl ArgumentType for PoolNameArgumentType {
    type Item = Identifier;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        Identifier::from_reader(reader)
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        let names = pumpkin_world::generation::structure::template::all_pool_names();
        Box::pin(async move {
            builder
                .filter_and_suggest_iter(names.iter().map(|n| format!("minecraft:{n}")))
                .build()
        })
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ResourceLocation
    }

    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }

    fn examples(&self) -> Vec<String> {
        vec!["minecraft:village/plains/houses".to_string()]
    }
}
