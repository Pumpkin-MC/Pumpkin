#![allow(clippy::unwrap_used, clippy::panic, clippy::print_stdout)]
//! Reports how much heap chunk generation keeps and needs at its peak
//!
//! - the numbers only depend on the seed, so they can be compared between commits
//! - measures bytes, not time
//!
//! Run with `cargo bench -p pumpkin-world --bench chunk_memory`.
//!
//! Baseline on master (1859221e7), bench profile:

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};

use pumpkin_data::BlockStateId;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::chunk_system::chunk_loading::ChunkLoading;
use pumpkin_world::chunk_system::generation::generate_single_chunk_with_radius;
use pumpkin_world::chunk_system::{Chunk, StagedChunkEnum};
use pumpkin_world::generation::get_world_gen;
use pumpkin_world::generation::structure::template::StructureTemplate;
use pumpkin_world::world::WorldPortalExt;

const SEED: Seed = Seed(42);
/// Fixed instead of derived from the machine so the numbers stay comparable.
const THREADS: usize = 16;
/// Every chunk a player at this view distance keeps loaded, plus the proto chunks the
/// scheduler keeps around them for the stages that read their neighbours.
const VIEW_DISTANCE: i32 = 16;

static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

// SAFETY: every method forwards its arguments to `System` unchanged; the counters are
// bookkeeping only.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        PEAK.fetch_max(
            LIVE.fetch_add(layout.size(), Relaxed) + layout.size(),
            Relaxed,
        );
        // SAFETY: same contract as the caller's.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Relaxed);
        // SAFETY: same contract as the caller's.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn live_kib() -> usize {
    LIVE.load(Relaxed) / 1024
}

/// Heap owned by `value` in KiB, measured by dropping it.
fn size_kib<T>(value: T) -> usize {
    let before = live_kib();
    drop(value);
    before - live_kib()
}

// Stub portal — allows all block placements, skips mob spawning.
struct BlockRegistry;
impl WorldPortalExt for BlockRegistry {
    fn can_place_at(
        &self,
        _block: &pumpkin_data::Block,
        _state: &pumpkin_data::BlockState,
        _block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        _block_pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        true
    }

    fn mirror(
        &self,
        block: &pumpkin_data::Block,
        state_id: BlockStateId,
        mirror: pumpkin_data::Mirror,
    ) -> &'static pumpkin_data::BlockState {
        block.mirror(state_id, mirror)
    }

    fn rotate(
        &self,
        block: &pumpkin_data::Block,
        state_id: BlockStateId,
        rotation: pumpkin_data::Rotation,
    ) -> &'static pumpkin_data::BlockState {
        block.rotate(state_id, rotation)
    }

    fn spawn_mobs_for_chunk_generation(
        &self,
        _cache: &mut dyn pumpkin_world::generation::proto_chunk::GenerationCache,
        _biome: &'static pumpkin_data::chunk::Biome,
        _chunk_x: i32,
        _chunk_z: i32,
    ) {
    }
}

fn main() {
    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD, false, Vec::new(), String::new());
    let generate = |x: i32, z: i32| {
        let ring = (x.abs().max(z.abs()) - VIEW_DISTANCE).max(0) as i8;
        let stage = StagedChunkEnum::level_to_stage(ChunkLoading::FULL_CHUNK_LEVEL + ring);
        generate_single_chunk_with_radius(&world_gen, &BlockRegistry, x, z, stage, 1)
    };
    let radius = VIEW_DISTANCE + StagedChunkEnum::FULL_RADIUS;
    let positions: Vec<(i32, i32)> = (-radius..=radius)
        .flat_map(|x| (-radius..=radius).map(move |z| (x, z)))
        .collect();

    // Generated on a fixed number of threads like the scheduler does.
    let before = live_kib();
    PEAK.store(LIVE.load(Relaxed), Relaxed);
    let chunks: Vec<Chunk> = std::thread::scope(|scope| {
        #[expect(
            clippy::needless_collect,
            reason = "every thread has to be spawned before the first one is joined"
        )]
        let handles: Vec<_> = positions
            .chunks(positions.len().div_ceil(THREADS))
            .map(|slice| {
                scope.spawn(|| {
                    slice
                        .iter()
                        .map(|&(x, z)| generate(x, z))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        // `join` returns once the thread's thread-local destructors have run, so the
        // per-thread buffer pools are gone afterwards.
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    });
    let peak = PEAK.load(Relaxed) / 1024 - before;
    let (mut full_light, mut light) = (0, 0);
    let (loaded, proto): (Vec<Chunk>, Vec<Chunk>) = chunks
        .into_iter()
        .partition(|chunk| matches!(chunk, Chunk::Level(_)));
    for chunk in &loaded {
        let Chunk::Level(data) = chunk else { continue };
        let engine = data.light_engine.lock().unwrap();
        let sections = engine.sky_light.iter().chain(engine.block_light.iter());
        light += engine.sky_light.len() + engine.block_light.len();
        full_light += sections.filter(|container| !container.is_empty()).count();
    }
    let (loaded_count, proto_count) = (loaded.len(), proto.len());
    let (loaded_total, proto_total) = (size_kib(loaded), size_kib(proto));
    println!(
        "{} chunks generated on {THREADS} threads: view distance {VIEW_DISTANCE} plus a ring of {}",
        loaded_count + proto_count,
        StagedChunkEnum::FULL_RADIUS
    );
    println!("  peak heap during generation: {peak} KiB");
    println!(
        "  {loaded_count} loaded chunks: {} KiB on average, {loaded_total} KiB in total",
        loaded_total / loaded_count
    );
    println!("    light sections stored as full arrays: {full_light} of {light}");
    println!(
        "  {proto_count} proto chunks in the ring: {} KiB on average, {proto_total} KiB in total",
        proto_total / proto_count
    );
    println!(
        "  left after dropping them (world generator, caches, feature tables): {} KiB",
        live_kib() - before
    );

    let before = live_kib();
    let mut blocks = 0;
    let templates: Vec<StructureTemplate> = pumpkin_data::template_bytes::all_template_names()
        .iter()
        .filter_map(|name| pumpkin_data::template_bytes::get_template_bytes(name))
        .filter_map(|bytes| StructureTemplate::from_nbt_bytes(bytes).ok())
        .inspect(|template| {
            blocks += template
                .palettes
                .iter()
                .map(|p| p.blocks().len())
                .sum::<usize>();
        })
        .collect();
    let total = live_kib() - before;
    println!(
        "{} structure templates parsed: {total} KiB, {} bytes per block",
        templates.len(),
        total * 1024 / blocks.max(1)
    );
}
