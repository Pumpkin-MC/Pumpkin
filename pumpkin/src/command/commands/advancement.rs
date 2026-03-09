use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::entities::EntitiesArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{Arg, ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["advancement"];

const DESCRIPTION: &str = "Gives, removes, or checks player advancements.";

const ARG_TARGETS: &str = "targets";
const ARG_ADVANCEMENT: &str = "advancement";

struct GrantEverythingExecutor;

impl CommandExecutor for GrantEverythingExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            if targets.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_ONE_FAILURE,
                    [],
                )));
            }

            // TODO: Implement advancement tracking system and grant all advancements

            let count = targets.len() as i32;
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_ONE_SUCCESS,
                        [
                            TextComponent::text("all".to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_MANY_SUCCESS,
                        [
                            TextComponent::text("all".to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count)
        })
    }
}

struct GrantOnlyExecutor;

impl CommandExecutor for GrantOnlyExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let Some(Arg::Simple(advancement)) = args.get(ARG_ADVANCEMENT) else {
                return Err(CommandError::InvalidConsumption(Some(
                    ARG_ADVANCEMENT.into(),
                )));
            };

            if targets.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_ADVANCEMENT_GRANT_ONE_TO_ONE_FAILURE,
                    [],
                )));
            }

            // TODO: Implement advancement tracking and grant specific advancement

            let count = targets.len() as i32;
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_GRANT_ONE_TO_ONE_SUCCESS,
                        [
                            TextComponent::text(advancement.to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_GRANT_ONE_TO_MANY_SUCCESS,
                        [
                            TextComponent::text(advancement.to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count)
        })
    }
}

struct RevokeEverythingExecutor;

impl CommandExecutor for RevokeEverythingExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            if targets.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_ONE_FAILURE,
                    [],
                )));
            }

            // TODO: Implement advancement tracking and revoke all advancements

            let count = targets.len() as i32;
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_ONE_SUCCESS,
                        [
                            TextComponent::text("all".to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_MANY_SUCCESS,
                        [
                            TextComponent::text("all".to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count)
        })
    }
}

struct RevokeOnlyExecutor;

impl CommandExecutor for RevokeOnlyExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = EntitiesArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let Some(Arg::Simple(advancement)) = args.get(ARG_ADVANCEMENT) else {
                return Err(CommandError::InvalidConsumption(Some(
                    ARG_ADVANCEMENT.into(),
                )));
            };

            if targets.is_empty() {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    translation::COMMANDS_ADVANCEMENT_REVOKE_ONE_TO_ONE_FAILURE,
                    [],
                )));
            }

            // TODO: Implement advancement tracking and revoke specific advancement

            let count = targets.len() as i32;
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_REVOKE_ONE_TO_ONE_SUCCESS,
                        [
                            TextComponent::text(advancement.to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_REVOKE_ONE_TO_MANY_SUCCESS,
                        [
                            TextComponent::text(advancement.to_string()),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count)
        })
    }
}

fn grant_tree() -> crate::command::tree::builder::NonLeafNodeBuilder {
    literal("grant").then(
        argument(ARG_TARGETS, EntitiesArgumentConsumer)
            .then(literal("everything").execute(GrantEverythingExecutor))
            .then(
                literal("only")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(GrantOnlyExecutor)),
            )
            .then(
                literal("from")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(GrantOnlyExecutor)),
            )
            .then(
                literal("through")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(GrantOnlyExecutor)),
            )
            .then(
                literal("until")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(GrantOnlyExecutor)),
            ),
    )
}

fn revoke_tree() -> crate::command::tree::builder::NonLeafNodeBuilder {
    literal("revoke").then(
        argument(ARG_TARGETS, EntitiesArgumentConsumer)
            .then(literal("everything").execute(RevokeEverythingExecutor))
            .then(
                literal("only")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(RevokeOnlyExecutor)),
            )
            .then(
                literal("from")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(RevokeOnlyExecutor)),
            )
            .then(
                literal("through")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(RevokeOnlyExecutor)),
            )
            .then(
                literal("until")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(RevokeOnlyExecutor)),
            ),
    )
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(grant_tree())
        .then(revoke_tree())
}
