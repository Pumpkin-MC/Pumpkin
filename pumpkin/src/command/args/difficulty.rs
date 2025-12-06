use std::str::FromStr;

use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::Difficulty;

use crate::{
    command::{
        CommandSender,
        args::{ConsumeResult, SuggestResult},
        dispatcher::CommandError,
        tree::RawArgs,
    },
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

pub struct DifficultyArgumentConsumer;

impl GetClientSideArgParser for DifficultyArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::String(
            pumpkin_protocol::java::client::play::StringProtoArgBehavior::SingleWord,
        )
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for DifficultyArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let result: Option<Arg<'a>> =
            s_opt.and_then(|s| Difficulty::from_str(s).map(Arg::Difficulty).ok());

        Box::pin(async move { result })
    }

    fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> SuggestResult<'a> {
        Box::pin(async move {
            let difficulties = ["easy", "normal", "hard", "peaceful"];
            let suggestions: Vec<CommandSuggestion> = difficulties
                .iter()
                .map(|difficulty| CommandSuggestion::new((*difficulty).to_string(), None))
                .collect();
            Ok(Some(suggestions))
        })
    }
}

impl DefaultNameArgConsumer for DifficultyArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "difficulty"
    }
}

impl<'a> FindArg<'a> for DifficultyArgumentConsumer {
    type Data = Difficulty;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Difficulty(data)) => Ok(*data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
