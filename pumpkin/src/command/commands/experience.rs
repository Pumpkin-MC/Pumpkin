use std::sync::atomic::Ordering;

use async_trait::async_trait;
use pumpkin_util::math::experience;
use pumpkin_util::text::TextComponent;

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, literal};
use crate::command::{CommandError, CommandExecutor, CommandSender};
use crate::entity::player::Player;

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

impl ExperienceExecutor {
    async fn handle_query(
        &self,
        sender: &mut CommandSender<'_>,
        target: &Player,
        exp_type: ExpType,
    ) {
        match exp_type {
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

    fn get_success_message(
        mode: Mode,
        exp_type: ExpType,
        amount: i32,
        targets_len: usize,
        target_name: Option<String>,
    ) -> TextComponent {
        match (mode, exp_type) {
            (Mode::Add, ExpType::Points) => {
                if targets_len > 1 {
                    TextComponent::translate(
                        "commands.experience.add.points.success.multiple",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(targets_len.to_string()),
                        ]
                        .into(),
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.add.points.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(target_name.unwrap()),
                        ]
                        .into(),
                    )
                }
            }
            (Mode::Add, ExpType::Levels) => {
                if targets_len > 1 {
                    TextComponent::translate(
                        "commands.experience.add.levels.success.multiple",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(targets_len.to_string()),
                        ]
                        .into(),
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.add.levels.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(target_name.unwrap()),
                        ]
                        .into(),
                    )
                }
            }
            (Mode::Set, ExpType::Points) => {
                if targets_len > 1 {
                    TextComponent::translate(
                        "commands.experience.set.points.success.multiple",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(targets_len.to_string()),
                        ]
                        .into(),
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.set.points.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(target_name.unwrap()),
                        ]
                        .into(),
                    )
                }
            }
            (Mode::Set, ExpType::Levels) => {
                if targets_len > 1 {
                    TextComponent::translate(
                        "commands.experience.set.levels.success.multiple",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(targets_len.to_string()),
                        ]
                        .into(),
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.set.levels.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(target_name.unwrap()),
                        ]
                        .into(),
                    )
                }
            }
            (Mode::Query, _) => unreachable!("Query mode doesn't use success messages"),
        }
    }

    async fn handle_modify(
        &self,
        _sender: &mut CommandSender<'_>,
        target: &Player,
        amount: i32,
        exp_type: ExpType,
        mode: Mode,
    ) -> bool {
        match exp_type {
            ExpType::Levels => {
                let current_level = target.experience_level.load(Ordering::Relaxed);
                let new_level = if mode == Mode::Add {
                    current_level + amount
                } else {
                    amount
                };

                if new_level < 0 {
                    return false;
                }

                target.set_level(new_level).await;
            }
            ExpType::Points => {
                if mode == Mode::Add {
                    target.add_experience(amount).await;
                } else {
                    let level = experience::get_level_from_total_exp(amount);
                    let progress = experience::get_progress_from_total_exp(amount);
                    target.set_experience(level, progress, amount).await;
                }
            }
        }
        true
    }
}

#[async_trait]
impl CommandExecutor for ExperienceExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

        match self.mode {
            Mode::Query => {
                if targets.len() != 1 {
                    // TODO: Add proper error message for multiple players in query mode
                    return Ok(());
                }
                self.handle_query(sender, &targets[0], self.exp_type.unwrap())
                    .await;
            }
            Mode::Add | Mode::Set => {
                let Ok(amount) = BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_AMOUNT)?
                else {
                    sender
                        .send_message(TextComponent::translate(
                            "commands.experience.set.points.invalid",
                            [].into(),
                        ))
                        .await;
                    return Ok(());
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

                for target in targets {
                    if !self
                        .handle_modify(sender, target, amount, self.exp_type.unwrap(), self.mode)
                        .await
                    {
                        sender
                            .send_message(TextComponent::translate(
                                "commands.experience.set.points.invalid",
                                [].into(),
                            ))
                            .await;
                        continue;
                    }

                    let msg = Self::get_success_message(
                        self.mode,
                        self.exp_type.unwrap(),
                        amount,
                        targets.len(),
                        Some(target.gameprofile.name.clone()),
                    );
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
            literal("add").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer).then(
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
                ),
            ),
        )
        .then(
            literal("set").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer).then(
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
                ),
            ),
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
