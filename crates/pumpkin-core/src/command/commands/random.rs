use pumpkin_data::translation;
use pumpkin_util::PermissionLvl;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;

use crate::command::argument_builder::{
    ArgumentBuilder, LiteralArgumentBuilder, argument, command, literal,
};
use crate::command::argument_types::core::bool::BoolArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::argument_types::range::IntRangeArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::suggestion::provider::{SuggestionProvider, SuggestionProviderResult};
use crate::command::suggestion::suggestions::SuggestionsBuilder;

const DESCRIPTION: &str = "Draws a random value.";
const PERMISSION: &str = "minecraft:command.random";
const PERMISSION_RESET: &str = "minecraft:command.random.reset";

const ARG_RANGE: &str = "range";
const ARG_SEQUENCE: &str = "sequence";
const ARG_SEED: &str = "seed";
const ARG_INCLUDE_WORLD_SEED: &str = "includeWorldSeed";
const ARG_INCLUDE_SEQUENCE_ID: &str = "includeSequenceId";

const RANGE_TOO_LARGE_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RANDOM_ERROR_RANGE_TOO_LARGE,
    translation::java::COMMANDS_RANDOM_ERROR_RANGE_TOO_LARGE,
);

const RANGE_TOO_SMALL_ERROR_TYPE: CommandErrorType<0> = CommandErrorType::new(
    translation::java::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL,
    translation::java::COMMANDS_RANDOM_ERROR_RANGE_TOO_SMALL,
);

struct RandomSequenceSuggestionProvider;

impl SuggestionProvider for RandomSequenceSuggestionProvider {
    fn suggest(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> SuggestionProviderResult {
        let sequences = context.server().random_sequences.lock().unwrap();
        for key in sequences.get_sequence_keys() {
            builder = builder.suggest(key);
        }
        builder.build()
    }
}

struct RandomExecutor {
    roll: bool,
    has_sequence: bool,
}

impl CommandExecutor for RandomExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let bounds = IntRangeArgumentType::get(context, ARG_RANGE)?;
        let min = bounds.min().unwrap_or(i32::MIN);
        let max = bounds.max().unwrap_or(i32::MAX);

        let span = i64::from(max) - i64::from(min);
        if span == 0 {
            return Err(RANGE_TOO_SMALL_ERROR_TYPE.create_without_context());
        }
        if span >= 2_147_483_647 {
            return Err(RANGE_TOO_LARGE_ERROR_TYPE.create_without_context());
        }

        let value = if self.has_sequence {
            let sequence = IdentifierArgumentType::get(context, ARG_SEQUENCE)?;
            let world_seed = context.server().level_info.load().world_gen_settings.seed;
            let mut sequences = context.server().random_sequences.lock().unwrap();
            sequences
                .get_or_create(&sequence, world_seed)
                .random_between_inclusive(min, max)
        } else {
            rand::random_range(min..=max)
        };

        if self.roll {
            let msg = TextComponent::translate_cross(
                translation::java::COMMANDS_RANDOM_ROLL,
                translation::java::COMMANDS_RANDOM_ROLL,
                [
                    context.source.display_name.clone(),
                    TextComponent::text(value.to_string()),
                    TextComponent::text(min.to_string()),
                    TextComponent::text(max.to_string()),
                ],
            );
            for player in context.server().get_all_players() {
                player.send_system_message(&msg);
            }
            if context.source.player_or_none().is_none() {
                context.source.send_message(msg);
            }
        } else {
            context.source.send_feedback(
                TextComponent::translate_cross(
                    translation::java::COMMANDS_RANDOM_SAMPLE_SUCCESS,
                    translation::java::COMMANDS_RANDOM_SAMPLE_SUCCESS,
                    [TextComponent::text(value.to_string())],
                ),
                false,
            );
        }

        Ok(value)
    }
}

struct ResetSequenceExecutor {
    has_seed: bool,
    has_include_world_seed: bool,
    has_include_sequence_id: bool,
}

impl CommandExecutor for ResetSequenceExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let sequence = IdentifierArgumentType::get(context, ARG_SEQUENCE)?;
        let world_seed = context.server().level_info.load().world_gen_settings.seed;
        let mut sequences = context.server().random_sequences.lock().unwrap();

        if self.has_seed {
            let seed = IntegerArgumentType::get(context, ARG_SEED)?;
            let include_world_seed = if self.has_include_world_seed {
                BoolArgumentType::get(context, ARG_INCLUDE_WORLD_SEED)?
            } else {
                true
            };
            let include_sequence_id = if self.has_include_sequence_id {
                BoolArgumentType::get(context, ARG_INCLUDE_SEQUENCE_ID)?
            } else {
                true
            };
            sequences.reset_with_options(
                &sequence,
                world_seed,
                seed,
                include_world_seed,
                include_sequence_id,
            );
        } else {
            sequences.reset(&sequence, world_seed);
        }

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_RANDOM_RESET_SUCCESS,
                translation::java::COMMANDS_RANDOM_RESET_SUCCESS,
                [TextComponent::text(sequence.to_string())],
            ),
            false,
        );

        Ok(1)
    }
}

struct ResetAllSequencesExecutor {
    has_seed: bool,
    has_include_world_seed: bool,
    has_include_sequence_id: bool,
}

impl CommandExecutor for ResetAllSequencesExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let mut sequences = context.server().random_sequences.lock().unwrap();

        if self.has_seed {
            let seed = IntegerArgumentType::get(context, ARG_SEED)?;
            let include_world_seed = if self.has_include_world_seed {
                BoolArgumentType::get(context, ARG_INCLUDE_WORLD_SEED)?
            } else {
                true
            };
            let include_sequence_id = if self.has_include_sequence_id {
                BoolArgumentType::get(context, ARG_INCLUDE_SEQUENCE_ID)?
            } else {
                true
            };
            sequences.set_seed_defaults(seed, include_world_seed, include_sequence_id);
        }

        let count = sequences.clear() as i32;

        context.source.send_feedback(
            TextComponent::translate_cross(
                translation::java::COMMANDS_RANDOM_RESET_ALL_SUCCESS,
                translation::java::COMMANDS_RANDOM_RESET_ALL_SUCCESS,
                [TextComponent::text(count.to_string())],
            ),
            false,
        );

        Ok(count)
    }
}

fn draw_random_value_tree(name: &'static str, announce: bool) -> LiteralArgumentBuilder {
    literal(name).then(
        argument(ARG_RANGE, IntRangeArgumentType)
            .executes(RandomExecutor {
                roll: announce,
                has_sequence: false,
            })
            .then(
                argument(ARG_SEQUENCE, IdentifierArgumentType)
                    .suggests(RandomSequenceSuggestionProvider)
                    .requires(PERMISSION_RESET)
                    .executes(RandomExecutor {
                        roll: announce,
                        has_sequence: true,
                    }),
            ),
    )
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Allow,
    ));

    registry.register_permission_or_panic(Permission::new(
        PERMISSION_RESET,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher.register(
        command("random", DESCRIPTION)
            .requires(PERMISSION)
            .then(draw_random_value_tree("value", false))
            .then(draw_random_value_tree("roll", true))
            .then(
                literal("reset")
                    .requires(PERMISSION_RESET)
                    .then(
                        literal("*")
                            .executes(ResetAllSequencesExecutor {
                                has_seed: false,
                                has_include_world_seed: false,
                                has_include_sequence_id: false,
                            })
                            .then(
                                argument(ARG_SEED, IntegerArgumentType::any())
                                    .executes(ResetAllSequencesExecutor {
                                        has_seed: true,
                                        has_include_world_seed: false,
                                        has_include_sequence_id: false,
                                    })
                                    .then(
                                        argument(ARG_INCLUDE_WORLD_SEED, BoolArgumentType)
                                            .executes(ResetAllSequencesExecutor {
                                                has_seed: true,
                                                has_include_world_seed: true,
                                                has_include_sequence_id: false,
                                            })
                                            .then(
                                                argument(ARG_INCLUDE_SEQUENCE_ID, BoolArgumentType)
                                                    .executes(ResetAllSequencesExecutor {
                                                        has_seed: true,
                                                        has_include_world_seed: true,
                                                        has_include_sequence_id: true,
                                                    }),
                                            ),
                                    ),
                            ),
                    )
                    .then(
                        argument(ARG_SEQUENCE, IdentifierArgumentType)
                            .suggests(RandomSequenceSuggestionProvider)
                            .executes(ResetSequenceExecutor {
                                has_seed: false,
                                has_include_world_seed: false,
                                has_include_sequence_id: false,
                            })
                            .then(
                                argument(ARG_SEED, IntegerArgumentType::any())
                                    .executes(ResetSequenceExecutor {
                                        has_seed: true,
                                        has_include_world_seed: false,
                                        has_include_sequence_id: false,
                                    })
                                    .then(
                                        argument(ARG_INCLUDE_WORLD_SEED, BoolArgumentType)
                                            .executes(ResetSequenceExecutor {
                                                has_seed: true,
                                                has_include_world_seed: true,
                                                has_include_sequence_id: false,
                                            })
                                            .then(
                                                argument(ARG_INCLUDE_SEQUENCE_ID, BoolArgumentType)
                                                    .executes(ResetSequenceExecutor {
                                                        has_seed: true,
                                                        has_include_world_seed: true,
                                                        has_include_sequence_id: true,
                                                    }),
                                            ),
                                    ),
                            ),
                    ),
            ),
    );
}
