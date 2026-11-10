use std::borrow::Cow;
use std::collections::HashSet;
use std::sync::{Arc, Weak};

use pumpkin_data::{Block, translation::java};
use pumpkin_gametest::{
    GameTestBatchReport, GameTestReporter, GameTestRetryOptions, GameTestRotation, GameTestWorld,
    VERIFY_TEST_BATCH_SIZE, clear_structure_area, resource_selector_matches,
};
use pumpkin_nbt::{NbtCompound, tag::NbtTag};
use pumpkin_protocol::java::client::play::{CGameTestRelativePos, SuggestionProviders};
use pumpkin_util::PermissionLvl;
use pumpkin_util::identifier::Identifier;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::permission::{Permission, PermissionDefault, PermissionRegistry};
use pumpkin_util::text::{TextComponent, click::ClickEvent, color::NamedColor, hover::HoverEvent};
use pumpkin_world::world::BlockFlags;
use tracing::info;

use crate::block::entities::test_instance_block::{
    TestInstanceBlockBlockEntity, TestInstanceRotation,
};
use crate::command::CommandSender;
use crate::command::CommandSource;
use crate::command::argument_builder::{ArgumentBuilder, argument, command, literal};
use crate::command::argument_types::argument_type::{ArgumentType, JavaClientArgumentType};
use crate::command::argument_types::core::bool::BoolArgumentType;
use crate::command::argument_types::core::integer::IntegerArgumentType;
use crate::command::argument_types::core::string::StringArgumentType;
use crate::command::argument_types::identifier::IdentifierArgumentType;
use crate::command::context::command_context::CommandContext;
use crate::command::errors::command_syntax_error::CommandSyntaxError;
use crate::command::errors::error_types::CommandErrorType;
use crate::command::node::dispatcher::CommandDispatcher;
use crate::command::node::{CommandExecutor, CommandExecutorResult};
use crate::command::string_reader::StringReader;
use crate::command::suggestion::suggestions::{Suggestions, SuggestionsBuilder};
use crate::entity::EntityBase;
use crate::server::server_test_manager::ServerGameTestWorld;
use crate::server::server_test_manager::{
    GameTestQueueEntry, enqueue_game_test_clear, enqueue_game_test_verification,
    reset_test_instance, stop_game_tests, test_instance_placement,
};
use crate::server::{GameTestOutcome, GameTestOutcomeStatus, GameTestRunRequest, Server};
use crate::world::POI_TYPE_TEST_INSTANCE;
use crate::world::World;

const DESCRIPTION: &str = "Runs a GameTest test instance.";
const PERMISSION: &str = "minecraft:command.test";

const ARG_TESTS: &str = "tests";
const ARG_NUMBER_OF_TIMES: &str = "numberOfTimes";
const ARG_UNTIL_FAILED: &str = "untilFailed";
const ARG_ROTATION_STEPS: &str = "rotationSteps";
const ARG_TESTS_PER_ROW: &str = "testsPerRow";
const ARG_AMOUNT: &str = "amount";
const ARG_ONLY_REQUIRED_TESTS: &str = "onlyRequiredTests";
const ARG_ID: &str = "id";
const ARG_WIDTH: &str = "width";
const ARG_HEIGHT: &str = "height";
const ARG_DEPTH: &str = "depth";
const ARG_RADIUS: &str = "radius";
const ARG_VAR: &str = "var";

const TEST_POS_Z_OFFSET_FROM_PLAYER: i32 = 3;
const TEST_GRID_SPACING: i32 = 64;
const DEFAULT_TESTS_PER_ROW: i32 = 8;
const DEFAULT_CLEAR_RADIUS: i32 = 250;
const MAX_CLEAR_RADIUS: i32 = 1024;
const TEST_NEARBY_SEARCH_RADIUS: i32 = 15;
const TEST_FULL_SEARCH_RADIUS: i32 = 250;
const TEST_POS_PICK_DISTANCE: f64 = 10.0;
const DEFAULT_CREATE_SIZE: i32 = 5;
const MAX_CREATE_SIZE: i32 = 48;

const TEST_INSTANCE_REGISTRY: Identifier = Identifier::parse_static("minecraft:test_instance");

const TEST_NOT_FOUND_ERROR: CommandErrorType<2> = CommandErrorType::new(
    java::ARGUMENT_RESOURCE_SELECTOR_NOT_FOUND,
    java::ARGUMENT_RESOURCE_SELECTOR_NOT_FOUND,
);
const CLEAR_NO_TESTS: CommandErrorType<0> = CommandErrorType::new(
    java::COMMANDS_TEST_CLEAR_ERROR_NO_TESTS,
    java::COMMANDS_TEST_CLEAR_ERROR_NO_TESTS,
);
const RESET_NO_TESTS: CommandErrorType<0> = CommandErrorType::new(
    java::COMMANDS_TEST_RESET_ERROR_NO_TESTS,
    java::COMMANDS_TEST_RESET_ERROR_NO_TESTS,
);
const NO_TEST_CONTAINING_POS: CommandErrorType<3> = CommandErrorType::new(
    java::COMMANDS_TEST_ERROR_NO_TEST_CONTAINING_POS,
    java::COMMANDS_TEST_ERROR_NO_TEST_CONTAINING_POS,
);
const TEST_INSTANCE_NOT_FOUND: CommandErrorType<0> = CommandErrorType::new(
    java::COMMANDS_TEST_ERROR_TEST_INSTANCE_NOT_FOUND,
    java::COMMANDS_TEST_ERROR_TEST_INSTANCE_NOT_FOUND,
);
const NO_TEST_INSTANCES: CommandErrorType<0> = CommandErrorType::new(
    java::COMMANDS_TEST_ERROR_NO_TEST_INSTANCES,
    java::COMMANDS_TEST_ERROR_NO_TEST_INSTANCES,
);
const CREATE_TOO_LARGE: CommandErrorType<1> = CommandErrorType::new(
    java::COMMANDS_TEST_ERROR_TOO_LARGE,
    java::COMMANDS_TEST_ERROR_TOO_LARGE,
);

struct CommandTestReporter {
    sender: CommandSender,
    server: Weak<Server>,
}

impl GameTestReporter for CommandTestReporter {
    fn send_message(&self, message: TextComponent) {
        self.sender.send_message(message);
    }

    fn on_outcome(&self, outcome: &GameTestOutcome) {
        if outcome.status == GameTestOutcomeStatus::Failed
            && let Some(server) = self.server.upgrade()
        {
            server.record_failed_game_test(&outcome.id, outcome.required);
        }
    }
}

/// `GameTest` selectors may contain `*` and `?`, like vanilla's resource selector.
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
        JavaClientArgumentType::ResourceSelector {
            identifier: TEST_INSTANCE_REGISTRY,
        }
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

#[derive(Clone, Copy)]
enum RunPlacedMode {
    Closest,
    These,
    That,
}

struct RunPlacedExecutor(RunPlacedMode);

fn retry_options(context: &CommandContext) -> Result<(i32, bool), CommandSyntaxError> {
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
    Ok((number_of_times, until_failed))
}

fn nearby_test_positions(world: &World, center: BlockPos, radius: i32) -> Vec<BlockPos> {
    let radius_squared = i64::from(radius) * i64::from(radius);
    world
        .portal_poi
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get_in_square(center, radius, Some(POI_TYPE_TEST_INSTANCE))
        .into_iter()
        .filter(|position| {
            let dx = i64::from(position.0.x) - i64::from(center.0.x);
            let dy = i64::from(position.0.y) - i64::from(center.0.y);
            let dz = i64::from(position.0.z) - i64::from(center.0.z);
            dx * dx + dy * dy + dz * dz <= radius_squared
        })
        .collect()
}

fn selected_placed_positions(
    context: &CommandContext,
    mode: RunPlacedMode,
) -> Result<Vec<BlockPos>, CommandSyntaxError> {
    let center = BlockPos::floored_v(context.source.position);
    let world = context.world();
    Ok(match mode {
        RunPlacedMode::Closest => nearby_test_positions(world, center, TEST_NEARBY_SEARCH_RADIUS)
            .into_iter()
            .min_by_key(|position| {
                (i64::from(position.0.x) - i64::from(center.0.x)).abs()
                    + (i64::from(position.0.y) - i64::from(center.0.y)).abs()
                    + (i64::from(position.0.z) - i64::from(center.0.z)).abs()
            })
            .into_iter()
            .collect(),
        RunPlacedMode::These => nearby_test_positions(world, center, TEST_FULL_SEARCH_RADIUS),
        RunPlacedMode::That => {
            let player = context.source.player_or_err()?;
            let start = player.get_eye_pos();
            let direction = player.get_looking_vector();
            nearby_test_positions(world, center, TEST_FULL_SEARCH_RADIUS)
                .into_iter()
                .filter(|&position| {
                    test_instance_placement(context.server(), world, position)
                        .ok()
                        .and_then(|placement| {
                            ray_hits_test_bounds(
                                start,
                                direction,
                                *placement.origin(),
                                placement.size(),
                            )
                        })
                        .is_some()
                })
                .min_by_key(|position| {
                    let dx = i64::from(position.0.x) - i64::from(center.0.x);
                    let dy = i64::from(position.0.y) - i64::from(center.0.y);
                    let dz = i64::from(position.0.z) - i64::from(center.0.z);
                    dx * dx + dy * dy + dz * dz
                })
                .into_iter()
                .collect()
        }
    })
}

impl CommandExecutor for RunPlacedExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let world = context.world().clone();
        let positions = selected_placed_positions(context, self.0)?;

        stop_game_tests(context.server());
        let (number_of_times, until_failed) = retry_options(context)?;
        let mut requests = Vec::with_capacity(positions.len());
        for position in positions {
            let Some(entity) = world.get_block_entity(&position) else {
                continue;
            };
            let Some(controller) = entity
                .as_any()
                .downcast_ref::<TestInstanceBlockBlockEntity>()
            else {
                continue;
            };
            let (test_id, rotation, ignore_entities) = {
                let data = controller
                    .data
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let data = data.as_ref();
                (
                    data.and_then(|data| data.get_string("test"))
                        .map(str::to_owned),
                    data.and_then(|data| data.get_string("rotation"))
                        .and_then(TestInstanceRotation::from_serialized_name)
                        .unwrap_or_default(),
                    data.and_then(|data| data.get_bool("ignore_entities"))
                        .unwrap_or(false),
                )
            };
            let Some(test_id) = test_id else {
                continue;
            };
            if context
                .server()
                .datapack_manager
                .get_test_instance(&test_id)
                .is_none()
            {
                context.source.send_error(pumpkin_macros::translate_cross!(
                    java::COMMANDS_TEST_ERROR_NON_EXISTANT_TEST,
                    java::COMMANDS_TEST_ERROR_NON_EXISTANT_TEST,
                    TextComponent::text(test_id),
                ));
                continue;
            }
            let rotation_steps = match rotation {
                TestInstanceRotation::None => 0,
                TestInstanceRotation::Clockwise90 => 1,
                TestInstanceRotation::Clockwise180 => 2,
                TestInstanceRotation::Counterclockwise90 => 3,
            };
            requests.push(
                GameTestRunRequest::new(
                    test_id,
                    world.clone(),
                    position.0.x,
                    position.0.z,
                    rotation_steps,
                    GameTestRetryOptions::new(number_of_times, until_failed),
                )
                .with_y(position.0.y)
                .with_ignore_entities(ignore_entities),
            );
        }
        if requests.is_empty() {
            context.source.send_feedback(
                pumpkin_macros::translate_cross!(
                    java::COMMANDS_TEST_NO_TESTS,
                    java::COMMANDS_TEST_NO_TESTS,
                ),
                false,
            );
            return Ok(0);
        }
        context.server().forget_failed_game_tests();
        context.source.send_feedback(
            pumpkin_macros::translate_cross!(
                java::COMMANDS_TEST_RUN_RUNNING,
                java::COMMANDS_TEST_RUN_RUNNING,
                TextComponent::text(requests.len().to_string()),
            ),
            false,
        );
        let reporter: Arc<dyn GameTestReporter> = Arc::new(CommandTestReporter {
            sender: context.source.output.clone(),
            server: Arc::downgrade(context.server()),
        });
        context.server().queue_game_test_batch(requests, reporter);
        Ok(1)
    }
}

struct ResetExecutor(RunPlacedMode);

impl CommandExecutor for ResetExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let positions = selected_placed_positions(context, self.0)?;
        let world = context.world().clone();
        let positions: Vec<_> = positions
            .into_iter()
            .filter(|position| {
                world
                    .get_block_entity(position)
                    .and_then(|entity| {
                        entity
                            .as_any()
                            .downcast_ref::<TestInstanceBlockBlockEntity>()
                            .and_then(|controller| {
                                controller
                                    .data
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                                    .as_ref()
                                    .and_then(|data| data.get_string("test"))
                                    .map(str::to_owned)
                            })
                    })
                    .is_some_and(|id| {
                        context
                            .server()
                            .datapack_manager
                            .get_test_instance(&id)
                            .is_some()
                    })
            })
            .collect();
        if positions.is_empty() {
            return Err(RESET_NO_TESTS.create_without_context());
        }
        let count = positions.len() as i32;
        let controllers: Vec<_> = positions
            .into_iter()
            .filter_map(|position| {
                world.get_block_entity(&position).map(|entity| {
                    let mut nbt = NbtCompound::new();
                    entity.write_internal(&mut nbt);
                    (position, nbt)
                })
            })
            .collect();
        stop_game_tests(context.server());
        let server = context.server().clone();
        let source = (*context.source).clone();
        server.clone().spawn_task(async move {
            server.clear_running_game_tests().await;
            let mut reset = 0;
            let adapter = ServerGameTestWorld::new(world.clone());
            for (position, nbt) in controllers {
                let result = async {
                    if world.get_block_entity(&position).is_none() {
                        adapter
                            .set_block_state(
                                &position,
                                Block::TEST_INSTANCE_BLOCK.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await?;
                        adapter.set_block_entity_nbt(&position, &nbt).await?;
                    }
                    reset_test_instance(&server, world.clone(), position).await
                }
                .await;
                match result {
                    Ok(true) => reset += 1,
                    Ok(false) => {}
                    Err(error) => source.send_error(TextComponent::text(error.to_string())),
                }
            }
            if reset == 0 {
                source.send_error(pumpkin_macros::translate_cross!(
                    java::COMMANDS_TEST_RESET_ERROR_NO_TESTS,
                    java::COMMANDS_TEST_RESET_ERROR_NO_TESTS,
                ));
            } else {
                source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        java::COMMANDS_TEST_RESET_SUCCESS,
                        java::COMMANDS_TEST_RESET_SUCCESS,
                        TextComponent::text(reset.to_string()),
                    ),
                    true,
                );
            }
        });
        Ok(count)
    }
}

struct LocateExecutor;

impl CommandExecutor for LocateExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let selected: HashSet<_> = selected_test_ids(context)?.into_iter().collect();
        let center = BlockPos::floored_v(context.source.position);
        context.source.send_feedback(
            pumpkin_macros::translate_cross!(
                java::COMMANDS_TEST_LOCATE_STARTED,
                java::COMMANDS_TEST_LOCATE_STARTED,
            ),
            false,
        );
        let mut found = 0;
        for world in context.server().worlds.load().iter() {
            for position in nearby_test_positions(world, center, TEST_FULL_SEARCH_RADIUS) {
                let Some(entity) = world.get_block_entity(&position) else {
                    continue;
                };
                let Some(controller) = entity
                    .as_any()
                    .downcast_ref::<TestInstanceBlockBlockEntity>()
                else {
                    continue;
                };
                let data = controller
                    .data
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let Some(test_id) = data.as_ref().and_then(|data| data.get_string("test")) else {
                    continue;
                };
                if !selected.contains(test_id) {
                    continue;
                }
                let rotation = data
                    .as_ref()
                    .and_then(|data| data.get_string("rotation"))
                    .and_then(TestInstanceRotation::from_serialized_name)
                    .unwrap_or_default();
                drop(data);

                let (dx, dz, yaw) = match rotation {
                    TestInstanceRotation::None => (0, -3, 0),
                    TestInstanceRotation::Clockwise90 => (3, 0, 90),
                    TestInstanceRotation::Clockwise180 => (0, 3, 180),
                    TestInstanceRotation::Counterclockwise90 => (-3, 0, 270),
                };
                let dimension = &world.dimension.minecraft_name;
                let teleport = format!(
                    "/execute in {dimension} run tp @s {} {} {} {yaw} 0",
                    position.0.x + dx,
                    position.0.y,
                    position.0.z + dz,
                );
                let coordinates = pumpkin_macros::translate_cross!(
                    java::CHAT_COORDINATES,
                    java::CHAT_COORDINATES,
                    TextComponent::text(position.0.x.to_string()),
                    TextComponent::text(position.0.y.to_string()),
                    TextComponent::text(position.0.z.to_string()),
                )
                .add_child(TextComponent::text(format!(" ->[{dimension}]")))
                .color_named(NamedColor::Green)
                .click_event(ClickEvent::SuggestCommand {
                    command: Cow::Owned(teleport),
                });
                let distance = if dimension == &context.world().dimension.minecraft_name {
                    let dx = f64::from(position.0.x) - f64::from(center.0.x);
                    let dz = f64::from(position.0.z) - f64::from(center.0.z);
                    (dx.hypot(dz).floor() as i32).to_string()
                } else {
                    "N/A".to_string()
                };
                context.source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        java::COMMANDS_TEST_LOCATE_FOUND,
                        java::COMMANDS_TEST_LOCATE_FOUND,
                        coordinates,
                        TextComponent::text(distance),
                    ),
                    false,
                );
                found += 1;
            }
        }
        if found == 0 {
            return Err(NO_TEST_INSTANCES.create_without_context());
        }
        context.source.send_feedback(
            pumpkin_macros::translate_cross!(
                java::COMMANDS_TEST_LOCATE_DONE,
                java::COMMANDS_TEST_LOCATE_DONE,
                TextComponent::text(found.to_string()),
            ),
            true,
        );
        Ok(found)
    }
}

struct CreateExecutor;

impl CommandExecutor for CreateExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let id = IdentifierArgumentType::get(context, ARG_ID)?.to_string();
        let width = if context.arguments.contains_key(ARG_WIDTH) {
            IntegerArgumentType::get(context, ARG_WIDTH)?
        } else {
            DEFAULT_CREATE_SIZE
        };
        let height = if context.arguments.contains_key(ARG_HEIGHT) {
            IntegerArgumentType::get(context, ARG_HEIGHT)?
        } else {
            width
        };
        let depth = if context.arguments.contains_key(ARG_DEPTH) {
            IntegerArgumentType::get(context, ARG_DEPTH)?
        } else {
            width
        };
        if [width, height, depth]
            .iter()
            .any(|size| !(1..=MAX_CREATE_SIZE).contains(size))
        {
            return Err(CREATE_TOO_LARGE
                .create_without_context(TextComponent::text(MAX_CREATE_SIZE.to_string())));
        }

        let world = context.world().clone();
        let x = context.source.position.x.floor() as i32;
        let z = context.source.position.z.floor() as i32 + TEST_POS_Z_OFFSET_FROM_PLAYER;
        let source = (*context.source).clone();
        context.server().clone().spawn_task(async move {
            let adapter = ServerGameTestWorld::new(world);
            let result = async {
                let y = adapter.surface_height(x, z).await + 1;
                let controller = BlockPos::new(x, y, z);
                let origin = BlockPos::new(x, y + 1, z + 1);
                let max = BlockPos::new(x + width, y + height + 1, z + depth + 1);
                adapter.clear_non_player_entities(&origin, &max).await?;
                clear_structure_area(&adapter, &origin, [width, height, depth]).await?;
                adapter.clear_scheduled_block_ticks(&origin, &max).await?;
                adapter.clear_block_events(&origin, &max).await?;
                adapter
                    .set_block_state(
                        &controller,
                        Block::TEST_INSTANCE_BLOCK.default_state.id,
                        BlockFlags::NOTIFY_ALL,
                    )
                    .await?;
                let mut data = NbtCompound::new();
                data.put_string("test", id.clone());
                data.put("size", NbtTag::IntArray(vec![width, height, depth]));
                data.put_string("rotation", "none".to_string());
                data.put_bool("ignore_entities", false);
                data.put_string("status", "cleared".to_string());
                let mut nbt = NbtCompound::new();
                nbt.put_string("id", TestInstanceBlockBlockEntity::ID.to_string());
                nbt.put_compound("data", data);
                adapter.set_block_entity_nbt(&controller, &nbt).await?;
                for floor_x in origin.0.x..origin.0.x + width {
                    for floor_z in origin.0.z..origin.0.z + depth {
                        adapter
                            .set_block_state(
                                &BlockPos::new(floor_x, origin.0.y, floor_z),
                                Block::BEDROCK.default_state.id,
                                BlockFlags::NOTIFY_ALL,
                            )
                            .await?;
                    }
                }
                Ok::<(), pumpkin_gametest::GameTestError>(())
            }
            .await;
            match result {
                Ok(()) => source.send_feedback(
                    pumpkin_macros::translate_cross!(
                        java::COMMANDS_TEST_CREATE_SUCCESS,
                        java::COMMANDS_TEST_CREATE_SUCCESS,
                        TextComponent::text(id),
                    ),
                    true,
                ),
                Err(error) => source.send_error(TextComponent::text(error.to_string())),
            }
        });
        Ok(1)
    }
}

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
        let amount = if context.arguments.contains_key(ARG_AMOUNT) {
            IntegerArgumentType::get(context, ARG_AMOUNT)?.max(0) as usize
        } else {
            1
        };
        let selected = (0..amount).flat_map(|_| selected.iter().cloned()).collect();
        run_selected_tests(context, selected)
    }
}

struct RunFailedExecutor;

impl CommandExecutor for RunFailedExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let only_required = if context.arguments.contains_key(ARG_ONLY_REQUIRED_TESTS) {
            BoolArgumentType::get(context, ARG_ONLY_REQUIRED_TESTS)?
        } else {
            false
        };
        let selected = context.server().failed_game_tests(only_required);
        run_selected_tests(context, selected)
    }
}

fn run_selected_tests(context: &CommandContext, selected: Vec<String>) -> CommandExecutorResult {
    // Vanilla TestCommand::run stops the current tests before starting a new run.
    // Clear this server's active tests before the replacement requests are drained.
    context.server().stop_game_tests();

    if selected.is_empty() {
        context.source.send_feedback(
            pumpkin_macros::translate_cross!(
                java::COMMANDS_TEST_NO_TESTS,
                java::COMMANDS_TEST_NO_TESTS,
            ),
            false,
        );
        return Ok(0);
    }
    context.server().forget_failed_game_tests();

    let (number_of_times, until_failed) = retry_options(context)?;

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
        server: Arc::downgrade(context.server()),
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

struct VerifyExecutor;

impl CommandExecutor for VerifyExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        let selected = selected_test_ids(context)?;
        stop_game_tests(context.server());
        context.server().forget_failed_game_tests();

        let world = context.world().clone();
        let base_x = context.source.position.x.floor() as i32;
        let base_z = context.source.position.z.floor() as i32 + TEST_POS_Z_OFFSET_FROM_PLAYER;
        let total = selected.len() * GameTestRotation::ALL.len() * VERIFY_TEST_BATCH_SIZE;
        let report = Arc::new(GameTestBatchReport::new(
            Arc::new(CommandTestReporter {
                sender: context.source.output.clone(),
                server: Arc::downgrade(context.server()),
            }),
            total,
        ));

        let mut batches = Vec::with_capacity(selected.len() * GameTestRotation::ALL.len());
        for test_id in selected {
            for &rotation in &GameTestRotation::ALL {
                let mut batch = Vec::with_capacity(VERIFY_TEST_BATCH_SIZE);
                for _ in 0..VERIFY_TEST_BATCH_SIZE {
                    batch.push(GameTestQueueEntry::new(
                        test_id.clone(),
                        world.clone(),
                        base_x,
                        base_z,
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
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
        context.server().stop_game_tests();
        Ok(1)
    }
}

struct PosExecutor;

impl CommandExecutor for PosExecutor {
    #[expect(
        clippy::too_many_lines,
        reason = "position lookup and feedback share the selected test"
    )]
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
                || {
                    TextComponent::translate_cross(
                        "test_instance_block.invalid_test",
                        "test_instance_block.invalid_test",
                        [],
                    )
                },
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
) -> Option<f64> {
    let mut entry = 0.0f64;
    let mut exit = f64::from(DEFAULT_CLEAR_RADIUS);
    for (start, direction, min, max) in [
        (start.x, direction.x, origin.0.x, origin.0.x + size[0]),
        (start.y, direction.y, origin.0.y, origin.0.y + size[1]),
        (start.z, direction.z, origin.0.z, origin.0.z + size[2]),
    ] {
        if direction.abs() < f64::EPSILON {
            if start < f64::from(min) || start > f64::from(max) {
                return None;
            }
            continue;
        }
        let near = (f64::from(min) - start) / direction;
        let far = (f64::from(max) - start) / direction;
        entry = entry.max(near.min(far));
        exit = exit.min(near.max(far));
        if entry > exit {
            return None;
        }
    }
    Some(entry)
}

impl CommandExecutor for ClearExecutor {
    fn execute(&self, context: &CommandContext) -> CommandExecutorResult {
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
        let candidates = world
            .portal_poi
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get_in_square(center, radius, Some(POI_TYPE_TEST_INSTANCE));
        let mut tests = Vec::new();
        for position in candidates {
            let dx = i64::from(position.0.x) - i64::from(center.0.x);
            let dy = i64::from(position.0.y) - i64::from(center.0.y);
            let dz = i64::from(position.0.z) - i64::from(center.0.z);
            if dx * dx + dy * dy + dz * dz > radius_squared {
                continue;
            }
            let entry = if let Some((start, direction)) = view {
                let Ok(placement) = test_instance_placement(context.server(), &world, position)
                else {
                    continue;
                };
                let Some(entry) =
                    ray_hits_test_bounds(start, direction, *placement.origin(), placement.size())
                else {
                    continue;
                };
                entry
            } else {
                0.0
            };
            if let Some(entity) = world.get_block_entity(&position) {
                tests.push((position, entry, entity));
            }
        }
        if matches!(self.0, ClearMode::That) {
            tests.sort_by(|(_, left, _), (_, right, _)| left.total_cmp(right));
            tests.truncate(1);
        }
        if tests.is_empty() {
            return Err(CLEAR_NO_TESTS.create_without_context());
        }
        let count = tests.len();
        stop_game_tests(context.server());
        enqueue_game_test_clear(
            world,
            tests
                .into_iter()
                .map(|(position, _, entity)| (position, entity))
                .collect(),
            (*context.source).clone(),
        );
        Ok(count as i32)
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the vanilla test command has many argument branches"
)]
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

    let runmultiple = literal("runmultiple").then(
        argument(ARG_TESTS, TestInstanceArgumentType)
            .executes(RunExecutor)
            .then(argument(ARG_AMOUNT, IntegerArgumentType::any()).executes(RunExecutor)),
    );

    let runfailed = literal("runfailed")
        .executes(RunFailedExecutor)
        .then(
            argument(ARG_NUMBER_OF_TIMES, IntegerArgumentType::with_min(0))
                .executes(RunFailedExecutor)
                .then(
                    argument(ARG_UNTIL_FAILED, BoolArgumentType)
                        .executes(RunFailedExecutor)
                        .then(
                            argument(ARG_ROTATION_STEPS, IntegerArgumentType::any())
                                .executes(RunFailedExecutor)
                                .then(
                                    argument(ARG_TESTS_PER_ROW, IntegerArgumentType::any())
                                        .executes(RunFailedExecutor),
                                ),
                        ),
                ),
        )
        .then(
            argument(ARG_ONLY_REQUIRED_TESTS, BoolArgumentType)
                .executes(RunFailedExecutor)
                .then(
                    argument(ARG_NUMBER_OF_TIMES, IntegerArgumentType::with_min(0))
                        .executes(RunFailedExecutor)
                        .then(
                            argument(ARG_UNTIL_FAILED, BoolArgumentType)
                                .executes(RunFailedExecutor)
                                .then(
                                    argument(ARG_ROTATION_STEPS, IntegerArgumentType::any())
                                        .executes(RunFailedExecutor)
                                        .then(
                                            argument(ARG_TESTS_PER_ROW, IntegerArgumentType::any())
                                                .executes(RunFailedExecutor),
                                        ),
                                ),
                        ),
                ),
        );

    let runclosest = literal("runclosest")
        .executes(RunPlacedExecutor(RunPlacedMode::Closest))
        .then(
            argument(ARG_NUMBER_OF_TIMES, IntegerArgumentType::with_min(0))
                .executes(RunPlacedExecutor(RunPlacedMode::Closest))
                .then(
                    argument(ARG_UNTIL_FAILED, BoolArgumentType)
                        .executes(RunPlacedExecutor(RunPlacedMode::Closest)),
                ),
        );
    let runthese = literal("runthese")
        .executes(RunPlacedExecutor(RunPlacedMode::These))
        .then(
            argument(ARG_NUMBER_OF_TIMES, IntegerArgumentType::with_min(0))
                .executes(RunPlacedExecutor(RunPlacedMode::These))
                .then(
                    argument(ARG_UNTIL_FAILED, BoolArgumentType)
                        .executes(RunPlacedExecutor(RunPlacedMode::These)),
                ),
        );
    let runthat = literal("runthat")
        .executes(RunPlacedExecutor(RunPlacedMode::That))
        .then(
            argument(ARG_NUMBER_OF_TIMES, IntegerArgumentType::with_min(0))
                .executes(RunPlacedExecutor(RunPlacedMode::That))
                .then(
                    argument(ARG_UNTIL_FAILED, BoolArgumentType)
                        .executes(RunPlacedExecutor(RunPlacedMode::That)),
                ),
        );

    dispatcher.register(
        command("test", DESCRIPTION)
            .requires(PERMISSION)
            .then(literal("run").then(tests))
            .then(runmultiple)
            .then(runfailed)
            .then(runclosest)
            .then(runthese)
            .then(runthat)
            .then(literal("resetclosest").executes(ResetExecutor(RunPlacedMode::Closest)))
            .then(literal("resetthese").executes(ResetExecutor(RunPlacedMode::These)))
            .then(literal("resetthat").executes(ResetExecutor(RunPlacedMode::That)))
            .then(
                literal("verify")
                    .then(argument(ARG_TESTS, TestInstanceArgumentType).executes(VerifyExecutor)),
            )
            .then(
                literal("locate")
                    .then(argument(ARG_TESTS, TestInstanceArgumentType).executes(LocateExecutor)),
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
            .then(
                literal("pos")
                    .executes(PosExecutor)
                    .then(argument(ARG_VAR, StringArgumentType::SingleWord).executes(PosExecutor)),
            )
            .then(
                literal("create").then(
                    argument(ARG_ID, IdentifierArgumentType)
                        .executes(CreateExecutor)
                        .then(
                            argument(ARG_WIDTH, IntegerArgumentType::any())
                                .executes(CreateExecutor)
                                .then(
                                    argument(ARG_HEIGHT, IntegerArgumentType::any()).then(
                                        argument(ARG_DEPTH, IntegerArgumentType::any())
                                            .executes(CreateExecutor),
                                    ),
                                ),
                        ),
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

#[cfg(test)]
mod tests {
    use pumpkin_util::math::vector3::Vector3;

    use super::{BlockPos, ray_hits_test_bounds};

    #[test]
    fn ray_hit_distance_uses_bounds_entry() {
        let start = Vector3::new(0.5, 1.5, 0.5);
        let direction = Vector3::new(0.0, 0.0, 1.0);
        let farther = ray_hits_test_bounds(start, direction, BlockPos::new(0, 1, 8), [1, 1, 2]);
        let nearer = ray_hits_test_bounds(start, direction, BlockPos::new(0, 1, 4), [1, 1, 2]);

        assert_eq!(nearer, Some(3.5));
        assert_eq!(farther, Some(7.5));
        assert!(nearer < farther);
    }
}
