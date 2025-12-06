use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{
        Arg, ConsumedArgs, FindArg, players::PlayersArgumentConsumer,
        textcomponent::TextComponentArgConsumer,
    },
    tree::{CommandTree, builder::argument},
};

const NAMES: [&str; 1] = ["tellraw"];

const DESCRIPTION: &str = "Send raw message to players.";

const ARG_TARGETS: &str = "targets";

const ARG_MESSAGE: &str = "message";

struct TellRawExecutor;

impl CommandExecutor for TellRawExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_TARGETS.into())));
            };

            let text = TextComponentArgConsumer::find_arg(args, ARG_MESSAGE)?;
            for target in targets {
                target.send_system_message(&text).await;
            }
            Ok(())
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_TARGETS, PlayersArgumentConsumer)
            .then(argument(ARG_MESSAGE, TextComponentArgConsumer).execute(TellRawExecutor)),
    )
}
