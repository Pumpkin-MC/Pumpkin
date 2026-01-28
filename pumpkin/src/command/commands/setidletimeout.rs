use std::sync::atomic::Ordering;

use pumpkin_util::text::TextComponent;

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::{Arg, GetCloned};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::argument;
use crate::command::{CommandExecutor, CommandResult, CommandSender, args::ConsumedArgs};

const NAMES: [&str; 1] = ["setidletimeout"];

const DESCRIPTION: &str = "Sets the time before idle players are kicked from the server.";

const ARG_MINUTES: &str = "minutes";

fn minutes_consumer() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new().min(0).name(ARG_MINUTES)
}

struct SetIdleTimeoutExecutor;

impl CommandExecutor for SetIdleTimeoutExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Num(Ok(minutes))) = args.get_cloned(&ARG_MINUTES) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_MINUTES.into())));
            };

            let crate::command::args::bounded_num::Number::I32(minutes) = minutes else {
                return Err(CommandError::InvalidConsumption(Some(ARG_MINUTES.into())));
            };

            server.player_idle_timeout.store(minutes, Ordering::Relaxed);

            sender
                .send_message(TextComponent::translate(
                    "commands.setidletimeout.success",
                    [TextComponent::text(minutes.to_string())],
                ))
                .await;

            Ok(())
        })
    }
}

#[must_use]
pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_MINUTES, minutes_consumer()).execute(SetIdleTimeoutExecutor))
}
