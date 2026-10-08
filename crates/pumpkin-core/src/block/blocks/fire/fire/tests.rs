use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, RwLock};

use pumpkin_config::{AdvancedConfiguration, BasicConfiguration, TelemetryConfig};
use pumpkin_util::difficulty::Difficulty;
use pumpkin_world::chunk::ChunkData;
use rand::{SeedableRng, rngs::StdRng};

use crate::data::VanillaData;
use crate::plugin::api::events::EventPriority;
use crate::plugin::api::events::block::{
    block_burn::BlockBurnEvent, block_spread::BlockSpreadEvent,
};
use crate::plugin::{BoxFuture, EventHandler};
use crate::server::Server;

use super::*;

const SOURCE: BlockPos = BlockPos::new(8, 100, 8);
const TARGET: BlockPos = BlockPos::new(7, 99, 7);

async fn fire_world() -> (tempfile::TempDir, Arc<Server>, Arc<World>) {
    let temp = tempfile::tempdir().unwrap();
    let basic = BasicConfiguration {
        default_level_name: temp.path().to_str().unwrap().to_owned(),
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
    .await
    .unwrap();
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);
    world.level_info.rcu(|info| {
        let mut info = (**info).clone();
        info.difficulty = Difficulty::Normal;
        info.game_rules.spawn_mobs = false;
        info.game_rules.random_tick_speed = 0;
        info.game_rules.fire_spread_radius_around_player = -1;
        info
    });
    world.set_raining(false);
    (temp, server, world)
}

fn fire_scene(world: &World, support: &Block, age: u8) -> Arc<ChunkData> {
    // Only one loaded chunk; the hay is not adjacent to the source fire.
    let chunk = ChunkData::empty_sync(0, 0);
    chunk.set_block_absolute_y(8, 99, 8, support.default_state.id);
    chunk.set_block_absolute_y(7, 98, 7, Block::HAY_BLOCK.default_state.id);
    let mut properties = FireProperties::from_state_id(Block::FIRE.default_state.id);
    properties.age = age;
    chunk.set_block_absolute_y(8, 100, 8, properties.to_state_id(&Block::FIRE));
    world
        .level
        .loaded_chunks
        .insert(SOURCE.chunk_position(), chunk.clone());
    chunk
}

fn fire_random(skipped_words: usize) -> StdRng {
    let mut seed = [0; 32];
    seed[0] = 1;
    let mut random = StdRng::from_seed(seed);
    for _ in 0..skipped_words {
        random.next_u32();
    }
    random
}

fn tick_fire(world: &Arc<World>, random: &mut StdRng) {
    FireBlock.tick(
        &OnScheduledTickArgs {
            world,
            block: &Block::FIRE,
            position: &SOURCE,
        },
        random,
    );
    assert!(world.is_block_tick_scheduled(&SOURCE, &Block::FIRE));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scheduled_tick_stops_without_adjacent_fuel() {
    let (_temp, server, world) = fire_world().await;
    let mut unfueled = Vec::new();
    for age in [0, 3] {
        fire_scene(&world, &Block::STONE, age);
        assert!(!FireBlock::are_blocks_around_flammable(
            world.as_ref(),
            &SOURCE
        ));
        // Fixed stream position: age increases, six adjacent burns miss, then
        // the first air target above the distant hay passes the spread roll.
        tick_fire(&world, &mut fire_random(14));
        let source_age = (world.get_block(&SOURCE) == &Block::FIRE)
            .then(|| FireProperties::from_state_id(world.get_block_state_id(&SOURCE)).age);
        unfueled.push((source_age, world.get_block(&TARGET).id));
        if age == 3 {
            // The next real scheduled callback must extinguish age-four fire.
            // Forty steps cover the existing 30..=39-tick delay, with no random ticks.
            for _ in 0..40 {
                world.tick_chunks(&server);
            }
            assert_eq!(world.get_block(&SOURCE), &Block::AIR);
        }
    }

    for support in [&Block::OAK_PLANKS, &Block::NETHERRACK] {
        fire_scene(&world, support, 0);
        tick_fire(&world, &mut fire_random(14));
        assert_eq!(world.get_block(&SOURCE), &Block::FIRE);
        assert_eq!(world.get_block(&TARGET), &Block::FIRE);
    }
    assert_eq!(
        unfueled,
        [(Some(1), Block::AIR.id), (Some(4), Block::AIR.id)]
    );
}

#[derive(Default)]
struct FireEvents {
    cancel: AtomicBool,
    burns: Mutex<Vec<BlockPos>>,
    spreads: Mutex<Vec<(BlockPos, u8)>>,
}

impl EventHandler<BlockBurnEvent> for FireEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut BlockBurnEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            assert_eq!(event.igniting_block, &Block::FIRE);
            assert_eq!(event.block, &Block::OAK_PLANKS);
            assert_eq!(event.world.get_block(&event.block_pos), event.block);
            self.burns.lock().unwrap().push(event.block_pos);
            event.cancelled = self.cancel.load(Ordering::Relaxed);
        })
    }
}

impl EventHandler<BlockSpreadEvent> for FireEvents {
    fn handle_blocking<'a>(
        &'a self,
        _server: &'a Arc<Server>,
        event: &'a mut BlockSpreadEvent,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            assert_eq!(event.source_pos, SOURCE);
            assert_eq!(event.world.get_block(&event.target_pos), &Block::AIR);
            assert_eq!(event.new_state_id.to_block(), &Block::FIRE);
            self.spreads.lock().unwrap().push((
                event.target_pos,
                FireProperties::from_state_id(event.new_state_id).age,
            ));
            event.cancelled = self.cancel.load(Ordering::Relaxed);
        })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scheduled_tick_respects_spread_cancellation() {
    let (_temp, server, world) = fire_world().await;
    let events = Arc::new(FireEvents::default());
    server.plugin_manager.register::<BlockSpreadEvent, _>(
        events.clone(),
        EventPriority::Normal,
        true,
    );
    for cancel in [false, true] {
        events.cancel.store(cancel, Ordering::Relaxed);
        events.spreads.lock().unwrap().clear();
        fire_scene(&world, &Block::NETHERRACK, 0);
        tick_fire(&world, &mut fire_random(14));
        assert!(
            events
                .spreads
                .lock()
                .unwrap()
                .iter()
                .any(|(pos, _)| *pos == TARGET)
        );
        let expected = if cancel { &Block::AIR } else { &Block::FIRE };
        assert_eq!(world.get_block(&TARGET), expected);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scheduled_tick_respects_burn_cancellation() {
    let (_temp, server, world) = fire_world().await;
    let events = Arc::new(FireEvents::default());
    server.plugin_manager.register::<BlockBurnEvent, _>(
        events.clone(),
        EventPriority::Normal,
        true,
    );
    for cancel in [false, true] {
        events.cancel.store(cancel, Ordering::Relaxed);
        events.burns.lock().unwrap().clear();
        let chunk = fire_scene(&world, &Block::NETHERRACK, 0);
        chunk.set_block_absolute_y(9, 99, 8, Block::STONE.default_state.id);
        chunk.set_block_absolute_y(9, 100, 8, Block::OAK_PLANKS.default_state.id);
        // This fixed stream position burns the eastern planks into fire.
        tick_fire(&world, &mut fire_random(2));
        assert_eq!(*events.burns.lock().unwrap(), [SOURCE.east()]);
        let expected = if cancel {
            &Block::OAK_PLANKS
        } else {
            &Block::FIRE
        };
        assert_eq!(world.get_block(&SOURCE.east()), expected);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scheduled_tick_max_age_extinction_uses_original_age() {
    let (_temp, server, world) = fire_world().await;
    let events = Arc::new(FireEvents::default());
    events.cancel.store(true, Ordering::Relaxed);
    server.plugin_manager.register::<BlockBurnEvent, _>(
        events.clone(),
        EventPriority::Normal,
        true,
    );
    server
        .plugin_manager
        .register::<BlockSpreadEvent, _>(events, EventPriority::Normal, true);
    // Age increment succeeds; the following max-age extinction roll is zero.
    let skipped = 2;
    let mut outcomes = Vec::new();
    for age in [15, 14] {
        let chunk = fire_scene(&world, &Block::STONE, age);
        chunk.set_block_absolute_y(9, 99, 8, Block::STONE.default_state.id);
        chunk.set_block_absolute_y(9, 100, 8, Block::OAK_PLANKS.default_state.id);
        assert!(FireBlock::are_blocks_around_flammable(
            world.as_ref(),
            &SOURCE
        ));
        tick_fire(&world, &mut fire_random(skipped));
        outcomes.push(
            (world.get_block(&SOURCE) == &Block::FIRE)
                .then(|| FireProperties::from_state_id(world.get_block_state_id(&SOURCE)).age),
        );
        assert_eq!(world.get_block(&SOURCE.east()), &Block::OAK_PLANKS);
        assert_eq!(world.get_block(&TARGET), &Block::AIR);
    }
    assert_eq!(outcomes, [None, Some(15)], "RNG offset {skipped}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scheduled_tick_adjacent_burn_uses_original_age() {
    let (_temp, server, world) = fire_world().await;
    let burns = Arc::new(FireEvents::default());
    let spreads = Arc::new(FireEvents::default());
    spreads.cancel.store(true, Ordering::Relaxed);
    server
        .plugin_manager
        .register::<BlockBurnEvent, _>(burns.clone(), EventPriority::Normal, true);
    server
        .plugin_manager
        .register::<BlockSpreadEvent, _>(spreads, EventPriority::Normal, true);
    // Age increases 13 -> 14 and the eastern burn succeeds. Its replacement
    // roll is four for the original age, but five for the incremented age.
    let skipped = 7197;
    let mut outcomes = Vec::new();
    for cancel in [false, true] {
        burns.cancel.store(cancel, Ordering::Relaxed);
        burns.burns.lock().unwrap().clear();
        let chunk = fire_scene(&world, &Block::NETHERRACK, 13);
        chunk.set_block_absolute_y(9, 99, 8, Block::STONE.default_state.id);
        chunk.set_block_absolute_y(9, 100, 8, Block::OAK_PLANKS.default_state.id);
        tick_fire(&world, &mut fire_random(skipped));
        assert_eq!(*burns.burns.lock().unwrap(), [SOURCE.east()]);
        outcomes.push(world.get_block(&SOURCE.east()).id);
        assert_eq!(world.get_block(&TARGET), &Block::AIR);
    }
    // Inspect replacement kind, not durable age: neighbor age preservation is separate.
    assert_eq!(
        outcomes,
        [Block::FIRE.id, Block::OAK_PLANKS.id],
        "RNG offset {skipped}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scheduled_tick_air_spread_odds_use_original_age() {
    let (_temp, server, world) = fire_world().await;
    let events = Arc::new(FireEvents::default());
    events.cancel.store(true, Ordering::Relaxed);
    server.plugin_manager.register::<BlockSpreadEvent, _>(
        events.clone(),
        EventPriority::Normal,
        true,
    );
    fire_scene(&world, &Block::NETHERRACK, 8);
    assert_eq!(FireBlock.get_burn_chance(&world, &TARGET), 60);
    assert!(!FireBlock::is_increased_burnout_biome(&world, &TARGET));
    // Age increases, then the first air target above the hay rolls three.
    let skipped = 14;
    tick_fire(&world, &mut fire_random(skipped));
    assert_eq!(world.get_block(&TARGET), &Block::AIR);
    assert_eq!(
        FireProperties::from_state_id(world.get_block_state_id(&SOURCE)).age,
        9
    );
    // At normal difficulty, original age eight accepts roll three; age nine does not.
    let targets: Vec<_> = events
        .spreads
        .lock()
        .unwrap()
        .iter()
        .map(|(pos, _)| *pos)
        .collect();
    assert_eq!(targets, [TARGET], "RNG offset {skipped}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scheduled_tick_air_spread_proposes_original_age() {
    let (_temp, server, world) = fire_world().await;
    let events = Arc::new(FireEvents::default());
    events.cancel.store(true, Ordering::Relaxed);
    server.plugin_manager.register::<BlockSpreadEvent, _>(
        events.clone(),
        EventPriority::Normal,
        true,
    );
    fire_scene(&world, &Block::NETHERRACK, 0);
    // The same stream accepts this target, with no child-age increment.
    let skipped = 14;
    tick_fire(&world, &mut fire_random(skipped));
    assert_eq!(world.get_block(&TARGET), &Block::AIR);
    assert_eq!(
        FireProperties::from_state_id(world.get_block_state_id(&SOURCE)).age,
        1
    );
    // Ages zero and one have the same spread odds; only the event's proposed age differs.
    assert_eq!(
        *events.spreads.lock().unwrap(),
        [(TARGET, 0)],
        "RNG offset {skipped}"
    );
}
