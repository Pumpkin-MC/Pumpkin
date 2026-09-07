//! Exercises the death notification, living-state reset and Bedrock acknowledgement.
//! World respawn scheduling is independent; no network writer is started by the fixture.

use std::io::Cursor;
use std::sync::atomic::Ordering;

use pumpkin_protocol::Packet;
use pumpkin_protocol::bedrock::packet_decoder::BedrockBatchDecoder;
use pumpkin_protocol::bedrock::server::respawn::{RespawnState, SRespawn};
use pumpkin_protocol::codec::var_ulong::VarULong;
use pumpkin_protocol::serial::PacketRead;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::text::TextComponent;

use crate::entity::EntityBase;
use crate::net::bedrock::BedrockClient;
use crate::test_support::TestServer;

async fn respawn_packets(client: &BedrockClient) -> Vec<SRespawn> {
    let mut decoder = BedrockBatchDecoder::new();
    let mut respawns = Vec::new();
    for data in client.drain_outgoing_packets_for_test().await {
        let payload = decoder.get_packet_payload(data.to_vec()).await.unwrap();
        let mut batch = Cursor::new(payload);
        let packet = decoder.get_game_packet(&mut batch).unwrap();
        assert_eq!(batch.position() as usize, batch.get_ref().len());
        if packet.id == SRespawn::PACKET_ID {
            let mut payload = Cursor::new(packet.payload);
            respawns.push(SRespawn::read(&mut payload).unwrap());
            assert_eq!(payload.position() as usize, payload.get_ref().len());
        }
    }
    respawns
}

async fn acknowledgement_flow(reset_before_ack: bool) {
    let fixture = TestServer::new().await;
    let player = fixture.new_bedrock_player().await;
    let client = player.client.bedrock().unwrap();
    let mut request = SRespawn {
        position: Vector3::new(999.0, 999.0, 999.0),
        state: RespawnState::ClientReadyToSpawn,
        player_runtime_id: VarULong(player.entity_id() as u64),
    };

    // Healthy players cannot start an unsolicited death handshake.
    client.handle_respawn(&player, &request);
    assert!(respawn_packets(client).await.is_empty());
    assert!(!player.bedrock_respawn_ack_pending.load(Ordering::Acquire));

    for _ in 0..2 {
        // The second iteration checks that another death rearms the handshake.
        player.living_entity.health.store(0.0);
        player.living_entity.dead.store(true, Ordering::Relaxed);
        player.handle_killed(&TextComponent::text("test death"));
        let notifications = respawn_packets(client).await;
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].state, RespawnState::SearchingForSpawn);
        assert!(player.bedrock_respawn_ack_pending.load(Ordering::Acquire));

        // Server-only states must not consume the pending client acknowledgement.
        for state in [RespawnState::SearchingForSpawn, RespawnState::ReadyToSpawn] {
            request.state = state;
            client.handle_respawn(&player, &request);
        }
        assert!(respawn_packets(client).await.is_empty());
        assert!(player.bedrock_respawn_ack_pending.load(Ordering::Acquire));
        request.state = RespawnState::ClientReadyToSpawn;

        // This is the actual reset performed by World::respawn_player, after the
        // asynchronous PlayerAction::Respawn task has progressed far enough.
        if reset_before_ack {
            player.living_entity.reset_state();
        }
        player.get_entity().set_pos(Vector3::new(5.5, 72.0, -8.5));
        client.handle_respawn(&player, &request);
        let responses = respawn_packets(client).await;
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].state, RespawnState::ReadyToSpawn);
        assert_eq!(responses[0].player_runtime_id.0, player.entity_id() as u64);
        assert_eq!(
            responses[0].position,
            Vector3::new(5.5, 72.0 + player.get_entity().entity_type.eye_height, -8.5)
        );
        assert!(!player.bedrock_respawn_ack_pending.load(Ordering::Acquire));

        // Repeated client requests cannot emit extra acknowledgements, even if
        // the world respawn has not yet reset the dead player.
        client.handle_respawn(&player, &request);
        assert!(respawn_packets(client).await.is_empty());
        if !reset_before_ack {
            player.living_entity.reset_state();
        }
        assert!(!player.living_entity.dead.load(Ordering::Relaxed));
        assert!(player.living_entity.health.load() > 0.0);
        client.handle_respawn(&player, &request);
        assert!(respawn_packets(client).await.is_empty());
    }

    drop(player);
    fixture.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_acknowledges_respawn_before_health_reset_once_per_death() {
    acknowledgement_flow(false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn bedrock_acknowledges_respawn_after_health_reset_once_per_death() {
    acknowledgement_flow(true).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn restored_dead_spawn_notification_arms_respawn_acknowledgement() {
    let fixture = TestServer::new().await;
    let player = fixture.new_bedrock_player().await;
    let client = player.client.bedrock().unwrap();
    player.living_entity.health.store(0.0);
    // Loading saved health need not have called the live death handler.
    assert!(!player.living_entity.dead.load(Ordering::Relaxed));
    // This shared sender is invoked after initial chunks for a restored-dead player.
    player.send_bedrock_respawn_state(RespawnState::SearchingForSpawn);
    let notifications = respawn_packets(client).await;
    assert_eq!(notifications.len(), 1);
    assert_eq!(notifications[0].state, RespawnState::SearchingForSpawn);
    player.living_entity.reset_state();
    client.handle_respawn(
        &player,
        &SRespawn {
            position: Vector3::default(),
            state: RespawnState::ClientReadyToSpawn,
            player_runtime_id: VarULong(player.entity_id() as u64),
        },
    );
    let responses = respawn_packets(client).await;
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0].state, RespawnState::ReadyToSpawn);
    assert!(!player.bedrock_respawn_ack_pending.load(Ordering::Acquire));
    drop(player);
    fixture.shutdown().await;
}
