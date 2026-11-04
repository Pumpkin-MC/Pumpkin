use std::sync::Arc;

use pumpkin_data::translation::java;
use pumpkin_gametest::{
    GameTestBatchReport, GameTestReporter, GameTestRetryOptions, GameTestRotation,
    VERIFY_TEST_BATCH_SIZE, resource_selector_matches,
};
use pumpkin_protocol::java::client::play::SuggestionProviders;
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use tracing::info;

use crate::command::CommandSender;
use crate::command::CommandSource;
use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::core::bool::BoolArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use crate::entity::EntityBase;
use crate::server::server_test_manager::{
    GameTestQueueEntry, enqueue_game_test, enqueue_game_test_clear, enqueue_game_test_verification,
    stop_game_tests, test_instance_placement,
};

const DESCRIPTION: &str = "Runs a GameTest test instance.";
const PERMISSION: &str = "minecraft:command.test";

const ARG_TESTS: &str = "tests";
const ARG_NUMBER_OF_TIMES: &str = "numberOfTimes";
const ARG_UNTIL_FAILED: &str = "untilFailed";
const ARG_ROTATION_STEPS: &str = "rotationSteps";
const ARG_TESTS_PER_ROW: &str = "testsPerRow";
const ARG_RADIUS: &str = "radius";

const TEST_POS_Z_OFFSET_FROM_PLAYER: i32 = 3;
const TEST_GRID_SPACING: i32 = 64;
const DEFAULT_TESTS_PER_ROW: i32 = 8;
const VERIFY_TEST_GRID_AXIS_SIZE: i32 = 10;
const VERIFY_TEST_GRID_SPACING: i32 = 128;
const DEFAULT_CLEAR_RADIUS: i32 = 250;
const MAX_CLEAR_RADIUS: i32 = 1024;

const TEST_INSTANCE_REGISTRY: Identifier = Identifier::parse_static("minecraft:test_instance");

const TEST_NOT_FOUND_ERROR: CommandErrorType<2> = CommandErrorType::new(
    java::ARGUMENT_RESOURCE_SELECTOR_NOT_FOUND,
    java::ARGUMENT_RESOURCE_SELECTOR_NOT_FOUND,
);
const CLEAR_NO_TESTS: CommandErrorType<0> = CommandErrorType::new(
    java::COMMANDS_TEST_CLEAR_ERROR_NO_TESTS,
    java::COMMANDS_TEST_CLEAR_ERROR_NO_TESTS,
);

struct CommandTestReporter {
    sender: CommandSender,
}

impl GameTestReporter for CommandTestReporter {
    fn send_message(&self, message: TextComponent) {
        self.sender.send_message(message);
    }
}

/// `GameTest` selectors are resource-location-like, but may also contain `*` and `?`.
/// Keep the Java client parser as `ResourceLocation` and ask the server for completions,
/// matching the behavior of the old command implementation.
struct TestInstanceArgumentType;

impl ArgumentType<CommandSource> for TestInstanceArgumentType {
    type Item = String;

    fn parse(&self, reader: &mut StringReader) -> Result<Self::Item, CommandSyntaxError> {
        let start = reader.cursor();
        while let Some(character) = reader.peek() {
            if is_allowed_in_test_selector(character) {
                reader.skip();
            } else {
                break;
            }
        }

        Ok(reader.string()[start..reader.cursor()].to_string())
    }

    fn list_suggestions(
        &self,
        context: &CommandContext,
        mut builder: SuggestionsBuilder,
    ) -> Suggestions {
        let current = builder.remaining().to_string();

        for name in context.server().datapack_manager.get_test_instance_names() {
            if resource_suggestion_matches(&current, &name) {
                builder = builder.suggest(name);
            }
        }

        builder.build()
    }

    fn client_side_parser(&self) -> JavaClientArgumentType {
        JavaClientArgumentType::ResourceLocation
    }

    fn override_suggestion_providers(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }

    fn examples(&self) -> Vec<String> {
        vec![
            "minecraft:test".to_string(),
            "minecraft:*".to_string(),
            "*".to_string(),
        ]
    }
}

impl TestInstanceArgumentType {
    fn get<'a>(context: &'a CommandContext, name: &str) -> Result<&'a str, CommandSyntaxError> {
        Ok(context.get_argument::<String>(name)?.as_str())
    }
}

struct RunExecutor;

fn selected_test_ids(context: &CommandContext) -> Result<Vec<String>, CommandSyntaxError> {
    let selector = TestInstanceArgumentType::get(context, ARG_TESTS)?;
    let selected: Vec<_> = context
        .server()
        .datapack_manager
        .get_test_instance_names()
        .into_iter()
        .filter(|name| resource_selector_matches(selector, name))
        .collect();

    if selected.is_empty() {
        return Err(TEST_NOT_FOUND_ERROR.create_without_context(
            TextComponent::text(selector.to_string()),
            TextComponent::text(TEST_INSTANCE_REGISTRY.to_string()),
        ));
    }
    Ok(selected)
}

impl CommandExecutor for RunExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let selected = selected_test_ids(context)?;

        // Vanilla TestCommand::run always clears the current GameTestTicker first.
        // Without this, issuing another /test run with a different retry mode lets
        // two runners own the same controller/structure concurrently.
        stop_game_tests();

        let number_was_supplied = context.arguments.contains_key(ARG_NUMBER_OF_TIMES);
        let number_of_times = if number_was_supplied {
            IntegerArgumentType::get(context, ARG_NUMBER_OF_TIMES)?
        } else {
            1
        };

        let until_failed = if context.arguments.contains_key(ARG_UNTIL_FAILED) {
            BoolArgumentType::get(context, ARG_UNTIL_FAILED)?
        } else {
            // Vanilla RetryOptions.noRetries() is (1, true), while specifying
            // numberOfTimes without untilFailed defaults haltOnFailure to false.
            !number_was_supplied
        };

        let rotation_steps = if context.arguments.contains_key(ARG_ROTATION_STEPS) {
            IntegerArgumentType::get(context, ARG_ROTATION_STEPS)?
        } else {
            0
        };

        let tests_per_row = if context.arguments.contains_key(ARG_TESTS_PER_ROW) {
            IntegerArgumentType::get(context, ARG_TESTS_PER_ROW)?
        } else {
            DEFAULT_TESTS_PER_ROW
        }
        .max(1);

        let world = context.world().clone();
        let base_x = context.source.position.x.floor() as i32;
        let base_z = context.source.position.z.floor() as i32 + TEST_POS_Z_OFFSET_FROM_PLAYER;

        context
            .source
            .send_message(pumpkin_macros::translate_cross!(
                java::COMMANDS_TEST_RUN_RUNNING,
                java::COMMANDS_TEST_RUN_RUNNING,
                TextComponent::text(selected.len().to_string()),
            ));

        let report = Arc::new(GameTestBatchReport::new(
            Arc::new(CommandTestReporter {
                sender: context.source.output.clone(),
            }),
            selected.len(),
        ));

        let retry_options = GameTestRetryOptions::new(number_of_times, until_failed);

        for (index, test_id) in selected.into_iter().enumerate() {
            let index = index as i32;
            let column = index % tests_per_row;
            let row = index / tests_per_row;
            let test_x = base_x + column * TEST_GRID_SPACING;
            let test_z = base_z + row * TEST_GRID_SPACING;

            enqueue_game_test(GameTestQueueEntry::new(
                test_id.clone(),
                world.clone(),
                test_x,
                test_z,
                rotation_steps,
                retry_options,
                report.clone(),
            ));

            info!(
                target: "pumpkin::gametest",
                test = %test_id,
                test_x,
                test_z,
                number_of_times,
                until_failed,
                rotation_steps,
                "Queued GameTest request"
            );
        }

        Ok(1)
    }
}

struct VerifyExecutor;

impl CommandExecutor for VerifyExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let selected = selected_test_ids(context)?;
        stop_game_tests();

        let world = context.world().clone();
        let base_x = context.source.position.x.floor() as i32;
        let base_z = context.source.position.z.floor() as i32 + TEST_POS_Z_OFFSET_FROM_PLAYER;
        let total = selected.len() * GameTestRotation::ALL.len() * VERIFY_TEST_BATCH_SIZE;
        let report = Arc::new(GameTestBatchReport::new(
            Arc::new(CommandTestReporter {
                sender: context.source.output.clone(),
            }),
            total,
        ));

        let mut batches = Vec::with_capacity(selected.len() * GameTestRotation::ALL.len());
        for test_id in selected {
            for &rotation in &GameTestRotation::ALL {
                let mut batch = Vec::with_capacity(VERIFY_TEST_BATCH_SIZE);
                for index in 0..VERIFY_TEST_BATCH_SIZE {
                    let index = index as i32;
                    let column = index % VERIFY_TEST_GRID_AXIS_SIZE;
                    let row = index / VERIFY_TEST_GRID_AXIS_SIZE;
                    batch.push(GameTestQueueEntry::new(
                        test_id.clone(),
                        world.clone(),
                        base_x + column * VERIFY_TEST_GRID_SPACING,
                        base_z + row * VERIFY_TEST_GRID_SPACING,
                        rotation.steps(),
                        GameTestRetryOptions::no_retries(),
                        report.clone(),
                    ));
                }
                batches.push(batch);
            }
        }

        enqueue_game_test_verification(batches, report);
        Ok(1)
    }
}

struct StopExecutor;

impl CommandExecutor for StopExecutor {
    fn execute(&self, _context: &CommandContext) -> CommandExecutorResult {
        stop_game_tests();
        Ok(1)
    }
}

#[derive(Clone, Copy)]
enum ClearMode {
    That,
    These,
    All,
}

struct ClearExecutor(ClearMode);

fn ray_hits_test_bounds(
    start: pumpkin_util::math::vector3::Vector3<f64>,
    direction: pumpkin_util::math::vector3::Vector3<f64>,
    origin: BlockPos,
    size: [i32; 3],
) -> bool {
    let mut entry = 0.0f64;
    let mut exit = f64::from(DEFAULT_CLEAR_RADIUS);
    for (start, direction, min, max) in [
        (start.x, direction.x, origin.0.x, origin.0.x + size[0]),
        (start.y, direction.y, origin.0.y, origin.0.y + size[1]),
        (start.z, direction.z, origin.0.z, origin.0.z + size[2]),
    ] {
        if direction.abs() < f64::EPSILON {
            if start < f64::from(min) || start > f64::from(max) {
                return false;
            }
            continue;
        }
        let near = (f64::from(min) - start) / direction;
        let far = (f64::from(max) - start) / direction;
        entry = entry.max(near.min(far));
        exit = exit.min(near.max(far));
        if entry > exit {
            return false;
        }
    }
    true
}

impl CommandExecutor for ClearExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        stop_game_tests();
        let world = context.world().clone();
        let center = BlockPos::new(
            context.source.position.x.floor() as i32,
            context.source.position.y.floor() as i32,
            context.source.position.z.floor() as i32,
        );
        let radius = if context.arguments.contains_key(ARG_RADIUS) {
            IntegerArgumentType::get(context, ARG_RADIUS)?.clamp(0, MAX_CLEAR_RADIUS)
        } else {
            DEFAULT_CLEAR_RADIUS
        };
        let radius_squared = i64::from(radius) * i64::from(radius);
        let view = if matches!(self.0, ClearMode::That) {
            let player = context.source.player_or_err()?;
            Some((player.get_eye_pos(), player.get_looking_vector()))
        } else {
            None
        };
        let candidates: Vec<_> = world
            .block_entities
            .iter()
            .flat_map(|chunk| {
                chunk
                    .value()
                    .iter()
                    .filter_map(|(&position, entity)| {
                        entity
                            .as_any()
                            .is::<crate::block::entities::test_instance_block::TestInstanceBlockBlockEntity>()
                            .then_some(position)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        let mut tests = Vec::new();
        for position in candidates {
            let dx = i64::from(position.0.x) - i64::from(center.0.x);
            let dy = i64::from(position.0.y) - i64::from(center.0.y);
            let dz = i64::from(position.0.z) - i64::from(center.0.z);
            if dx * dx + dy * dy + dz * dz > radius_squared {
                continue;
            }
            if let Ok(placement) = test_instance_placement(context.server(), &world, position) {
                if let Some((start, direction)) = view
                    && !ray_hits_test_bounds(
                        start,
                        direction,
                        *placement.origin(),
                        placement.size(),
                    )
                {
                    continue;
                }
                tests.push(position);
            }
        }
        if matches!(self.0, ClearMode::That) {
            tests.sort_by_key(|position| {
                let dx = i64::from(position.0.x) - i64::from(center.0.x);
                let dy = i64::from(position.0.y) - i64::from(center.0.y);
                let dz = i64::from(position.0.z) - i64::from(center.0.z);
                dx * dx + dy * dy + dz * dz
            });
            tests.truncate(1);
        }
        if tests.is_empty() {
            return Err(CLEAR_NO_TESTS.create_without_context());
        }
        let count = tests.len();
        enqueue_game_test_clear(world, tests);
        context.source.send_feedback(
            TextComponent::translate_cross(
                java::COMMANDS_TEST_CLEAR_SUCCESS,
                java::COMMANDS_TEST_CLEAR_SUCCESS,
                [TextComponent::text(count.to_string())],
            ),
            true,
        );
        Ok(count as i32)
    }
}

pub fn register(dispatcher: &mut CommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    let tests_per_row =
        argument(ARG_TESTS_PER_ROW, IntegerArgumentType::any()).executes(RunExecutor);

    let rotation_steps = argument(ARG_ROTATION_STEPS, IntegerArgumentType::any())
        .executes(RunExecutor)
        .then(tests_per_row);

    let until_failed = argument(ARG_UNTIL_FAILED, BoolArgumentType)
        .executes(RunExecutor)
        .then(rotation_steps);

    let number_of_times = argument(ARG_NUMBER_OF_TIMES, IntegerArgumentType::with_min(0))
        .executes(RunExecutor)
        .then(until_failed);

    let tests = argument(ARG_TESTS, TestInstanceArgumentType)
        .executes(RunExecutor)
        .then(number_of_times);

    dispatcher.register(
        command("test", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("run").then(tests))
            .then(
                literal("verify")
                    .then(argument(ARG_TESTS, TestInstanceArgumentType).executes(VerifyExecutor)),
            )
            .then(literal("clearthat").executes(ClearExecutor(ClearMode::That)))
            .then(literal("clearthese").executes(ClearExecutor(ClearMode::These)))
            .then(
                literal("clearall")
                    .executes(ClearExecutor(ClearMode::All))
                    .then(
                        argument(ARG_RADIUS, IntegerArgumentType::any())
                            .executes(ClearExecutor(ClearMode::All)),
                    ),
            )
            .then(literal("stop").executes(StopExecutor)),
    );
}

const fn is_allowed_in_test_selector(character: char) -> bool {
    matches!(
        character,
        '0'..='9' | 'a'..='z' | '_' | '-' | '.' | '/' | ':' | '*' | '?'
    )
}

fn resource_suggestion_matches(input: &str, candidate: &str) -> bool {
    if input.is_empty() {
        return true;
    }

    let input = input.to_ascii_lowercase();
    let candidate = candidate.to_ascii_lowercase();

    if input.contains(':') {
        matches_suggestion_substring(&input, &candidate)
    } else if let Some((namespace, path)) = candidate.split_once(':') {
        matches_suggestion_substring(&input, namespace)
            || matches_suggestion_substring(&input, path)
    } else {
        matches_suggestion_substring(&input, &candidate)
    }
}

fn matches_suggestion_substring(pattern: &str, input: &str) -> bool {
    if input.starts_with(pattern) {
        return true;
    }

    input.char_indices().any(|(index, character)| {
        matches!(character, '.' | '_' | '/')
            && input[index + character.len_utf8()..].starts_with(pattern)
    })
}
