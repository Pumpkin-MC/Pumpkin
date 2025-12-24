use pumpkin_data::entity::EntityType;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::text::TextComponent;

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

pub struct SummonableEntitiesArgumentConsumer;

impl GetClientSideArgParser for SummonableEntitiesArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::SummonableEntities)
    }
}

impl ArgumentConsumer for SummonableEntitiesArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        Box::pin(async move { s_opt.map(Arg::Block) })
    }
}

impl DefaultNameArgConsumer for SummonableEntitiesArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "summonable_entities"
    }
}

impl<'a> FindArg<'a> for SummonableEntitiesArgumentConsumer {
    type Data = &'static EntityType;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Block(name)) => {
                EntityType::from_name(name.strip_prefix("minecraft:").unwrap_or(name)).map_or_else(
                    || {
                        Err(CommandError::CommandFailed(TextComponent::text(
                            "Can't find Entity",
                        )))
                    },
                    Result::Ok,
                )
            }
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
