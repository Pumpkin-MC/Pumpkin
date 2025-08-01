use std::sync::Arc;

use crate::command::CommandSender;
use crate::command::args::entities::TargetSelector;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::entity::EntityBase;
use crate::server::Server;
use async_trait::async_trait;
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// todo: implement for entities that aren't players
///
/// For selecting a single entity, eg. using @s, a player name or entity uuid.
///
/// Use [`super::arg_entities::EntitiesArgumentConsumer`] when there may be multiple targets.
pub struct EntityArgumentConsumer;

impl GetClientSideArgParser for EntityArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        // todo: investigate why this does not accept target selectors
        ArgumentType::Entity {
            flags: ArgumentType::ENTITY_FLAG_ONLY_SINGLE,
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for EntityArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        src: &CommandSender,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let s = args.pop()?;

        let entity_selector = match s.parse::<TargetSelector>() {
            Ok(selector) => selector,
            Err(e) => {
                log::debug!("Failed to parse target selector '{s}': {e}");
                return None;
            }
        };
        if entity_selector.get_limit() > 1 {
            log::debug!("Target selector '{s}' has limit > 1, expected single entity");
            return None;
        }
        // todo: command context
        let entities = server.select_entities(&entity_selector, Some(src)).await;

        entities.first().map(Arg::Entity)
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl DefaultNameArgConsumer for EntityArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "target"
    }
}

impl<'a> FindArg<'a> for EntityArgumentConsumer {
    type Data = Arc<dyn EntityBase>;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Entity(data)) => Ok(data.clone()),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
