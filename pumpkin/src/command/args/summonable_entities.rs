use async_trait::async_trait;
use pumpkin_data::entity::EntityType;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

use crate::{command::dispatcher::CommandError, server::Server};

use super::{
    super::{
        CommandSender,
        args::{ArgumentConsumer, RawArgs},
    },
    Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};

pub(crate) struct SummonableEntitiesArgumentConsumer;

impl GetClientSideArgParser for SummonableEntitiesArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::SummonableEntities)
    }
}

#[async_trait]
impl ArgumentConsumer for SummonableEntitiesArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;
        Some(Arg::Block(s))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for SummonableEntitiesArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "summonable_entities"
    }
}

impl<'a> FindArg<'a> for SummonableEntitiesArgumentConsumer {
    type Data = EntityType;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Block(name)) => EntityType::from_name(&name.replace("minecraft:", ""))
                .map_or_else(
                    || {
                        Err(CommandError::GeneralCommandIssue(format!(
                            "Entity {name} does not exist."
                        )))
                    },
                    Result::Ok,
                ),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
