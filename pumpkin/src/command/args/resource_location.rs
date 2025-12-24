use crate::command::CommandSender;
use crate::command::args::{
    Arg, ArgumentConsumer, ConsumeResult, DefaultNameArgConsumer, FindArg, GetClientSideArgParser,
};
use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::server::Server;
use pumpkin_protocol::java::client::play::{ArgumentType, SuggestionProviders};

pub struct ResourceLocationArgumentConsumer {
    autocomplete: bool,
}

impl GetClientSideArgParser for ResourceLocationArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for ResourceLocationArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        if !self.autocomplete {
            return Box::pin(async move { None });
        }
        let s_opt: Option<&'a str> = args.pop();

        Box::pin(async move { s_opt.map(Arg::ResourceLocation) })
    }

    // async fn suggest<'a>(
    //     &'a self,
    //     _sender: &CommandSender,
    //     _server: &'a Server,
    //     _input: &'a str,
    // ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
    //     if !self.autocomplete {
    //         return Ok(None);
    //     }
    //     // TODO

    //     // let suggestions = server
    //     //     .bossbars
    //     //     .lock()
    //     //     .await
    //     //     .custom_bossbars
    //     //     .keys()
    //     //     .map(|suggestion| CommandSuggestion::new(suggestion, None))
    //     //     .collect();

    //     Ok(None)
    // }
}

impl DefaultNameArgConsumer for ResourceLocationArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "id"
    }
}

impl<'a> FindArg<'a> for ResourceLocationArgumentConsumer {
    type Data = &'a str;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::ResourceLocation(data)) => Ok(data),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

impl ResourceLocationArgumentConsumer {
    #[must_use]
    pub const fn new(autocomplete: bool) -> Self {
        Self { autocomplete }
    }
}
