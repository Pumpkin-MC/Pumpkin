#[allow(clippy::wildcard_imports)]
use super::*;
use crate::block::entities::structure_block::parse_structure_identifier;
use crate::block::entities::test_instance_block::{
    MAX_TEST_STRUCTURE_SIZE, TestInstanceBlockBlockEntity, TestInstanceData, TestInstanceRotation,
    TestInstanceStatus, describe_test_instance, lookup_test_instance, resolve_test_structure_size,
};
use pumpkin_data::translation::java as translation;
use pumpkin_protocol::java::client::play::CTestInstanceBlockStatus;
use pumpkin_protocol::java::server::play::{
    TestInstanceBlockAction, TestInstanceBlockData,
    TestInstanceBlockStatus as PacketTestInstanceBlockStatus,
};
use pumpkin_util::text::color::NamedColor;

const fn status_from_packet(status: PacketTestInstanceBlockStatus) -> TestInstanceStatus {
    match status {
        PacketTestInstanceBlockStatus::Cleared => TestInstanceStatus::Cleared,
        PacketTestInstanceBlockStatus::Running => TestInstanceStatus::Running,
        PacketTestInstanceBlockStatus::Finished => TestInstanceStatus::Finished,
    }
}

fn data_from_packet(data: &TestInstanceBlockData<'_>) -> TestInstanceData {
    let test = data
        .test
        .and_then(|raw| parse_structure_identifier(raw).map(|(_, _, canonical)| canonical));
    let size = [
        data.size.x.0.clamp(0, MAX_TEST_STRUCTURE_SIZE),
        data.size.y.0.clamp(0, MAX_TEST_STRUCTURE_SIZE),
        data.size.z.0.clamp(0, MAX_TEST_STRUCTURE_SIZE),
    ];

    TestInstanceData {
        test,
        size,
        rotation: TestInstanceRotation::from_block_rotation(data.rotation),
        ignore_entities: data.ignore_entities,
        status: status_from_packet(data.status),
        error_message: data.error_message.clone(),
    }
}

impl JavaClient {
    pub fn handle_test_instance_block_action(
        &self,
        player: &Arc<Player>,
        packet: &STestInstanceBlockAction<'_>,
    ) {
        if !player.is_creative() || player.permission_lvl.load() < PermissionLvl::Two {
            return;
        }

        let pos = packet.pos;
        let world = player.world();
        let Some(block_entity) = world.get_block_entity(&pos) else {
            return;
        };
        let Some(test_instance_block) = block_entity
            .as_any()
            .downcast_ref::<TestInstanceBlockBlockEntity>()
        else {
            return;
        };

        if matches!(
            packet.action,
            TestInstanceBlockAction::Query | TestInstanceBlockAction::Init
        ) {
            let server = world.server.upgrade();
            let status = packet
                .data
                .test
                .and_then(|test_id| {
                    server
                        .as_deref()
                        .and_then(|srv| lookup_test_instance(srv, test_id))
                })
                .map_or_else(
                    || {
                        TextComponent::translate(
                            translation::TEST_INSTANCE_DESCRIPTION_NO_TEST,
                            Vec::new(),
                        )
                        .color_named(NamedColor::Red)
                    },
                    |def| describe_test_instance(&def),
                );

            let size = if packet.action == TestInstanceBlockAction::Query {
                packet.data.test.and_then(|test_id| {
                    server.as_deref().and_then(|srv| {
                        resolve_test_structure_size(&world, srv, test_id)
                            .map(|[sx, sy, sz]| [VarInt(sx), VarInt(sy), VarInt(sz)])
                    })
                })
            } else {
                None
            };

            self.try_send_packet(&CTestInstanceBlockStatus::new(status, size));
            return;
        }

        test_instance_block.set_data(&data_from_packet(&packet.data));

        match packet.action {
            TestInstanceBlockAction::Reset => {
                if let Some(feedback) = test_instance_block.reset_test(&world) {
                    player.send_system_message(&feedback);
                }
            }
            TestInstanceBlockAction::Save => {
                if let Err(err) = test_instance_block.save_test(&world) {
                    player.send_system_message(&err);
                }
            }
            TestInstanceBlockAction::Export => match test_instance_block.save_test(&world) {
                Ok(_) => {
                    player.send_system_message(
                        &TextComponent::text("Test structure exporting is disabled")
                            .color_named(NamedColor::Red),
                    );
                }
                Err(err) => {
                    player.send_system_message(&err);
                }
            },
            TestInstanceBlockAction::Run => {
                let feedback = test_instance_block.run_test(&world, player);
                player.send_system_message(&feedback);
            }
            TestInstanceBlockAction::Set
            | TestInstanceBlockAction::Init
            | TestInstanceBlockAction::Query => {}
        }

        test_instance_block.mark_dirty();
        world.update_block_entity(&block_entity);
        let state_id = world.get_block_state_id(&pos);
        world.register_block_change(pos, state_id);
    }
}
