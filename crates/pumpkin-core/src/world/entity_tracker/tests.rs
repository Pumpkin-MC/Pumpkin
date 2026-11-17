use pumpkin_protocol::java::client::play::CSpawnEntity;
use pumpkin_protocol::ser::NetworkReadExt;
use pumpkin_protocol::{RawPacket, ServerPacket};

use super::*;
use crate::entity::passive::cow::CowEntity;
use crate::plugin::server::packet::PacketSentEvent;
use crate::plugin::{BoxFuture, EventHandler, EventPriority};
use crate::server::Server;
use crate::test_support::{Client, TestResult, count, id, server, shutdown};
use pumpkin_util::version::JavaMinecraftVersion;

/// Where a client that got `packets` draws entity `entity_id`, starting from `pos`.
fn drawn_at(
    mut pos: Option<Vector3<f64>>,
    entity_id: i32,
    packets: &[RawPacket],
) -> Result<Option<Vector3<f64>>, Box<dyn std::error::Error>> {
    let version = CURRENT_MC_VERSION;
    let step = |pos: Option<Vector3<f64>>, delta: Vector3<i16>| {
        let delta = Vector3::new(f64::from(delta.x), f64::from(delta.y), f64::from(delta.z));
        pos.map(|pos| pos + delta * (1.0 / 4096.0))
    };
    for packet in packets {
        let mut payload = &packet.payload[..];
        if packet.id == id::<CSpawnEntity>() {
            let spawn = CSpawnEntity::read_packet_data(payload, &version)?;
            if spawn.entity_id.0 == entity_id {
                pos = Some(spawn.position);
            }
        } else if packet.id == id::<CUpdateEntityPos>() {
            let packet = CUpdateEntityPos::read(&mut payload, &version)?;
            if packet.entity_id.0 == entity_id {
                pos = step(pos, packet.delta);
            }
        } else if packet.id == id::<CUpdateEntityPosRot>() {
            let packet = CUpdateEntityPosRot::read(&mut payload, &version)?;
            if packet.entity_id.0 == entity_id {
                pos = step(pos, packet.delta);
            }
        } else if packet.id == id::<CRemoveEntities>() {
            for _ in 0..payload.get_var_int()?.0 {
                if payload.get_var_int()?.0 == entity_id {
                    pos = None;
                }
            }
        } else if packet.id == id::<CEntityPositionSync>() && payload.get_var_int()?.0 == entity_id
        {
            // A linear path: just the end position.
            payload.get_var_int()?;
            pos = Some(Vector3::new(
                payload.get_f64_be()?,
                payload.get_f64_be()?,
                payload.get_f64_be()?,
            ));
        }
    }
    Ok(pos)
}

fn assert_close(drawn: Option<Vector3<f64>>, actual: Vector3<f64>) {
    // Deltas are sent in 1/4096 of a block.
    assert!(
        drawn.is_some_and(|drawn| (drawn - actual).length() < 0.001),
        "drawn at {drawn:?}, is at {actual:?}"
    );
}

#[tokio::test]
async fn watchers_follow_a_player_from_the_tracker() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    let entity = walker.player.get_entity();
    world.entity_tracker.update_all(&world);
    let mut drawn = drawn_at(None, walker.entity_id(), &watcher.received().await?)?;
    assert_close(drawn, entity.pos.load());

    // Vanilla sends a player every `updateInterval` (2) ticks.
    let mut moves = 0;
    for step in 1..=6 {
        entity.set_pos(Vector3::new(
            0.2f64.mul_add(f64::from(step), 0.5),
            100.0,
            0.5,
        ));
        world.entity_tracker.update_all(&world);
        let packets = watcher.received().await?;
        moves += count::<CUpdateEntityPos>(&packets);
        drawn = drawn_at(drawn, walker.entity_id(), &packets)?;
    }
    assert_eq!(moves, 3);
    assert_close(drawn, entity.pos.load());

    // A short teleport and a long one, each followed by a step.
    for jump in [3.0, 40.0] {
        let pos = entity.pos.load();
        let target = Vector3::new(pos.x + jump, pos.y, pos.z);
        walker.player.teleport(target, None, None, world.clone());
        for step in 1..=2 {
            entity.set_pos(Vector3::new(
                target.x,
                pos.y,
                0.1f64.mul_add(f64::from(step), pos.z),
            ));
            world.entity_tracker.update_all(&world);
        }
        drawn = drawn_at(drawn, walker.entity_id(), &watcher.received().await?)?;
        assert_close(drawn, entity.pos.load());
    }
    shutdown(&server).await;
    Ok(())
}

#[tokio::test]
async fn head_turns_wait_for_the_update_interval() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let turner = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    world.entity_tracker.update_all(&world);
    watcher.received().await?;

    let mut turns = 0;
    for tick in 1..=4u8 {
        turner
            .player
            .get_entity()
            .head_yaw
            .store(10.0 * f32::from(tick));
        world.entity_tracker.update_all(&world);
        turns += count::<CHeadRot>(&watcher.received().await?);
    }
    // Vanilla sends the head inside the same `updateInterval` block as the body.
    assert_eq!(turns, 2);
    shutdown(&server).await;
    Ok(())
}

#[tokio::test]
async fn the_entity_broadcast_range_scales_tracking_ranges() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let mut watcher = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let cow: Arc<dyn EntityBase> = CowEntity::new(Entity::new(
        world.clone(),
        Vector3::new(60.5, 100.0, 0.5),
        &EntityType::COW,
    ));
    world.entity_tracker.add_entity(&cow, &world);
    assert_eq!(count::<CSpawnEntity>(&watcher.received().await?), 1);

    // A cow is tracked from 160 blocks, and 30% of that does not reach 60 blocks.
    world.entity_tracker.set_entity_broadcast_range(30);
    world.entity_tracker.update_all(&world);
    assert_eq!(count::<CRemoveEntities>(&watcher.received().await?), 1);

    world.entity_tracker.set_entity_broadcast_range(100);
    world.entity_tracker.update_all(&world);
    assert_eq!(count::<CSpawnEntity>(&watcher.received().await?), 1);
    shutdown(&server).await;
    Ok(())
}

#[tokio::test]
async fn a_respawned_or_removed_player_is_redrawn_or_gone() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    let entity = walker.player.get_entity();
    world.entity_tracker.update_all(&world);
    let mut drawn = drawn_at(None, walker.entity_id(), &watcher.received().await?)?;

    // Vanilla `PlayerList.respawn`: the old entity goes, a fresh one appears elsewhere.
    let respawn = Vector3::new(30.5, 100.0, 0.5);
    entity.set_pos(respawn);
    let tracked: Arc<dyn EntityBase> = walker.player.clone();
    world.entity_tracker.respawn_entity(&tracked, &world);
    entity.set_pos(Vector3::new(30.5, 100.0, 1.5));
    for _ in 0..2 {
        world.entity_tracker.update_all(&world);
    }
    let packets = watcher.received().await?;
    assert_eq!(count::<CRemoveEntities>(&packets), 1);
    drawn = drawn_at(drawn, walker.entity_id(), &packets)?;
    assert_close(drawn, entity.pos.load());

    world
        .entity_tracker
        .remove_entity(walker.player.as_ref(), &world);
    world.entity_tracker.update_all(&world);
    drawn = drawn_at(drawn, walker.entity_id(), &watcher.received().await?)?;
    assert!(drawn.is_none());
    shutdown(&server).await;
    Ok(())
}

#[tokio::test]
async fn a_spectator_is_hidden_and_its_moves_stay_private() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let mut ghost = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    world.entity_tracker.update_all(&world);
    let mut drawn = drawn_at(None, ghost.entity_id(), &watcher.received().await?)?;
    assert!(drawn.is_some());

    ghost.player.gamemode.store(GameMode::Spectator);
    world
        .entity_tracker
        .update_player_position(&ghost.player, &world);
    world.entity_tracker.update_all(&world);
    drawn = drawn_at(drawn, ghost.entity_id(), &watcher.received().await?)?;
    assert!(drawn.is_none());

    // Its movement reaches nobody, while it still follows the others.
    ghost.received().await?;
    for step in 1..=4 {
        let x = 0.2 * f64::from(step);
        ghost
            .player
            .get_entity()
            .set_pos(Vector3::new(0.5 + x, 100.0, 0.5));
        watcher
            .player
            .get_entity()
            .set_pos(Vector3::new(3.5 + x, 100.0, 0.5));
        world.entity_tracker.update_all(&world);
    }
    assert!(watcher.received().await?.is_empty());
    assert_eq!(count::<CUpdateEntityPos>(&ghost.received().await?), 2);
    shutdown(&server).await;
    Ok(())
}

#[tokio::test]
async fn a_riding_player_turns_but_moves_with_its_vehicle() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let rider = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    let cow: Arc<dyn EntityBase> = CowEntity::new(Entity::new(
        world.clone(),
        Vector3::new(0.5, 100.0, 0.5),
        &EntityType::COW,
    ));
    world.entity_tracker.add_entity(&cow, &world);
    let entity = rider.player.get_entity();
    cow.get_entity()
        .add_passenger_silently(cow.clone(), rider.player.clone());
    world.entity_tracker.update_all(&world);
    let mut drawn = drawn_at(None, rider.entity_id(), &watcher.received().await?)?;

    // Vanilla sends a passenger's rotation only: the client moves it with the vehicle.
    for step in 1..=4u8 {
        let pos = Vector3::new(0.2f64.mul_add(f64::from(step), 0.5), 100.0, 0.5);
        cow.get_entity().set_pos(pos);
        entity.set_pos(pos);
        entity.set_rotation(10.0 * f32::from(step), 0.0);
        world.entity_tracker.update_all(&world);
    }
    let packets = watcher.received().await?;
    let moved = |packet: &RawPacket| {
        let mut payload = &packet.payload[..];
        (packet.id == id::<CUpdateEntityPos>() || packet.id == id::<CUpdateEntityPosRot>())
            && payload
                .get_var_int()
                .is_ok_and(|id| id.0 == rider.entity_id())
    };
    assert!(!packets.iter().any(moved));
    assert_eq!(count::<CUpdateEntityRot>(&packets), 2);

    // Off the vehicle the next update is a full position.
    cow.get_entity().remove_passenger(rider.entity_id());
    entity.set_pos(Vector3::new(2.0, 100.0, 2.5));
    for _ in 0..2 {
        world.entity_tracker.update_all(&world);
    }
    let packets = watcher.received().await?;
    assert_eq!(count::<CEntityPositionSync>(&packets), 1);
    drawn = drawn_at(drawn, rider.entity_id(), &packets)?;
    assert_close(drawn, entity.pos.load());
    shutdown(&server).await;
    Ok(())
}

struct CountsPackets(AtomicU32);
impl EventHandler<PacketSentEvent> for CountsPackets {
    fn handle<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        _event: &'a PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        self.0.fetch_add(1, Relaxed);
        Box::pin(async {})
    }
}

/// Stands in for a plugin that translates packets for an older client.
struct RenumbersPackets;
const RENUMBERED: i32 = 1000;
impl EventHandler<PacketSentEvent> for RenumbersPackets {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        event.packet_id += RENUMBERED;
        Box::pin(async {})
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn watchers_follow_a_player_while_a_plugin_watches_packets() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    let entity = walker.player.get_entity();
    world.entity_tracker.update_all(&world);
    let mut drawn = drawn_at(None, walker.entity_id(), &watcher.received().await?)?;
    let seen = Arc::new(CountsPackets(AtomicU32::new(0)));
    server
        .plugin_manager
        .register(seen.clone(), EventPriority::Normal, false);

    for step in 1..=6 {
        entity.set_pos(Vector3::new(
            0.2f64.mul_add(f64::from(step), 0.5),
            100.0,
            0.5,
        ));
        entity.head_yaw.store(10.0 * step as f32);
        world.entity_tracker.update_all(&world);
    }
    let packets = watcher.received().await?;
    drawn = drawn_at(drawn, walker.entity_id(), &packets)?;
    assert_close(drawn, entity.pos.load());
    // Every packet of the tracker went through the event, one by one.
    assert_eq!(seen.0.load(Relaxed) as usize, packets.len());
    shutdown(&server).await;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn an_older_client_gets_the_tracker_packets_translated() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    world.entity_tracker.update_all(&world);
    watcher.received().await?;
    if let Some(java) = watcher.player.client.java() {
        java.version.store(JavaMinecraftVersion::V_1_21_4);
    }
    server
        .plugin_manager
        .register(Arc::new(RenumbersPackets), EventPriority::Normal, true);

    for step in 1..=6 {
        let x = 0.2f64.mul_add(f64::from(step), 0.5);
        walker
            .player
            .get_entity()
            .set_pos(Vector3::new(x, 100.0, 0.5));
        world.entity_tracker.update_all(&world);
    }
    let ids: Vec<i32> = watcher.received().await?.iter().map(|p| p.id).collect();
    assert_eq!(ids, [id::<CUpdateEntityPos>() + RENUMBERED; 3]);
    shutdown(&server).await;
    Ok(())
}

#[tokio::test]
async fn watchers_follow_a_player_while_the_game_is_frozen() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    let entity = walker.player.get_entity();
    world.entity_tracker.update_all(&world);
    let drawn = drawn_at(None, walker.entity_id(), &watcher.received().await?)?;

    server.tick_rate_manager.set_frozen(&server, true);
    entity.set_pos(Vector3::new(1.5, 100.0, 0.5));
    for _ in 0..2 {
        server.tick_rate_manager.tick();
        server.tick();
    }
    assert!(!server.tick_rate_manager.runs_normally());
    let drawn = drawn_at(drawn, walker.entity_id(), &watcher.received().await?)?;
    assert_close(drawn, entity.pos.load());
    shutdown(&server).await;
    Ok(())
}
