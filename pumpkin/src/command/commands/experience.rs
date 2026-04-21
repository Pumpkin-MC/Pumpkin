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
    exp_type: ExpType,
}

impl Executor {
    async fn handle_query(&self, sender: &CommandSender, target: &Player) -> i32 {
        let (val, translation_key) = match self.exp_type {
            ExpType::Levels => (
                target.experience_level.load(Ordering::Relaxed),
                "commands.experience.query.levels",
            ),
            ExpType::Points => (
                target.experience_points.load(Ordering::Relaxed),
                "commands.experience.query.points",
            ),
        };

        sender
            .send_message(TextComponent::translate(
                translation_key,
                [
                    target.get_display_name().await,
                    TextComponent::text(val.to_string()),
                ],
            ))
            .await;

        val
    }

    fn get_success_message(
        &self,
        amount: i32,
        targets: &[Arc<Player>],
        first_target_name: TextComponent,
    ) -> TextComponent {
        let type_str = match self.exp_type {
            ExpType::Points => "points",
            ExpType::Levels => "levels",
        };
        let mode_str = match self.mode {
            Mode::Add => "add",
            Mode::Set => "set",
            Mode::Query => "query",
        };

        if targets.len() > 1 {
            TextComponent::translate(
                format!("commands.experience.{mode_str}.{type_str}.success.multiple"),
                [
                    TextComponent::text(amount.to_string()),
                    TextComponent::text(targets.len().to_string()),
                ],
            )
        } else {
            TextComponent::translate(
                format!("commands.experience.{mode_str}.{type_str}.success.single"),
                [TextComponent::text(amount.to_string()), first_target_name],
            )
        }
    }

    /// Returns `true` if successful. Otherwise, there was a problem setting the points of a player.
    async fn handle_modify(&self, target: &Arc<Player>, amount: i32) -> bool {
        match self.exp_type {
            ExpType::Levels => {
                if self.mode == Mode::Add {
                    target.add_experience_levels(amount).await;
                } else {
                    target.set_experience_level(amount, true).await;
                }
            }
            ExpType::Points => {
                if self.mode == Mode::Add {
                    target.add_experience_points(amount).await;
                } else {
                    let current_lvl = target.experience_level.load(Ordering::Relaxed);
                    if amount > experience::points_in_level(current_lvl) {
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

            if self.mode == Mode::Query {
                let target = targets.first().ok_or_else(|| {
                    CommandError::CommandFailed(TextComponent::translate(
                        "argument.player.unknown",
                        [],
                    ))
                })?;

                if targets.len() > 1 {
                    return Err(CommandError::CommandFailed(TextComponent::translate(
                        "argument.player.toomany",
                        [],
                    )));
                }
                return Ok(self.handle_query(sender, target).await);
            }

            // Handle Add/Set
            let amount = BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_AMOUNT)??;

            let mut successes = 0;
            for target in targets {
                if self.handle_modify(target, amount).await {
                    successes += 1;
                }
            }

            if successes == 0 {
                return Err(CommandError::CommandFailed(TextComponent::translate(
                    "commands.experience.set.points.invalid",
                    [],
                )));
            }

            // Safe to access first() because successes > 0
            let first_name = targets[0].get_display_name().await;
            let msg = self.get_success_message(amount, targets, first_name);
            sender.send_message(msg).await;

            Ok(successes)
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
                            exp_type: ExpType::Levels,
                        }))
                        .then(literal("points").execute(Executor {
                            mode: Mode::Add,
                            exp_type: ExpType::Points,
                        }))
                        .execute(Executor {
                            mode: Mode::Add,
                            exp_type: ExpType::Points,
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
                            exp_type: ExpType::Levels,
                        }))
                        .then(literal("points").execute(Executor {
                            mode: Mode::Set,
                            exp_type: ExpType::Points,
                        }))
                        .execute(Executor {
                            mode: Mode::Set,
                            exp_type: ExpType::Points,
                        }),
                ),
            ),
        )
        .then(
            literal("query").then(
                argument(ARG_TARGETS, PlayersArgumentConsumer)
                    .then(literal("levels").execute(Executor {
                        mode: Mode::Query,
                        exp_type: ExpType::Levels,
                    }))
                    .then(literal("points").execute(Executor {
                        mode: Mode::Query,
                        exp_type: ExpType::Points,
                    })),
            ),
        )
}
