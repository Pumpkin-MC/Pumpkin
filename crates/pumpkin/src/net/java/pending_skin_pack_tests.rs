use std::sync::atomic::Ordering;
use std::time::Duration;

use pumpkin_protocol::{
    KnownPack,
    bedrock::client::Skin,
    codec::var_int::VarInt,
    java::client::{
        config::{CConfigAddResourcePack, CConfigRemoveResourcePack, CFinishConfig, CKnownPacks},
        play::CAddResourcePack,
    },
    ser::NetworkReadExt,
};
use tokio::net::TcpListener;

use super::*;
use crate::net::ClientPlatform;
use crate::net::bedrock::skin_pack::BedrockSkinPack;
use crate::test_support::TestServer;

type Peer = TCPNetworkDecoder<BufReader<TcpStream>>;

async fn connection() -> (PendingConnection, Peer) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (peer, accepted) = tokio::join!(
        TcpStream::connect(listener.local_addr().unwrap()),
        listener.accept(),
    );
    let (socket, address) = accepted.unwrap();
    let mut pending = PendingConnection::new(
        socket,
        address,
        1,
        PacketRateLimiter::new(false, 100.0, 100.0),
    );
    pending.server_address = "127.0.0.1".to_string();
    pending.version.store(JavaMinecraftVersion::V_26_2);
    pending.connection_state.store(ConnectionState::Config);
    pending.gameprofile = Some(GameProfile {
        id: uuid::Uuid::new_v4(),
        name: "SkinPackTest".to_string(),
        properties: arc_swap::ArcSwap::from_pointee(Vec::new()),
        profile_actions: None,
    });
    (
        pending,
        TCPNetworkDecoder::new(BufReader::new(peer.unwrap())),
    )
}

async fn revision(fixture: &TestServer, marker: u8) -> Arc<BedrockSkinPack> {
    let mut skin = Skin::steve();
    skin.skin_data[0] = marker;
    fixture
        .server
        .bedrock_skin_packs
        .register(uuid::Uuid::from_u128(7), &skin)
        .await
        .unwrap();
    let pack = fixture.server.bedrock_skin_packs.current().await.unwrap();
    // Initial connections are not yet among these broadcast recipients.
    fixture.server.push_bedrock_skin_pack(pack.clone()).await;
    pack
}

async fn receive(peer: &mut Peer) -> RawPacket {
    tokio::time::timeout(Duration::from_secs(5), peer.get_raw_packet())
        .await
        .expect("configuration packet must be sent")
        .unwrap()
}

async fn expect_packet(peer: &mut Peer, packet: &impl ClientPacket) {
    let mut expected = Vec::new();
    JavaClient::write_packet_for_version(packet, JavaMinecraftVersion::V_26_2, &mut expected)
        .unwrap();
    let mut expected = expected.as_slice();
    let id = expected.get_var_int().unwrap().0;
    let actual = receive(peer).await;
    assert_eq!(actual.id, id);
    assert_eq!(actual.payload.as_ref(), expected);
}

async fn expect_no_packet(peer: &mut Peer) {
    assert!(
        tokio::time::timeout(Duration::from_millis(20), peer.get_raw_packet())
            .await
            .is_err(),
        "an ignored response must not advance configuration"
    );
}

fn pack_url(fixture: &TestServer, pack: &BedrockSkinPack) -> String {
    crate::net::bedrock::skin_pack::resource_url(
        "127.0.0.1",
        fixture
            .server
            .advanced_config
            .networking
            .bedrock
            .nethernet
            .address
            .port(),
        fixture
            .server
            .advanced_config
            .networking
            .bedrock
            .skins
            .resource_pack_url
            .as_deref(),
        pack.id,
    )
}

async fn expect_offer(peer: &mut Peer, fixture: &TestServer, pack: &BedrockSkinPack) {
    expect_packet(
        peer,
        &CConfigAddResourcePack::new(&pack.id, &pack_url(fixture, pack), &pack.hash, false, None),
    )
    .await;
}

async fn expect_known_packs(peer: &mut Peer) {
    let version = JavaMinecraftVersion::V_26_2.to_string();
    expect_packet(
        peer,
        &CKnownPacks::new(&[KnownPack {
            namespace: "minecraft",
            id: "core",
            version: &version,
        }]),
    )
    .await;
}

async fn reply(pending: &mut PendingConnection, server: &Server, id: uuid::Uuid, status: i32) {
    pending
        .handle_resource_pack_response(
            server,
            SConfigResourcePack {
                uuid: id,
                result: VarInt(status),
            },
        )
        .await;
}

fn config_packet(id: i32) -> RawPacket {
    RawPacket {
        id,
        payload: Bytes::new(),
    }
}

async fn finish_and_reconcile(
    mut pending: PendingConnection,
    peer: &mut Peer,
    fixture: &TestServer,
    expected_play_offer: Option<&BedrockSkinPack>,
) {
    let known = config_packet(SKnownPacks::to_id(JavaMinecraftVersion::V_26_2));
    let (result, ()) = tokio::join!(
        pending.handle_config_packet(&fixture.server, &known),
        async {
            while receive(peer).await.id != CFinishConfig::to_id(JavaMinecraftVersion::V_26_2) {}
        },
    );
    assert!(result.unwrap().is_none());
    let result = pending
        .handle_config_packet(
            &fixture.server,
            &config_packet(SAcknowledgeFinishConfig::to_id(
                JavaMinecraftVersion::V_26_2,
            )),
        )
        .await
        .unwrap();
    let Some(PacketHandlerResult::ReadyToPlay(profile, config)) = result else {
        panic!("configuration must finish after the offered packs settle");
    };
    let mut client = JavaClient::from_pending(pending, profile.clone(), config.clone());
    let mut outbound = client.outgoing_packet_queue_recv.take().unwrap();
    let (player, _) = fixture
        .server
        .add_player(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            Some(config),
        )
        .unwrap();
    let client = player.client.java().unwrap();
    client.set_player(player.clone());
    // Publication alone must not allow a Play resource-pack offer before CLogin.
    let latest = fixture.server.bedrock_skin_packs.current().await.unwrap();
    fixture.server.push_bedrock_skin_pack(latest).await;
    assert!(outbound.try_recv().is_err());
    assert!(client.pending_bedrock_skin_pack.lock().await.is_none());
    fixture.world.start_bedrock_player_tracking(&player);
    // This is the post-spawn handoff hook: the player is now published, so a
    // revision cannot be missed by both this catch-up and future broadcasts.
    client.reconcile_bedrock_skin_pack(&fixture.server).await;
    if let Some(pack) = expected_play_offer {
        let expected = client
            .serialize_packet(&CAddResourcePack::new(
                &pack.id,
                &pack_url(fixture, pack),
                &pack.hash,
                false,
                None,
            ))
            .unwrap();
        assert_eq!(outbound.try_recv().unwrap().data, expected);
        assert_eq!(
            client
                .pending_bedrock_skin_pack
                .lock()
                .await
                .as_ref()
                .unwrap()
                .id,
            pack.id
        );
    } else {
        assert!(client.pending_bedrock_skin_pack.lock().await.is_none());
    }
    client.reconcile_bedrock_skin_pack(&fixture.server).await;
    assert!(
        outbound.try_recv().is_err(),
        "handoff catch-up is idempotent"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn initial_download_catches_latest_revision_and_ignores_stale_responses() {
    let fixture = TestServer::new().await;
    fixture
        .server
        .bedrock_skin_pack_endpoint
        .store(true, Ordering::Release);
    let first = revision(&fixture, 1).await;
    let (mut pending, mut peer) = connection().await;
    pending.send_bedrock_skin_pack(&fixture.server).await;
    expect_offer(&mut peer, &fixture, &first).await;
    assert!(pending.bedrock_skin_pack.is_none());

    let latest = revision(&fixture, 2).await;
    for (id, status) in [(latest.id, 0), (first.id, 3), (first.id, 4)] {
        reply(&mut pending, &fixture.server, id, status).await;
    }
    expect_no_packet(&mut peer).await;
    assert_eq!(
        pending.pending_bedrock_skin_pack.as_ref().unwrap().id,
        first.id
    );

    reply(&mut pending, &fixture.server, first.id, 0).await;
    expect_offer(&mut peer, &fixture, &latest).await;
    assert_eq!(pending.bedrock_skin_pack.as_ref().unwrap().id, first.id);
    assert_eq!(pending.pending_resource_packs, HashSet::from([latest.id]));
    reply(&mut pending, &fixture.server, first.id, 0).await;
    for id in [
        SKnownPacks::to_id(JavaMinecraftVersion::V_26_2),
        SAcknowledgeFinishConfig::to_id(JavaMinecraftVersion::V_26_2),
    ] {
        assert!(
            pending
                .handle_config_packet(&fixture.server, &config_packet(id))
                .await
                .unwrap()
                .is_none()
        );
    }
    expect_no_packet(&mut peer).await;

    reply(&mut pending, &fixture.server, latest.id, 0).await;
    expect_packet(&mut peer, &CConfigRemoveResourcePack::new(Some(&first.id))).await;
    expect_known_packs(&mut peer).await;
    assert_eq!(pending.bedrock_skin_pack.as_ref().unwrap().id, latest.id);
    assert!(pending.pending_resource_packs.is_empty());
    reply(&mut pending, &fixture.server, latest.id, 0).await;
    expect_no_packet(&mut peer).await;
    finish_and_reconcile(pending, &mut peer, &fixture, None).await;
    fixture.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn handoff_catches_revision_published_after_configuration_download() {
    let fixture = TestServer::new().await;
    fixture
        .server
        .bedrock_skin_pack_endpoint
        .store(true, Ordering::Release);
    let first = revision(&fixture, 1).await;
    let (mut pending, mut peer) = connection().await;
    pending.send_bedrock_skin_pack(&fixture.server).await;
    expect_offer(&mut peer, &fixture, &first).await;
    reply(&mut pending, &fixture.server, first.id, 0).await;
    expect_known_packs(&mut peer).await;

    let latest = revision(&fixture, 2).await;
    finish_and_reconcile(pending, &mut peer, &fixture, Some(&latest)).await;
    fixture.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_configuration_replacement_keeps_loaded_pack_without_handoff_retry() {
    for status in [1, 2] {
        let fixture = TestServer::new().await;
        fixture
            .server
            .bedrock_skin_pack_endpoint
            .store(true, Ordering::Release);
        let first = revision(&fixture, 1).await;
        let (mut pending, mut peer) = connection().await;
        pending.send_bedrock_skin_pack(&fixture.server).await;
        expect_offer(&mut peer, &fixture, &first).await;
        let latest = revision(&fixture, 2).await;
        reply(&mut pending, &fixture.server, first.id, 0).await;
        expect_offer(&mut peer, &fixture, &latest).await;

        reply(&mut pending, &fixture.server, latest.id, status).await;
        expect_known_packs(&mut peer).await;
        assert_eq!(pending.bedrock_skin_pack.as_ref().unwrap().id, first.id);
        assert!(pending.pending_resource_packs.is_empty());
        assert!(!pending.is_closed());
        reply(&mut pending, &fixture.server, latest.id, status).await;
        expect_no_packet(&mut peer).await;
        finish_and_reconcile(pending, &mut peer, &fixture, None).await;
        fixture.shutdown().await;
    }
}
