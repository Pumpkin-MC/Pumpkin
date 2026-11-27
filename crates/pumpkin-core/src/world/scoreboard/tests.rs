use std::{
    error::Error,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::packet::CURRENT_MC_VERSION;
use pumpkin_protocol::{
    ConnectionState,
    java::{client::play::CKeepAlive, packet_decoder::TCPNetworkDecoder},
    packet::MultiVersionJavaPacket,
    ser::NetworkReadExt,
};
use pumpkin_util::GameMode;
use tokio::io::BufReader;
use uuid::Uuid;

use super::*;
use crate::{
    data::VanillaData,
    entity::player::CustomScoreboard,
    net::{
        ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig,
        java::{JavaClient, pending::PendingConnection},
    },
    server::Server,
};

fn test_team(name: &str, players: Vec<String>) -> Team {
    Team {
        name: name.into(),
        display_name: TextComponent::text(""),
        options: 0,
        nametag_visibility: NameTagVisibility::Always,
        collision_rule: CollisionRule::Always,
        color: NamedColor::White,
        player_prefix: TextComponent::text("Online players: "),
        player_suffix: TextComponent::text(""),
        players,
    }
}

fn populate(board: &mut Scoreboard, target: &impl ScoreboardTarget) {
    let mut objective = ScoreboardObjective::new(
        "gone",
        TextComponent::text("Test Scoreboard"),
        RenderType::Integer,
        Some(NumberFormat::Blank),
        "dummy",
    );
    board.add_objective(target, objective.clone());
    board.remove_objective(target, "gone");
    objective.name = "test_board".into();
    board.add_objective(target, objective);
    board.update_score(
        target,
        ScoreboardScore::new("entry", "test_board", VarInt(7), None, None),
    );
    let mut team = test_team("gone_team", vec!["entry".into()]);
    board.add_team(target, team.clone());
    board.remove_team(target, "gone_team");
    team.name = "test_team".into();
    board.add_team(target, team);
    board.set_display_objective(target, ScoreboardDisplaySlot::Sidebar, Some("test_board"));
}

fn player_with_transport(
    server: &Arc<Server>,
    world: &Arc<World>,
) -> Result<(Arc<Player>, tokio::io::DuplexStream), Box<dyn Error>> {
    let (transport, peer) = tokio::io::duplex(8192);
    let (read, write) = tokio::io::split(transport);
    let pending = PendingConnection::from_transport(
        Box::new(read),
        Box::new(write),
        "127.0.0.1:25565".parse()?,
        0,
        PacketRateLimiter::new(false, 0.0, 0.0),
        Arc::downgrade(server),
    );
    let profile = GameProfile {
        id: Uuid::new_v4(),
        name: "ScoreboardTest".into(),
        properties: ArcSwap::from_pointee(vec![]),
        profile_actions: None,
    };
    let mut client = JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
    client.connection_state.store(ConnectionState::Play);
    client.start_outgoing_packet_task();
    let player = Arc::new(Player::new(
        Arc::new(ClientPlatform::Java(client)),
        profile,
        PlayerConfig::default(),
        world,
        GameMode::Survival,
    ));
    Ok((player, peer))
}

async fn read_scoreboard_packets(
    peer: tokio::io::DuplexStream,
) -> Result<Vec<pumpkin_protocol::RawPacket>, Box<dyn Error>> {
    let mut decoder = TCPNetworkDecoder::new(BufReader::new(peer));
    let packets = tokio::time::timeout(Duration::from_secs(30), async {
        let mut packets = vec![];
        loop {
            let packet = decoder.get_raw_packet().await?;
            if packet.id == CKeepAlive::to_id(CURRENT_MC_VERSION) {
                break;
            }
            packets.push(packet);
        }
        Ok::<_, pumpkin_protocol::PacketDecodeError>(packets)
    })
    .await??;
    Ok(packets)
}

fn assert_login_packets(packets: &[pumpkin_protocol::RawPacket]) -> Result<(), Box<dyn Error>> {
    assert_eq!(
        packets.iter().map(|packet| packet.id).collect::<Vec<_>>(),
        [
            CUpdateObjectives::to_id(CURRENT_MC_VERSION),
            CDisplayObjective::to_id(CURRENT_MC_VERSION),
            CUpdateScore::to_id(CURRENT_MC_VERSION),
            CSetPlayerTeam::to_id(CURRENT_MC_VERSION),
            CUpdateScore::to_id(CURRENT_MC_VERSION),
        ]
    );
    let names: Vec<_> = packets
        .iter()
        .filter(|packet| {
            packet.id == CUpdateObjectives::to_id(CURRENT_MC_VERSION)
                || packet.id == CSetPlayerTeam::to_id(CURRENT_MC_VERSION)
        })
        .map(|packet| {
            let mut payload = &packet.payload[..];
            payload.get_str().map(String::from)
        })
        .collect::<Result<_, _>>()?;
    assert_eq!(names, ["test_board", "test_team"]);
    let scores: Vec<_> = packets
        .iter()
        .filter(|p| p.id == CUpdateScore::to_id(CURRENT_MC_VERSION))
        .map(|p| {
            let mut payload = &p.payload[..];
            let _ = payload.get_str()?;
            let _ = payload.get_str()?;
            Ok::<_, pumpkin_protocol::ser::ReadingError>(payload.get_var_int()?.0)
        })
        .collect::<Result<_, _>>()?;
    assert_eq!(scores, [7, 9]);
    Ok(())
}

async fn test_server(directory: &tempfile::TempDir) -> Result<Arc<Server>, Box<dyn Error>> {
    let basic = BasicConfiguration {
        default_level_name: directory
            .path()
            .join("world")
            .to_string_lossy()
            .into_owned(),
        allow_nether: false,
        allow_end: false,
        ..Default::default()
    };
    let mut advanced = AdvancedConfiguration::default();
    advanced.networking.bedrock.online_mode = false;
    let server = Server::new(
        basic,
        advanced,
        TelemetryConfig::default(),
        VanillaData::default(),
        vec![],
    )
    .await?;
    Ok(server)
}

#[tokio::test]
async fn login_sends_preexisting_scoreboard_once() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let server = test_server(&directory).await?;
    let world = server.worlds.load().first().cloned().ok_or("No world")?;

    for custom in [false, true] {
        let (player, peer) = player_with_transport(&server, &world)?;
        world.add_player(&player)?;
        let client = player.client.java().ok_or("No Java client")?;

        if custom {
            let mut board = Scoreboard::default();
            populate(&mut board, &player);
            player
                .as_java()
                .ok_or("No Java player")?
                .set_scoreboard(Some(board));
        } else {
            let mut board = world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            populate(&mut board, &world);
        }
        // No writer task can run on this current-thread runtime until the next await.
        assert_eq!(client.pending_bytes.load(Ordering::Acquire), 0);

        player.initialize_scoreboard();
        let snapshot_bytes = client.pending_bytes.load(Ordering::Acquire);
        assert!(snapshot_bytes > 0);
        player.initialize_scoreboard();
        assert_eq!(client.pending_bytes.load(Ordering::Acquire), snapshot_bytes);

        if custom {
            let mut guard = player
                .custom_scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(CustomScoreboard::Java(board)) = guard.as_mut() {
                board.update_score(
                    &player,
                    ScoreboardScore::new("entry", "test_board", VarInt(9), None, None),
                );
            }
        } else {
            world
                .scoreboard
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .update_score(
                    &world,
                    ScoreboardScore::new("entry", "test_board", VarInt(9), None, None),
                );
        }
        client.try_send_packet(&CKeepAlive::new(1));

        let packets = read_scoreboard_packets(peer).await?;
        assert_login_packets(&packets)?;

        client.close();
        client.await_tasks().await;
        let _ = world.remove_player(&player, false).await;
    }
    server.stop();
    server.shutdown().await;
    Ok(())
}

#[tokio::test]
async fn custom_scoreboard_login_replays_fallback_world_teams() -> Result<(), Box<dyn Error>> {
    let directory = tempfile::tempdir()?;
    let server = test_server(&directory).await?;
    let world = server.worlds.load().first().cloned().ok_or("No world")?;
    for custom_member in [None, Some("someone_else"), Some("ScoreboardTest")] {
        let (player, peer) = player_with_transport(&server, &world)?;
        world.add_player(&player)?;
        let client = player.client.java().ok_or("No Java client")?;
        let team = test_team("world_team", vec![player.gameprofile.name.clone()]);
        let mut custom = Scoreboard::new();
        custom.add_objective(
            &NoTarget,
            ScoreboardObjective::new(
                "custom",
                TextComponent::text("Custom"),
                RenderType::Integer,
                None,
                "dummy",
            ),
        );
        if let Some(member) = custom_member {
            custom.add_team(
                &NoTarget,
                Team {
                    name: "custom_team".into(),
                    players: vec![member.into()],
                    ..team.clone()
                },
            );
        }
        player
            .as_java()
            .ok_or("No Java player")?
            .set_scoreboard(Some(custom));
        world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .add_team(&world, team);
        let overridden = custom_member == Some(player.gameprofile.name.as_str());
        assert_eq!(
            player.get_team_name().as_deref(),
            Some(if overridden {
                "custom_team"
            } else {
                "world_team"
            })
        );
        assert_eq!(client.pending_bytes.load(Ordering::Acquire), 0);
        player.initialize_scoreboard();
        player.initialize_scoreboard();
        client.try_send_packet(&CKeepAlive::new(1));
        let packets = read_scoreboard_packets(peer).await?;
        let objective_names = packets
            .iter()
            .filter(|p| p.id == CUpdateObjectives::to_id(CURRENT_MC_VERSION))
            .map(|p| (&p.payload[..]).get_str().map(String::from))
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(objective_names, ["custom"]);
        let mut team_names = packets
            .iter()
            .filter(|p| p.id == CSetPlayerTeam::to_id(CURRENT_MC_VERSION))
            .map(|p| (&p.payload[..]).get_str().map(String::from))
            .collect::<Result<Vec<_>, _>>()?;
        team_names.sort();
        let expected = if custom_member.is_some() {
            vec!["custom_team", "world_team"]
        } else {
            vec!["world_team"]
        };
        assert_eq!(team_names, expected);
        let world_team = packets
            .iter()
            .find(|packet| {
                packet.id == CSetPlayerTeam::to_id(CURRENT_MC_VERSION)
                    && (&packet.payload[..])
                        .get_str()
                        .is_ok_and(|name| name.as_ref() == "world_team")
            })
            .ok_or("Missing world team packet")?;
        let name = player.gameprofile.name.as_bytes();
        assert_eq!(
            world_team
                .payload
                .windows(name.len())
                .any(|bytes| bytes == name),
            !overridden
        );
        client.close();
        client.await_tasks().await;
        let _ = world.remove_player(&player, false).await;
        world
            .scoreboard
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove_team(&NoTarget, "world_team");
    }
    server.stop();
    server.shutdown().await;
    Ok(())
}
