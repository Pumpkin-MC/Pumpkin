use std::sync::Arc;
use std::sync::atomic::Ordering;

use pumpkin_util::PermissionLvl;
use pumpkin_util::math::experience;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::entity::EntityBase;
use crate::entity::player::Player;

const DESCRIPTION: &str = "Add, set or query player experience.";
const PERMISSION: &str = "minecraft:command.experience";

const ERROR_SET_POINTS_INVALID: CommandErrorType<0> = CommandErrorType::new(
    "commands.experience.set.points.invalid",
    "commands.experience.set.points.invalid",
);

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
    exp_type: ExpType,
}

impl ExperienceExecutor {
    fn handle_query(&self, context: &CommandContext, target: &Player) -> i32 {
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

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation_key,
                translation_key,
                [
                    target.get_display_name(),
                    TextComponent::text(val.to_string()),
                ],
            ),
            false,
        );

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

        let bedrock_key = match self.exp_type {
            ExpType::Points => pumpkin_data::translation::bedrock::COMMANDS_XP_SUCCESS,
            ExpType::Levels => {
                if amount >= 0 {
                    pumpkin_data::translation::bedrock::COMMANDS_XP_SUCCESS_LEVELS
                } else {
                    pumpkin_data::translation::bedrock::COMMANDS_XP_SUCCESS_NEGATIVE_LEVELS
                }
            }
        };

        let bedrock_amount = if amount >= 0 { amount } else { amount.abs() };

        if targets.len() > 1 {
            TextComponent::translate_cross(
                format!("commands.experience.{mode_str}.{type_str}.success.multiple"),
                bedrock_key,
                [
                    TextComponent::text(bedrock_amount.to_string()),
                    TextComponent::text(targets.len().to_string()),
                ],
            )
        } else {
            TextComponent::translate_cross(
                format!("commands.experience.{mode_str}.{type_str}.success.single"),
                bedrock_key,
                [
                    TextComponent::text(bedrock_amount.to_string()),
                    first_target_name,
                ],
            )
        }
    }

    fn handle_modify(&self, target: &Arc<Player>, amount: i32) -> bool {
        match self.exp_type {
            ExpType::Levels => {
                if self.mode == Mode::Add {
                    target.add_experience_levels(amount);
                } else {
                    target.set_experience_level(amount, true);
                }
            }
            ExpType::Points => {
                if self.mode == Mode::Add {
                    target.add_experience_points(amount);
                } else {
                    let current_lvl = target.experience_level.load(Ordering::Relaxed);
                    if amount >= experience::points_in_level(current_lvl) {
                        return false;
                    }
                    target.set_experience_points(amount);
                }
            }
        }
        true
    }
}

impl CommandExecutor for ExperienceExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_players(context, "target")?;

        if self.mode == Mode::Query {
            let target = targets[0].clone();
            return Ok(self.handle_query(context, &target));
        }

        let amount = IntegerArgumentType::get(context, "amount")?;

        let mut successes = 0;
        for target in &targets {
            if self.handle_modify(target, amount) {
                successes += 1;
            }
        }

        if self.mode == Mode::Set && successes == 0 {
            return Err(ERROR_SET_POINTS_INVALID.create_without_context());
        }

        let first_name = targets[0].as_ref().get_display_name();
        let msg = self.get_success_message(amount, &targets, first_name);
        context.source.send_feedback(msg, true);

        Ok(targets.len() as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let add_node = literal("add").then(
        argument("target", EntityArgumentType::Players).then(
            argument("amount", IntegerArgumentType::any())
                .executes(ExperienceExecutor {
                    mode: Mode::Add,
                    exp_type: ExpType::Points,
                })
                .then(literal("points").executes(ExperienceExecutor {
                    mode: Mode::Add,
                    exp_type: ExpType::Points,
                }))
                .then(literal("levels").executes(ExperienceExecutor {
                    mode: Mode::Add,
                    exp_type: ExpType::Levels,
                })),
        ),
    );

    let set_node = literal("set").then(
        argument("target", EntityArgumentType::Players).then(
            argument("amount", IntegerArgumentType::with_min(0))
                .executes(ExperienceExecutor {
                    mode: Mode::Set,
                    exp_type: ExpType::Points,
                })
                .then(literal("points").executes(ExperienceExecutor {
                    mode: Mode::Set,
                    exp_type: ExpType::Points,
                }))
                .then(literal("levels").executes(ExperienceExecutor {
                    mode: Mode::Set,
                    exp_type: ExpType::Levels,
                })),
        ),
    );

    let query_node = literal("query").then(
        argument("target", EntityArgumentType::Player)
            .then(literal("points").executes(ExperienceExecutor {
                mode: Mode::Query,
                exp_type: ExpType::Points,
            }))
            .then(literal("levels").executes(ExperienceExecutor {
                mode: Mode::Query,
                exp_type: ExpType::Levels,
            })),
    );

    let cmd = command("experience", DESCRIPTION)
        .requires(PERMISSION)
        .then(add_node)
        .then(set_node)
        .then(query_node);

    dispatcher.register_with_aliases(cmd, &["xp"]);
}
