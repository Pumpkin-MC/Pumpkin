use std::sync::Arc;

use crate::command::CommandSender;
use crate::command::args::entities::TargetSelector;
use crate::command::args::{ConsumeResult, ConsumeResultWithSyntax};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::entity::EntityBase;
use crate::server::Server;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use tracing::debug;

use super::super::args::ArgumentConsumer;
use super::entities::parse_target_selector_with_context;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// todo: implement for entities that aren't players
///
/// For selecting a single entity, eg. using @s, a player name or entity uuid.
///
/// Use [`super::entities::EntitiesArgumentConsumer`] when there may be multiple targets.
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

impl ArgumentConsumer for EntityArgumentConsumer {
    fn consume<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s = args.pop().map(|arg| arg.value)?;

        let entity_selector = match s.parse::<TargetSelector>() {
            Ok(selector) => selector,
            Err(e) => {
                debug!("Failed to parse target selector '{s}': {e}");
                return None;
            }
        };

        if entity_selector.get_limit() > 1 {
            debug!("Target selector '{s}' has limit > 1, expected single entity");
            return None;
        }

        // todo: command context
        let entities = server.select_entities(&entity_selector, Some(sender));

        entities.into_iter().next().map(Arg::Entity)
    }

    fn consume_with_syntax<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResultWithSyntax<'a> {
        let Some(raw_arg) = args.pop() else {
            return Ok(None);
        };

        let selector = parse_target_selector_with_context(raw_arg)?;

        if selector.get_limit() > 1 {
            return Ok(None);
        }

        let entities = server.select_entities(&selector, Some(sender));
        Ok(entities.into_iter().next().map(Arg::Entity))
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
