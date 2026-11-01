use crate::command::{
    CommandSource,
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    errors::error_types::CommandErrorType,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};
use crate::world::scoreboard::{Scoreboard, ScoreboardObjective};
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

pub(crate) const OBJECTIVE_NOT_FOUND_ERROR: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_OBJECTIVE_NOTFOUND,
    translation::java::ARGUMENTS_OBJECTIVE_NOTFOUND,
);

pub(crate) const OBJECTIVE_READ_ONLY_ERROR: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENTS_OBJECTIVE_READONLY,
    translation::java::ARGUMENTS_OBJECTIVE_READONLY,
);

/// Represents an argument type parsing a scoreboard objective name.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ObjectiveArgumentType;

impl ArgumentType<CommandSource> for ObjectiveArgumentType {
    type Item = String;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let name = reader.read_unquoted_string();
        Ok(name)
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::Objective
    }

    fn list_suggestions(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> Suggestions {
        let scoreboard = context
            .world()
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for objective_name in scoreboard.get_objectives().keys() {
            builder = builder.filter_and_suggest_one(objective_name.as_str());
        }
        builder.build()
    }

    fn examples(&self) -> Vec<String> {
        vec!["objective".to_string(), "trigger_obj".to_string()]
    }
}

impl ObjectiveArgumentType {
    /// Returns a [`CommandContext`]'s parsed `String` argument as a string slice.
    pub fn get<'a>(context: &'a CommandContext, name: &str) -> Result<&'a str, CommandSyntaxError> {
        Ok(context.get_argument::<String>(name)?.as_str())
    }

    /// Resolves the parsed objective against the scoreboard, like vanilla's
    /// `ObjectiveArgument.getObjective`, and fails when it does not exist.
    pub fn objective_or_error<'a>(
        scoreboard: &'a Scoreboard,
        name: &str,
    ) -> Result<&'a ScoreboardObjective, CommandSyntaxError> {
        scoreboard.get_objective(name).ok_or_else(|| {
            OBJECTIVE_NOT_FOUND_ERROR.create_without_context(TextComponent::text(name.to_string()))
        })
    }

    /// Resolves the objective and rejects criteria that vanilla registers as
    /// read-only, like `ObjectiveArgument.getWritableObjective`.
    pub fn writable_objective_or_error<'a>(
        scoreboard: &'a Scoreboard,
        name: &str,
    ) -> Result<&'a ScoreboardObjective, CommandSyntaxError> {
        let objective = Self::objective_or_error(scoreboard, name)?;
        // These are the six criteria registered as read-only by vanilla ObjectiveCriteria.
        let read_only = matches!(
            objective.criterion.as_str(),
            "health" | "food" | "air" | "armor" | "xp" | "level"
        );
        if read_only {
            return Err(OBJECTIVE_READ_ONLY_ERROR
                .create_without_context(TextComponent::text(name.to_string())));
        }
        Ok(objective)
    }
}
