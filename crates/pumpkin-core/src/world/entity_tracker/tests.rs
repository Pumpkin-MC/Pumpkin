use pumpkin_config::EntityTrackingConfig;
use pumpkin_protocol::java::client::play::CSpawnEntity;
use pumpkin_protocol::ser::NetworkReadExt;
use pumpkin_protocol::{RawPacket, ServerPacket};

use super::*;
use crate::entity::passive::cow::CowEntity;
use crate::plugin::server::packet::PacketSentEvent;
use crate::plugin::{BoxFuture, EventHandler, EventPriority};
use crate::server::Server;
use crate::test_support::{
    Client, TestResult, count, id, server, server_with_entity_tracking, shutdown,
};
use pumpkin_util::version::JavaMinecraftVersion;
use pumpkin_world::cylindrical_chunk_iterator::Cylindrical;

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
        } else if packet.id == id::<CEntityPositionSync>()
            && payload.get_var_int()?.0 == entity_id
            && pos.is_some()
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

/// Whether a packet with this id and payload spawns entity `entity_id`.
fn spawns_entity(packet_id: i32, payload: &[u8], entity_id: i32) -> bool {
    packet_id == id::<CSpawnEntity>()
        && CSpawnEntity::read_packet_data(payload, &CURRENT_MC_VERSION)
            .is_ok_and(|spawn| spawn.entity_id.0 == entity_id)
}

/// How many of `packets` spawn entity `entity_id`.
fn spawns(packets: &[RawPacket], entity_id: i32) -> usize {
    packets
        .iter()
        .filter(|packet| spawns_entity(packet.id, &packet.payload, entity_id))
        .count()
}

/// How many of `packets` remove entity `entity_id`.
fn removals(packets: &[RawPacket], entity_id: i32) -> usize {
    let removed = |packet: &&RawPacket| {
        let mut payload = &packet.payload[..];
        packet.id == id::<CRemoveEntities>()
            && payload.get_var_int().is_ok_and(|count| {
                (0..count.0).any(|_| payload.get_var_int().is_ok_and(|id| id.0 == entity_id))
            })
    };
    packets.iter().filter(removed).count()
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
        // The teleport alone reaches the watcher with the next update.
        for _ in 0..2 {
            world.entity_tracker.update_all(&world);
        }
        drawn = drawn_at(drawn, walker.entity_id(), &watcher.received().await?)?;
        assert_close(drawn, target);
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

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_watcher_follows_every_player_of_a_crowd_through_a_tick() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let mut watcher = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let walkers: Vec<Client> = (0..12u32)
        .map(|i| {
            let pos = Vector3::new(2.5, 100.0, f64::from(i) + 0.5);
            Client::join(&server, 2 + u64::from(i), pos)
        })
        .collect();
    world.entity_tracker.update_all(&world);
    let spawned = watcher.received().await?;
    let mut drawn = Vec::new();
    for walker in &walkers {
        drawn.push(drawn_at(None, walker.entity_id(), &spawned)?);
    }

    // As in a tick: what the passes queue waits for the end of it.
    let java = watcher.player.client.java().expect("a Java player");
    java.suspend_flushing();
    for step in 1..=4 {
        for (i, walker) in walkers.iter().enumerate() {
            let pos = walker.player.get_entity().pos.load();
            let stride = 0.05 * f64::from(step) * (i + 1) as f64 / 4.0;
            walker
                .player
                .get_entity()
                .set_pos(Vector3::new(pos.x + stride, pos.y, pos.z));
        }
        world.entity_tracker.update_all(&world);
    }
    java.resume_flushing();

    let packets = watcher.received().await?;
    // Each player twice: vanilla sends one every `updateInterval` (2) ticks.
    assert_eq!(count::<CUpdateEntityPos>(&packets), 2 * walkers.len());
    for (walker, drawn) in walkers.iter().zip(drawn) {
        let drawn = drawn_at(drawn, walker.entity_id(), &packets)?;
        assert_close(drawn, walker.player.get_entity().pos.load());
    }
    shutdown(&server).await;
    Ok(())
}

#[tokio::test]
async fn a_pass_pairs_and_unpairs_who_changed_and_leaves_the_rest_alone() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let mut watcher = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let stays = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    let leaves = Client::join(&server, 3, Vector3::new(0.5, 100.0, 3.5));
    let arrives = Client::join(&server, 4, Vector3::new(4000.5, 100.0, 0.5));
    world.entity_tracker.update_all(&world);
    let packets = watcher.received().await?;
    assert!(drawn_at(None, stays.entity_id(), &packets)?.is_some());
    assert!(drawn_at(None, leaves.entity_id(), &packets)?.is_some());
    assert!(drawn_at(None, arrives.entity_id(), &packets)?.is_none());

    // Whether `packets` leave the entity drawn.
    let kept = |packets: &[RawPacket], entity_id: i32| {
        drawn_at(Some(Vector3::default()), entity_id, packets).map(|drawn| drawn.is_some())
    };

    // One walks off, one walks up and one takes a step, all in the same tick.
    let far = Vector3::new(4000.5, 100.0, 40.5);
    leaves.player.get_entity().set_pos(far);
    let near = Vector3::new(0.5, 100.0, -3.5);
    arrives.player.get_entity().set_pos(near);
    let step = Vector3::new(3.7, 100.0, 0.5);
    stays.player.get_entity().set_pos(step);
    world.entity_tracker.update_all(&world);
    let packets = watcher.received().await?;
    assert_eq!(spawns(&packets, stays.entity_id()), 0);
    assert!(kept(&packets, stays.entity_id())?);
    assert!(!kept(&packets, leaves.entity_id())?);
    assert_eq!(spawns(&packets, arrives.entity_id()), 1);
    let mut drawn = drawn_at(None, arrives.entity_id(), &packets)?;

    // Nothing changes any more: nobody is drawn or removed a second time.
    for _ in 0..2 {
        world.entity_tracker.update_all(&world);
        let packets = watcher.received().await?;
        for entity_id in [stays.entity_id(), arrives.entity_id(), leaves.entity_id()] {
            assert_eq!(spawns(&packets, entity_id), 0);
        }
        assert_eq!(count::<CRemoveEntities>(&packets), 0);
        drawn = drawn_at(drawn, arrives.entity_id(), &packets)?;
    }
    assert_close(drawn, near);
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
async fn a_world_created_later_gets_the_entity_broadcast_range_that_was_set() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    // What `minecraft:serversettings/entity_broadcast_range/set` does.
    server
        .management_hub
        .settings
        .entity_broadcast_range
        .store(30, Relaxed);
    for world in server.worlds.load().iter() {
        world.entity_tracker.set_entity_broadcast_range(30);
    }

    let world = server.create_world(
        "later".to_string(),
        pumpkin_data::dimension::Dimension::OVERWORLD,
    );
    let tracker = &world.entity_tracker;
    // A cow is tracked from 160 blocks, 30% of that are 48.
    assert_eq!(tracker.scaled_range(&EntityType::COW), 48);
    shutdown(&server).await;
    Ok(())
}

/// A walker at the origin, one watcher 3 blocks away and one 40 blocks away, with players
/// from 16 blocks on the far rhythm of 4 ticks.
async fn near_and_far_watchers(
    folder: &tempfile::TempDir,
) -> Result<(Arc<Server>, Client, Client, Client), Box<dyn std::error::Error>> {
    let entity_tracking = EntityTrackingConfig {
        far_player_distance: 16,
        far_player_update_interval: 4,
    };
    let server = server_with_entity_tracking(folder, entity_tracking).await?;
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let near = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    let far = Client::join(&server, 3, Vector3::new(40.5, 100.0, 0.5));
    Ok((server, walker, near, far))
}

#[tokio::test]
async fn far_watchers_follow_a_player_on_a_slower_rhythm() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, walker, mut near, mut far) = near_and_far_watchers(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let entity = walker.player.get_entity();
    world.entity_tracker.update_all(&world);
    let mut drawn_near = drawn_at(None, walker.entity_id(), &near.received().await?)?;
    let mut drawn_far = drawn_at(None, walker.entity_id(), &far.received().await?)?;

    let (mut near_moves, mut far_moves) = (0, 0);
    for step in 1..=8 {
        entity.set_pos(Vector3::new(
            0.5,
            100.0,
            0.2f64.mul_add(f64::from(step), 0.5),
        ));
        world.entity_tracker.update_all(&world);
        let (near_packets, far_packets) = (near.received().await?, far.received().await?);
        near_moves += count::<CUpdateEntityPos>(&near_packets);
        far_moves += count::<CUpdateEntityPos>(&far_packets);
        drawn_near = drawn_at(drawn_near, walker.entity_id(), &near_packets)?;
        drawn_far = drawn_at(drawn_far, walker.entity_id(), &far_packets)?;
    }
    assert_eq!((near_moves, far_moves), (4, 2));
    assert_close(drawn_near, entity.pos.load());
    assert_close(drawn_far, entity.pos.load());

    // A jump too long for a delta reaches the far watcher in the same tick.
    entity.set_pos(Vector3::new(0.5, 100.0, 12.5));
    world.entity_tracker.update_all(&world);
    let packets = far.received().await?;
    assert_eq!(count::<CEntityPositionSync>(&packets), 1);
    assert_close(
        drawn_at(drawn_far, walker.entity_id(), &packets)?,
        entity.pos.load(),
    );
    shutdown(&server).await;
    Ok(())
}

#[tokio::test]
async fn a_watcher_changes_rhythm_without_losing_the_player() -> TestResult {
    let folder = tempfile::tempdir()?;
    let (server, walker, _near, mut watcher) = near_and_far_watchers(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let entity = walker.player.get_entity();
    world.entity_tracker.update_all(&world);
    let mut drawn = drawn_at(None, walker.entity_id(), &watcher.received().await?)?;
    let mut z = 0.5;
    // The walker takes `ticks` steps; returns how many moves the watcher got.
    let mut walk = async |watcher: &mut Client,
                          drawn: &mut Option<Vector3<f64>>,
                          ticks: u32|
           -> Result<usize, Box<dyn std::error::Error>> {
        let mut moves = 0;
        for _ in 0..ticks {
            z += 0.2;
            entity.set_pos(Vector3::new(0.5, 100.0, z));
            world.entity_tracker.update_all(&world);
            let packets = watcher.received().await?;
            moves += count::<CUpdateEntityPos>(&packets);
            *drawn = drawn_at(*drawn, walker.entity_id(), &packets)?;
        }
        Ok(moves)
    };
    let come_to = async |watcher: &Client, x: f64| {
        watcher
            .player
            .get_entity()
            .set_pos(Vector3::new(x, 100.0, 0.5));
        world
            .entity_tracker
            .update_player_position(&watcher.player, &world);
    };

    // Far: every fourth tick. Close by: every other tick, from the right place.
    assert_eq!(walk(&mut watcher, &mut drawn, 7).await?, 1);
    come_to(&watcher, 6.5).await;
    assert_eq!(walk(&mut watcher, &mut drawn, 9).await?, 4);
    assert_close(drawn, entity.pos.load());

    // Away again, and the far rhythm picks up from its own base.
    come_to(&watcher, 40.5).await;
    assert_eq!(walk(&mut watcher, &mut drawn, 8).await?, 1);
    assert_close(drawn, entity.pos.load());

    // A player it fights is followed at full rate from any distance.
    let attacker = walker.player.gameprofile.id;
    watcher
        .player
        .living_entity
        .last_hurt_by_player
        .store(Some(attacker));
    come_to(&watcher, 40.5).await;
    assert_eq!(walk(&mut watcher, &mut drawn, 8).await?, 3);
    assert_close(drawn, entity.pos.load());
    shutdown(&server).await;
    Ok(())
}

/// The far interval is not a multiple of the near one, so a far update can come between a
/// watcher's return and the near update that places the player for it again.
#[tokio::test]
async fn a_watcher_back_before_its_first_far_update_gets_no_far_delta() -> TestResult {
    let folder = tempfile::tempdir()?;
    let entity_tracking = EntityTrackingConfig {
        far_player_distance: 16,
        far_player_update_interval: 5,
    };
    let server = server_with_entity_tracking(&folder, entity_tracking).await?;
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(6.5, 100.0, 0.5));
    let world = server.worlds.load()[0].clone();
    let entity = walker.player.get_entity();
    world.entity_tracker.update_all(&world);
    let mut drawn = drawn_at(None, walker.entity_id(), &watcher.received().await?)?;

    // Near updates in every other tick, far updates in tick 5.
    for tick in 1..=8 {
        // Away after the near update of tick 2, back before the far update of tick 5.
        if let Some(x) = [(3, 40.5), (5, 6.5)]
            .iter()
            .find_map(|&(at, x)| (at == tick).then_some(x))
        {
            let watcher = &watcher.player;
            watcher.get_entity().set_pos(Vector3::new(x, 100.0, 0.5));
            world.entity_tracker.update_player_position(watcher, &world);
        }
        let z = 0.2f64.mul_add(f64::from(tick), 0.5);
        entity.set_pos(Vector3::new(0.5, 100.0, z));
        world.entity_tracker.update_all(&world);
        drawn = drawn_at(drawn, walker.entity_id(), &watcher.received().await?)?;
        // A delta from a base the watcher never got would draw the walker ahead of itself.
        assert!(
            drawn.is_some_and(|drawn| drawn.z < z + 0.001),
            "tick {tick}: drawn at {drawn:?}, is at z {z}"
        );
    }
    assert_close(drawn, entity.pos.load());
    shutdown(&server).await;
    Ok(())
}

/// A far update can also come while a watcher is back for a moment. It gets no delta then,
/// so it needs the far base anew when it goes away again before a near update.
#[tokio::test]
async fn a_watcher_away_again_before_a_near_update_gets_the_far_base_anew() -> TestResult {
    let folder = tempfile::tempdir()?;
    let entity_tracking = EntityTrackingConfig {
        far_player_distance: 16,
        far_player_update_interval: 5,
    };
    let server = server_with_entity_tracking(&folder, entity_tracking).await?;
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let mut watcher = Client::join(&server, 2, Vector3::new(40.5, 100.0, 0.5));
    let world = server.worlds.load()[0].clone();
    let entity = walker.player.get_entity();
    let mut drawn = None;

    // Far updates in ticks 0, 5 and 10, near updates in every other tick.
    for tick in 0..=10 {
        // Back before the far update of tick 5, away before the near update of tick 6.
        if let Some(x) = [(5, 6.5), (6, 40.5)]
            .iter()
            .find_map(|&(at, x)| (at == tick).then_some(x))
        {
            let watcher = &watcher.player;
            watcher.get_entity().set_pos(Vector3::new(x, 100.0, 0.5));
            world.entity_tracker.update_player_position(watcher, &world);
        }
        let z = 0.2f64.mul_add(f64::from(tick), 0.5);
        entity.set_pos(Vector3::new(0.5, 100.0, z));
        world.entity_tracker.update_all(&world);
        drawn = drawn_at(drawn, walker.entity_id(), &watcher.received().await?)?;
    }
    assert_close(drawn, entity.pos.load());
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
    assert_eq!(removals(&packets, walker.entity_id()), 1);
    assert_eq!(spawns(&packets, walker.entity_id()), 1);
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

    // Neither of them moves: the change of game mode alone hides the player.
    assert!(ghost.player.set_gamemode(GameMode::Spectator));
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
        (packet.id == id::<CUpdateEntityPos>()
            || packet.id == id::<CUpdateEntityPosRot>()
            || packet.id == id::<CEntityPositionSync>())
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

/// Makes `viewer` a player that got the chunk of `entity`, wherever it stands.
fn send_chunk_of(entity: &Entity, viewer: &Client) {
    let chunk = entity.chunk_pos.load();
    viewer
        .player
        .watched_section
        .store(Cylindrical::new(chunk, get_view_distance(&viewer.player)));
    lock_briefly(&viewer.player.chunk_sender).mark_sent_out_of_band(chunk);
}

/// Stands in for a join that reaches the tracker while it sends a movement.
struct PairsMidPass {
    walker: i32,
    viewer: Arc<Player>,
    near: Vector3<f64>,
    paired: AtomicBool,
}
impl EventHandler<PacketSentEvent> for PairsMidPass {
    fn handle_blocking<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        // The walker's own copy of its metadata is queued right after its movement.
        if event.packet_id == id::<CSetEntityMetadata>()
            && event.player.entity_id() == self.walker
            && !self.paired.swap(true, Relaxed)
        {
            let world = server.worlds.load()[0].clone();
            self.viewer.get_entity().set_pos(self.near);
            world
                .entity_tracker
                .pair_new_player_with_tracked_entities(&self.viewer, &world);
        }
        Box::pin(async {})
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_watcher_paired_right_after_a_move_draws_the_player_where_it_is() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    let _watcher = Client::join(&server, 2, Vector3::new(3.5, 100.0, 0.5));
    // Out of range, with the walker's chunk already sent.
    let mut viewer = Client::join(&server, 3, Vector3::new(1000.5, 100.0, 0.5));
    let entity = walker.player.get_entity();
    send_chunk_of(entity, &viewer);
    world.entity_tracker.update_all(&world);
    viewer.received().await?;
    server.plugin_manager.register(
        Arc::new(PairsMidPass {
            walker: walker.entity_id(),
            viewer: viewer.player.clone(),
            near: Vector3::new(5.5, 100.0, 0.5),
            paired: AtomicBool::new(false),
        }),
        EventPriority::Normal,
        true,
    );

    entity.set_pos(Vector3::new(1.5, 100.0, 0.5));
    entity.set_sneaking(true);
    world.entity_tracker.update_all(&world);
    let mut drawn = drawn_at(None, walker.entity_id(), &viewer.received().await?)?;
    assert_close(drawn, entity.pos.load());

    for step in 1..=4 {
        entity.set_pos(Vector3::new(
            0.2f64.mul_add(f64::from(step), 1.5),
            100.0,
            0.5,
        ));
        world.entity_tracker.update_all(&world);
    }
    // The last step goes out on the next even tick.
    world.entity_tracker.update_all(&world);
    drawn = drawn_at(drawn, walker.entity_id(), &viewer.received().await?)?;
    assert_close(drawn, entity.pos.load());
    shutdown(&server).await;
    Ok(())
}

/// Stands in for a plugin that asks the tracker about an entity when its spawn packet goes
/// out, at the moment the entity's next movement is sent.
struct MovesWhileSpawned {
    walker: Arc<TrackedEntity>,
    step: Vector3<f64>,
    spawned: AtomicBool,
    locked: AtomicBool,
}
impl EventHandler<PacketSentEvent> for MovesWhileSpawned {
    fn handle_blocking<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        if spawns_entity(event.packet_id, &event.payload, self.walker.entity_id)
            && !self.spawned.swap(true, Relaxed)
        {
            if self.walker.seen_by.try_read().is_err() {
                self.locked.store(true, Relaxed);
            } else {
                let world = server.worlds.load()[0].clone();
                self.walker.entity.get_entity().set_pos(self.step);
                // A player is sent in every other pass.
                for _ in 0..2 {
                    self.walker.send_changes(&world);
                }
            }
        }
        Box::pin(async {})
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_plugin_sees_a_spawn_packet_with_the_watchers_unlocked() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    // Out of range, with the walker's chunk already sent.
    let mut viewer = Client::join(&server, 2, Vector3::new(1000.5, 100.0, 0.5));
    let entity = walker.player.get_entity();
    send_chunk_of(entity, &viewer);
    world.entity_tracker.update_all(&world);
    viewer.received().await?;
    let tracked = world
        .entity_tracker
        .get_tracked_entity(walker.entity_id())
        .ok_or("the walker is tracked")?;
    let plugin = Arc::new(MovesWhileSpawned {
        walker: tracked,
        step: Vector3::new(1.5, 100.0, 0.5),
        spawned: AtomicBool::new(false),
        locked: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register(plugin.clone(), EventPriority::Normal, true);

    viewer
        .player
        .get_entity()
        .set_pos(Vector3::new(5.5, 100.0, 0.5));
    world
        .entity_tracker
        .pair_new_player_with_tracked_entities(&viewer.player, &world);
    assert!(plugin.spawned.load(Relaxed));
    assert!(
        !plugin.locked.load(Relaxed),
        "the watchers are locked while a plugin sees the spawn packet"
    );

    // The move went out while the spawn packet did, so the next pass pairs the two anew.
    world.entity_tracker.update_all(&world);
    let mut drawn = drawn_at(None, walker.entity_id(), &viewer.received().await?)?;
    assert_close(drawn, entity.pos.load());
    for step in 1..=4 {
        entity.set_pos(Vector3::new(
            0.2f64.mul_add(f64::from(step), 1.5),
            100.0,
            0.5,
        ));
        world.entity_tracker.update_all(&world);
    }
    world.entity_tracker.update_all(&world);
    drawn = drawn_at(drawn, walker.entity_id(), &viewer.received().await?)?;
    assert_close(drawn, entity.pos.load());
    shutdown(&server).await;
    Ok(())
}

/// Stands in for a player that respawns while its spawn packet for a new watcher goes out.
struct RespawnsWhileSpawned {
    walker: Arc<Player>,
    spawned: AtomicBool,
}
impl EventHandler<PacketSentEvent> for RespawnsWhileSpawned {
    fn handle_blocking<'a>(
        &'a self,
        server: &'a Arc<Server>,
        event: &'a mut PacketSentEvent,
    ) -> BoxFuture<'a, ()> {
        if spawns_entity(event.packet_id, &event.payload, self.walker.entity_id())
            && !self.spawned.swap(true, Relaxed)
        {
            let world = server.worlds.load()[0].clone();
            let entity: Arc<dyn EntityBase> = self.walker.clone();
            world.entity_tracker.respawn_entity(&entity, &world);
        }
        Box::pin(async {})
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_player_respawning_while_its_spawn_packet_goes_out_stays_drawn() -> TestResult {
    let folder = tempfile::tempdir()?;
    let server = server(&folder).await?;
    let world = server.worlds.load()[0].clone();
    let walker = Client::join(&server, 1, Vector3::new(0.5, 100.0, 0.5));
    // Out of range, with the walker's chunk already sent.
    let mut viewer = Client::join(&server, 2, Vector3::new(1000.5, 100.0, 0.5));
    let entity = walker.player.get_entity();
    send_chunk_of(entity, &viewer);
    world.entity_tracker.update_all(&world);
    viewer.received().await?;
    let plugin = Arc::new(RespawnsWhileSpawned {
        walker: walker.player.clone(),
        spawned: AtomicBool::new(false),
    });
    server
        .plugin_manager
        .register(plugin.clone(), EventPriority::Normal, true);

    // Paired by the tracker pass: a join iterates the entity map, which a respawn waits for.
    viewer
        .player
        .get_entity()
        .set_pos(Vector3::new(5.5, 100.0, 0.5));
    world
        .entity_tracker
        .update_player_position(&viewer.player, &world);
    world.entity_tracker.update_all(&world);
    assert!(plugin.spawned.load(Relaxed));

    // The removal took the respawned player from the watcher too, so the next pass pairs
    // the two anew.
    world.entity_tracker.update_all(&world);
    let drawn = drawn_at(None, walker.entity_id(), &viewer.received().await?)?;
    assert_close(drawn, entity.pos.load());
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
