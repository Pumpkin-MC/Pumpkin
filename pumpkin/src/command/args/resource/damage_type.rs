use async_trait::async_trait;
use pumpkin_data::damage::DamageType;
use pumpkin_protocol::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};

use crate::command::{
    CommandSender,
    args::{
        Arg, ArgumentConsumer, ConsumedArgs, DefaultNameArgConsumer, FindArg,
        GetClientSideArgParser,
    },
    dispatcher::CommandError,
    tree::RawArgs,
};
use crate::server::Server;

pub struct DamageTypeArgumentConsumer;

impl GetClientSideArgParser for DamageTypeArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Resource {
            identifier: "damage_type",
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for DamageTypeArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let name = args.pop()?;

        // Create a static damage type first
        let damage_type = DamageType::from_name(name)?;
        // Find matching static damage type from values array
        Some(Arg::DamageType(damage_type))
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
