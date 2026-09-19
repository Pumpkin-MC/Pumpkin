use crate::command::{
    CommandSource,
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    argument_types::entity_selector::EntitySelector,
    argument_types::entity_selector::parser::EntitySelectorParser,
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};

/// A scoreboard holder, which is either a fake entry name or an entity selector.
pub enum ScoreHolder {
    Fake(String),
    Selector(Box<EntitySelector>),
    /// Every holder already tracked by the scoreboard, written as `*`.
    Wildcard,
}

/// Parses a scoreboard holder, which is a fake name or an entity selector.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ScoreHolderArgumentType;

impl ArgumentType<CommandSource> for ScoreHolderArgumentType {
    type Item = ScoreHolder;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        if reader.peek() == Some('*') {
            reader.set_cursor(reader.cursor() + 1);
            return Ok(ScoreHolder::Wildcard);
        }
        if reader.peek() == Some('@') {
            let parser = EntitySelectorParser::new(reader, true);
            Ok(ScoreHolder::Selector(Box::new(parser.parse_and_consume()?)))
        } else {
            Ok(ScoreHolder::Fake(reader.read_unquoted_string()))
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ScoreHolder {
            flags: JavaClientArgumentType::SCORE_HOLDER_FLAG_ALLOW_MULTIPLE,
        }
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
        for objective_scores in scoreboard.get_scores().values() {
            for holder in objective_scores.keys() {
                builder = builder.filter_and_suggest_one(holder.as_str());
            }
        }
        builder.build()
    }

    fn examples(&self) -> Vec<String> {
        examples!("detectGen", "Diamond", "@a", "@s")
    }
}

impl ScoreHolderArgumentType {
    /// Resolves a parsed scoreboard holder to the names the scores are stored
    /// under: the fake name itself, or the scoreboard name of every selected
    /// entity.
    pub fn get_score_holder_names(
        context: &CommandContext,
        name: &str,
    ) -> Result<Vec<String>, CommandSyntaxError> {
        match context.get_argument::<ScoreHolder>(name)? {
            ScoreHolder::Fake(fake) => Ok(vec![fake.clone()]),
            ScoreHolder::Selector(selector) => Ok(selector
                .find_entities(context.source.as_ref())?
                .into_iter()
                .map(|entity| entity.get_scoreboard_name())
                .collect()),
            ScoreHolder::Wildcard => {
                let scoreboard = context
                    .world()
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut names: Vec<String> = Vec::new();
                for objective_scores in scoreboard.get_scores().values() {
                    for holder in objective_scores.keys() {
                        if !names.contains(holder) {
                            names.push(holder.clone());
                        }
                    }
                }
                Ok(names)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_names_parse_as_fake_holders() {
        let mut reader = StringReader::new("detectGen");
        let Ok(ScoreHolder::Fake(name)) = ScoreHolderArgumentType.parse(&mut reader) else {
            panic!("a plain name must parse as a fake holder");
        };
        assert_eq!(name, "detectGen");
        assert_eq!(reader.remaining_length(), 0);
    }

    #[test]
    fn a_star_parses_as_the_wildcard() {
        let mut reader = StringReader::new("*");
        assert!(matches!(
            ScoreHolderArgumentType.parse(&mut reader),
            Ok(ScoreHolder::Wildcard)
        ));
        assert_eq!(reader.remaining_length(), 0);
    }

    #[test]
    fn selectors_parse_as_selectors() {
        let mut reader = StringReader::new("@s");
        assert!(matches!(
            ScoreHolderArgumentType.parse(&mut reader),
            Ok(ScoreHolder::Selector(_))
        ));
    }
}
