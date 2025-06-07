use std::str::FromStr;

use async_trait::async_trait;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::Difficulty;

use crate::{
    command::{CommandSender, dispatcher::CommandError, tree::RawArgs},
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

pub struct DifficultyArgumentConsumer;

impl GetClientSideArgParser for DifficultyArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::String(pumpkin_protocol::client::play::StringProtoArgBehavior::SingleWord)
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

#[async_trait]
impl ArgumentConsumer for DifficultyArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        Difficulty::from_str(s)
            .map_or_else(|_| None, |difficulty| Some(Arg::Difficulty(difficulty)))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        let difficulties = ["easy", "normal", "hard", "peaceful"];
        let suggestions: Vec<CommandSuggestion> = difficulties
            .iter()
            .map(|difficulty| CommandSuggestion::new((*difficulty).to_string(), None))
            .collect();
        Ok(Some(suggestions))
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
