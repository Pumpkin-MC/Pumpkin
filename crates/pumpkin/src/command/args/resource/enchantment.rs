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
use pumpkin_data::Enchantment;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};
use pumpkin_util::identifier::Identifier;

pub struct EnchantmentArgumentConsumer;

impl GetClientSideArgParser for EnchantmentArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        ArgumentType::Resource {
            identifier: Identifier::vanilla_static("enchantment"),
        }
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        None
    }
}

impl ArgumentConsumer for EnchantmentArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let name = args.pop().map(|arg| arg.value)?;
        Enchantment::from_name(name).map(Arg::Enchantment)
    }
}

impl DefaultNameArgConsumer for EnchantmentArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "enchantment"
    }
}

impl<'a> FindArg<'a> for EnchantmentArgumentConsumer {
    type Data = &'static Enchantment;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Enchantment(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
