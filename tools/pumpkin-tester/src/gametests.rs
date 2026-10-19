use std::sync::{Arc, Mutex as StdMutex};

use pumpkin_core::{
    server::{
        Server,
        builtin_game_tests::builtin_game_test_ids,
        server_test_manager::{
            GameTestBatchReport, GameTestQueueEntry, GameTestRetryOptions,
            prepare_test_run_with_reporter,
        },
    },
    world::{BlockFlags, World},
};
use pumpkin_data::{Block, dimension::Dimension};
use pumpkin_gametest::{GameTestReporter, GameTestRunner};
use pumpkin_util::{
    math::{position::BlockPos, vector2::Vector2},
    text::TextComponent,
};

const TEST_GRID_SPACING: i32 = 16;
const TESTS_PER_ROW: i32 = 4;
const TEST_ROWS_PER_LAYER: i32 = 4;
const TEST_LAYER_SPACING_Y: i32 = 12;
const TEST_BASE_X: i32 = 65;
const TEST_BASE_Y: i32 = 80;
const TEST_BASE_Z: i32 = 66;
const CREEPER_TEST_X: i32 = -64;
const CREEPER_TEST_Z: i32 = -64;
const MAX_GAMETEST_TICKS: usize = 500;

#[derive(Default)]
struct CaptureReporter {
    messages: StdMutex<Vec<String>>,
}

impl CaptureReporter {
    fn snapshot(&self) -> Vec<String> {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl GameTestReporter for CaptureReporter {
    fn send_message(&self, message: TextComponent) {
        self.messages
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(message.get_text());
    }
}

pub async fn run_gametests(
    server: &Arc<Server>,
    runner: &mut GameTestRunner,
    summary_lines: &mut Vec<String>,
) -> Result<(), String> {
    let world = server.get_world_from_dimension(&Dimension::OVERWORLD);

    let mut all_tests = vec![
        "minecraft:always_pass",
        "pumpkin:summon_command_regression",
        "pumpkin:creeper_should_run_from_cat",
    ];
    all_tests.extend_from_slice(builtin_game_test_ids());

    let reporter = Arc::new(CaptureReporter::default());
    let report_sink: Arc<dyn GameTestReporter> = reporter.clone();
    let report = Arc::new(GameTestBatchReport::new(
        report_sink.clone(),
        all_tests.len(),
    ));

    // Pre-load the 4x4 chunk test arena once so all 3D-stacked tests run in memory
    let base_cx = TEST_BASE_X >> 4;
    let base_cz = TEST_BASE_Z >> 4;
    force_load_surroundings_chunks(
        &world,
        base_cx,
        base_cx + TESTS_PER_ROW - 1,
        base_cz,
        base_cz + TEST_ROWS_PER_LAYER - 1,
    );
    for cx in base_cx..(base_cx + TESTS_PER_ROW) {
        for cz in base_cz..(base_cz + TEST_ROWS_PER_LAYER) {
            world
                .level
                .get_or_fetch_chunk(Vector2::new(cx, cz), |_| ())
                .await;
        }
    }

    let mut grid_slot = 0i32;
    for &test_id in &all_tests {
        let entry = if test_id == "pumpkin:creeper_should_run_from_cat" {
            prepare_flat_test_surroundings(&world, CREEPER_TEST_X, CREEPER_TEST_Z).await;
            GameTestQueueEntry::new(
                test_id,
                world.clone(),
                CREEPER_TEST_X,
                CREEPER_TEST_Z,
                0,
                GameTestRetryOptions::no_retries(),
                report.clone(),
            )
        } else {
            let slots_per_layer = TESTS_PER_ROW * TEST_ROWS_PER_LAYER;
            let layer = grid_slot / slots_per_layer;
            let in_layer = grid_slot % slots_per_layer;
            let column = in_layer % TESTS_PER_ROW;
            let row = in_layer / TESTS_PER_ROW;
            grid_slot += 1;

            let test_x = TEST_BASE_X + column * TEST_GRID_SPACING;
            let test_y = TEST_BASE_Y + layer * TEST_LAYER_SPACING_Y;
            let test_z = TEST_BASE_Z + row * TEST_GRID_SPACING;

            GameTestQueueEntry::new(
                test_id,
                world.clone(),
                test_x,
                test_z,
                0,
                GameTestRetryOptions::no_retries(),
                report.clone(),
            )
            .with_y(test_y)
        };

        let manager = prepare_test_run_with_reporter(server, entry, report_sink.clone())
            .await
            .map_err(|err| format!("Failed to prepare GameTest '{test_id}': {err}"))?;
        runner.enqueue(manager);
    }

    for _ in 0..MAX_GAMETEST_TICKS {
        if runner.is_empty() {
            break;
        }
        server.tick();
        runner.tick().await;
    }

    let finished = runner.is_empty();
    let passed = report.all_required_passed();
    let messages = reporter.snapshot();
    for msg in &messages {
        summary_lines.push(format!("  {msg}"));
    }

    runner.clear();

    if !finished {
        return Err(format!(
            "GameTest suite did not finish within {MAX_GAMETEST_TICKS} ticks:\n{}",
            messages.join("\n")
        ));
    }
    if !passed {
        return Err(format!(
            "One or more required GameTests failed ({} failed):\n{}",
            report.failed_required(),
            messages.join("\n")
        ));
    }

    Ok(())
}

fn force_load_surroundings_chunks(
    world: &World,
    min_cx: i32,
    max_cx: i32,
    min_cz: i32,
    max_cz: i32,
) {
    let mut forced = world
        .forced_chunks
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut chunk_loading = world
        .level
        .chunk_loading
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for cx in min_cx..=max_cx {
        for cz in min_cz..=max_cz {
            let chunk = Vector2::new(cx, cz);
            forced.insert(chunk);
            chunk_loading.add_force_ticket(chunk);
        }
    }
    chunk_loading.send_change();
}

async fn prepare_flat_test_surroundings(world: &Arc<World>, test_x: i32, test_z: i32) {
    let min_cx = (test_x - 20) >> 4;
    let max_cx = (test_x + 26) >> 4;
    let min_cz = (test_z - 20) >> 4;
    let max_cz = (test_z + 26) >> 4;
    force_load_surroundings_chunks(world, min_cx, max_cx, min_cz, max_cz);
    for cx in min_cx..=max_cx {
        for cz in min_cz..=max_cz {
            world
                .level
                .get_or_fetch_chunk(Vector2::new(cx, cz), |_| ())
                .await;
        }
    }

    let mut surface_y = 64;
    for y in (40..=200).rev() {
        if !world
            .get_block_state(&BlockPos::new(test_x, y, test_z))
            .is_air()
        {
            surface_y = y;
            break;
        }
    }

    let stone_id = Block::STONE.default_state.id;
    let magma_id = Block::MAGMA_BLOCK.default_state.id;
    let air_id = Block::AIR.default_state.id;
    let flags = BlockFlags::FORCE_STATE
        | BlockFlags::SKIP_BLOCK_ADDED_CALLBACK
        | BlockFlags::SKIP_BLOCK_ENTITY_REPLACED_CALLBACK;

    for dx in -20..=26 {
        for dz in -20..=26 {
            let x = test_x + dx;
            let z = test_z + dz;
            let is_origin_col = dx == 0 && dz == 0;
            let in_enclosure = (-1..=5).contains(&dx) && (0..=6).contains(&dz);
            let in_sw_dead_strip = dx <= 1 && dz >= 4 && (4..=6).contains(&(dx + dz));
            let in_valid_flee_sector = !in_enclosure && !in_sw_dead_strip;
            for y in (surface_y - 8)..=(surface_y + 20) {
                let state_id = if is_origin_col {
                    match y.cmp(&surface_y) {
                        std::cmp::Ordering::Less => stone_id,
                        std::cmp::Ordering::Equal => magma_id,
                        std::cmp::Ordering::Greater => air_id,
                    }
                } else if in_valid_flee_sector && (surface_y - 2..=surface_y + 16).contains(&y) {
                    if y == surface_y + 16 {
                        magma_id
                    } else if (y - surface_y).rem_euclid(2) == 0 {
                        stone_id
                    } else {
                        air_id
                    }
                } else {
                    air_id
                };
                world.set_block_state(&BlockPos::new(x, y, z), state_id, flags);
            }
        }
    }
}
