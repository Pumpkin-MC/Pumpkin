use std::str::FromStr;

use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::GameMode;

use crate::{
    command::{CommandSender, args::ConsumeResult, dispatcher::CommandError, tree::RawArgs},
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

pub struct GamemodeArgumentConsumer;

impl GetClientSideArgParser for GamemodeArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Gamemode
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for GamemodeArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let result: Option<Arg<'a>> = s_opt.and_then(|s| {
            if let Ok(id) = s.parse::<i8>()
                && let Ok(gamemode) = GameMode::try_from(id)
            {
                return Some(Arg::GameMode(gamemode));
            }

            GameMode::from_str(s).map(Arg::GameMode).ok() // Convert Result to Option<T>
        });

        Box::pin(async move { result })
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
