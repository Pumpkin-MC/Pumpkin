use pumpkin_data::Advancement;
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

pub struct AdvancementArgumentConsumer;

impl GetClientSideArgParser for AdvancementArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Resource {
            identifier: "advancement",
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for AdvancementArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let name_opt: Option<&'a str> = args.pop();

        let result: Option<Arg<'a>> = name_opt.map_or_else(
            || None,
            |name| Advancement::from_name(name).map(Arg::Advancement),
        );
        Box::pin(async move { result })
    }
}

impl DefaultNameArgConsumer for AdvancementArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "advancement"
    }
}

impl<'a> FindArg<'a> for AdvancementArgumentConsumer {
    type Data = &'static Advancement;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Advancement(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
