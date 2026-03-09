use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{ConsumedArgs, FindArg, simple::SimpleArgConsumer},
    tree::{CommandTree, builder::argument},
};

const NAMES: [&str; 1] = ["function"];
const DESCRIPTION: &str = "Runs a function.";
const ARG_NAME: &str = "name";

struct Executor;

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = SimpleArgConsumer::find_arg(args, ARG_NAME)?;

            // TODO: Implement function execution when mcfunction parser is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_FUNCTION_SCHEDULED_NO_FUNCTIONS,
                    [],
                ))
                .await;

            Err(CommandError::InvalidConsumption(Some(name.to_string())))
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_NAME, SimpleArgConsumer).execute(Executor))
}
