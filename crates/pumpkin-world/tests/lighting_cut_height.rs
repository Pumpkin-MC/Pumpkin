//! Phase 5 regression tests for the sky light cut height under runtime terrain edits.
//!
//! The cut height is a cache: it promises that every column ceiling of a non-diverged
//! quadrant sits inside `[cut, cut + spread]`. Runtime digging and building break that
//! promise, and the quadrant divergence flag is what stops the engine from trusting a
//! stale promise. These tests exercise that flag end to end through the real engine.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::print_stdout)]

use pumpkin_config::world::LevelConfig;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::level::Level;
use pumpkin_world::lighting::sky_light_height::SkyLightTier;
use pumpkin_world::lighting::{DynamicLightEngine, SkyLightHeight, SkyLightHeightMigration};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tempfile::TempDir;

const SECTION_COUNT: usize = 24;
const MIN_Y: i32 = -64;
/// Highest stone block of the flat test terrain; everything above is open air.
const SURFACE_Y: i32 = 60;

fn make_level() -> Arc<Level> {
    let temp_dir = TempDir::new().unwrap();
    Level::from_root_folder(
        &LevelConfig::default(),
        temp_dir.path().to_path_buf(),
        42,
        Dimension::OVERWORLD,
    )
}

/// One loaded chunk of flat stone up to [`SURFACE_Y`], correctly lit: sky 15 above the
/// surface, 0 inside the ground. That is the steady state the engine would converge to,
/// so any darkness or brightness the tests observe comes from their own edits.
fn insert_flat_chunk(level: &Arc<Level>) -> Arc<ChunkData> {
    let chunk = ChunkData::empty(0, 0);

    for z in 0..16usize {
        for x in 0..16usize {
            for y in MIN_Y..=SURFACE_Y {
                chunk.set_block_absolute_y(x, y, z, Block::STONE.default_state.id);
            }
        }
    }

    {
        let mut light = chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        light.sky_light = (0..SECTION_COUNT)
            .map(|section| {
                let section_bottom = MIN_Y + (section as i32) * 16;
                if section_bottom > SURFACE_Y {
                    LightContainer::new_empty(15)
                } else {
                    LightContainer::new_empty(0)
                }
            })
            .collect();
        light.block_light = (0..SECTION_COUNT)
            .map(|_| LightContainer::new_empty(0))
            .collect();

        // The section straddling the surface is half ground, half sky.
        let straddling = ((SURFACE_Y - MIN_Y) / 16) as usize;
        for y in (SURFACE_Y + 1)..(MIN_Y + ((straddling as i32) + 1) * 16) {
            for z in 0..16usize {
                for x in 0..16usize {
                    light.sky_light[straddling].set(x, (y - MIN_Y) as usize % 16, z, 15);
                }
            }
        }
    }

    let chunk = Arc::new(chunk);
    level.loaded_chunks.insert(Vector2::new(0, 0), chunk.clone());
    chunk
}

/// Runs the queued light work to convergence, the way several ticks would.
fn settle(engine: &DynamicLightEngine, level: &Arc<Level>) {
    for _ in 0..2000 {
        if !engine.drain_queued(level).leftover {
            return;
        }
    }
    panic!("light updates did not converge");
}

fn set_block(level: &Arc<Level>, engine: &DynamicLightEngine, pos: BlockPos, block: BlockStateId) {
    level
        .read_chunk_sync(&Vector2::new(0, 0), |chunk| {
            chunk.set_block_absolute_y(pos.0.x as usize, pos.0.y, pos.0.z as usize, block);
        })
        .unwrap();
    engine.update_lighting_at(level, pos);
}

fn sky(engine: &DynamicLightEngine, level: &Arc<Level>, pos: BlockPos) -> u8 {
    engine.get_sky_light_level(level, &pos)
}

fn cut_of(chunk: &Arc<ChunkData>) -> SkyLightHeight {
    SkyLightHeightMigration::get(chunk)
}

/// A two-block-tall serpentine staircase descending from the surface, inside the NW
/// quadrant (`local_x < 8 && local_z < 8`).
///
/// Two blocks tall on purpose: single diagonal steps only touch at an edge, and light
/// crosses faces, not edges — a one-block staircase would not be connected at all.
fn staircase(steps: usize) -> Vec<BlockPos> {
    let mut out = Vec::with_capacity(steps);
    for i in 0..steps {
        let y = SURFACE_Y - i as i32;
        let row = i / 6;
        let col = (i % 6) as i32;
        let x = if row % 2 == 0 { 1 + col } else { 6 - col };
        out.push(BlockPos::new(x, y, 2));
    }
    out
}

fn dig_staircase(
    level: &Arc<Level>,
    engine: &DynamicLightEngine,
    steps: &[BlockPos],
) {
    for step in steps {
        // Block by block, head height first, exactly as a player would mine it.
        set_block(level, engine, *step, Block::AIR.default_state.id);
        set_block(
            level,
            engine,
            BlockPos::new(step.0.x, step.0.y + 1, step.0.z),
            Block::AIR.default_state.id,
        );
    }
    settle(engine, level);
}

/// Digging a covered staircase must light it far below the cut — and must *not* flag the
/// quadrant, because a tunnel leaves the surface intact.
///
/// This is the plan's Staircase-Dig-and-Block-Test, steps 1-3, with step 4 corrected. The
/// plan expected the quadrant to diverge after digging. It does not, and it should not:
/// the cut tracks each column's **ceiling**, not its floor. Mining a tunnel under an
/// unbroken surface leaves every ceiling at 60, still inside the band, so the cached
/// promise is still true. Tier 1 correctly answers "no open sky" down there — the
/// staircase is lit by sideways propagation from its mouth, not by direct sunlight, and
/// Tier 1 never suppresses that propagation. See `an_open_trench_...` for the dig that
/// really does invalidate the cut.
#[tokio::test(flavor = "multi_thread")]
async fn digging_a_covered_staircase_lights_it_below_the_cut() {
    let level = make_level();
    let chunk = insert_flat_chunk(&level);
    let engine = DynamicLightEngine::new();

    let before = cut_of(&chunk);
    let min_y = chunk.section.min_y;
    let height = SkyLightHeight::chunk_height(&chunk);
    let cut = before.decode(min_y, height);

    for (x, z) in [(2, 2), (12, 2), (2, 12), (12, 12)] {
        assert!(before.quadrant_uses_limit(x, z));
    }
    let steps = staircase(14);
    let bottom = *steps.last().unwrap();
    assert!(
        bottom.0.y < cut,
        "staircase bottom {} must sit below the cached cut {cut}",
        bottom.0.y
    );
    assert_eq!(
        before.tier(bottom.0.y, bottom.0.x, bottom.0.z, min_y, height),
        SkyLightTier::NoOpenSky,
        "the bottom is below the cut, so Tier 1 answers it without a column scan"
    );

    dig_staircase(&level, &engine, &steps);

    // The surface above the tunnel is untouched, so every quadrant keeps the fast path.
    let after = cut_of(&chunk);
    for (x, z) in [(2, 2), (12, 2), (2, 12), (12, 12)] {
        assert!(
            after.quadrant_uses_limit(x, z),
            "a tunnel does not move any ceiling, so quadrant ({x},{z}) stays valid"
        );
    }

    // The point of step 3: light still reaches below the cut. A Tier 1 answer must skip
    // the column scan, never the update itself.
    //
    // Not to the very bottom, though, and that is correct physics rather than a defect:
    // sky light drops by one per block travelled, a step costs two blocks of path, so a
    // staircase goes dark after roughly seven steps no matter what the cache says. The
    // plan's "no unlit blocks below Y=60" only ever held for a vertical shaft.
    let top = sky(&engine, &level, steps[0]);
    assert!(
        top >= 14,
        "the staircase mouth should be near full sky, got {top}"
    );
    let deepest_lit = steps
        .iter()
        .filter(|step| sky(&engine, &level, **step) > 0)
        .last()
        .expect("some part of the staircase must be lit");
    assert!(
        deepest_lit.0.y < cut,
        "light stopped at y={}, at or above the cut {cut} — Tier 1 swallowed the update \
         instead of only skipping the column scan",
        deepest_lit.0.y
    );
    assert_eq!(
        sky(&engine, &level, bottom),
        0,
        "the far bottom is out of reach of sky light and must be dark"
    );

    let mut previous = u8::MAX;
    for step in &steps {
        let value = sky(&engine, &level, *step);
        assert!(
            value <= previous,
            "sky light must not increase going down: {value} after {previous} at {step:?}"
        );
        previous = value;
    }
}

/// An open trench does move a ceiling out of the band, and must flag its quadrant.
///
/// This is what the plan's step 4 was reaching for: not the depth of the dig, but whether
/// the column's ceiling leaves `[cut, cut + spread]`. Removing the surface itself does
/// that; tunnelling underneath it does not.
#[tokio::test(flavor = "multi_thread")]
async fn an_open_trench_flags_only_its_own_quadrant() {
    let level = make_level();
    let chunk = insert_flat_chunk(&level);
    let engine = DynamicLightEngine::new();

    let cut = cut_of(&chunk).decode(chunk.section.min_y, SkyLightHeight::chunk_height(&chunk));

    // Clear one full column in the NW quadrant, top down, the way a trench is dug.
    for y in (MIN_Y..=SURFACE_Y).rev() {
        set_block(
            &level,
            &engine,
            BlockPos::new(2, y, 2),
            Block::AIR.default_state.id,
        );
    }
    settle(&engine, &level);

    let after = cut_of(&chunk);
    assert!(
        !after.quadrant_uses_limit(2, 2),
        "the trench dropped this column's ceiling below the cut {cut}, so its quadrant \
         must lose the fast path"
    );
    for (x, z) in [(12, 2), (2, 12), (12, 12)] {
        assert!(
            after.quadrant_uses_limit(x, z),
            "quadrant ({x},{z}) is untouched and must keep the fast path"
        );
    }

    // A shaft open to the sky carries full daylight all the way down.
    assert_eq!(
        sky(&engine, &level, BlockPos::new(2, 0, 2)),
        15,
        "an open shaft should be fully sunlit to the bottom"
    );
}

/// Blocking the staircase below the original cut must darken everything under the block.
///
/// Steps 5-6 of the plan: the decrease pass may not be short-circuited by a stale cut.
#[tokio::test(flavor = "multi_thread")]
async fn blocking_the_staircase_darkens_everything_below_the_block() {
    for block_above_cut in [false, true] {
        let level = make_level();
        let chunk = insert_flat_chunk(&level);
        let engine = DynamicLightEngine::new();

        let cut = cut_of(&chunk).decode(chunk.section.min_y, SkyLightHeight::chunk_height(&chunk));
        let steps = staircase(14);
        dig_staircase(&level, &engine, &steps);

        // Pick a step above or below the original cut, per the plan's two cases.
        let plug_index = steps
            .iter()
            .position(|s| {
                if block_above_cut {
                    s.0.y < cut + 4
                } else {
                    s.0.y < cut
                }
            })
            .expect("staircase crosses the cut");
        let plug = steps[plug_index];
        assert!(
            sky(&engine, &level, plug) > 0,
            "the step to be plugged must be lit beforehand"
        );

        // Fill both halves of the two-block step so no light can pass.
        set_block(&level, &engine, plug, Block::STONE.default_state.id);
        set_block(
            &level,
            &engine,
            BlockPos::new(plug.0.x, plug.0.y + 1, plug.0.z),
            Block::STONE.default_state.id,
        );
        settle(&engine, &level);

        for step in &steps[(plug_index + 1)..] {
            assert_eq!(
                sky(&engine, &level, *step),
                0,
                "step {step:?} below the plug (cut={cut}, above_cut={block_above_cut}) stayed lit"
            );
        }
    }
}

/// Roofing over the chunk must not leave the interior "open sky".
///
/// This is the case where a stale cut is genuinely destructive. A wrong Tier 1 is largely
/// self-correcting — sky light still flows in sideways from neighbours, so a too-dark
/// answer gets filled in by propagation. A wrong Tier 2 is not: it asserts full sunlight
/// outright, so an interior under a fresh roof would stay lit at 15 forever.
#[tokio::test(flavor = "multi_thread")]
async fn a_new_roof_must_not_be_treated_as_open_sky() {
    let level = make_level();
    let chunk = insert_flat_chunk(&level);
    let engine = DynamicLightEngine::new();

    let min_y = chunk.section.min_y;
    let height = SkyLightHeight::chunk_height(&chunk);
    let roof_y = 100;
    let probe = BlockPos::new(4, 70, 4);

    let before = cut_of(&chunk);
    assert_eq!(
        before.tier(probe.0.y, probe.0.x, probe.0.z, min_y, height),
        SkyLightTier::OpenSky,
        "before the roof this position really does see the sky"
    );
    assert_eq!(sky(&engine, &level, probe), 15);

    for z in 0..16 {
        for x in 0..16 {
            set_block(
                &level,
                &engine,
                BlockPos::new(x, roof_y, z),
                Block::STONE.default_state.id,
            );
        }
    }
    settle(&engine, &level);

    let after = cut_of(&chunk);
    assert!(
        !after.quadrant_uses_limit(probe.0.x, probe.0.z),
        "the roofed quadrant must be flagged as diverged"
    );
    assert_eq!(
        sky(&engine, &level, probe),
        0,
        "the room under the new roof must be dark"
    );
}

/// The plan's step 7: the same scenario must fail without the divergence flag.
///
/// Proves the flag is load-bearing rather than the test being accidentally green. The
/// stale value is produced from an untouched chunk, so it is exactly what the cache would
/// hold if invalidation had never run. `check_sky_light_updates` is called directly to
/// bypass the invalidation hook that `update_lighting_at` performs first.
#[tokio::test(flavor = "multi_thread")]
async fn without_the_divergence_flag_the_roofed_room_is_wrongly_sunlit() {
    let level = make_level();
    let chunk = insert_flat_chunk(&level);
    let engine = DynamicLightEngine::new();
    let probe = BlockPos::new(4, 70, 4);

    // What the cache holds for pristine flat terrain — no quadrant diverged.
    let pristine = cut_of(&insert_flat_chunk(&make_level()));
    assert!(pristine.quadrant_uses_limit(probe.0.x, probe.0.z));

    for z in 0..16 {
        for x in 0..16 {
            set_block(
                &level,
                &engine,
                BlockPos::new(x, 100, z),
                Block::STONE.default_state.id,
            );
        }
    }
    settle(&engine, &level);
    assert_eq!(sky(&engine, &level, probe), 0, "roofed room starts dark");

    // Force the stale, non-diverged value back in and re-run only the sky pass.
    chunk
        .sky_light_height_cache
        .store(pristine.raw(), Ordering::Relaxed);
    SkyLightHeightMigration::persist(&chunk, pristine);

    engine.check_sky_light_updates(&level, probe);
    settle(&engine, &level);

    assert_eq!(
        sky(&engine, &level, probe),
        15,
        "with a stale cut the engine must wrongly report full sunlight — if this ever \
         reads 0, the divergence flag is no longer what makes the roof test pass and \
         that test has stopped covering the gap it was written for"
    );
}

/// Phase 5 profiling: how sky light work splits across the three tiers on real terrain.
///
/// The tier counters exist for exactly this question. Tier 3 is the only one that pays for
/// a column scan, so its share decides whether the cut height cache earns its keep. Run on
/// flat ground with an intact surface, sampling from bedrock to well above the surface.
///
/// Deliberately not run on the all-air chunk the throughput bench uses: there every column
/// ceiling sits at the world bottom, which pushes the whole sample into Tier 2 or the band
/// around it and would report a tier split that no real world produces.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "manual perf sanity check, not part of the normal test suite"]
async fn tier_distribution_on_flat_terrain() {
    let level = make_level();
    let chunk = insert_flat_chunk(&level);
    let engine = DynamicLightEngine::new();

    let cut = cut_of(&chunk).decode(chunk.section.min_y, SkyLightHeight::chunk_height(&chunk));
    let spread = cut_of(&chunk).spread();
    println!("cut={cut} spread={spread} band=[{cut}, {}]", cut + spread);

    let mut sampled = 0;
    for y in (MIN_Y..=120).step_by(4) {
        for z in 0..16 {
            for x in 0..16 {
                engine.check_sky_light_updates(&level, BlockPos::new(x, y, z));
                sampled += 1;
            }
        }
    }
    println!("sampled {sampled} positions");
    for pass in 1..=50 {
        let stats = engine.drain_queued(&level);
        println!("pass {pass}: {stats}");
        if !stats.leftover {
            break;
        }
    }
}
