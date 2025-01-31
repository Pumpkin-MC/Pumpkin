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
        target: &Player,  // Remove sender parameter since we'll handle errors in execute
        amount: i32,
        exp_type: ExpType,
        mode: Mode,
    ) -> Result<(), &'static str> {  // Change return type to indicate error reason
        match exp_type {
            ExpType::Levels => {
                let current_level = target.experience_level.load(Ordering::Relaxed);
                let new_level = if mode == Mode::Add {
                    current_level + amount
                } else {
                    amount
                };

                if new_level < 0 {
                    return Err("commands.experience.set.points.invalid");
                }

                target.set_level(new_level).await;
            }
            ExpType::Points => {
                if mode == Mode::Add {
                    target.add_experience(amount).await;
                } else {
                    // When setting points, check if they exceed current level's max
                    let current_level = target.experience_level.load(Ordering::Relaxed);
                    let current_level_start = experience::get_total_exp_to_level(current_level);
                    let next_level_start = experience::get_total_exp_to_level(current_level + 1);
                    
                    // Amount must be between current level's start and next level's start (exclusive)
                    if amount < current_level_start || amount >= next_level_start {
                        return Err("commands.experience.set.points.invalid");
                    }
                    
                    // Calculate progress within current level
                    let level_points = amount - current_level_start;
                    let points_needed = next_level_start - current_level_start;
                    let progress = level_points as f32 / points_needed as f32;
                    
                    target.set_experience(current_level, progress, amount).await;
                }
            }
        }
        Ok(())
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
                    match self.handle_modify(target, amount, self.exp_type.unwrap(), self.mode).await {
                        Ok(()) => {
                            let msg = Self::get_success_message(
                                self.mode,
                                self.exp_type.unwrap(),
                                amount,
                                targets.len(),
                                Some(target.gameprofile.name.clone()),
                            );
                            sender.send_message(msg).await;
                        }
                        Err(error_msg) => {
                            sender
                                .send_message(TextComponent::translate(error_msg, [].into()))
                                .await;
                            continue;
                        }
                    }
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
