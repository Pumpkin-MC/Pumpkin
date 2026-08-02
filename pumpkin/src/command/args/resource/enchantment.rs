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

fn parse_enchantment_name(name: &str) -> Option<&'static Enchantment> {
    Enchantment::from_name(name.strip_prefix("minecraft:").unwrap_or(name))
}

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
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let name_opt: Option<&'a str> = args.pop().map(|arg| arg.value);

        let result: Option<Arg<'a>> = name_opt.map_or_else(
            || None,
            |name| parse_enchantment_name(name).map(Arg::Enchantment),
        );
        Box::pin(async move { result })
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

#[cfg(test)]
mod tests {
    use super::parse_enchantment_name;

    #[test]
    fn accepts_short_and_namespaced_enchantment_names() {
        assert_eq!(
            parse_enchantment_name("flame").map(|enchantment| enchantment.registry_key),
            parse_enchantment_name("minecraft:flame").map(|enchantment| enchantment.registry_key)
        );
    }
}
