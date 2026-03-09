use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandExecutor, CommandResult, CommandSender,
    args::{ConsumedArgs, FindArg, players::PlayersArgumentConsumer, simple::SimpleArgConsumer},
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
};

const NAMES: [&str; 1] = ["dialog"];
const DESCRIPTION: &str = "Shows or clears dialogs for players.";
const ARG_TARGETS: &str = "targets";
const ARG_DIALOG: &str = "dialog";

struct ShowExecutor;

impl CommandExecutor for ShowExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let _dialog = SimpleArgConsumer::find_arg(args, ARG_DIALOG)?;

            // TODO: Implement dialog show when dialog packet support is available
            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_DIALOG_SHOW_SINGLE,
                        [TextComponent::text(targets[0].gameprofile.name.clone())],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_DIALOG_SHOW_MULTIPLE,
                        [TextComponent::text(count.to_string())],
                    ))
                    .await;
            }

            Ok(count as i32)
        })
    }
}

struct ClearExecutor;

impl CommandExecutor for ClearExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            // TODO: Implement dialog clear when dialog packet support is available
            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_DIALOG_CLEAR_SINGLE,
                        [TextComponent::text(targets[0].gameprofile.name.clone())],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_DIALOG_CLEAR_MULTIPLE,
                        [TextComponent::text(count.to_string())],
                    ))
                    .await;
            }

            Ok(count as i32)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("show").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer)
                    .then(argument(ARG_DIALOG, SimpleArgConsumer).execute(ShowExecutor)),
            ),
        )
        .then(
            literal("clear")
                .then(argument(ARG_TARGETS, PlayersArgumentConsumer).execute(ClearExecutor)),
        )
}
