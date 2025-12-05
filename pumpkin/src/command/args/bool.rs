use crate::command::CommandSender;
use crate::command::args::{Arg, ArgumentConsumer, ConsumeResult, FindArg, GetClientSideArgParser};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};

pub struct BoolArgConsumer;

impl GetClientSideArgParser for BoolArgConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Bool
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for BoolArgConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let result: Option<Arg<'a>> = s_opt.map_or_else(
            || None,
            |s| match s {
                "false" => Some(Arg::Bool(false)),
                "true" => Some(Arg::Bool(true)),
                _ => None,
            },
        );

        Box::pin(async move { result })
    }
}

impl<'a> FindArg<'a> for BoolArgConsumer {
    type Data = bool;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Bool(data)) => Ok(*data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
