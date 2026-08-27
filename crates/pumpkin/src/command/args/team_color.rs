use pumpkin_data::translation;
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::text::TextComponent;
use pumpkin_util::text::color::NamedColor;

use crate::command::CommandSender;
use crate::command::args::{
    Arg, ArgumentConsumer, ConsumeResult, ConsumeResultWithSyntax, DefaultNameArgConsumer, FindArg,
    GetClientSideArgParser, SuggestResult,
};
use crate::command::dispatcher::CommandError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::tree::RawArgs;
use crate::server::Server;

pub const INVALID_COLOR_ERROR_TYPE: CommandErrorType<1> = CommandErrorType::new(
    translation::java::ARGUMENT_COLOR_INVALID,
    translation::java::ARGUMENT_COLOR_INVALID,
);

const TEAM_COLORS: [&str; 16] = [
    "black",
    "dark_blue",
    "dark_green",
    "dark_aqua",
    "dark_red",
    "dark_purple",
    "gold",
    "gray",
    "dark_gray",
    "blue",
    "green",
    "aqua",
    "red",
    "light_purple",
    "yellow",
    "white",
];

pub struct TeamColorArgumentConsumer;

impl GetClientSideArgParser for TeamColorArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Color
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for TeamColorArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop().map(|arg| arg.value);

        s_opt.and_then(|s| NamedColor::try_from(s).ok().map(Arg::TeamColor))
    }

    fn consume_with_syntax<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResultWithSyntax<'a> {
        let Some(raw_arg) = args.pop() else {
            return Ok(None);
        };

        NamedColor::try_from(raw_arg.value)
            .map(|color| Some(Arg::TeamColor(color)))
            .map_err(|()| {
                INVALID_COLOR_ERROR_TYPE.create_without_context_args_slice(&[TextComponent::text(
                    raw_arg.value.to_string(),
                )])
            })
    }

    fn suggest(&self, _sender: &CommandSender, _server: &Server, input: &str) -> SuggestResult {
        let suggestions: Vec<CommandSuggestion> = TEAM_COLORS
            .iter()
            .filter(|color| color.starts_with(input))
            .map(|color| CommandSuggestion::new((*color).to_string(), None))
            .collect();
        Ok(Some(suggestions))
    }
}

impl DefaultNameArgConsumer for TeamColorArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "color"
    }
}

impl<'a> FindArg<'a> for TeamColorArgumentConsumer {
    type Data = NamedColor;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::TeamColor(color)) => Ok(*color),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_team_color() {
        assert_eq!(NamedColor::try_from("red"), Ok(NamedColor::Red));
        assert_eq!(NamedColor::try_from("dark_blue"), Ok(NamedColor::DarkBlue));
        assert_eq!(NamedColor::try_from("invalid"), Err(()));
    }
}
