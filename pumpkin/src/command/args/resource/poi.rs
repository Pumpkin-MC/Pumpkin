use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};

use crate::command::{
    CommandSender,
    args::{
        Arg, ArgumentConsumer, ConsumeResult, ConsumedArgs, DefaultNameArgConsumer, FindArg,
        GetClientSideArgParser,
    },
    dispatcher::CommandError,
    tree::RawArgs,
};
use crate::server::Server;

pub struct PoiArgumentConsumer;

impl GetClientSideArgParser for PoiArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::ResourceOrTag {
            identifier: "minecraft:point_of_interest_type",
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for PoiArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let name_opt = args.pop().map(|arg| arg.value);
        Box::pin(async move { name_opt.map(Arg::ResourceLocation) })
    }
}

impl DefaultNameArgConsumer for PoiArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "poi"
    }
}

impl<'a> FindArg<'a> for PoiArgumentConsumer {
    type Data = &'a str;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::ResourceLocation(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
