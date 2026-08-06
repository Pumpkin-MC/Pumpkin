use crate::command::{
    argument_types::argument_type::{ArgumentType, JavaClientArgumentType},
    context::command_context::CommandContext,
    errors::command_syntax_error::CommandSyntaxError,
    errors::error_types::CommandErrorType,
    string_reader::StringReader,
    suggestion::suggestions::{Suggestions, SuggestionsBuilder},
};
use pumpkin_data::scoreboard::ScoreboardDisplaySlot;
use pumpkin_util::text::TextComponent;
use std::pin::Pin;

pub const UNKNOWN_DISPLAY_SLOT_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    pumpkin_data::translation::java::ARGUMENT_SCOREBOARDDISPLAYSLOT_INVALID,
    pumpkin_data::translation::java::ARGUMENT_SCOREBOARDDISPLAYSLOT_INVALID,
);

pub struct ScoreboardDisplaySlotArgumentType;

impl ArgumentType for ScoreboardDisplaySlotArgumentType {
    type Item = ScoreboardDisplaySlot;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        reader.read_unquoted_string();
        let name = &reader.string()[start..reader.cursor()];

        match name {
            "list" => Ok(ScoreboardDisplaySlot::List),
            "sidebar" => Ok(ScoreboardDisplaySlot::Sidebar),
            "belowName" | "below_name" => Ok(ScoreboardDisplaySlot::BelowName),
            "sidebar.team.black" => Ok(ScoreboardDisplaySlot::TeamBlack),
            "sidebar.team.dark_blue" => Ok(ScoreboardDisplaySlot::TeamDarkBlue),
            "sidebar.team.dark_green" => Ok(ScoreboardDisplaySlot::TeamDarkGreen),
            "sidebar.team.dark_aqua" => Ok(ScoreboardDisplaySlot::TeamDarkAqua),
            "sidebar.team.dark_red" => Ok(ScoreboardDisplaySlot::TeamDarkRed),
            "sidebar.team.dark_purple" => Ok(ScoreboardDisplaySlot::TeamDarkPurple),
            "sidebar.team.gold" => Ok(ScoreboardDisplaySlot::TeamGold),
            "sidebar.team.gray" => Ok(ScoreboardDisplaySlot::TeamGray),
            "sidebar.team.dark_gray" => Ok(ScoreboardDisplaySlot::TeamDarkGray),
            "sidebar.team.blue" => Ok(ScoreboardDisplaySlot::TeamBlue),
            "sidebar.team.green" => Ok(ScoreboardDisplaySlot::TeamGreen),
            "sidebar.team.aqua" => Ok(ScoreboardDisplaySlot::TeamAqua),
            "sidebar.team.red" => Ok(ScoreboardDisplaySlot::TeamRed),
            "sidebar.team.light_purple" => Ok(ScoreboardDisplaySlot::TeamLightPurple),
            "sidebar.team.yellow" => Ok(ScoreboardDisplaySlot::TeamYellow),
            "sidebar.team.white" => Ok(ScoreboardDisplaySlot::TeamWhite),
            _ => Err(UNKNOWN_DISPLAY_SLOT_ERROR_TYPE
                .create(reader, TextComponent::text(name.to_string()))),
        }
    }

    fn client_side_parser(&'_ self) -> JavaClientArgumentType {
        JavaClientArgumentType::ScoreboardSlot
    }

    fn list_suggestions<'a>(
        &'a self,
        _context: &'a CommandContext,
        builder: SuggestionsBuilder,
    ) -> Pin<Box<dyn Future<Output = Suggestions> + Send + 'a>> {
        Box::pin(async move {
            builder
                .filter_and_suggest(&[
                    "list",
                    "sidebar",
                    "belowName",
                    "sidebar.team.black",
                    "sidebar.team.dark_blue",
                    "sidebar.team.dark_green",
                    "sidebar.team.dark_aqua",
                    "sidebar.team.dark_red",
                    "sidebar.team.dark_purple",
                    "sidebar.team.gold",
                    "sidebar.team.gray",
                    "sidebar.team.dark_gray",
                    "sidebar.team.blue",
                    "sidebar.team.green",
                    "sidebar.team.aqua",
                    "sidebar.team.red",
                    "sidebar.team.light_purple",
                    "sidebar.team.yellow",
                    "sidebar.team.white",
                ])
                .build()
        })
    }

    fn examples(&self) -> Vec<String> {
        examples!("list", "sidebar", "belowName")
    }
}

impl ScoreboardDisplaySlotArgumentType {
    /// Returns a [`CommandContext`]'s parsed `ScoreboardDisplaySlot` argument.
    pub fn get(
        context: &crate::command::context::command_context::CommandContext,
        name: &str,
    ) -> Result<
        ScoreboardDisplaySlot,
        crate::command::errors::command_syntax_error::CommandSyntaxError,
    > {
        Ok(*context.get_argument::<ScoreboardDisplaySlot>(name)?)
    }
}
