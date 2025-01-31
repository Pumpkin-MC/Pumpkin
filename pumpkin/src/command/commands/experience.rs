use std::sync::atomic::Ordering;

use async_trait::async_trait;
use pumpkin_util::text::TextComponent;
use pumpkin_util::math::experience;

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

#[derive(Clone, Copy, PartialEq)]
enum ExpType {
    Points,
    Levels,
}

struct ExperienceExecutor {
    mode: Mode,
    exp_type: Option<ExpType>,
}

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

        match self.mode {
            Mode::Query => {
                // For query mode, we can only target a single player
                if targets.len() != 1 {
                    // TODO: Add proper error message for multiple players in query mode
                    return Ok(());
                }

                let target = &targets[0];
                match self.exp_type.unwrap() {
                    ExpType::Levels => {
                        let level = target.experience_level.load(Ordering::Relaxed);
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
                    }
                    ExpType::Points => {
                        let total = target.total_experience.load(Ordering::Relaxed);
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
            }
            Mode::Add | Mode::Set => {
                let amount = match BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_AMOUNT)? {
                    Ok(value) => value,
                    Err(_) => {
                        sender
                            .send_message(TextComponent::translate(
                                "commands.experience.set.points.invalid",
                                [].into(),
                            ))
                            .await;
                        return Ok(());
                    }
                };

                if self.mode == Mode::Set && amount < 0 {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.experience.set.points.invalid",
                            [].into(),
                        ))
                        .await;
                    return Ok(());
                }

                for target in targets.iter() {
                    match self.exp_type.unwrap() {
                        ExpType::Levels => {
                            let current_level = target.experience_level.load(Ordering::Relaxed);
                            let new_level = if self.mode == Mode::Add {
                                current_level + amount
                            } else {
                                amount
                            };

                            if new_level < 0 {
                                sender
                                    .send_message(TextComponent::translate(
                                        "commands.experience.set.points.invalid",
                                        [].into(),
                                    ))
                                    .await;
                                continue;
                            }

                            target.set_level(new_level).await;
                        }
                        ExpType::Points => {
                            if self.mode == Mode::Add {
                                target.add_experience(amount).await;
                            } else {
                                let level = experience::get_level_from_total_exp(amount);
                                let progress = experience::get_progress_from_total_exp(amount);
                                target.set_experience(level, progress, amount).await;
                            }
                        }
                    }

                    // Send appropriate success message
                    let msg = match (self.mode, self.exp_type.unwrap()) {
                        (Mode::Add, ExpType::Points) => {
                            if targets.len() > 1 {
                                TextComponent::translate(
                                    "commands.experience.add.points.success.multiple",
                                    [
                                        TextComponent::text(amount.to_string()),
                                        TextComponent::text(targets.len().to_string()),
                                    ]
                                    .into(),
                                )
                            } else {
                                TextComponent::translate(
                                    "commands.experience.add.points.success.single",
                                    [
                                        TextComponent::text(amount.to_string()),
                                        TextComponent::text(target.gameprofile.name.clone()),
                                    ]
                                    .into(),
                                )
                            }
                        }
                        (Mode::Add, ExpType::Levels) => {
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
                        }
                        (Mode::Set, exp_type) => {
                            if targets.len() > 1 {
                                TextComponent::translate(
                                    if exp_type == ExpType::Levels {
                                        "commands.experience.set.levels.success.multiple"
                                    } else {
                                        "commands.experience.set.points.success.multiple"
                                    },
                                    [
                                        TextComponent::text(amount.to_string()),
                                        TextComponent::text(targets.len().to_string()),
                                    ]
                                    .into(),
                                )
                            } else {
                                TextComponent::translate(
                                    if exp_type == ExpType::Levels {
                                        "commands.experience.set.levels.success.single"
                                    } else {
                                        "commands.experience.set.points.success.single"
                                    },
                                    [
                                        TextComponent::text(amount.to_string()),
                                        TextComponent::text(target.gameprofile.name.clone()),
                                    ]
                                    .into(),
                                )
                            }
                        }
                        _ => unreachable!(),
                    };
                    sender.send_message(msg).await;
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
                    argument(ARG_AMOUNT, xp_amount())
                        .then(literal("levels").execute(ExperienceExecutor {
                            mode: Mode::Add,
                            exp_type: Some(ExpType::Levels),
                        }))
                        .then(literal("points").execute(ExperienceExecutor {
                            mode: Mode::Add,
                            exp_type: Some(ExpType::Points),
                        }))
                        .execute(ExperienceExecutor {
                            mode: Mode::Add,
                            exp_type: Some(ExpType::Points),
                        }),
                )),
        )
        .then(
            literal("set")
                .then(argument(ARG_TARGETS, PlayersArgumentConsumer).then(
                    argument(ARG_AMOUNT, xp_amount())
                        .then(literal("levels").execute(ExperienceExecutor {
                            mode: Mode::Set,
                            exp_type: Some(ExpType::Levels),
                        }))
                        .then(literal("points").execute(ExperienceExecutor {
                            mode: Mode::Set,
                            exp_type: Some(ExpType::Points),
                        }))
                        .execute(ExperienceExecutor {
                            mode: Mode::Set,
                            exp_type: Some(ExpType::Points),
                        }),
                )),
        )
        .then(
            literal("query").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer)
                    .then(literal("levels").execute(ExperienceExecutor {
                        mode: Mode::Query,
                        exp_type: Some(ExpType::Levels),
                    }))
                    .then(literal("points").execute(ExperienceExecutor {
                        mode: Mode::Query,
                        exp_type: Some(ExpType::Points),
                    })),
            ),
        )
}
