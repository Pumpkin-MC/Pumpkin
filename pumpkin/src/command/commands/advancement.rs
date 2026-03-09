use pumpkin_data::translation;
use pumpkin_util::text::TextComponent;

use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::simple::SimpleArgConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};

const NAMES: [&str; 1] = ["advancement"];

const DESCRIPTION: &str = "Gives, removes, or checks player advancements.";

const ARG_TARGETS: &str = "targets";
const ARG_ADVANCEMENT: &str = "advancement";
const ARG_CRITERION: &str = "criterion";

struct GrantEverythingExecutor;

impl CommandExecutor for GrantEverythingExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            // TODO: Implement advancement tracking system and grant all advancements
            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_ONE_SUCCESS,
                        [
                            TextComponent::text("0"),
                            TextComponent::text(targets[0].gameprofile.name.clone()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_MANY_SUCCESS,
                        [
                            TextComponent::text("0"),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count as i32)
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
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let advancement = SimpleArgConsumer::find_arg(args, ARG_ADVANCEMENT)?;

            // TODO: Implement advancement tracking and grant specific advancement
            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_GRANT_ONE_TO_ONE_SUCCESS,
                        [
                            TextComponent::text(advancement.to_string()),
                            TextComponent::text(targets[0].gameprofile.name.clone()),
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
            Ok(count as i32)
        })
    }
}

struct GrantTreeExecutor;

impl CommandExecutor for GrantTreeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let _advancement = SimpleArgConsumer::find_arg(args, ARG_ADVANCEMENT)?;

            // TODO: Implement from/through/until advancement tree traversal
            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_ONE_SUCCESS,
                        [
                            TextComponent::text("0"),
                            TextComponent::text(targets[0].gameprofile.name.clone()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_GRANT_MANY_TO_MANY_SUCCESS,
                        [
                            TextComponent::text("0"),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count as i32)
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
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            // TODO: Implement advancement tracking and revoke all advancements
            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_ONE_SUCCESS,
                        [
                            TextComponent::text("0"),
                            TextComponent::text(targets[0].gameprofile.name.clone()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_MANY_SUCCESS,
                        [
                            TextComponent::text("0"),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count as i32)
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
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let advancement = SimpleArgConsumer::find_arg(args, ARG_ADVANCEMENT)?;

            // TODO: Implement advancement tracking and revoke specific advancement
            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_REVOKE_ONE_TO_ONE_SUCCESS,
                        [
                            TextComponent::text(advancement.to_string()),
                            TextComponent::text(targets[0].gameprofile.name.clone()),
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
            Ok(count as i32)
        })
    }
}

struct RevokeTreeExecutor;

impl CommandExecutor for RevokeTreeExecutor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;
            let _advancement = SimpleArgConsumer::find_arg(args, ARG_ADVANCEMENT)?;

            // TODO: Implement from/through/until advancement tree traversal
            let count = targets.len();
            if count == 1 {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_ONE_SUCCESS,
                        [
                            TextComponent::text("0"),
                            TextComponent::text(targets[0].gameprofile.name.clone()),
                        ],
                    ))
                    .await;
            } else {
                sender
                    .send_message(TextComponent::translate(
                        translation::COMMANDS_ADVANCEMENT_REVOKE_MANY_TO_MANY_SUCCESS,
                        [
                            TextComponent::text("0"),
                            TextComponent::text(count.to_string()),
                        ],
                    ))
                    .await;
            }
            Ok(count as i32)
        })
    }
}

fn grant_tree() -> crate::command::tree::builder::NonLeafNodeBuilder {
    literal("grant").then(
        argument(ARG_TARGETS, PlayersArgumentConsumer)
            .then(literal("everything").execute(GrantEverythingExecutor))
            .then(
                literal("only").then(
                    argument(ARG_ADVANCEMENT, SimpleArgConsumer)
                        .then(argument(ARG_CRITERION, SimpleArgConsumer).execute(GrantOnlyExecutor))
                        .execute(GrantOnlyExecutor),
                ),
            )
            .then(
                literal("from")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(GrantTreeExecutor)),
            )
            .then(
                literal("through")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(GrantTreeExecutor)),
            )
            .then(
                literal("until")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(GrantTreeExecutor)),
            ),
    )
}

fn revoke_tree() -> crate::command::tree::builder::NonLeafNodeBuilder {
    literal("revoke").then(
        argument(ARG_TARGETS, PlayersArgumentConsumer)
            .then(literal("everything").execute(RevokeEverythingExecutor))
            .then(
                literal("only").then(
                    argument(ARG_ADVANCEMENT, SimpleArgConsumer)
                        .then(
                            argument(ARG_CRITERION, SimpleArgConsumer).execute(RevokeOnlyExecutor),
                        )
                        .execute(RevokeOnlyExecutor),
                ),
            )
            .then(
                literal("from")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(RevokeTreeExecutor)),
            )
            .then(
                literal("through")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(RevokeTreeExecutor)),
            )
            .then(
                literal("until")
                    .then(argument(ARG_ADVANCEMENT, SimpleArgConsumer).execute(RevokeTreeExecutor)),
            ),
    )
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(grant_tree())
        .then(revoke_tree())
}
