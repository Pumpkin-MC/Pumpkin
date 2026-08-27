use std::sync::Arc;

use futures::executor::block_on;
use pumpkin_protocol::java::client::play::{ArgumentType, CommandSuggestion, SuggestionProviders};
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::TextComponent;
use tracing::info;

use crate::command::args::bool::BoolArgConsumer;
use crate::command::args::bounded_num::BoundedNumArgumentConsumer;
use crate::command::args::{
    Arg, ArgumentConsumer, ConsumeResult, ConsumedArgs, FindArg, GetClientSideArgParser,
    SuggestResult,
};
use crate::command::node::dispatcher::CommandDispatcher as LegacyCommandDispatcher;
use crate::command::tree::builder::{argument, literal};
use crate::command::tree::{CommandTree, RawArgs};
use crate::command::{CommandError, CommandExecutor, CommandResult, CommandSender};
use crate::server::Server;
use crate::server::ticker::{
    GameTestBatchReport, GameTestRequest, GameTestRetryOptions, enqueue_game_test, stop_game_tests,
};

const NAMES: [&str; 1] = ["test"];
const DESCRIPTION: &str = "Runs a GameTest test instance.";
const PERMISSION: &str = "minecraft:command.test";
const ARG_TESTS: &str = "tests";
const ARG_NUMBER_OF_TIMES: &str = "numberOfTimes";
const ARG_UNTIL_FAILED: &str = "untilFailed";
const ARG_ROTATION_STEPS: &str = "rotationSteps";
const ARG_TESTS_PER_ROW: &str = "testsPerRow";
const TEST_POS_Z_OFFSET_FROM_PLAYER: i32 = 3;
const TEST_GRID_SPACING: i32 = 64;
const DEFAULT_TESTS_PER_ROW: i32 = 8;
const TEST_INSTANCE_REGISTRY: Identifier = Identifier::parse_static("minecraft:test_instance");

struct TestInstanceArgumentConsumer;

impl GetClientSideArgParser for TestInstanceArgumentConsumer {
    fn get_client_side_parser(&self) -> ArgumentType {
        // Pumpkin currently supplies these completions through ask_server. Use the
        // resource-location parser client-side so namespaced IDs such as
        // `pumpkin:creeper_should_run_from_cat` parse as one argument without making
        // the client validate them against its local dynamic registry snapshot.
        ArgumentType::ResourceLocation
    }

    fn get_client_side_suggestion_type_override(&self) -> Option<SuggestionProviders> {
        Some(SuggestionProviders::AskServer)
    }
}

impl ArgumentConsumer for TestInstanceArgumentConsumer {
    fn consume<'a>(
        &'a self,
        _sender: &'a CommandSender,
        server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> ConsumeResult<'a> {
        let selector = args.pop().map(|arg| arg.value)?;
        let names = server.datapack_manager.get_test_instance_names();
        names
            .iter()
            .any(|name| resource_selector_matches(selector, name))
            .then_some(Arg::Simple(selector))
    }

    fn suggest(&self, _sender: &CommandSender, server: &Server, input: &str) -> SuggestResult {
        let current = current_suggestion_token(input);
        let suggestions = server
            .datapack_manager
            .get_test_instance_names()
            .into_iter()
            .filter(|name| resource_suggestion_matches(current, name))
            .map(|name| CommandSuggestion::new(name, None))
            .collect();
        Ok(Some(suggestions))
    }
}

impl<'a> FindArg<'a> for TestInstanceArgumentConsumer {
    type Data = &'a str;

    fn find_arg(args: &'a ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        match args.get(name) {
            Some(Arg::Simple(value)) => Ok(value),
            _ => Err(CommandError::InvalidConsumption(Some(name.to_string()))),
        }
    }
}

struct RunExecutor;

impl CommandExecutor for RunExecutor {
    fn execute(
        &self,
        sender: &CommandSender,
        server: &Server,
        args: &ConsumedArgs,
    ) -> CommandResult {
        let selector = TestInstanceArgumentConsumer::find_arg(args, ARG_TESTS)?;
        let names = server.datapack_manager.get_test_instance_names();
        let selected: Vec<_> = names
            .into_iter()
            .filter(|name| resource_selector_matches(selector, name))
            .collect();
        if selected.is_empty() {
            return Err(CommandError::CommandFailed(TextComponent::translate_cross(
                "argument.resource_selector.not_found",
                "argument.resource_selector.not_found",
                [
                    TextComponent::text(selector.to_string()),
                    TextComponent::text(TEST_INSTANCE_REGISTRY.to_string()),
                ],
            )));
        }

        // Vanilla TestCommand::run always clears the current GameTestTicker first.
        // Without this, issuing another /test run with a different retry mode lets
        // two runners own the same controller/structure concurrently.
        block_on(stop_game_tests());

        let number_was_supplied = args.contains_key(ARG_NUMBER_OF_TIMES);
        let number_of_times = if number_was_supplied {
            BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_NUMBER_OF_TIMES)??
        } else {
            1
        };
        let until_failed = if args.contains_key(ARG_UNTIL_FAILED) {
            BoolArgConsumer::find_arg(args, ARG_UNTIL_FAILED)?
        } else {
            // Vanilla RetryOptions.noRetries() is (1, true), while specifying
            // numberOfTimes without untilFailed defaults haltOnFailure to false.
            !number_was_supplied
        };
        let rotation_steps = if args.contains_key(ARG_ROTATION_STEPS) {
            BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_ROTATION_STEPS)??
        } else {
            0
        };
        let tests_per_row = if args.contains_key(ARG_TESTS_PER_ROW) {
            BoundedNumArgumentConsumer::<i32>::find_arg(args, ARG_TESTS_PER_ROW)??
        } else {
            DEFAULT_TESTS_PER_ROW
        }
        .max(1);

        let world = sender
            .world_or_first(server)
            .ok_or(CommandError::InvalidRequirement)?;
        let (base_x, base_z) = sender.position().map_or_else(
            || {
                let level_info = world.level_info.load();
                (
                    level_info.spawn_x,
                    level_info.spawn_z + TEST_POS_Z_OFFSET_FROM_PLAYER,
                )
            },
            |source_pos| {
                (
                    source_pos.x.floor() as i32,
                    source_pos.z.floor() as i32 + TEST_POS_Z_OFFSET_FROM_PLAYER,
                )
            },
        );

        sender.send_message(TextComponent::translate_cross(
            "commands.test.run.running",
            "commands.test.run.running",
            [TextComponent::text(selected.len().to_string())],
        ));

        let report = Arc::new(GameTestBatchReport::new(sender.clone(), selected.len()));
        let retry_options = GameTestRetryOptions::new(number_of_times, until_failed);
        for (index, test_id) in selected.into_iter().enumerate() {
            let index = index as i32;
            let column = index % tests_per_row;
            let row = index / tests_per_row;
            let test_x = base_x + column * TEST_GRID_SPACING;
            let test_z = base_z + row * TEST_GRID_SPACING;
            block_on(enqueue_game_test(GameTestRequest::new(
                test_id.clone(),
                world.clone(),
                test_x,
                test_z,
                rotation_steps,
                retry_options,
                report.clone(),
            )));

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

struct StopExecutor;

impl CommandExecutor for StopExecutor {
    fn execute(
        &self,
        _sender: &CommandSender,
        _server: &Server,
        _args: &ConsumedArgs,
    ) -> CommandResult {
        block_on(stop_game_tests());
        Ok(1)
    }
}

pub fn init_command_tree() -> CommandTree {
    let tests_per_row =
        argument(ARG_TESTS_PER_ROW, BoundedNumArgumentConsumer::<i32>::new()).execute(RunExecutor);
    let rotation_steps = argument(ARG_ROTATION_STEPS, BoundedNumArgumentConsumer::<i32>::new())
        .execute(RunExecutor)
        .then(tests_per_row);
    let until_failed = argument(ARG_UNTIL_FAILED, BoolArgConsumer)
        .execute(RunExecutor)
        .then(rotation_steps);
    let number_of_times = argument(
        ARG_NUMBER_OF_TIMES,
        BoundedNumArgumentConsumer::<i32>::new().min(0),
    )
    .execute(RunExecutor)
    .then(until_failed);
    let tests = argument(ARG_TESTS, TestInstanceArgumentConsumer)
        .execute(RunExecutor)
        .then(number_of_times);

    CommandTree::new(NAMES, DESCRIPTION)
        .then(literal("run").then(tests))
        .then(literal("stop").execute(StopExecutor))
}

pub fn register(dispatcher: &mut LegacyCommandDispatcher, registry: &PermissionRegistry) {
    registry.register_permission_or_panic(Permission::new(
        PERMISSION,
        DESCRIPTION,
        PermissionDefault::Op(PermissionLvl::Two),
    ));

    dispatcher
        .fallback_dispatcher
        .register(init_command_tree(), PERMISSION);
}

fn current_suggestion_token(input: &str) -> &str {
    if input.chars().next_back().is_some_and(char::is_whitespace) {
        ""
    } else {
        input.rsplit(char::is_whitespace).next().unwrap_or("")
    }
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
