#![expect(clippy::expect_used, reason = "test setup and packet assertions")]

use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use arc_swap::ArcSwap;
use futures::future::BoxFuture;
use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, telemetry::TelemetryConfig};
use pumpkin_data::{
    Block,
    block_properties::{ChestLikeProperties, ChestType, HorizontalFacing},
    dimension::Dimension,
    item::Item,
    item_stack::ItemStack,
    packet::CURRENT_MC_VERSION,
};
use pumpkin_inventory::Inventory;
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_protocol::{
    ConnectionState, VarInt,
    java::{
        client::play::{COpenScreen, CSystemChatMessage},
        server::play::SUseItemOn,
    },
    packet::MultiVersionJavaPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt},
};
use pumpkin_util::{
    GameMode,
    math::{position::BlockPos, vector2::Vector2, vector3::Vector3},
    world_seed::Seed,
};
use pumpkin_world::chunk::ChunkData;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::UnboundedReceiver,
};

use super::{JavaClient, outgoing::OutgoingPacket, pending::PendingConnection};
use crate::{
    block::entities::{BlockEntity, chest::ChestBlockEntity},
    data::VanillaData,
    entity::{EntityBase, player::Player},
    net::{ClientPlatform, GameProfile, PacketRateLimiter, PlayerConfig},
    plugin::{
        EventHandler,
        api::events::{
            EventPriority, inventory::inventory_open::InventoryOpenEvent,
            player::player_interact_event::PlayerInteractEvent,
        },
    },
    server::Server,
    world::World,
};

const LOOT_TABLE: &str = "minecraft:chests/simple_dungeon";
const LOOT_SEED: i64 = 3732;
const POSITIONS: [BlockPos; 2] = [BlockPos::new(4, 64, 4), BlockPos::new(3, 64, 4)];

struct ChestFixture {
    server: Arc<Server>,
    world: Arc<World>,
    player: Arc<Player>,
    outgoing: UnboundedReceiver<OutgoingPacket>,
    _peer: TcpStream,
    _directory: tempfile::TempDir,
}

impl ChestFixture {
    async fn new() -> Self {
        let directory = tempfile::tempdir().expect("temporary world");
        let basic = BasicConfiguration {
            default_level_name: directory.path().to_string_lossy().into_owned(),
            seed: Seed(0),
            allow_nether: false,
            allow_end: false,
            use_favicon: false,
            ..Default::default()
        };
        let mut advanced = AdvancedConfiguration::default();
        advanced.networking.bedrock.online_mode = false;
        advanced.player_data.save_player_data = false;
        advanced.advancement.save_advancements = false;
        let server = Server::new(
            basic,
            advanced,
            TelemetryConfig {
                enabled: false,
                ..Default::default()
            },
            VanillaData {
                banned_ip_list: RwLock::default(),
                banned_player_list: RwLock::default(),
                operator_config: RwLock::default(),
                user_cache: RwLock::default(),
                whitelist_config: RwLock::default(),
            },
            vec![],
        )
        .await
        .expect("create server");
        let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
        world
            .level
            .loaded_chunks
            .insert(Vector2::new(0, 0), ChunkData::empty_sync(0, 0));

        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("local test socket");
        let peer = TcpStream::connect(listener.local_addr().expect("listener address"))
            .await
            .expect("connect test peer");
        let (stream, address) = listener.accept().await.expect("accept test peer");
        let pending = PendingConnection::new(
            stream,
            address,
            0,
            PacketRateLimiter::new(false, 0.0, 0.0),
            Arc::downgrade(&server),
        );
        pending.connection_state.store(ConnectionState::Play);
        let profile = GameProfile {
            id: uuid::Uuid::new_v4(),
            name: "ChestTest".into(),
            properties: ArcSwap::from_pointee(vec![]),
            profile_actions: None,
        };
        let mut client =
            JavaClient::from_pending(pending, profile.clone(), PlayerConfig::default());
        // Observe the real serialized output without starting a socket writer.
        let outgoing = client
            .outgoing_packet_queue_recv
            .take()
            .expect("outgoing queue");
        let player = Arc::new(Player::new(
            Arc::new(ClientPlatform::Java(client)),
            profile,
            PlayerConfig::default(),
            &world,
            GameMode::Spectator,
        ));
        player.set_client_loaded(true);
        player.get_entity().set_pos(Vector3::new(4.5, 64.0, 5.5));
        world.players.store(Arc::new(vec![player.clone()]));
        Self {
            server,
            world,
            player,
            outgoing,
            _peer: peer,
            _directory: directory,
        }
    }

    fn chest(&self, index: usize, chest_type: ChestType, pending: bool) -> Arc<ChestBlockEntity> {
        let position = POSITIONS[index];
        let mut properties = ChestLikeProperties::default(&Block::CHEST);
        properties.facing = HorizontalFacing::North;
        properties.r#type = chest_type;
        self.world
            .level
            .set_block_state(&position, properties.to_state_id(&Block::CHEST));
        let mut nbt = NbtCompound::new();
        if pending {
            nbt.put_string("LootTable", LOOT_TABLE.to_string());
            nbt.put_long("LootTableSeed", LOOT_SEED);
        }
        let chest = Arc::new(ChestBlockEntity::from_nbt(&nbt, position));
        self.world.add_block_entity(chest.clone());
        chest
    }

    fn open(&self, position: BlockPos) -> Option<u8> {
        let factory = self
            .server
            .block_registry
            .get_screen_handler_factory(
                &Block::CHEST,
                &self.player,
                &position,
                &self.server,
                &self.world,
            )
            .expect("unobstructed chest has a factory");
        self.player
            .open_handled_screen(factory.as_ref(), Some(position))
    }

    fn packets(&mut self) -> Vec<(i32, Vec<u8>)> {
        let mut packets = Vec::new();
        while let Ok(packet) = self.outgoing.try_recv() {
            if let OutgoingPacket::Data { data, .. } = packet {
                let mut reader = &data[..];
                let id = reader.get_var_int().expect("packet id").0;
                packets.push((id, reader.to_vec()));
            }
        }
        packets
    }

    fn assert_refusal(&mut self) {
        let packets = self.packets();
        assert_eq!(
            packets.len(),
            1,
            "refusal sends only the overlay, never OpenScreen"
        );
        assert_eq!(packets[0].0, CSystemChatMessage::to_id(CURRENT_MC_VERSION));
        let mut reader = packets[0].1.as_slice();
        let component = reader
            .get_compound_nbt_borrowed(&CURRENT_MC_VERSION)
            .expect("chat component NBT")
            .expect("chat component compound");
        assert_eq!(
            component.get_string("translate"),
            Some("container.spectatorCantOpen")
        );
        assert_eq!(component.get_string("color"), Some("red"));
        assert!(reader.get_bool().expect("overlay flag"));
        assert!(reader.is_empty());
        assert!(self.player.open_container_pos.load().is_none());
    }

    fn assert_opened(&mut self) {
        let packets = self.packets();
        assert_eq!(
            packets
                .iter()
                .filter(|(id, _)| *id == COpenScreen::to_id(CURRENT_MC_VERSION))
                .count(),
            1
        );
        assert!(
            packets
                .iter()
                .all(|(id, _)| *id != CSystemChatMessage::to_id(CURRENT_MC_VERSION))
        );
        self.player.close_handled_screen();
        self.packets();
    }

    async fn shutdown(self) {
        self.world.players.store(Arc::new(vec![]));
        self.server.shutdown().await;
    }
}

fn assert_pending(chest: &ChestBlockEntity) {
    let mut saved = NbtCompound::new();
    chest.write_nbt(&mut saved);
    assert_eq!(saved.get_string("LootTable"), Some(LOOT_TABLE));
    assert_eq!(saved.get_long("LootTableSeed"), Some(LOOT_SEED));
    assert!(chest.is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spectator_pending_chest_loot() {
    let mut fixture = ChestFixture::new().await;
    for (double, pending) in [
        (false, [true, false]),
        (true, [true, false]),
        (true, [false, true]),
        (true, [true, true]),
    ] {
        fixture.player.gamemode.store(GameMode::Spectator);
        let first = fixture.chest(
            0,
            if double {
                ChestType::Right
            } else {
                ChestType::Single
            },
            pending[0],
        );
        let mut chests = vec![first];
        if double {
            chests.push(fixture.chest(1, ChestType::Left, pending[1]));
        }
        fixture.packets();
        for position in POSITIONS.iter().take(chests.len()) {
            let factory = fixture
                .server
                .block_registry
                .get_screen_handler_factory(
                    &Block::CHEST,
                    &fixture.player,
                    position,
                    &fixture.server,
                    &fixture.world,
                )
                .expect("pending loot must not hide the factory");
            assert!(
                factory
                    .create_screen_handler(1, &fixture.player.inventory, fixture.player.as_ref())
                    .is_none(),
                "spectator menu must reject pending loot, including the other half"
            );
            assert!(fixture.open(*position).is_none());
            fixture.assert_refusal();
            for (chest, was_pending) in chests.iter().zip(pending) {
                if was_pending {
                    assert_pending(chest);
                }
            }
            assert!(chests.iter().all(|chest| chest.is_empty()));
        }
        fixture.player.gamemode.store(GameMode::Creative);
        assert!(fixture.open(POSITIONS[0]).is_some());
        for (chest, was_pending) in chests.iter().zip(pending) {
            assert!(!chest.has_loot_table());
            if was_pending {
                assert!(
                    !chest.is_empty(),
                    "Creative still generates real loot after refusal"
                );
            }
        }
        fixture.assert_opened();
        fixture.player.gamemode.store(GameMode::Spectator);
        assert!(fixture.open(POSITIONS[0]).is_some());
        fixture.assert_opened();
    }
    fixture.shutdown().await;
}

#[derive(Default)]
struct CancelOpening {
    interact: AtomicBool,
    inventory: AtomicBool,
    interact_calls: AtomicUsize,
    inventory_calls: AtomicUsize,
}

impl EventHandler<PlayerInteractEvent> for CancelOpening {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut PlayerInteractEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            self.interact_calls.fetch_add(1, Ordering::Relaxed);
            event.cancelled = self.interact.load(Ordering::Relaxed);
        })
    }
}

impl EventHandler<InventoryOpenEvent> for CancelOpening {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut InventoryOpenEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            self.inventory_calls.fetch_add(1, Ordering::Relaxed);
            event.cancelled = self.inventory.load(Ordering::Relaxed);
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn spectator_chest_generated_and_cancelled() {
    let mut fixture = ChestFixture::new().await;
    for (chest_type, filled) in [
        (ChestType::Single, false),
        (ChestType::Single, true),
        (ChestType::Right, false),
        (ChestType::Right, true),
    ] {
        let chest = fixture.chest(0, chest_type, false);
        if chest_type == ChestType::Right {
            fixture.chest(1, ChestType::Left, false);
        }
        if filled {
            chest.set_stack(0, ItemStack::new(3, &Item::DIAMOND));
        }
        fixture.packets();
        assert!(fixture.open(POSITIONS[0]).is_some());
        fixture.assert_opened();
        assert_eq!(chest.is_empty(), !filled);
        if filled {
            let stack = chest.get_stack(0);
            assert_eq!(stack.item.id, Item::DIAMOND.id);
            assert_eq!(stack.item_count, 3);
        }
    }
    let chest = fixture.chest(0, ChestType::Single, true);
    fixture.packets();
    let cancel = Arc::new(CancelOpening::default());
    fixture
        .server
        .plugin_manager
        .register::<PlayerInteractEvent, _>(cancel.clone(), EventPriority::Normal, true);
    fixture
        .server
        .plugin_manager
        .register::<InventoryOpenEvent, _>(cancel.clone(), EventPriority::Normal, true);
    let packet = SUseItemOn {
        hand: VarInt(0),
        position: POSITIONS[0],
        face: VarInt(1),
        cursor_pos: Vector3::new(0.5, 1.0, 0.5),
        inside_block: false,
        is_against_world_border: false,
        sequence: VarInt(0),
    };
    for cancel_interact in [true, false] {
        cancel.interact.store(cancel_interact, Ordering::Relaxed);
        cancel.inventory.store(!cancel_interact, Ordering::Relaxed);
        let client = fixture.player.client.java().expect("Java test player");
        client
            .handle_use_item_on(&fixture.player, &packet, &fixture.server)
            .expect("valid spectator interaction");
        assert_pending(&chest);
        assert!(fixture.player.open_container_pos.load().is_none());
        assert!(
            fixture
                .packets()
                .iter()
                .all(|(id, _)| *id != COpenScreen::to_id(CURRENT_MC_VERSION)
                    && *id != CSystemChatMessage::to_id(CURRENT_MC_VERSION))
        );
    }
    assert_eq!(cancel.interact_calls.load(Ordering::Relaxed), 2);
    assert_eq!(cancel.inventory_calls.load(Ordering::Relaxed), 1);
    cancel.inventory.store(false, Ordering::Relaxed);
    let client = fixture.player.client.java().expect("Java test player");
    client
        .handle_use_item_on(&fixture.player, &packet, &fixture.server)
        .expect("uncancelled spectator interaction");
    fixture.assert_refusal();
    assert_pending(&chest);
    fixture.shutdown().await;
}
