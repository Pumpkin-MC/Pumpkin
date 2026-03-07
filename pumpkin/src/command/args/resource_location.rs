use crate::command::CommandSender;
use crate::command::args::{
    Arg, ArgumentConsumer, ConsumeResult, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::identifier::{Identifier, IdentifierCreationResult};

// TODO: Add proper autocomplete
pub struct IdentifierArgumentConsumer;

impl GetClientSideArgParser for IdentifierArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Identifier
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for IdentifierArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        Box::pin(async move { s_opt.map(Identifier::parse).map(Arg::Identifier) })
    }

    // async fn suggest<'a>(
    //     &'a self,
    //     _sender: &CommandSender,
    //     _server: &'a Server,
    //     _input: &'a str,
    // ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
    //     if !self.autocomplete {
    //         return Ok(None);
    //     }
    //     // TODO

    //     // let suggestions = server
    //     //     .bossbars
    //     //     .lock()
    //     //     .await
    //     //     .custom_bossbars
    //     //     .keys()
    //     //     .map(|suggestion| CommandSuggestion::new(suggestion, None))
    //     //     .collect();

    //     Ok(None)
    // }
}

impl DefaultNameArgConsumer for IdentifierArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "id"
    }
}

impl<'a> FindArg<'a> for IdentifierArgumentConsumer {
    type Data = &'a IdentifierCreationResult;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Identifier(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
