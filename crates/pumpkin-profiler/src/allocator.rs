use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

use crate::collector::{COLLECTOR, MAX_FRAMES, Sample};

pub const SAMPLE_BYTES: usize = 512 * 1024;

static REGISTERED: AtomicBool = AtomicBool::new(false);
static SEED: AtomicU64 = AtomicU64::new(1);

thread_local! {
    static IN_HOOK: Cell<bool> = const { Cell::new(false) };
    static SUPPRESSED: Cell<bool> = const { Cell::new(false) };
    static RANDOM: Cell<u64> = const { Cell::new(0) };
}

struct HookGuard;

impl HookGuard {
    fn enter() -> Option<Self> {
        IN_HOOK
            .try_with(|busy| (!busy.replace(true)).then(|| Self))
            .ok()
            .flatten()
    }
}

impl Drop for HookGuard {
    fn drop(&mut self) {
        let _ = IN_HOOK.try_with(|busy| busy.set(false));
    }
}

/// Excludes allocations on the calling thread from heap profiles.
/// Frees of previously sampled allocations remain visible. Other threads keep collecting.
pub fn without_tracking<T>(f: impl FnOnce() -> T) -> T {
    let _guard = Suppress::enter();
    f()
}

pub struct Suppress(bool);

impl Suppress {
    pub fn enter() -> Self {
        Self(
            SUPPRESSED
                .try_with(|value| value.replace(true))
                .unwrap_or(true),
        )
    }
}

impl Drop for Suppress {
    fn drop(&mut self) {
        let _ = SUPPRESSED.try_with(|value| value.set(self.0));
    }
}

pub const fn sampling_weight(size: usize) -> usize {
    if size >= SAMPLE_BYTES {
        1
    } else {
        SAMPLE_BYTES / size.next_power_of_two()
    }
}

fn selected(size: usize) -> bool {
    let weight = sampling_weight(size);
    if weight == 1 {
        return true;
    }
    RANDOM
        .try_with(|random| {
            let mut value = random.get();
            if value == 0 {
                value = SEED.fetch_add(1, Ordering::Relaxed);
            }
            value ^= value << 13;
            value ^= value >> 7;
            value ^= value << 17;
            random.set(value);
            value as usize & (weight - 1) == 0
        })
        .unwrap_or(false)
}

fn record(address: usize, size: usize, generation: u64) {
    if address == 0 || generation == 0 || SUPPRESSED.try_with(Cell::get).unwrap_or(true) {
        return;
    }
    let Some(_guard) = HookGuard::enter() else {
        return;
    };
    if !selected(size) {
        return;
    }
    let mut frames = [std::ptr::null_mut(); MAX_FRAMES];
    // SAFETY: The writable buffer holds MAX_FRAMES pointers. HookGuard prevents
    // recursive collector access if the unwinder allocates or frees Rust memory.
    let frame_count =
        unsafe { libc::backtrace(frames.as_mut_ptr(), MAX_FRAMES as i32) }.max(0) as usize;
    COLLECTOR.insert(
        &Sample {
            address,
            size,
            frames: frames.map(|frame| frame as usize),
            frame_count,
        },
        generation,
    );
}

fn remove(address: usize, generation: u64) -> Option<Sample> {
    if generation == 0 {
        return None;
    }
    let _guard = HookGuard::enter()?;
    COLLECTOR.remove(address, generation)
}

/// Forwards allocations to `System` and samples their lifetimes while profiling is active.
pub struct TrackingAllocator;

// SAFETY: Each operation forwards the caller's pointer and layout to System.
// Collector inserts use pre-reserved capacity, and recursive bookkeeping is suppressed.
unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let generation = COLLECTOR.generation();
        // SAFETY: The caller supplies a valid non-zero allocation layout.
        let pointer = unsafe { System.alloc(layout) };
        record(pointer as usize, layout.size(), generation);
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let generation = COLLECTOR.generation();
        // SAFETY: The caller supplies a valid non-zero allocation layout.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        record(pointer as usize, layout.size(), generation);
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // Remove before freeing, so address reuse cannot erase a newer allocation.
        remove(pointer as usize, COLLECTOR.generation());
        // SAFETY: The caller supplies a live allocation and its matching layout.
        unsafe { System.dealloc(pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let generation = COLLECTOR.generation();
        let previous = remove(pointer as usize, generation);
        // SAFETY: The caller supplies a live allocation, matching layout and valid new size.
        let result = unsafe { System.realloc(pointer, layout, new_size) };
        if result.is_null() {
            if let Some(sample) = previous {
                let _guard = HookGuard::enter();
                COLLECTOR.insert(&sample, generation);
            }
        } else {
            record(result as usize, new_size, generation);
        }
        result
    }
}

/// Marks the binary's global allocator as wired to this collector.
pub fn register_allocator() {
    REGISTERED.store(true, Ordering::Release);
}

pub fn start(generation: u64) -> std::io::Result<()> {
    if !REGISTERED.load(Ordering::Acquire) {
        return Err(std::io::Error::other(
            "the binary has not installed the memory profiler allocator wrapper",
        ));
    }
    COLLECTOR.start(generation)
}
