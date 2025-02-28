use std::str::FromStr;

use async_trait::async_trait;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::GameMode;

use crate::{
    command::{CommandSender, dispatcher::CommandError, tree::RawArgs},
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

pub struct GamemodeArgumentConsumer;

impl GetClientSideArgParser for GamemodeArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Gamemode
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for GamemodeArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        if let Ok(id) = s.parse::<i8>() {
            if let Ok(gamemode) = GameMode::try_from(id) {
                return Some(Arg::GameMode(gamemode));
            }
        };

        GameMode::from_str(s).map_or_else(|_| None, |gamemode| Some(Arg::GameMode(gamemode)))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for GamemodeArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "gamemode"
    }
}

impl<'a> FindArg<'a> for GamemodeArgumentConsumer {
    type Data = GameMode;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::GameMode(data)) => Ok(*data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
