use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_util::math::experience;
use pumpkin_util::text::TextComponent;

use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::players::PlayersArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::dispatcher::CommandError;
use crate::command::tree::CommandTree;
use crate::command::tree::builder::{argument, literal};
use crate::command::{CommandExecutor, CommandResult, CommandSender};
use crate::entity::EntityBase;
use crate::entity::player::Player;

const NAMES: [&str; 2] = ["experience", "xp"];
const DESCRIPTION: &str = "Add, set or query player experience.";
const ARG_TARGETS: &str = "targets";
const ARG_AMOUNT: &str = "amount";

const fn xp_amount() -> BoundedNumArgumentConsumer<i32> {
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

struct Executor {
    mode: Mode,
    exp_type: Option<ExpType>,
}

impl Executor {
    async fn handle_query(
        &self,
        sender: &CommandSender,
        target: &Player,
        exp_type: ExpType,
    ) -> i32 {
        match exp_type {
            ExpType::Levels => {
                let level = target.experience_level.load(Ordering::Relaxed);
                sender
                    .send_message(TextComponent::translate(
                        "commands.experience.query.levels",
                        [
                            target.get_display_name().await,
                            TextComponent::text(level.to_string()),
                        ],
                    ))
                    .await;
                level
            }
            ExpType::Points => {
                let points = target.experience_points.load(Ordering::Relaxed);
                sender
                    .send_message(TextComponent::translate(
                        "commands.experience.query.points",
                        [
                            target.get_display_name().await,
                            TextComponent::text(points.to_string()),
                        ],
                    ))
                    .await;
                points
            }
        }
    }

    fn get_success_message(
        mode: Mode,
        exp_type: ExpType,
        amount: i32,
        targets_len: usize,
        target_name: Option<TextComponent>,
    ) -> TextComponent {
        match (mode, exp_type) {
            (Mode::Add, ExpType::Points) => {
                if targets_len > 1 {
                    TextComponent::translate(
                        "commands.experience.add.points.success.multiple",
                        [
                            TextComponent::text(amount.to_string()),
                            TextComponent::text(targets_len.to_string()),
                        ],
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.add.points.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            target_name.unwrap(),
                        ],
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
                        ],
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.add.levels.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            target_name.unwrap(),
                        ],
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
                        ],
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.set.points.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            target_name.unwrap(),
                        ],
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
                        ],
                    )
                } else {
                    TextComponent::translate(
                        "commands.experience.set.levels.success.single",
                        [
                            TextComponent::text(amount.to_string()),
                            target_name.unwrap(),
                        ],
                    )
                }
            }
            (Mode::Query, _) => unreachable!("Query mode doesn't use success messages"),
        }
    }

    /// Returns `true` if successful. Otherwise, there was a problem setting the points of a player.
    async fn handle_modify(
        &self,
        target: &Arc<Player>,
        amount: i32,
        exp_type: ExpType,
        mode: Mode,
    ) -> bool {
        match exp_type {
            ExpType::Levels => {
                if mode == Mode::Add {
                    target.add_experience_levels(amount).await;
                } else {
                    target.set_experience_level(amount, true).await;
                }
            }
            ExpType::Points => {
                if mode == Mode::Add {
                    target.add_experience_points(amount).await;
                } else {
                    let current_level = target.experience_level.load(Ordering::Relaxed);
                    let current_max_points = experience::points_in_level(current_level);

                    if amount > current_max_points {
                        return false;
                    }

                    target.set_experience_points(amount).await;
                }
            }
        }
        true
    }
}

impl CommandExecutor for Executor {
    fn execute<'a>(
        &'a self,
        sender: &'a CommandSender,
        _server: &'a crate::server::Server,
        args: &'a ConsumedArgs<'a>,
    ) -> CommandResult<'a> {
        Box::pin(async move {
            let targets = PlayersArgumentConsumer::find_arg(args, ARG_TARGETS)?;

            match self.mode {
                Mode::Query => {
                    if targets.len() != 1 {
                        // TODO: Add proper error message for multiple players in query mode during parsing itself and not here
                        return Err(CommandError::CommandFailed(TextComponent::translate(
                            "argument.player.toomany",
                            [],
                        )));
                    }
                    Ok(self
                        .handle_query(sender, &targets[0], self.exp_type.unwrap())
                        .await)
                }
                Mode::Add | Mode::Set => {
                    let Ok(amount) = BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_AMOUNT)?
                    else {
                        return Err(CommandError::CommandFailed(TextComponent::translate(
                            "commands.experience.set.points.invalid",
                            [],
                        )));
                    };

                    if self.mode == Mode::Set && amount < 0 {
                        return Err(CommandError::CommandFailed(TextComponent::translate(
                            "commands.experience.set.points.invalid",
                            [],
                        )));
                    }

                    let mut successes: i32 = 0;
                    for target in targets {
                        let succeeded = self
                            .handle_modify(target, amount, self.exp_type.unwrap(), self.mode)
                            .await;

                        if succeeded {
                            successes += 1;
                        }
                    }

                    if successes == 0 {
                        Err(CommandError::CommandFailed(TextComponent::translate(
                            "commands.experience.set.points.invalid",
                            [],
                        )))
                    } else {
                        // This should not panic as we already check the number of successes to not be equal to `0`.
                        let target = targets
                            .first()
                            .expect("expected at least one player in targets")
                            .get_display_name()
                            .await;

                        let msg = Self::get_success_message(
                            self.mode,
                            self.exp_type.unwrap(),
                            amount,
                            targets.len(),
                            Some(target),
                        );
                        sender.send_message(msg).await;

                        Ok(successes)
                    }
                }
            }
        })
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(
            literal("add").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer).then(
                    argument(ARG_AMOUNT, xp_amount())
                        .then(literal("levels").execute(Executor {
                            mode: Mode::Add,
                            exp_type: Some(ExpType::Levels),
                        }))
                        .then(literal("points").execute(Executor {
                            mode: Mode::Add,
                            exp_type: Some(ExpType::Points),
                        }))
                        .execute(Executor {
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
                        .then(literal("levels").execute(Executor {
                            mode: Mode::Set,
                            exp_type: Some(ExpType::Levels),
                        }))
                        .then(literal("points").execute(Executor {
                            mode: Mode::Set,
                            exp_type: Some(ExpType::Points),
                        }))
                        .execute(Executor {
                            mode: Mode::Set,
                            exp_type: Some(ExpType::Points),
                        }),
                ),
            ),
        )
        .then(
            literal("query").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer)
                    .then(literal("levels").execute(Executor {
                        mode: Mode::Query,
                        exp_type: Some(ExpType::Levels),
                    }))
                    .then(literal("points").execute(Executor {
                        mode: Mode::Query,
                        exp_type: Some(ExpType::Points),
                    })),
            ),
        )
}
