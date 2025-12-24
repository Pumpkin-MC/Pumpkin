use pumpkin_data::item::Item;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::text::TextComponent;

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

pub struct ItemArgumentConsumer;

impl GetClientSideArgParser for ItemArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::ItemStack
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for ItemArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let item = args.pop();
        match item {
            Some(s) => Box::pin(async move { Some(Arg::Item(s)) }),
            None => Box::pin(async move { None }),
        }
    }
}

impl DefaultNameArgConsumer for ItemArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "item"
    }
}

impl<'a> FindArg<'a> for ItemArgumentConsumer {
    type Data = (&'a str, &'static Item);

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Item(name)) => {
                Item::from_registry_key(name.strip_prefix("minecraft:").unwrap_or(name))
                    .map_or_else(
                        || {
                            if name.starts_with("minecraft:") {
                                Err(CommandError::CommandFailed(TextComponent::translate(
                                    "argument.item.id.invalid",
                                    [TextComponent::text((*name).to_string())],
                                )))
                            } else {
                                Err(CommandError::CommandFailed(TextComponent::translate(
                                    "argument.item.id.invalid",
                                    [TextComponent::text("minecraft:".to_string() + *name)],
                                )))
                            }
                        },
                        |item| Ok((*name, item)),
                    )
            }
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
