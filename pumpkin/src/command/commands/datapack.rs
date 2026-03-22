use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::{
    CommandError, CommandExecutor, CommandResult, CommandSender,
    args::{ConsumedArgs, FindArg, simple::SimpleArgConsumer},
    tree::{
        CommandTree,
        builder::{argument, literal},
    },
};

const NAMES: [&str; 1] = ["datapack"];
const DESCRIPTION: &str = "Controls loaded data packs.";
const ARG_NAME: &str = "name";
const ARG_EXISTING: &str = "existing";

struct ListExecutor {
    mode: ListMode,
}

enum ListMode {
    Available,
    Enabled,
}

impl CommandExecutor for ListExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        _args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            // TODO: Implement datapack listing when datapack management system is available
            match self.mode {
                ListMode::Available => {
                    sender
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_DATAPACK_LIST_AVAILABLE_NONE,
                            [],
                        ))
                        .await;
                }
                ListMode::Enabled => {
                    // The vanilla built-in pack is always enabled
                    sender
                        .send_message(TextComponent::translate(
                            translation::COMMANDS_DATAPACK_LIST_ENABLED_SUCCESS,
                            [TextComponent::text("1")],
                        ))
                        .await;
                }
            }
            Ok(0)
        })
    }
}

struct EnableExecutor;

impl CommandExecutor for EnableExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = SimpleArgConsumer::find_arg(args, ARG_NAME)?;

            // TODO: Implement datapack enable when datapack management system is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_DATAPACK_ENABLE_FAILED,
                    [TextComponent::text(name.to_string())],
                ))
                .await;

            Err(CommandError::CommandFailed(TextComponent::text(format!(
                "Unknown data pack: {name}"
            ))))
        })
    }
}

struct DisableExecutor;

impl CommandExecutor for DisableExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let name = SimpleArgConsumer::find_arg(args, ARG_NAME)?;

            // TODO: Implement datapack disable when datapack management system is available
            sender
                .send_message(TextComponent::translate(
                    translation::COMMANDS_DATAPACK_DISABLE_FAILED,
                    [TextComponent::text(name.to_string())],
                ))
                .await;

            Err(CommandError::CommandFailed(TextComponent::text(format!(
                "Unknown data pack: {name}"
            ))))
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("list")
                .then(literal("available").execute(ListExecutor {
                    mode: ListMode::Available,
                }))
                .then(literal("enabled").execute(ListExecutor {
                    mode: ListMode::Enabled,
                }))
                .execute(ListExecutor {
                    mode: ListMode::Enabled,
                }),
        )
        .then(
            literal("enable").then(
                argument(ARG_NAME, SimpleArgConsumer)
                    // /datapack enable <name> [first|last|before <existing>|after <existing>]
                    .then(literal("first").execute(EnableExecutor))
                    .then(literal("last").execute(EnableExecutor))
                    .then(
                        literal("before").then(
                            argument(ARG_EXISTING, SimpleArgConsumer).execute(EnableExecutor),
                        ),
                    )
                    .then(
                        literal("after").then(
                            argument(ARG_EXISTING, SimpleArgConsumer).execute(EnableExecutor),
                        ),
                    )
                    .execute(EnableExecutor),
            ),
        )
        .then(
            literal("disable").then(argument(ARG_NAME, SimpleArgConsumer).execute(DisableExecutor)),
        )
}
