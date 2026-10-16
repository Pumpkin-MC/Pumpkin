use std::{
    cell::RefCell,
    cmp::Reverse,
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
    sync::{
        Arc, Mutex, MutexGuard, PoisonError,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use serde::Serialize;

use crate::collector::COLLECTOR;

const PHASE_COUNT: usize = 16;
#[cfg(unix)]
const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;
// Chunks get their own budget so they cannot crowd out world, type and plugin keys.
const MAX_DETAILS: usize = 4096;
const MAX_CHUNK_DETAILS: usize = 4096;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Tick,
    World,
    Functions,
    Players,
    Entities,
    BlockEntities,
    Blocks,
    Fluids,
    RandomTicks,
    Spawning,
    WorldSetup,
    EntityTracking,
    ChunkGeneration,
    ChunkLoading,
    ChunkSaving,
    Lighting,
}

const NAMES: [&str; PHASE_COUNT] = [
    "tick",
    "world",
    "functions/plugins",
    "players",
    "entities",
    "block entities",
    "scheduled blocks",
    "scheduled fluids",
    "random ticks",
    "spawning",
    "world setup/environment",
    "entity tracking",
    "chunk generation",
    "chunk loading",
    "chunk saving",
    "lighting",
];

#[derive(Clone, Copy, Serialize)]
pub struct Timing {
    pub calls: u64,
    pub wall_nanos: u64,
    pub cpu_nanos: Option<u64>,
    pub max_wall_nanos: u64,
    pub over_budget: u64,
}

impl Timing {
    const EMPTY: Self = Self {
        calls: 0,
        wall_nanos: 0,
        cpu_nanos: Some(0),
        max_wall_nanos: 0,
        over_budget: 0,
    };

    fn record(&mut self, wall: u64, cpu: Option<u64>, budget: u64) {
        self.calls += 1;
        self.wall_nanos += wall;
        self.cpu_nanos = self
            .cpu_nanos
            .zip(cpu)
            .map(|(total, elapsed)| total + elapsed);
        self.max_wall_nanos = self.max_wall_nanos.max(wall);
        self.over_budget += u64::from(budget != 0 && wall > budget);
    }

    fn merge(&mut self, other: &Self) {
        self.calls += other.calls;
        self.wall_nanos += other.wall_nanos;
        self.cpu_nanos = self
            .cpu_nanos
            .zip(other.cpu_nanos)
            .map(|(total, elapsed)| total + elapsed);
        self.max_wall_nanos = self.max_wall_nanos.max(other.max_wall_nanos);
        self.over_budget += other.over_budget;
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Detail {
    World {
        id: uuid::Uuid,
        dimension: &'static str,
    },
    Entity {
        name: &'static str,
    },
    BlockEntity {
        name: &'static str,
    },
    Block {
        name: &'static str,
        phase: Phase,
    },
    Fluid {
        name: &'static str,
        phase: Phase,
    },
    Chunk {
        world: uuid::Uuid,
        x: i32,
        z: i32,
    },
    Plugin {
        name: String,
        event: &'static str,
    },
}

impl Detail {
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::World { dimension, id } => format!("world {dimension} ({id})"),
            Self::Entity { name } => format!("entity {name}"),
            Self::BlockEntity { name } => format!("block entity {name}"),
            Self::Block { name, phase } => format!("block {name} / {}", NAMES[*phase as usize]),
            Self::Fluid { name, phase } => format!("fluid {name} / {}", NAMES[*phase as usize]),
            Self::Chunk { world, x, z } => format!("chunk [{x}, {z}] in {world}"),
            Self::Plugin { name, event } => format!("plugin {name} / {event}"),
        }
    }
}

#[derive(Serialize)]
pub struct DetailTiming {
    pub detail: Detail,
    #[serde(flatten)]
    pub timing: Timing,
}

#[derive(Serialize)]
pub struct Details {
    pub entries: Vec<DetailTiming>,
    pub dropped_keys: u64,
}

#[derive(Default)]
struct DetailTable {
    entries: HashMap<Detail, Timing>,
    chunk_keys: usize,
    dropped_keys: u64,
}

impl DetailTable {
    fn entry(&mut self, detail: Detail) -> Option<&mut Timing> {
        if !self.entries.contains_key(&detail) {
            let chunk = matches!(detail, Detail::Chunk { .. });
            let (used, limit) = if chunk {
                (self.chunk_keys, MAX_CHUNK_DETAILS)
            } else {
                (self.entries.len() - self.chunk_keys, MAX_DETAILS)
            };
            if used >= limit {
                self.dropped_keys += 1;
                return None;
            }
            self.chunk_keys += usize::from(chunk);
        }
        Some(self.entries.entry(detail).or_insert(Timing::EMPTY))
    }
}

struct Buffer {
    generation: u64,
    timings: [Timing; PHASE_COUNT],
    details: DetailTable,
}

impl Buffer {
    fn new(generation: u64) -> Self {
        Self {
            generation,
            timings: [Timing::EMPTY; PHASE_COUNT],
            details: DetailTable::default(),
        }
    }
}

static GENERATION: AtomicU64 = AtomicU64::new(0);
// Spans record into a buffer owned by their thread, so parallel workers never wait on each
// other. The list is only locked to register a thread's buffer and to merge them on snapshot.
static BUFFERS: Mutex<Vec<Arc<Mutex<Buffer>>>> = Mutex::new(Vec::new());

thread_local! {
    static LOCAL: RefCell<Option<Arc<Mutex<Buffer>>>> = const { RefCell::new(None) };
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn record(generation: u64, f: impl FnOnce(&mut Buffer)) {
    crate::without_tracking(|| {
        let _ = LOCAL.try_with(|local| {
            let Ok(mut local) = local.try_borrow_mut() else {
                return;
            };
            if let Some(buffer) = local.as_ref() {
                let mut buffer = lock(buffer);
                if buffer.generation == generation {
                    f(&mut buffer);
                    return;
                }
            }
            let mut buffers = lock(&BUFFERS);
            if GENERATION.load(Ordering::Acquire) != generation {
                return;
            }
            let mut buffer = Buffer::new(generation);
            f(&mut buffer);
            let buffer = Arc::new(Mutex::new(buffer));
            buffers.push(Arc::clone(&buffer));
            *local = Some(buffer);
        });
    });
}

fn buffers() -> Vec<Arc<Mutex<Buffer>>> {
    lock(&BUFFERS).clone()
}

pub struct Span {
    phase: Option<Phase>,
    details: [Option<Detail>; 2],
    generation: u64,
    wall: Instant,
    cpu: Option<u64>,
    budget: u64,
    thread: PhantomData<Rc<()>>,
}

pub fn span(phase: Phase) -> Option<Span> {
    let generation = COLLECTOR.generation();
    (generation != 0).then(|| Span {
        phase: Some(phase),
        details: [None, None],
        generation,
        wall: Instant::now(),
        cpu: thread_cpu_nanos(),
        budget: 0,
        thread: PhantomData,
    })
}

#[must_use]
pub fn detail(primary: Detail, secondary: Option<Detail>) -> Option<Span> {
    span(Phase::World).map(|mut span| {
        span.phase = None;
        span.details = [Some(primary), secondary];
        span
    })
}

#[must_use]
pub fn world(id: uuid::Uuid, dimension: &'static str) -> Option<Span> {
    span(Phase::World).map(|mut span| {
        span.details[0] = Some(Detail::World { id, dimension });
        span
    })
}

#[must_use]
pub fn tick(budget_nanos: u64) -> Option<Span> {
    span(Phase::Tick).map(|mut span| {
        span.budget = budget_nanos;
        span
    })
}

impl Drop for Span {
    fn drop(&mut self) {
        let cpu = self
            .cpu
            .zip(thread_cpu_nanos())
            .and_then(|(before, after)| after.checked_sub(before));
        let wall = self.wall.elapsed().as_nanos() as u64;
        let (phase, budget) = (self.phase, self.budget);
        let details = self.details.each_mut().map(Option::take);
        record(self.generation, |buffer| {
            if let Some(phase) = phase {
                buffer.timings[phase as usize].record(wall, cpu, budget);
            }
            for detail in details.into_iter().flatten() {
                if let Some(timing) = buffer.details.entry(detail) {
                    timing.record(wall, cpu, 0);
                }
            }
        });
    }
}

pub fn start(generation: u64) {
    let mut buffers = lock(&BUFFERS);
    buffers.clear();
    GENERATION.store(generation, Ordering::Release);
}

pub fn stop() {
    let mut buffers = lock(&BUFFERS);
    GENERATION.store(0, Ordering::Release);
    // Threads keep their buffer until their next span, so release the contents now.
    for buffer in buffers.drain(..) {
        *lock(&buffer) = Buffer::new(0);
    }
}

#[must_use]
pub fn snapshot() -> Vec<(&'static str, Timing)> {
    let mut timings = [Timing::EMPTY; PHASE_COUNT];
    for buffer in buffers() {
        let buffer = lock(&buffer);
        for (total, timing) in timings.iter_mut().zip(&buffer.timings) {
            total.merge(timing);
        }
    }
    NAMES.into_iter().zip(timings).collect()
}

#[must_use]
pub fn detail_snapshot() -> Details {
    let mut merged = DetailTable::default();
    for buffer in buffers() {
        let buffer = lock(&buffer);
        merged.dropped_keys += buffer.details.dropped_keys;
        for (detail, timing) in &buffer.details.entries {
            if let Some(total) = merged.entry(detail.clone()) {
                total.merge(timing);
            }
        }
    }
    let mut entries: Vec<_> = merged
        .entries
        .into_iter()
        .map(|(detail, timing)| DetailTiming { detail, timing })
        .collect();
    entries.sort_unstable_by_key(|entry| {
        Reverse(entry.timing.cpu_nanos.unwrap_or(entry.timing.wall_nanos))
    });
    Details {
        entries,
        dropped_keys: merged.dropped_keys,
    }
}

pub struct AsyncSpan {
    phase: Option<Phase>,
    generation: u64,
    detail: Option<Detail>,
    wall: Instant,
}

pub fn plugin_event(name: &str, event: &'static str) -> Option<AsyncSpan> {
    let generation = COLLECTOR.generation();
    (generation != 0).then(|| {
        crate::without_tracking(|| AsyncSpan {
            phase: None,
            generation,
            detail: Some(Detail::Plugin {
                name: name.to_owned(),
                event,
            }),
            wall: Instant::now(),
        })
    })
}

pub fn async_span(phase: Phase) -> Option<AsyncSpan> {
    let generation = COLLECTOR.generation();
    (generation != 0).then(|| AsyncSpan {
        phase: Some(phase),
        generation,
        detail: None,
        wall: Instant::now(),
    })
}

impl Drop for AsyncSpan {
    fn drop(&mut self) {
        let wall = self.wall.elapsed().as_nanos() as u64;
        let phase = self.phase;
        let detail = self.detail.take();
        record(self.generation, |buffer| {
            if let Some(phase) = phase {
                buffer.timings[phase as usize].record(wall, None, 0);
            }
            if let Some(timing) = detail.and_then(|detail| buffer.details.entry(detail)) {
                timing.record(wall, None, 0);
            }
        });
    }
}

#[cfg(unix)]
fn thread_cpu_nanos() -> Option<u64> {
    clock_nanos(libc::CLOCK_THREAD_CPUTIME_ID)
}

#[cfg(unix)]
#[must_use]
pub fn process_cpu_nanos() -> Option<u64> {
    clock_nanos(libc::CLOCK_PROCESS_CPUTIME_ID)
}

#[cfg(unix)]
fn clock_nanos(clock: libc::clockid_t) -> Option<u64> {
    let mut value = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: value is a writable timespec and clock is a supported CPU clock.
    if unsafe { libc::clock_gettime(clock, &raw mut value) } != 0 {
        return None;
    }
    Some(
        u64::try_from(value.tv_sec).ok()? * NANOSECONDS_PER_SECOND
            + u64::try_from(value.tv_nsec).ok()?,
    )
}

// GetThreadTimes only advances on scheduler clock ticks (about 15.6 ms), which is
// far coarser than a single span, so Windows spans record wall time only.
#[cfg(windows)]
const fn thread_cpu_nanos() -> Option<u64> {
    None
}

#[cfg(windows)]
#[must_use]
pub fn process_cpu_nanos() -> Option<u64> {
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::Threading::{GetCurrentProcess, GetProcessTimes},
    };
    let mut created = FILETIME::default();
    let mut exited = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    // SAFETY: The pseudo-handle denotes the current process and is never closed.
    // GetProcessTimes accepts it and four writable FILETIME buffers.
    if unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &raw mut created,
            &raw mut exited,
            &raw mut kernel,
            &raw mut user,
        )
    } == 0
    {
        return None;
    }
    let ticks =
        |time: FILETIME| (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
    ticks(kernel).checked_add(ticks(user))?.checked_mul(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_from_exited_threads_are_merged_until_stop() {
        COLLECTOR.start(7).unwrap();
        start(7);
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    for _ in 0..10 {
                        drop(span(Phase::Entities));
                        drop(detail(Detail::Entity { name: "zombie" }, None));
                    }
                });
            }
        });
        assert_eq!(snapshot()[Phase::Entities as usize].1.calls, 40);
        let details = detail_snapshot();
        assert_eq!(details.entries.len(), 1);
        assert_eq!(details.entries[0].timing.calls, 40);

        stop();
        COLLECTOR.stop();
        assert_eq!(snapshot()[Phase::Entities as usize].1.calls, 0);
        assert!(detail_snapshot().entries.is_empty());
    }
}
