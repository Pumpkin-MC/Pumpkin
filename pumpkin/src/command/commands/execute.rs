use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::message::MsgArgConsumer;
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["execute"];

const DESCRIPTION: &str = "Executes a command with context modifiers and conditions.";

const ARG_COMMAND: &str = "command";
const ARG_TARGETS: &str = "targets";

struct RunExecutor;

impl CommandExecutor for RunExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let Some(Arg::Msg(command)) = args.get(ARG_COMMAND) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_COMMAND.into())));
            };

            let cmd = if command.starts_with('/') {
                command.clone()
            } else {
                format!("/{command}")
            };

            let dispatcher = server.command_dispatcher.read().await;
            dispatcher.fallback_dispatcher.dispatch(sender, server, &cmd).await?;
            Ok(1)
        })
    }
}

struct AsExecutor;

impl CommandExecutor for AsExecutor {
    fn execute<'a>(
        &'a self,
        _sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            let Some(Arg::Msg(command)) = args.get(ARG_COMMAND) else {
                return Err(CommandError::InvalidConsumption(Some(ARG_COMMAND.into())));
            };

            let cmd = if command.starts_with('/') {
                command.clone()
            } else {
                format!("/{command}")
            };

            let dispatcher = server.command_dispatcher.read().await;
            let mut success = 0i32;

            // TODO: /execute as should work on any entity, not just players.
            // Currently limited to players because CommandSender has no entity variant.
            for target in targets {
                let entity = target.get_entity();
                if let Some(player) = entity.world.load().get_player_by_id(entity.entity_id) {
                    let new_sender = CommandSender::Player(player);
                    if dispatcher.fallback_dispatcher.dispatch(&new_sender, server, &cmd).await.is_ok() {
                        success += 1;
                    }
                }
            }

            if success == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_EXECUTE_CONDITIONAL_FAIL,
                    [],
                )));
            }
            Ok(success)
        })
    }
}

// TODO: Use BlockPredicateArgumentConsumer instead of SimpleArgConsumer for block matching.
// TODO: Support chaining multiple subcommands (e.g., `execute if ... if ... run`).
struct IfBlockExecutor {
    inverted: bool,
}

impl CommandExecutor for IfBlockExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        let inverted = self.inverted;
        Box::pin(async move {
            let pos = BlockPosArgumentConsumer::find_arg(args, "pos")?;
            let Some(Arg::Simple(block_name)) = args.get("block") else {
                return Err(CommandError::InvalidConsumption(Some("block".into())));
            };

            let world = sender.world().ok_or(CommandError::InvalidRequirement)?;
            let block = world.get_block(&pos).await;

            let block_name_clean = block_name.strip_prefix("minecraft:").unwrap_or(block_name);
            let matches = block.name == block_name_clean;
            let condition_met = if inverted { !matches } else { matches };

            if !condition_met {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_EXECUTE_CONDITIONAL_FAIL,
                    [],
                )));
            }

            // If there's a run command, execute it; otherwise report pass
            if let Some(Arg::Msg(command)) = args.get(ARG_COMMAND) {
                let cmd = if command.starts_with('/') {
                    command.clone()
                } else {
                    format!("/{command}")
                };
                let dispatcher = server.command_dispatcher.read().await;
                dispatcher.fallback_dispatcher.dispatch(sender, server, &cmd).await?;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_EXECUTE_CONDITIONAL_PASS,
                        [],
                    ))
                    .await;
            }
            Ok(1)
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("run").then(argument(ARG_COMMAND, MsgArgConsumer).execute(RunExecutor)))
        .then(literal("as").then(
            argument(ARG_TARGETS, EntitiesArgumentConsumer).then(
                literal("run").then(argument(ARG_COMMAND, MsgArgConsumer).execute(AsExecutor)),
            ),
        ))
        .then(
            literal("if").then(
                literal("block").then(
                    argument("pos", BlockPosArgumentConsumer).then(
                        argument("block", crate::command::args::simple::SimpleArgConsumer)
                            .then(
                                literal("run").then(
                                    argument(ARG_COMMAND, MsgArgConsumer)
                                        .execute(IfBlockExecutor { inverted: false }),
                                ),
                            )
                            .execute(IfBlockExecutor { inverted: false }),
                    ),
                ),
            ),
        )
        .then(
            literal("unless").then(
                literal("block").then(
                    argument("pos", BlockPosArgumentConsumer).then(
                        argument("block", crate::command::args::simple::SimpleArgConsumer)
                            .then(
                                literal("run").then(
                                    argument(ARG_COMMAND, MsgArgConsumer)
                                        .execute(IfBlockExecutor { inverted: true }),
                                ),
                            )
                            .execute(IfBlockExecutor { inverted: true }),
                    ),
                ),
            ),
        )
}
