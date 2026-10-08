use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use arc_swap::ArcSwap;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_data::{
    data_component_impl::EquipmentSlot, dimension::Dimension, entity::EntityType, item::Item,
    item_stack::ItemStack, packet::CURRENT_MC_VERSION, particle::Particle, sound::Sound,
};
use pumpkin_protocol::{
    ConnectionState,
    java::client::play::{CParticle, CSoundEffect},
    packet::MultiVersionJavaPacket,
    ser::NetworkReadExt,
};
use pumpkin_util::{
    GameMode,
    math::{vector2::Vector2, vector3::Vector3},
};
use pumpkin_world::chunk::ChunkData;
use tokio::sync::mpsc::UnboundedReceiver;

use super::{JavaClient, outgoing::OutgoingPacket};
use crate::{
    data::VanillaData,
    entity::{Entity, EntityBase, combat::AttackType, passive::cow::CowEntity, player::Player},
    net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig, decrement_pending_bytes},
    server::Server,
    world::World,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct SweepFixture {
    _directory: tempfile::TempDir,
    server: Arc<Server>,
    world: Arc<World>,
    player: Arc<Player>,
    packets: UnboundedReceiver<OutgoingPacket>,
}

impl SweepFixture {
    async fn new() -> TestResult<Self> {
        let directory = tempfile::tempdir()?;
        let basic = BasicConfiguration {
            default_level_name: directory
                .path()
                .to_str()
                .ok_or("non-UTF8 test directory")?
                .to_owned(),
            allow_nether: false,
            allow_end: false,
            allow_chat_reports: false,
            use_favicon: false,
            ..Default::default()
        };
        let mut advanced = AdvancedConfiguration::default();
        advanced.networking.bedrock.online_mode = false;
        let server = Server::new(
            basic,
            advanced,
            TelemetryConfig::default(),
            VanillaData {
                banned_ip_list: RwLock::default(),
                banned_player_list: RwLock::default(),
                operator_config: RwLock::default(),
                user_cache: RwLock::default(),
                whitelist_config: RwLock::default(),
            },
            Vec::new(),
        )
        .await?;
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));
        let profile = GameProfile {
            id: uuid::Uuid::new_v4(),
            name: "Sweeper".to_owned(),
            properties: ArcSwap::from_pointee(Vec::new()),
            profile_actions: None,
        };
        let (send, packets) = tokio::sync::mpsc::unbounded_channel();
        // Exercise the production outgoing queue without sockets or a writer task.
        let client = JavaClient {
            id: 1,
            version: CURRENT_MC_VERSION.into(),
            gameprofile: profile.clone(),
            config: ArcSwap::from_pointee(PlayerConfig::default()),
            server_address: String::new(),
            connection_state: ConnectionState::Play.into(),
            address: ([127, 0, 0, 1], 0).into(),
            brand: ArcSwap::from_pointee(None),
            player: ArcSwap::from_pointee(None),
            tasks: tokio_util::task::TaskTracker::new(),
            rt_handle: tokio::runtime::Handle::current(),
            close_token: tokio_util::sync::CancellationToken::new(),
            outgoing_packet_queue_send: send,
            outgoing_packet_queue_recv: None,
            pending_bytes: Arc::default(),
            network_writer: std::sync::Mutex::new(None),
            network_reader: std::sync::Mutex::new(None),
            wait_for_keep_alive: false.into(),
            received_movement_this_tick: false.into(),
            keep_alive_id: 0.into(),
            last_keep_alive_time: Instant::now().into(),
            last_packet_time: Instant::now().into(),
            pending_keep_alives: std::sync::Mutex::default(),
            packet_sequence: (-1).into(),
            packet_limiter: PacketRateLimiter::new(false, 0.0, 0.0),
            suspend_flushing: Arc::default(),
        };
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            PlayerConfig::default(),
            &world,
            GameMode::Survival,
        ));
        player.set_client_loaded(true);
        player.get_entity().set_pos(Vector3::new(8.0, 100.0, 8.0));
        world.add_player(&player)?;
        let sword = ItemStack::new(1, &Item::IRON_SWORD);
        player.inventory().set_held_item(sword.clone());
        player
            .living_entity
            .send_equipment_changes(&[(EquipmentSlot::MAIN_HAND, sword)]);
        Ok(Self {
            _directory: directory,
            server,
            world,
            player,
            packets,
        })
    }

    fn cow(&self, pos: Vector3<f64>) -> Arc<dyn EntityBase> {
        let cow = CowEntity::new(Entity::new(self.world.clone(), pos, &EntityType::COW));
        self.world.add_entity_silent(cow.clone());
        cow
    }

    fn effects(&mut self) -> TestResult<(usize, Vec<i32>)> {
        let mut sweeps = 0;
        let mut sounds = Vec::new();
        while let Ok(packet) = self.packets.try_recv() {
            let OutgoingPacket::Data { data, .. } = packet else {
                continue;
            };
            decrement_pending_bytes(
                &self.player.client.java().ok_or("not Java")?.pending_bytes,
                data.len(),
            );
            let mut bytes = data.as_ref();
            let id = bytes.get_var_int()?.0;
            if id == CSoundEffect::to_id(CURRENT_MC_VERSION) {
                sounds.push(bytes.get_var_int()?.0 - 1);
            } else if id == CParticle::to_id(CURRENT_MC_VERSION)
                && bytes.get_var_int()?.0 == Particle::SweepAttack as i32
            {
                sweeps += 1;
            }
        }
        Ok((sweeps, sounds))
    }

    async fn close(self) {
        if let Some(client) = self.player.client.java() {
            client.close();
        }
        self.world.players.store(Arc::new(Vec::new()));
        self.server.shutdown().await;
        self.world.level.world_portal.store(Arc::new(None));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn airborne_sword_attacks_do_not_sweep_bystanders() -> TestResult {
    let mut fixture = SweepFixture::new().await?;
    let mut results = Vec::new();
    // Vanilla Player.isSweepAttack requires onGround; ascent is strong, descent can crit.
    for (on_ground, fall_distance) in [(true, 0.0), (false, 0.0), (false, 0.25)] {
        let player = &fixture.player;
        player
            .get_entity()
            .on_ground
            .store(on_ground, Ordering::Relaxed);
        player.living_entity.fall_distance.store(fall_distance);
        player.last_attacked_ticks.store(100, Ordering::Relaxed);
        let target = fixture.cow(Vector3::new(8.0, 100.0, 10.0));
        let bystander = fixture.cow(Vector3::new(8.5, 100.0, 10.0));
        let target_living = target.get_living_entity().ok_or("target not living")?;
        let bystander_living = bystander
            .get_living_entity()
            .ok_or("bystander not living")?;
        let target_health = target_living.health.load();
        let bystander_health = bystander_living.health.load();
        fixture.effects()?;

        fixture.player.attack(&target);

        let primary_damage = target_health - target_living.health.load();
        let sweep_damage = bystander_health - bystander_living.health.load();
        let (particles, sounds) = fixture.effects()?;
        results.push((primary_damage, sweep_damage, particles, sounds));
        fixture.world.remove_entity(target.as_ref());
        fixture.world.remove_entity(bystander.as_ref());
    }
    fixture.close().await;

    for (index, expected_sound) in [
        Sound::EntityPlayerAttackSweep,
        Sound::EntityPlayerAttackStrong,
        Sound::EntityPlayerAttackCrit,
    ]
    .into_iter()
    .enumerate()
    {
        let (damage, sweep_damage, particles, sounds) = &results[index];
        assert!(*damage > 0.0, "primary target must be hurt in case {index}");
        assert_eq!(
            *sweep_damage,
            if index == 0 { 1.0 } else { 0.0 },
            "bystander damage in case {index}"
        );
        assert_eq!(
            *particles,
            usize::from(index == 0),
            "sweep particles in case {index}"
        );
        assert!(
            sounds.contains(&(expected_sound as i32)),
            "attack sound in case {index}: {sounds:?}"
        );
        assert_eq!(
            sounds.contains(&(Sound::EntityPlayerAttackSweep as i32)),
            index == 0
        );
    }
    assert_eq!(
        results[0].0, results[1].0,
        "ascent must retain ordinary strong damage"
    );
    assert_eq!(
        results[2].0,
        results[1].0 * 1.5,
        "falling critical damage must be retained"
    );
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sword_sweep_gate_preserves_attack_priority_and_charge() -> TestResult {
    let fixture = SweepFixture::new().await?;
    let player = &fixture.player;
    let mut actual = Vec::new();
    for (ground, fall, sprint, charge, item) in [
        (true, 0.0, false, 1.0, &Item::IRON_SWORD),
        (false, 0.0, false, 1.0, &Item::IRON_SWORD),
        (false, 0.25, false, 1.0, &Item::IRON_SWORD),
        (false, 0.25, true, 1.0, &Item::IRON_SWORD),
        (true, 0.0, false, 0.9, &Item::IRON_SWORD),
        (true, 0.0, false, 0.91, &Item::IRON_SWORD),
        (true, 0.0, false, 1.0, &Item::IRON_AXE),
    ] {
        player
            .get_entity()
            .on_ground
            .store(ground, Ordering::Relaxed);
        player.living_entity.fall_distance.store(fall);
        player.set_sprinting(sprint);
        player.inventory().set_held_item(ItemStack::new(1, item));
        actual.push(AttackType::new(player, charge));
    }
    fixture.close().await;
    assert_eq!(
        actual,
        [
            AttackType::Sweeping,
            AttackType::Strong,
            AttackType::Critical,
            AttackType::Knockback,
            AttackType::Weak,
            AttackType::Sweeping,
            AttackType::Strong
        ]
    );
    Ok(())
}
