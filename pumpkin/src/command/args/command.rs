use pumpkin_protocol::java::client::play::{
    ArgumentType, CommandSuggestion, StringProtoArgBehavior, SuggestionProviders,
};

use crate::{
    command::{
        CommandSender,
        args::{ConsumeResult, SplitSingleWhitespaceIncludingEmptyParts, SuggestResult},
        dispatcher::CommandError,
        tree::{CommandTree, RawArgs},
    },
    server::Server,
};

use super::{Arg, ArgumentConsumer, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

pub struct CommandTreeArgumentConsumer;

impl GetClientSideArgParser for CommandTreeArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType<'_> {
        ArgumentType::String(StringProtoArgBehavior::SingleWord)
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for CommandTreeArgumentConsumer {
    fn consume<'a, 'b>(
        &'a self,
        _sender: &'a CommandSender,
        server: &'a Server,
        args: &'b mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let s_opt: Option<&'a str> = args.pop();

        let Some(s) = s_opt else {
            return Box::pin(async move { None });
        };

        Box::pin(async move {
            let dispatcher = server.command_dispatcher.read().await;

            dispatcher
                .get_tree(s)
                .ok()
                .map(|tree| Arg::CommandTree(tree.clone()))
        })
    }

    fn suggest<'a>(
        &'a self,
        _sender: &CommandSender,
        server: &'a Server,
        input: &'a str,
    ) -> SuggestResult<'a> {
        Box::pin(async move {
            let Some(input) = input.split_single_whitespace_including_empty_parts().last() else {
                return Ok(None);
            };

            let dispatcher = server.command_dispatcher.read().await;
            let suggestions = dispatcher
                .commands
                .keys()
                .filter(|suggestion| suggestion.starts_with(input))
                .map(|suggestion| CommandSuggestion::new(suggestion.clone(), None))
                .collect();
            Ok(Some(suggestions))
        })
    }
}

impl DefaultNameArgConsumer for CommandTreeArgumentConsumer {
    fn default_name(&self) -> &'static str {
        "cmd"
    }
}

impl<'a> FindArg<'a> for CommandTreeArgumentConsumer {
    type Data = &'a CommandTree;

    fn find_arg(args: &'a super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::CommandTree(tree)) => Ok(tree),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}
