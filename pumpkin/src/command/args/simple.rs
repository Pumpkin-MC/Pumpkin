use pumpkin_protocol::java::client::play::{
    ArgumentType, StringProtoArgBehavior, SuggestionProviders,
};

use crate::{
    command::{args::ConsumeResult, dispatcher::CommandError},
    server::Server,
};

use super::{
    super::{
        CommandSender,
        args::{ArgumentConsumer, RawArgs},
    },
    Arg, FindArg, GetClientSideArgParser,
};

/// Should never be a permanent solution
#[allow(unused)]
pub struct SimpleArgConsumer;

impl GetClientSideArgParser for SimpleArgConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::String(StringProtoArgBehavior::SingleWord)
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for SimpleArgConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        Box::pin(async move { s_opt.map(Arg::Simple) })
    }
}

impl<'a> FindArg<'a> for SimpleArgConsumer {
    type Data = &'a str;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Simple(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
