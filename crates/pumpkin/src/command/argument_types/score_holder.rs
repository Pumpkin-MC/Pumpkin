use crate::command::{
    CommandSource,
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    argument_types::entity::{
        ENTITY_SELECTOR_PERMISSION, NO_ENTITIES_ERROR_TYPE, NOT_SINGLE_ENTITY_ERROR_TYPE,
    },
    argument_types::entity_selector::EntitySelector,
    argument_types::entity_selector::parser::EntitySelectorParser,
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    errors::error_types::CommandErrorType,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};
use crate::entity::EntityBase;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;
use rustc_hash::FxHashSet;
use uuid::Uuid;

const NO_WILDCARD_RESULTS: CommandErrorType<0> = CommandErrorType::new(
    translation::java::ARGUMENT_SCOREHOLDER_EMPTY,
    translation::java::ARGUMENT_SCOREHOLDER_EMPTY,
);

/// A parsed score-holder name, entity selector, or wildcard.
pub enum ScoreHolder {
    Name(String),
    Selector(Box<EntitySelector>),
    /// Every holder already tracked by the scoreboard, written as `*`.
    Wildcard,
}

/// A resolved holder's storage key and its user-facing name.
pub struct ResolvedScoreHolder {
    pub name: String,
    pub display_name: TextComponent,
}

impl ResolvedScoreHolder {
    fn named(name: String) -> Self {
        Self {
            display_name: TextComponent::text(name.clone()),
            name,
        }
    }

    fn entity(entity: &dyn EntityBase) -> Self {
        Self {
            name: entity.get_scoreboard_name(),
            display_name: entity.get_display_name(),
        }
    }

    /// Named online players and UUIDs retain entity feedback; offline names stay literal.
    fn resolve_name(source: &CommandSource, name: &str) -> Vec<Self> {
        if !name.starts_with('#')
            && let Some(server) = source.server.as_ref()
        {
            if let Ok(uuid) = Uuid::parse_str(name) {
                let entities: Vec<Self> = server
                    .worlds
                    .load()
                    .iter()
                    .filter_map(|world| {
                        world
                            .get_entity_by_uuid(uuid)
                            .map(|entity| Self::entity(entity.as_ref()))
                            .or_else(|| {
                                world
                                    .get_player_by_uuid(uuid)
                                    .map(|player| Self::entity(player.as_ref()))
                            })
                    })
                    .collect();
                if !entities.is_empty() {
                    return entities;
                }
            } else if let Some(player) = server.get_player_by_name(name) {
                return vec![Self::entity(player.as_ref())];
            }
        }
        vec![Self::named(name.to_string())]
    }
}

/// Parses one or multiple scoreboard holders, including fake names and selectors.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ScoreHolderArgumentType {
    Single,
    Multiple,
}

impl ArgumentType<CommandSource> for ScoreHolderArgumentType {
    type Item = ScoreHolder;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        self.parse_with_allow_selectors(reader, true)
    }

    fn parse_with_source(
        &self,
        reader: &mut StringReader,
        source: &CommandSource,
    ) -> Result<Self::Item, CommandSyntaxError> {
        self.parse_with_allow_selectors(reader, source.has_permission(ENTITY_SELECTOR_PERMISSION))
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ScoreHolder {
            flags: if *self == Self::Multiple {
                JavaClientArgumentType::SCORE_HOLDER_FLAG_ALLOW_MULTIPLE
            } else {
                0
            },
        }
    }

    fn list_suggestions(
        &self,
        context: &CommandContext,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        let mut reader = StringReader::new(builder.input.clone());
        reader.set_cursor(builder.start);
        let mut parser = EntitySelectorParser::new(
            &mut reader,
            context.source.has_permission(ENTITY_SELECTOR_PERMISSION),
        );
        let _ = parser.parse();
        parser.fill_suggestions(&builder, |mut suggestions| {
            for player in context.server().get_all_players() {
                suggestions = suggestions.filter_and_suggest_one(player.gameprofile.name.clone());
            }
            suggestions
        })
    }

    fn examples(&self) -> Vec<String> {
        examples!("detectGen", "#temp", "$x", "*", "@s")
    }
}

impl ScoreHolderArgumentType {
    fn parse_with_allow_selectors(
        self,
        reader: &mut StringReader,
        allow_selectors: bool,
    ) -> Result<ScoreHolder, CommandSyntaxError> {
        let start = reader.cursor();
        if reader.peek() == Some('@') {
            let selector =
                EntitySelectorParser::new(reader, allow_selectors).parse_and_consume()?;
            if self == Self::Single && selector.max_selected > 1 {
                return Err(NOT_SINGLE_ENTITY_ERROR_TYPE.create(reader));
            }
            return Ok(ScoreHolder::Selector(Box::new(selector)));
        }

        while reader.peek().is_some_and(|c| c != ' ') {
            reader.skip();
        }
        let name = &reader.string()[start..reader.cursor()];
        match name {
            "" => Err(NO_ENTITIES_ERROR_TYPE.create(reader)),
            "*" => Ok(ScoreHolder::Wildcard),
            _ => Ok(ScoreHolder::Name(name.to_string())),
        }
    }

    /// Resolves holders without losing entity display names used in feedback.
    pub fn get_score_holders(
        context: &CommandContext,
        name: &str,
    ) -> Result<Vec<ResolvedScoreHolder>, CommandSyntaxError> {
        let holders = match context.get_argument::<ScoreHolder>(name)? {
            ScoreHolder::Name(name) => {
                ResolvedScoreHolder::resolve_name(context.source.as_ref(), name)
            }
            ScoreHolder::Selector(selector) => selector
                .find_entities(context.source.as_ref())?
                .into_iter()
                .map(|entity| ResolvedScoreHolder::entity(entity.as_ref()))
                .collect(),
            ScoreHolder::Wildcard => {
                let scoreboard = context
                    .world()
                    .scoreboard
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let mut names: Vec<String> = Vec::new();
                let mut seen = FxHashSet::default();
                for objective_scores in scoreboard.get_scores().values() {
                    for holder in objective_scores.keys() {
                        if seen.insert(holder) {
                            names.push(holder.clone());
                        }
                    }
                }
                if names.is_empty() {
                    return Err(NO_WILDCARD_RESULTS.create_without_context());
                }
                names.into_iter().map(ResolvedScoreHolder::named).collect()
            }
        };
        if holders.is_empty() {
            return Err(NO_ENTITIES_ERROR_TYPE.create_without_context());
        }
        Ok(holders)
    }

    /// Resolves a single score holder; wildcards are not accepted.
    pub fn get_score_holder(
        context: &CommandContext,
        name: &str,
    ) -> Result<ResolvedScoreHolder, CommandSyntaxError> {
        if matches!(
            context.get_argument::<ScoreHolder>(name)?,
            ScoreHolder::Wildcard
        ) {
            return Err(NO_WILDCARD_RESULTS.create_without_context());
        }
        let mut holders = Self::get_score_holders(context, name)?;
        Ok(holders.remove(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_names_parse_as_names() {
        for name in [
            "detectGen",
            "#temp",
            "$x",
            "map:counter",
            "*suffix",
            "日本語",
        ] {
            let input = format!("{name} objective");
            let mut reader = StringReader::new(&input);
            let Ok(ScoreHolder::Name(actual)) =
                ScoreHolderArgumentType::Multiple.parse(&mut reader)
            else {
                panic!("{name} must parse as a holder name");
            };
            assert_eq!(actual, name);
            assert_eq!(reader.remaining_part(), " objective");
        }
    }

    #[test]
    fn a_star_parses_as_the_wildcard() {
        let mut reader = StringReader::new("*");
        assert!(matches!(
            ScoreHolderArgumentType::Multiple.parse(&mut reader),
            Ok(ScoreHolder::Wildcard)
        ));
        assert_eq!(reader.remaining_length(), 0);
    }

    #[test]
    fn selectors_parse_as_selectors() {
        let mut reader = StringReader::new("@s");
        assert!(matches!(
            ScoreHolderArgumentType::Multiple.parse(&mut reader),
            Ok(ScoreHolder::Selector(_))
        ));
    }

    #[test]
    fn single_holder_rejects_selectors_that_can_select_multiple_entities() {
        for selector in ["@a", "@e", "@p[limit=2]"] {
            let mut reader = StringReader::new(selector);
            let error = ScoreHolderArgumentType::Single
                .parse(&mut reader)
                .err()
                .unwrap();
            assert!(error.is(&NOT_SINGLE_ENTITY_ERROR_TYPE));
            assert_eq!(reader.cursor(), selector.len());
        }
        for selector in ["@s", "@p", "@e[limit=1]", "#temp"] {
            assert!(
                ScoreHolderArgumentType::Single
                    .parse(&mut StringReader::new(selector))
                    .is_ok()
            );
        }
        assert!(matches!(
            ScoreHolderArgumentType::Single.client_side_parser(),
            JavaClientArgumentType::ScoreHolder { flags: 0 }
        ));
    }

    #[test]
    fn empty_input_is_not_a_holder_name() {
        for input in ["", " objective"] {
            assert!(
                ScoreHolderArgumentType::Multiple
                    .parse(&mut StringReader::new(input))
                    .is_err()
            );
        }
    }
}
