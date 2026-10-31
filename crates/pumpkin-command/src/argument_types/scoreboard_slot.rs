use crate::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::context::command_context::CommandContext;
use crate::errors::command_syntax_error::CommandSyntaxError;
use crate::errors::error_types::CommandErrorType;
use crate::string_reader::StringReader;
use crate::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

pub const INVALID_VALUE_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_SCOREBOARDDISPLAYSLOT_INVALID,
    translation::java::ARGUMENT_SCOREBOARDDISPLAYSLOT_INVALID,
);

/// Every display slot, in vanilla's `DisplaySlot` declaration order.
const VALUES: [ScoreboardDisplaySlot; 19] = [
    ScoreboardDisplaySlot::List,
    ScoreboardDisplaySlot::Sidebar,
    ScoreboardDisplaySlot::BelowName,
    ScoreboardDisplaySlot::TeamBlack,
    ScoreboardDisplaySlot::TeamDarkBlue,
    ScoreboardDisplaySlot::TeamDarkGreen,
    ScoreboardDisplaySlot::TeamDarkAqua,
    ScoreboardDisplaySlot::TeamDarkRed,
    ScoreboardDisplaySlot::TeamDarkPurple,
    ScoreboardDisplaySlot::TeamGold,
    ScoreboardDisplaySlot::TeamGray,
    ScoreboardDisplaySlot::TeamDarkGray,
    ScoreboardDisplaySlot::TeamBlue,
    ScoreboardDisplaySlot::TeamGreen,
    ScoreboardDisplaySlot::TeamAqua,
    ScoreboardDisplaySlot::TeamRed,
    ScoreboardDisplaySlot::TeamLightPurple,
    ScoreboardDisplaySlot::TeamYellow,
    ScoreboardDisplaySlot::TeamWhite,
];

/// Represents an argument type parsing a scoreboard display slot name.
pub struct ScoreboardSlotArgumentType;

impl<S: crate::source::CommandSource> ArgumentType<S> for ScoreboardSlotArgumentType {
    type Item = ScoreboardDisplaySlot;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let string = reader.read_unquoted_string();
        VALUES
            .into_iter()
            .find(|slot| Self::serialized_name(*slot) == string.as_str())
            .ok_or_else(|| INVALID_VALUE_ERROR_TYPE.create(reader, TextComponent::text(string)))
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ScoreboardSlot
    }

    fn list_suggestions(
        &self,
        _context: &CommandContext<S>,
        builder: SuggestionsBuilder,
    ) -> Suggestions {
        builder
            .filter_and_suggest_iter(VALUES.map(Self::serialized_name))
            .build()
    }

    fn examples(&self) -> Vec<String> {
        examples!("sidebar", "foo.bar")
    }
}

impl ScoreboardSlotArgumentType {
    /// The name vanilla's `DisplaySlot` serializes to.
    #[must_use]
    pub const fn serialized_name(slot: ScoreboardDisplaySlot) -> &'static str {
        match slot {
            ScoreboardDisplaySlot::List => "list",
            ScoreboardDisplaySlot::Sidebar => "sidebar",
            ScoreboardDisplaySlot::BelowName => "below_name",
            ScoreboardDisplaySlot::TeamBlack => "sidebar.team.black",
            ScoreboardDisplaySlot::TeamDarkBlue => "sidebar.team.dark_blue",
            ScoreboardDisplaySlot::TeamDarkGreen => "sidebar.team.dark_green",
            ScoreboardDisplaySlot::TeamDarkAqua => "sidebar.team.dark_aqua",
            ScoreboardDisplaySlot::TeamDarkRed => "sidebar.team.dark_red",
            ScoreboardDisplaySlot::TeamDarkPurple => "sidebar.team.dark_purple",
            ScoreboardDisplaySlot::TeamGold => "sidebar.team.gold",
            ScoreboardDisplaySlot::TeamGray => "sidebar.team.gray",
            ScoreboardDisplaySlot::TeamDarkGray => "sidebar.team.dark_gray",
            ScoreboardDisplaySlot::TeamBlue => "sidebar.team.blue",
            ScoreboardDisplaySlot::TeamGreen => "sidebar.team.green",
            ScoreboardDisplaySlot::TeamAqua => "sidebar.team.aqua",
            ScoreboardDisplaySlot::TeamRed => "sidebar.team.red",
            ScoreboardDisplaySlot::TeamLightPurple => "sidebar.team.light_purple",
            ScoreboardDisplaySlot::TeamYellow => "sidebar.team.yellow",
            ScoreboardDisplaySlot::TeamWhite => "sidebar.team.white",
        }
    }
}

impl_copy_get!(ScoreboardSlotArgumentType, ScoreboardDisplaySlot);

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_vanilla_slot_names_only() {
        let mut reader = StringReader::new("sidebar.team.dark_blue");
        assert_parse_ok_reset!(
            &mut reader,
            ScoreboardSlotArgumentType,
            ScoreboardDisplaySlot::TeamDarkBlue
        );

        let mut reader = StringReader::new("below_name");
        assert_parse_ok_reset!(
            &mut reader,
            ScoreboardSlotArgumentType,
            ScoreboardDisplaySlot::BelowName
        );

        // The Bedrock/internal spelling is not a vanilla slot name.
        let mut reader = StringReader::new("belowname");
        assert_parse_err_reset!(
            &mut reader,
            ScoreboardSlotArgumentType,
            &INVALID_VALUE_ERROR_TYPE
        );
    }
}
