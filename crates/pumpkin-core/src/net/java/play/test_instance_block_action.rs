#[allow(clippy::wildcard_imports)]
use super::*;
use pumpkin_gametest::{
    GameTestBatchReport, GameTestDefinition, GameTestReporter, GameTestRetryOptions,
    GameTestStructureTemplate, TestType,
};
use pumpkin_nbt::{NbtCompound, tag::NbtTag};
use pumpkin_protocol::java::client::play::CTestInstanceBlockStatus;
use pumpkin_protocol::java::server::play::TestInstanceBlockAction;
use pumpkin_util::text::color::NamedColor;

fn description_row(key: &'static str, value: TextComponent) -> TextComponent {
    TextComponent::translate_cross(key, key, [value.color_named(NamedColor::Blue)])
        .add_child(TextComponent::text("\n"))
}

fn test_instance_description(test: &GameTestDefinition) -> TextComponent {
    let kind = match test.instance_type {
        TestType::BlockBased => pumpkin_data::translation::java::TEST_INSTANCE_TYPE_BLOCK_BASED,
        TestType::Function => pumpkin_data::translation::java::TEST_INSTANCE_TYPE_FUNCTION,
    };
    let mut description = description_row(
        pumpkin_data::translation::java::TEST_INSTANCE_DESCRIPTION_TYPE,
        TextComponent::translate_cross(kind, kind, []),
    );
    if test.instance_type == TestType::Function
        && let Some(function) = &test.function
    {
        description = description.add_child(description_row(
            pumpkin_data::translation::java::TEST_INSTANCE_DESCRIPTION_FUNCTION,
            TextComponent::text(function.clone()),
        ));
    }
    description
        .add_child(description_row(
            pumpkin_data::translation::java::TEST_INSTANCE_DESCRIPTION_STRUCTURE,
            TextComponent::text(test.structure.clone()),
        ))
        .add_child(description_row(
            pumpkin_data::translation::java::TEST_INSTANCE_DESCRIPTION_BATCH,
            TextComponent::text(test.environment.as_str().unwrap_or_default().to_string()),
        ))
        .add_child(description_row(
            pumpkin_data::translation::java::TEST_INSTANCE_DESCRIPTION_DIMENSION,
            TextComponent::text("minecraft:overworld"),
        ))
}

use crate::block::entities::test_instance_block::{
    TestInstanceBlockBlockEntity, TestInstanceRotation, TestInstanceStatus,
};
use crate::server::server_test_manager::{
    GameTestQueueEntry, enqueue_game_test, reset_test_instance, stop_game_tests,
};
use crate::server::structure_editor::capture_structure;

struct TestInstancePlayerReporter(Arc<Player>);

impl GameTestReporter for TestInstancePlayerReporter {
    fn send_message(&self, message: TextComponent) {
        self.0.send_system_message(&message);
    }
}

impl JavaClient {
    #[expect(
        clippy::too_many_lines,
        reason = "test editor actions share packet state and validation"
    )]
    pub fn handle_test_instance_block_action(
        &self,
        player: &Arc<Player>,
        packet: &STestInstanceBlockAction<'_>,
    ) {
        if !player.has_client_loaded()
            || !player.is_creative()
            || player.permission_lvl.load() < PermissionLvl::Two
        {
            return;
        }
        player.update_last_action_time();
        let world = player.world();
        let Some(entity) = world.get_block_entity(&packet.pos) else {
            return;
        };
        let Some(controller) = entity
            .as_any()
            .downcast_ref::<TestInstanceBlockBlockEntity>()
        else {
            return;
        };
        let Some(server) = world.server.upgrade() else {
            return;
        };

        if matches!(
            packet.action,
            TestInstanceBlockAction::Query | TestInstanceBlockAction::Init
        ) {
            let test = packet
                .data
                .test
                .and_then(|id| server.datapack_manager.get_test_instance(id));
            let status = test.as_ref().map_or_else(
                || {
                    TextComponent::translate_cross(
                        pumpkin_data::translation::java::TEST_INSTANCE_DESCRIPTION_NO_TEST,
                        pumpkin_data::translation::java::TEST_INSTANCE_DESCRIPTION_NO_TEST,
                        [],
                    )
                    .color_named(NamedColor::Red)
                },
                test_instance_description,
            );
            let query = matches!(packet.action, TestInstanceBlockAction::Query);
            let player = player.clone();
            server.clone().spawn_task(async move {
                let size = if query {
                    if let Some(test) = test {
                        server
                            .datapack_manager
                            .load_structure(&test.structure)
                            .await
                            .ok()
                            .and_then(|nbt| GameTestStructureTemplate::from_nbt(&nbt).ok())
                            .map(|template| template.size())
                    } else {
                        None
                    }
                } else {
                    None
                };
                player
                    .send_client_packet(&CTestInstanceBlockStatus::new(status, size))
                    .await;
            });
            return;
        }

        let rotation = match packet.data.rotation {
            pumpkin_data::block_rotation::Rotation::None => TestInstanceRotation::None,
            pumpkin_data::block_rotation::Rotation::Clockwise90 => {
                TestInstanceRotation::Clockwise90
            }
            pumpkin_data::block_rotation::Rotation::Rotate180 => TestInstanceRotation::Clockwise180,
            pumpkin_data::block_rotation::Rotation::CounterClockwise90 => {
                TestInstanceRotation::Counterclockwise90
            }
        };
        let size = [
            packet.data.size.x.0,
            packet.data.size.y.0,
            packet.data.size.z.0,
        ];
        if size.iter().any(|axis| !(0..=256).contains(axis)) {
            return;
        }
        let mut data = NbtCompound::new();
        if let Some(test) = packet.data.test {
            data.put_string("test", test.to_string());
        }
        data.put("size", NbtTag::IntArray(size.to_vec()));
        data.put_string("rotation", rotation.serialized_name().to_string());
        data.put_bool("ignore_entities", packet.data.ignore_entities);
        let status = match packet.data.status {
            pumpkin_protocol::java::server::play::TestInstanceBlockStatus::Cleared => {
                TestInstanceStatus::Cleared
            }
            pumpkin_protocol::java::server::play::TestInstanceBlockStatus::Running => {
                TestInstanceStatus::Running
            }
            pumpkin_protocol::java::server::play::TestInstanceBlockStatus::Success
            | pumpkin_protocol::java::server::play::TestInstanceBlockStatus::Failed => {
                TestInstanceStatus::Finished
            }
        };
        data.put_string("status", status.serialized_name().to_string());
        if let Some(error) = packet.data.error_message {
            data.put_string("error_message", error.to_string());
        }
        *controller
            .data
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(data);
        world.update_block_entity(&entity);

        let test_id = packet.data.test.map(str::to_owned);
        match packet.action {
            TestInstanceBlockAction::Run => {
                if let Some(test_id) =
                    test_id.filter(|id| server.datapack_manager.get_test_instance(id).is_some())
                {
                    let player = player.clone();
                    let position = packet.pos;
                    let ignore_entities = packet.data.ignore_entities;
                    server.clone().spawn_task(async move {
                        match reset_test_instance(&server, world.clone(), position).await {
                            Ok(true) => {
                                stop_game_tests(&server);
                                player.send_system_message(&TextComponent::translate_cross(
                                    pumpkin_data::translation::java::TEST_INSTANCE_BLOCK_STARTING,
                                    pumpkin_data::translation::java::TEST_INSTANCE_BLOCK_STARTING,
                                    [TextComponent::text(test_id.clone())],
                                ));
                                let report = Arc::new(GameTestBatchReport::new(
                                    Arc::new(TestInstancePlayerReporter(player.clone())),
                                    1,
                                ));
                                enqueue_game_test(
                                    GameTestQueueEntry::new(
                                        test_id,
                                        world,
                                        position.0.x,
                                        position.0.z,
                                        rotation as i32,
                                        GameTestRetryOptions::no_retries(),
                                        report,
                                    )
                                    .with_y(position.0.y)
                                    .with_ignore_entities(ignore_entities),
                                );
                            }
                            Ok(false) => player.send_system_message(
                                &TextComponent::translate_cross(
                                    pumpkin_data::translation::java::TEST_INSTANCE_BLOCK_ERROR_NO_TEST_STRUCTURE,
                                    pumpkin_data::translation::java::TEST_INSTANCE_BLOCK_ERROR_NO_TEST_STRUCTURE,
                                    [
                                        TextComponent::text(position.0.x.to_string()),
                                        TextComponent::text(position.0.y.to_string()),
                                        TextComponent::text(position.0.z.to_string()),
                                    ],
                                )
                                .color_named(NamedColor::Red),
                            ),
                            Err(error) => player
                                .send_system_message(&TextComponent::text(error.to_string())),
                        }
                    });
                } else {
                    player.send_system_message(
                        &TextComponent::translate_cross(
                            pumpkin_data::translation::java::TEST_INSTANCE_BLOCK_ERROR_NO_TEST,
                            pumpkin_data::translation::java::TEST_INSTANCE_BLOCK_ERROR_NO_TEST,
                            [
                                TextComponent::text(packet.pos.0.x.to_string()),
                                TextComponent::text(packet.pos.0.y.to_string()),
                                TextComponent::text(packet.pos.0.z.to_string()),
                            ],
                        )
                        .color_named(NamedColor::Red),
                    );
                }
            }
            TestInstanceBlockAction::Save | TestInstanceBlockAction::Export => {
                let export = matches!(packet.action, TestInstanceBlockAction::Export);
                let name = test_id
                    .as_ref()
                    .and_then(|id| {
                        server
                            .datapack_manager
                            .get_test_instance(id)
                            .map(|test| test.structure)
                    })
                    .or(test_id);
                let Some(name) = name else {
                    player.send_system_message(&TextComponent::text("No test instance selected"));
                    return;
                };
                let padding = packet
                    .data
                    .test
                    .and_then(|id| server.datapack_manager.get_test_instance(id))
                    .map_or(0, |test| test.padding);
                let origin = BlockPos::new(
                    packet.pos.0.x + padding,
                    packet.pos.0.y + padding + 1,
                    packet.pos.0.z + padding + 1,
                );
                let include_entities = !packet.data.ignore_entities;
                let player = player.clone();
                server.clone().spawn_task(async move {
                    let result =
                        match capture_structure(&world, origin, size, include_entities, true).await
                        {
                            Ok(nbt) => server
                                .datapack_manager
                                .save_generated_structure(&name, nbt)
                                .await
                                .map(|_| ()),
                            Err(error) => Err(error),
                        };
                    if let Err(error) = result {
                        player.send_system_message(&TextComponent::text(format!(
                            "Failed to save test structure '{name}': {error}"
                        )));
                    } else if export {
                        player.send_system_message(&TextComponent::text(
                            "Test structure exporting is disabled",
                        ));
                    }
                });
            }
            TestInstanceBlockAction::Reset => {
                let player = player.clone();
                let position = packet.pos;
                let test_name = test_id.unwrap_or_default();
                server.clone().spawn_task(async move {
                    match reset_test_instance(&server, world, position).await {
                        Ok(true) => {
                            player.send_system_message(
                                &TextComponent::translate_cross(
                                    pumpkin_data::translation::java::TEST_INSTANCE_BLOCK_RESET_SUCCESS,
                                    pumpkin_data::translation::java::TEST_INSTANCE_BLOCK_RESET_SUCCESS,
                                    [TextComponent::text(test_name)],
                                )
                                .color_named(NamedColor::Green),
                            );
                        }
                        Ok(false) => {}
                        Err(error) => {
                            player.send_system_message(&TextComponent::text(error.to_string()));
                        }
                    }
                });
            }
            TestInstanceBlockAction::Set => {}
            TestInstanceBlockAction::Query | TestInstanceBlockAction::Init => (),
        }
    }
}
