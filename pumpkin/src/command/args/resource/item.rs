use async_trait::async_trait;
use pumpkin_data::item::Item;
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

pub struct ItemArgumentConsumer;

impl GetClientSideArgParser for ItemArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Resource { identifier: "item" }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

#[async_trait]
impl ArgumentConsumer for ItemArgumentConsumer {
    async fn consume<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        // todo: get an actual item
        Some(Arg::Item(args.pop()?))
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

impl DefaultNameArgConsumer for ItemArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "item"
    }
}

impl<'a> FindArg<'a> for ItemArgumentConsumer {
    type Data = (&'a str, Item);

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Item(name)) => Item::from_name(&name.replace("minecraft:", "")).map_or_else(
                || {
                    Err(CommandError::GeneralCommandIssue(format!(
                        "Item {name} does not exist."
                    )))
                },
                |item| Ok((*name, item)),
            ),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
