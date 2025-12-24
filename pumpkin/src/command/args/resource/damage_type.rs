use pumpkin_data::damage::DamageType;
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

pub struct DamageTypeArgumentConsumer;

impl GetClientSideArgParser for DamageTypeArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Resource {
            identifier: "damage_type",
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for DamageTypeArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let name_opt: Option<&'a str> = args.pop();

        let result: Option<Arg<'a>> = name_opt.map_or_else(
            || None,
            |name| {
                let name = name.strip_prefix("minecraft:").unwrap_or(name);
                DamageType::from_name(name).map(Arg::DamageType)
            },
        );

        Box::pin(async move { result })
    }
}

impl DefaultNameArgConsumer for DamageTypeArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "damage_type"
    }
}

impl<'a> FindArg<'a> for DamageTypeArgumentConsumer {
    type Data = &'a DamageType;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::DamageType(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
