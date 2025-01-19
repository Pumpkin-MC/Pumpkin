use std::{net::IpAddr, str::FromStr};

use async_trait::async_trait;
use pumpkin_protocol::client::play::{
    CommandSuggestion, ProtoCmdArgParser, ProtoCmdArgSuggestionType,
};

use crate::{command::dispatcher::CommandError, server::Server};

use super::{
    super::{
        args::{ArgumentConsumer, RawArgs},
        CommandSender,
    },
    Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};

/// Consumes all remaining words/args. Does not consume if there is no word.
pub(crate) struct IpConsumer;

impl GetClientSideArgParser for IpConsumer {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::GameProfile
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<ProtoCmdArgSuggestionType> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for IpConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let profile = args.pop()?;

        let ip = match IpAddr::from_str(profile) {
            Ok(ip) => ip,
            Err(_) => server
                .get_player_by_name(profile)
                .await?
                .client
                .address
                .lock()
                .await
                .ip(),
        };

        Some(Arg::Ip(ip))
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

impl DefaultNameArgConsumer for IpConsumer {
    fn default_name(&self) -> String {
        "ip".to_string()
    }
}

impl<'a> FindArg<'a> for IpConsumer {
    type Data = &'a str;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Msg(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
