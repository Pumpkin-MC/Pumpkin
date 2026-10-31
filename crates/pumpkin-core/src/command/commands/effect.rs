use pumpkin_data::potion::Effect;
use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::core::bool::BoolArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::entity::EntityArgumentType;
use crate::command::argument_types::resource::{MOB_EFFECT_ARGUMENT, ResourceArgument};
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};

const DESCRIPTION: &str = "Adds or removes the status effects of players and other entities.";
const PERMISSION: &str = "minecraft:command.effect";

const ERROR_GIVE_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_EFFECT_GIVE_FAILED,
    translation::java::COMMANDS_EFFECT_GIVE_FAILED,
);

const ERROR_CLEAR_EVERYTHING_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_EFFECT_CLEAR_EVERYTHING_FAILED,
    translation::java::COMMANDS_EFFECT_CLEAR_EVERYTHING_FAILED,
);

const ERROR_CLEAR_SPECIFIC_FAILED: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_EFFECT_CLEAR_SPECIFIC_FAILED,
    translation::java::COMMANDS_EFFECT_CLEAR_SPECIFIC_FAILED,
);

#[derive(Clone, Copy)]
enum Duration {
    Default,
    Specified,
    Infinite,
}

struct GiveExecutor {
    duration: Duration,
    has_amplifier: bool,
    has_hide_particles: bool,
}

impl CommandExecutor for GiveExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = EntityArgumentType::get_entities(context, "targets")?;
        let effect = ResourceArgument::get_mob_effect(context, "effect")?;

        let duration_ticks = match self.duration {
            Duration::Default => 30 * 20,
            Duration::Specified => IntegerArgumentType::get(context, "seconds")? * 20,
            Duration::Infinite => -1,
        };

        let amplifier = if self.has_amplifier {
            IntegerArgumentType::get(context, "amplifier")? as u8
        } else {
            0
        };

        let hide_particles = if self.has_hide_particles {
            BoolArgumentType::get(context, "hideParticles")?
        } else {
            false
        };

        let instance = Effect {
            effect_type: effect,
            duration: duration_ticks,
            amplifier,
            ambient: false,
            show_particles: !hide_particles,
            show_icon: true,
            blend: false,
        };

        // Vanilla only tracks successful applications, so the feedback below counts those.
        let mut successes = 0;
        let mut only_success = None;

        for target in &targets {
            // Vanilla skips every selected entity that isn't a `LivingEntity`.
            let Some(living) = target.get_living_entity() else {
                continue;
            };

            let should_skip = living
                .get_effect(effect)
                .is_some_and(|existing| existing.amplifier >= amplifier);

            if !should_skip {
                // The player path also fires the effects-changed advancement, as `ServerPlayer` does.
                if let Some(player) = target.get_player() {
                    player.add_effect(instance.clone());
                } else {
                    living.add_effect(instance.clone());
                }
                successes += 1;
                only_success = (successes == 1).then_some(target.as_ref());
            }
        }

        let translation_name = TextComponent::translate_cross(
            effect.translation_key.to_string(),
            effect.translation_key.to_string(),
            [],
        );

        if successes == 0 {
            return Err(ERROR_GIVE_FAILED.create_without_context());
        }

        if let Some(target) = only_success {
            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_EFFECT_GIVE_SUCCESS_SINGLE,
                    translation::java::COMMANDS_EFFECT_GIVE_SUCCESS_SINGLE,
                    [translation_name, target.get_display_name()],
                ),
                true,
            );
        } else {
            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_EFFECT_GIVE_SUCCESS_MULTIPLE,
                    translation::java::COMMANDS_EFFECT_GIVE_SUCCESS_MULTIPLE,
                    [translation_name, TextComponent::text(successes.to_string())],
                ),
                true,
            );
        }

        Ok(successes)
    }
}

enum ClearMode {
    SelfAll,
    TargetsAll,
    TargetsSpecific,
}

struct ClearExecutor(ClearMode);

impl CommandExecutor for ClearExecutor {
    #[allow(clippy::too_many_lines, clippy::option_if_let_else)]
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let targets = match self.0 {
            ClearMode::SelfAll => vec![context.source.entity_or_err()?],
            ClearMode::TargetsAll | ClearMode::TargetsSpecific => {
                EntityArgumentType::get_entities(context, "targets")?
            }
        };

        match self.0 {
            ClearMode::SelfAll | ClearMode::TargetsAll => {
                let mut succeeded_clears = 0;
                let mut only_success = None;
                for target in &targets {
                    // Vanilla skips every selected entity that isn't a `LivingEntity`.
                    let Some(living) = target.get_living_entity() else {
                        continue;
                    };
                    let cleared = if let Some(player) = target.get_player() {
                        player.remove_all_effects()
                    } else {
                        living.remove_all_effects()
                    };

                    if cleared {
                        succeeded_clears += 1;
                        only_success = (succeeded_clears == 1).then_some(target.as_ref());
                    }
                }

                if succeeded_clears == 0 {
                    return Err(ERROR_CLEAR_EVERYTHING_FAILED.create_without_context());
                }

                if let Some(target) = only_success {
                    context.source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_EFFECT_CLEAR_EVERYTHING_SUCCESS_SINGLE,
                            translation::java::COMMANDS_EFFECT_CLEAR_EVERYTHING_SUCCESS_SINGLE,
                            [target.get_display_name()],
                        ),
                        true,
                    );
                } else {
                    context.source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_EFFECT_CLEAR_EVERYTHING_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_EFFECT_CLEAR_EVERYTHING_SUCCESS_MULTIPLE,
                            [TextComponent::text(succeeded_clears.to_string())],
                        ),
                        true,
                    );
                }
                Ok(succeeded_clears)
            }
            ClearMode::TargetsSpecific => {
                let effect = ResourceArgument::get_mob_effect(context, "effect")?;
                let mut succeeded_clears = 0;
                let mut only_success = None;
                for target in &targets {
                    // Vanilla skips every selected entity that isn't a `LivingEntity`.
                    let Some(living) = target.get_living_entity() else {
                        continue;
                    };
                    let removed = if let Some(player) = target.get_player() {
                        player.remove_effect(effect)
                    } else {
                        living.remove_effect(effect)
                    };

                    if removed {
                        succeeded_clears += 1;
                        only_success = (succeeded_clears == 1).then_some(target.as_ref());
                    }
                }

                if succeeded_clears == 0 {
                    return Err(ERROR_CLEAR_SPECIFIC_FAILED.create_without_context());
                }

                if let Some(target) = only_success {
                    context.source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_EFFECT_CLEAR_SPECIFIC_SUCCESS_SINGLE,
                            translation::java::COMMANDS_EFFECT_CLEAR_SPECIFIC_SUCCESS_SINGLE,
                            [
                                TextComponent::translate_cross(
                                    effect.translation_key,
                                    effect.translation_key,
                                    [],
                                ),
                                target.get_display_name(),
                            ],
                        ),
                        true,
                    );
                } else {
                    context.source.send_feedback(
                        TextComponent::translate_cross(
                            translation::java::COMMANDS_EFFECT_CLEAR_SPECIFIC_SUCCESS_MULTIPLE,
                            translation::java::COMMANDS_EFFECT_CLEAR_SPECIFIC_SUCCESS_MULTIPLE,
                            [
                                TextComponent::translate_cross(
                                    effect.translation_key,
                                    effect.translation_key,
                                    [],
                                ),
                                TextComponent::text(succeeded_clears.to_string()),
                            ],
                        ),
                        true,
                    );
                }
                Ok(succeeded_clears)
            }
        }
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let seconds_node = argument("seconds", IntegerArgumentType::new(0, 1_000_000))
        .executes(GiveExecutor {
            duration: Duration::Specified,
            has_amplifier: false,
            has_hide_particles: false,
        })
        .then(
            argument("amplifier", IntegerArgumentType::new(0, 255))
                .executes(GiveExecutor {
                    duration: Duration::Specified,
                    has_amplifier: true,
                    has_hide_particles: false,
                })
                .then(
                    argument("hideParticles", BoolArgumentType).executes(GiveExecutor {
                        duration: Duration::Specified,
                        has_amplifier: true,
                        has_hide_particles: true,
                    }),
                ),
        );

    let infinite_node = literal("infinite")
        .executes(GiveExecutor {
            duration: Duration::Infinite,
            has_amplifier: false,
            has_hide_particles: false,
        })
        .then(
            argument("amplifier", IntegerArgumentType::new(0, 255))
                .executes(GiveExecutor {
                    duration: Duration::Infinite,
                    has_amplifier: true,
                    has_hide_particles: false,
                })
                .then(
                    argument("hideParticles", BoolArgumentType).executes(GiveExecutor {
                        duration: Duration::Infinite,
                        has_amplifier: true,
                        has_hide_particles: true,
                    }),
                ),
        );

    let give_node = literal("give").then(
        argument("targets", EntityArgumentType::Entities).then(
            argument("effect", MOB_EFFECT_ARGUMENT.clone())
                .executes(GiveExecutor {
                    duration: Duration::Default,
                    has_amplifier: false,
                    has_hide_particles: false,
                })
                .then(seconds_node)
                .then(infinite_node),
        ),
    );

    let clear_node = literal("clear")
        .executes(ClearExecutor(ClearMode::SelfAll))
        .then(
            argument("targets", EntityArgumentType::Entities)
                .executes(ClearExecutor(ClearMode::TargetsAll))
                .then(
                    argument("effect", MOB_EFFECT_ARGUMENT.clone())
                        .executes(ClearExecutor(ClearMode::TargetsSpecific)),
                ),
        );

    dispatcher.register(
        command("effect", DESCRIPTION)
            .requires(PERMISSION)
            .then(clear_node)
            .then(give_node),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::context::command_source::CommandSource;
    use std::sync::Arc;

    #[test]
    fn parse_effect_with_entity_targets() {
        let mut dispatcher = CommandDispatcher::new();
        let registry = PermissionRegistry::default();
        register(&mut dispatcher, &registry);

        let source = Arc::new(CommandSource::dummy());

        // Vanilla declares `targets` as `EntityArgument.entities()`, so selectors that include
        // non-player entities must parse for both subcommands.
        for input in [
            "effect give @e minecraft:glowing",
            "effect give @e[type=pig] minecraft:glowing 30 0 true",
            "effect give @e minecraft:glowing infinite",
            "effect clear @e",
            "effect clear @e minecraft:glowing",
        ] {
            let result = dispatcher.parse_input(input, &source);
            assert!(
                result.errors.is_empty(),
                "`{input}` failed to parse: {:?}",
                result.errors
            );
        }
    }
}
