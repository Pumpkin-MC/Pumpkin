use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};

use crate::command::CommandSender;
use crate::command::args::ConsumeResult;
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// Entity anchor point for facing commands (eyes or feet).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityAnchor {
    Feet,
    Eyes,
}

impl EntityAnchor {
    /// Returns the name used in commands.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Feet => "feet",
            Self::Eyes => "eyes",
        }
    }
}

/// Argument consumer for entity anchor (eyes/feet).
pub struct EntityAnchorArgumentConsumer;

impl GetClientSideArgParser for EntityAnchorArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::EntityAnchor
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for EntityAnchorArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let Some(anchor_str) = args.pop() else {
            return Box::pin(async move { None });
        };

        let anchor = match anchor_str.to_lowercase().as_str() {
            "feet" => EntityAnchor::Feet,
            "eyes" => EntityAnchor::Eyes,
            _ => return Box::pin(async move { None }),
        };

        Box::pin(async move { Some(Arg::EntityAnchor(anchor)) })
    }
}

impl DefaultNameArgConsumer for EntityAnchorArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "anchor"
    }
}

impl<'a> FindArg<'a> for EntityAnchorArgumentConsumer {
    type Data = EntityAnchor;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::EntityAnchor(anchor)) => Ok(*anchor),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
