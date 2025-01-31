use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, literal};
use crate::command::{CommandError, CommandExecutor, CommandSender};

const NAMES: [&str; 2] = ["experience", "xp"];
const DESCRIPTION: &str = "Add, set or query player experience.";
const ARG_TARGETS: &str = "targets";
const ARG_AMOUNT: &str = "amount";

fn xp_amount() -> BoundedNumArgumentConsumer<i32> {
    BoundedNumArgumentConsumer::new()
        .name(ARG_AMOUNT)
        .min(0)
        .max(i32::MAX)
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Add,
    Set,
    Query,
}

struct ExperienceExecutor(Mode);

#[async_trait]
impl CommandExecutor for ExperienceExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        // Get target players
        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

        match self.0 {
            Mode::Query => {
                for target in targets {
                    let level = target
                        .experience_level
                        .load(std::sync::atomic::Ordering::Relaxed);
                    let progress = target.experience_progress.load();
                    let total = target
                        .total_experience
                        .load(std::sync::atomic::Ordering::Relaxed);

                    // Send both levels and points queries
                    sender
                        .send_message(TextComponent::translate(
                            "commands.experience.query.levels",
                            [
                                TextComponent::text(target.gameprofile.name.clone()),
                                TextComponent::text(level.to_string()),
                            ]
                            .into(),
                        ))
                        .await;

                    sender
                        .send_message(TextComponent::translate(
                            "commands.experience.query.points",
                            [
                                TextComponent::text(target.gameprofile.name.clone()),
                                TextComponent::text(total.to_string()),
                            ]
                            .into(),
                        ))
                        .await;
                }
            }
            Mode::Add | Mode::Set => {
                // Use proper FindArg trait function call and handle both error cases
                let amount = match BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_AMOUNT) {
                    Ok(Ok(value)) => value,
                    Ok(Err(_)) => {
                        sender
                            .send_message(TextComponent::translate(
                                "commands.experience.set.points.invalid",
                                [].into(),
                            ))
                            .await;
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                };

                for target in targets {
                    let current_level = target
                        .experience_level
                        .load(std::sync::atomic::Ordering::Relaxed);
                    let new_level = if self.0 == Mode::Add {
                        current_level + amount
                    } else {
                        amount
                    };

                    // Ensure level doesn't go below 0
                    if new_level < 0 {
                        sender
                            .send_message(TextComponent::translate(
                                "commands.experience.set.points.invalid",
                                [].into(),
                            ))
                            .await;
                        continue;
                    }

                    target.set_experience(new_level, 0.0, new_level * 100).await;

                    // Send appropriate success message based on operation and number of targets
                    let msg = if self.0 == Mode::Add {
                        if targets.len() > 1 {
                            TextComponent::translate(
                                "commands.experience.add.levels.success.multiple",
                                [
                                    TextComponent::text(amount.to_string()),
                                    TextComponent::text(targets.len().to_string()),
                                ]
                                .into(),
                            )
                        } else {
                            TextComponent::translate(
                                "commands.experience.add.levels.success.single",
                                [
                                    TextComponent::text(amount.to_string()),
                                    TextComponent::text(target.gameprofile.name.clone()),
                                ]
                                .into(),
                            )
                        }
                    } else if targets.len() > 1 {
                        TextComponent::translate(
                            "commands.experience.set.levels.success.multiple",
                            [
                                TextComponent::text(new_level.to_string()),
                                TextComponent::text(targets.len().to_string()),
                            ]
                            .into(),
                        )
                    } else {
                        TextComponent::translate(
                            "commands.experience.set.levels.success.single",
                            [
                                TextComponent::text(new_level.to_string()),
                                TextComponent::text(target.gameprofile.name.clone()),
                            ]
                            .into(),
                        )
                    };
                    sender.send_message(msg).await;
                    break; // Only send message once for multiple targets
                }
            }
        }

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("add")
                .then(argument(ARG_TARGETS, PlayersArgumentConsumer).then(
                    argument(ARG_AMOUNT, xp_amount()).execute(ExperienceExecutor(Mode::Add)),
                )),
        )
        .then(
            literal("set")
                .then(argument(ARG_TARGETS, PlayersArgumentConsumer).then(
                    argument(ARG_AMOUNT, xp_amount()).execute(ExperienceExecutor(Mode::Set)),
                )),
        )
        .then(literal("query").then(
            argument(ARG_TARGETS, PlayersArgumentConsumer).execute(ExperienceExecutor(Mode::Query)),
        ))
}
