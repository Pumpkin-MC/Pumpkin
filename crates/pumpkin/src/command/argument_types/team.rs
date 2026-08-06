use crate::command::{
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};
use std::pin::Pin;

/// Represents an argument type parsing a team name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct TeamArgumentType;

impl ArgumentType for TeamArgumentType {
    type Item = String;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let name = reader.read_unquoted_string();
        Ok(name)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Team
    }

    fn list_suggestions<'a>(
        &'a self,
        context: &'a CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move {
            let scoreboard = context.world().scoreboard.lock().await;
            for team_name in scoreboard.get_teams().keys() {
                builder = builder.filter_and_suggest_one(team_name.as_str());
            }
            builder.build()
        })
    }

    fn examples(&self) -> Vec<String> {
        vec!["foo".to_string(), "foo_bar".to_string()]
    }
}

impl TeamArgumentType {
    /// Returns a [`CommandContext`]'s parsed `String` argument as a string slice.
    pub fn get<'a>(context: &'a CommandContext, name: &str) -> Result<&'a str, CommandSyntaxError> {
        Ok(context.get_argument::<String>(name)?.as_str())
    }
}
