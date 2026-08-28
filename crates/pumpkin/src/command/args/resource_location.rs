use crate::command::CommandSender;
use crate::command::args::{
    Arg, ArgumentConsumer, ConsumeResult, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::identifier::Identifier;

// TODO: Add proper autocomplete
pub struct ResourceLocationArgumentConsumer;

impl GetClientSideArgParser for ResourceLocationArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for ResourceLocationArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let identifier = args.pop().and_then(|arg| Identifier::parse(arg.value).ok());

        identifier.map(Arg::ResourceLocation)
    }
}

impl DefaultNameArgConsumer for ResourceLocationArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "id"
    }
}

impl<'a> FindArg<'a> for ResourceLocationArgumentConsumer {
    type Data = &'a Identifier;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::ResourceLocation(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
