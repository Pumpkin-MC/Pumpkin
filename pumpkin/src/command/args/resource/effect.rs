use pumpkin_data::effect::StatusEffect;
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

pub struct EffectTypeArgumentConsumer;

impl GetClientSideArgParser for EffectTypeArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::Resource {
            identifier: "mob_effect",
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for EffectTypeArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let status_effect: Option<&'a str> = args.pop();

        match status_effect {
            Some(name) => Box::pin(async move {
                let status_effect =
                    StatusEffect::from_name(name.strip_prefix("minecraft:").unwrap_or(name))?;
                Some(Arg::Effect(status_effect))
            }),
            None => Box::pin(async move { None }),
        }
    }
}

impl DefaultNameArgConsumer for EffectTypeArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "mob_effect"
    }
}

impl<'a> FindArg<'a> for EffectTypeArgumentConsumer {
    type Data = &'static StatusEffect;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Effect(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
