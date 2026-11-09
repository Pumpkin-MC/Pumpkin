use std::borrow::Cow;
use std::sync::Arc;

use pumpkin_data::translation::java;
use pumpkin_gametest::{GameTestReporter, GameTestRetryOptions};
use pumpkin_protocol::java::client::play::{CGameTestRelativePos, SuggestionProviders};
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::{
    TextComponent, click::ClickEvent, color::NamedColor, hover::HoverEvent,
};
use tracing::info;

use crate::command::CommandSender;
use crate::command::CommandSource;
use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::core::bool::BoolArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use crate::block::entities::test_instance_block::TestInstanceBlockBlockEntity;
use crate::entity::EntityBase;
use crate::server::GameTestRunRequest;
use crate::server::server_test_manager::test_instance_placement;
use crate::world::POI_TYPE_TEST_INSTANCE;

const DESCRIPTION: &str = "Runs a GameTest test instance.";
const PERMISSION: &str = "minecraft:command.test";

const ARG_TESTS: &str = "tests";
const ARG_NUMBER_OF_TIMES: &str = "numberOfTimes";
const ARG_UNTIL_FAILED: &str = "untilFailed";
const ARG_ROTATION_STEPS: &str = "rotationSteps";
const ARG_TESTS_PER_ROW: &str = "testsPerRow";
const ARG_VAR: &str = "var";

const TEST_POS_Z_OFFSET_FROM_PLAYER: i32 = 3;
const TEST_GRID_SPACING: i32 = 64;
const DEFAULT_TESTS_PER_ROW: i32 = 8;
const TEST_NEARBY_SEARCH_RADIUS: i32 = 15;
const TEST_FULL_SEARCH_RADIUS: i32 = 250;
const TEST_POS_PICK_DISTANCE: f64 = 10.0;

const TEST_INSTANCE_REGISTRY: Identifier = Identifier::parse_static("minecraft:test_instance");

const TEST_NOT_FOUND_ERROR: CommandErrorType<2> = CommandErrorType::new(
    java::ARGUMENT_RESOURCE_SELECTOR_NOT_FOUND,
    java::ARGUMENT_RESOURCE_SELECTOR_NOT_FOUND,
);
const NO_TEST_CONTAINING_POS: CommandErrorType<3> = CommandErrorType::new(
    java::COMMANDS_TEST_ERROR_NO_TEST_CONTAINING_POS,
    java::COMMANDS_TEST_ERROR_NO_TEST_CONTAINING_POS,
);
const TEST_INSTANCE_NOT_FOUND: CommandErrorType<0> = CommandErrorType::new(
    java::COMMANDS_TEST_ERROR_TEST_INSTANCE_NOT_FOUND,
    java::COMMANDS_TEST_ERROR_TEST_INSTANCE_NOT_FOUND,
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

impl CommandExecutor for RunExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
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

        // Vanilla TestCommand::run stops the current tests before starting a new run.
        // Clear this server's active tests before the replacement requests are drained.
        context.server().stop_game_tests();

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

        let reporter: Arc<dyn GameTestReporter> = Arc::new(CommandTestReporter {
            sender: context.source.output.clone(),
        });

        let retry_options = GameTestRetryOptions::new(number_of_times, until_failed);
        let mut requests = Vec::with_capacity(selected.len());

        for (index, test_id) in selected.into_iter().enumerate() {
            let index = index as i32;
            let column = index % tests_per_row;
            let row = index / tests_per_row;
            let test_x = base_x + column * TEST_GRID_SPACING;
            let test_z = base_z + row * TEST_GRID_SPACING;

            requests.push(GameTestRunRequest::new(
                test_id.clone(),
                world.clone(),
                test_x,
                test_z,
                rotation_steps,
                retry_options,
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

        let _report = context.server().queue_game_test_batch(requests, reporter);

        Ok(1)
    }
}

struct StopExecutor;

impl CommandExecutor for StopExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        context.server().stop_game_tests();
        Ok(1)
    }
}

struct PosExecutor;

impl CommandExecutor for PosExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let player = context.source.player_or_err()?;
        let var_name = if context.arguments.contains_key(ARG_VAR) {
            StringArgumentType::get(context, ARG_VAR)?
        } else {
            "pos"
        };
        let world = context.world();
        let start = player.get_eye_pos();
        let direction = player.get_looking_vector();
        let end = pumpkin_util::math::vector3::Vector3::new(
            start.x + direction.x * TEST_POS_PICK_DISTANCE,
            start.y + direction.y * TEST_POS_PICK_DISTANCE,
            start.z + direction.z * TEST_POS_PICK_DISTANCE,
        );
        let target = world
            .ray_trace_block(start, end, false)
            .map_or_else(|| BlockPos::floored_v(end), |(position, _, _)| position);

        let mut containing = None;
        for radius in [TEST_NEARBY_SEARCH_RADIUS, TEST_FULL_SEARCH_RADIUS] {
            let candidates = world
                .portal_poi
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get_in_square(target, radius, Some(POI_TYPE_TEST_INSTANCE));
            containing = candidates.into_iter().find_map(|position| {
                let placement = test_instance_placement(context.server(), world, position).ok()?;
                let origin = placement.origin();
                let size = placement.size();
                let inside = target.0.x >= origin.0.x
                    && target.0.x < origin.0.x + size[0]
                    && target.0.y >= origin.0.y
                    && target.0.y < origin.0.y + size[1]
                    && target.0.z >= origin.0.z
                    && target.0.z < origin.0.z + size[2];
                inside.then_some((position, *origin))
            });
            if containing.is_some() {
                break;
            }
        }

        let Some((controller_pos, origin)) = containing else {
            return Err(NO_TEST_CONTAINING_POS.create_without_context(
                TextComponent::text(target.0.x.to_string()),
                TextComponent::text(target.0.y.to_string()),
                TextComponent::text(target.0.z.to_string()),
            ));
        };
        let entity = world
            .get_block_entity(&controller_pos)
            .ok_or_else(|| TEST_INSTANCE_NOT_FOUND.create_without_context())?;
        let controller = entity
            .as_any()
            .downcast_ref::<TestInstanceBlockBlockEntity>()
            .ok_or_else(|| TEST_INSTANCE_NOT_FOUND.create_without_context())?;
        let test_name = controller
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
            .and_then(|data| data.get_string("test"))
            .map_or_else(
                || TextComponent::translate_cross(
                    "test_instance_block.invalid_test",
                    "test_instance_block.invalid_test",
                    [],
                ),
                |name| TextComponent::text(name.to_string()),
            );
        let relative = BlockPos::new(
            target.0.x - origin.0.x,
            target.0.y - origin.0.y,
            target.0.z - origin.0.z,
        );
        let coords = TextComponent::translate_cross(
            java::COMMANDS_TEST_COORDINATES,
            java::COMMANDS_TEST_COORDINATES,
            [
                TextComponent::text(relative.0.x.to_string()),
                TextComponent::text(relative.0.y.to_string()),
                TextComponent::text(relative.0.z.to_string()),
            ],
        )
        .bold()
        .color_named(NamedColor::Green)
        .hover_event(HoverEvent::show_text(TextComponent::translate_cross(
            java::COMMANDS_TEST_COORDINATES_COPY,
            java::COMMANDS_TEST_COORDINATES_COPY,
            [],
        )))
        .click_event(ClickEvent::CopyToClipboard {
            value: Cow::Owned(format!(
                "final BlockPos {var_name} = new BlockPos({}, {}, {});",
                relative.0.x, relative.0.y, relative.0.z
            )),
        });
        context.source.send_feedback(
            TextComponent::translate_cross(
                java::COMMANDS_TEST_RELATIVE_POSITION,
                java::COMMANDS_TEST_RELATIVE_POSITION,
                [test_name, coords],
            ),
            false,
        );
        player.try_send_client_packet(&CGameTestRelativePos::new(target, relative));
        Ok(1)
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
                literal("pos").executes(PosExecutor).then(
                    argument(ARG_VAR, StringArgumentType::SingleWord).executes(PosExecutor),
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

fn resource_selector_matches(selector: &str, name: &str) -> bool {
    let selector = if selector.contains(':') {
        selector.to_string()
    } else {
        format!("minecraft:{selector}")
    };

    wildcard_match(selector.as_bytes(), name.as_bytes())
}

const fn wildcard_match(pattern: &[u8], value: &[u8]) -> bool {
    let (mut p, mut v) = (0usize, 0usize);
    let mut star = None;
    let mut retry_v = 0usize;

    while v < value.len() {
        if p < pattern.len() && (pattern[p] == b'?' || pattern[p] == value[v]) {
            p += 1;
            v += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star = Some(p);
            p += 1;
            retry_v = v;
        } else if let Some(star_index) = star {
            p = star_index + 1;
            retry_v += 1;
            v = retry_v;
        } else {
            return false;
        }
    }

    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }

    p == pattern.len()
}
