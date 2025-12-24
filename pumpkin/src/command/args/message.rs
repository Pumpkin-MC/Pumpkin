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
    Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};

/// Consumes all remaining words/args. Does not consume if there is no word.
pub struct MsgArgConsumer;

impl GetClientSideArgParser for MsgArgConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::String(StringProtoArgBehavior::GreedyPhrase)
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for MsgArgConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let first_word_opt = args.pop();

        let mut msg = match first_word_opt {
            Some(word) => word.to_string(),
            None => return Box::pin(async { None }),
        };

        while let Some(word) = args.pop() {
            msg.push(' ');
            msg.push_str(word);
        }

        Box::pin(async move { Some(Arg::Msg(msg)) })
    }
}

impl DefaultNameArgConsumer for MsgArgConsumer {
    fn default_name(&self) -> &'static str {
        "msg"
    }
}

impl<'a> FindArg<'a> for MsgArgConsumer {
    type Data = &'a str;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Msg(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
