use std::{
    cmp::Reverse, collections::HashMap, marker::PhantomData, rc::Rc, sync::Mutex, time::Instant,
};

use serde::Serialize;

use crate::collector::COLLECTOR;

const PHASE_COUNT: usize = 16;
const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;
const MAX_DETAILS: usize = 4096;

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

struct State {
    generation: u64,
    timings: [Timing; PHASE_COUNT],
    details: Option<HashMap<Detail, Timing>>,
    dropped_keys: u64,
}

static STATE: Mutex<State> = Mutex::new(State {
    generation: 0,
    timings: [Timing::EMPTY; PHASE_COUNT],
    details: None,
    dropped_keys: 0,
});

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
        crate::without_tracking(|| {
            let mut state = STATE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.generation != self.generation {
                return;
            }
            if let Some(phase) = self.phase {
                state.timings[phase as usize].record(wall, cpu, self.budget);
            }
            for detail in self.details.iter_mut().filter_map(Option::take) {
                state.record_detail(detail, wall, cpu);
            }
        });
    }
}

pub fn start(generation: u64) {
    *STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = State {
        generation,
        timings: [Timing::EMPTY; PHASE_COUNT],
        details: Some(HashMap::new()),
        dropped_keys: 0,
    };
}

pub fn stop() {
    let mut state = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    state.generation = 0;
    state.details = None;
}

pub fn snapshot() -> Vec<(&'static str, Timing)> {
    let timings = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .timings;
    NAMES.into_iter().zip(timings).collect()
}

impl State {
    fn record_detail(&mut self, detail: Detail, wall: u64, cpu: Option<u64>) {
        let Some(details) = self.details.as_mut() else {
            return;
        };
        if details.len() >= MAX_DETAILS && !details.contains_key(&detail) {
            self.dropped_keys += 1;
            return;
        }
        details
            .entry(detail)
            .or_insert(Timing::EMPTY)
            .record(wall, cpu, 0);
    }
}

pub fn detail_snapshot() -> Details {
    let state = STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut entries: Vec<_> = state
        .details
        .iter()
        .flat_map(|details| details.iter())
        .map(|(detail, &timing)| DetailTiming {
            detail: detail.clone(),
            timing,
        })
        .collect();
    entries.sort_unstable_by_key(|entry| {
        Reverse(entry.timing.cpu_nanos.unwrap_or(entry.timing.wall_nanos))
    });
    Details {
        entries,
        dropped_keys: state.dropped_keys,
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
        crate::without_tracking(|| {
            let mut state = STATE
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if state.generation != self.generation {
                return;
            }
            if let Some(phase) = self.phase {
                state.timings[phase as usize].record(wall, None, 0);
            }
            if let Some(detail) = self.detail.take() {
                state.record_detail(detail, wall, None);
            }
        });
    }
}

fn thread_cpu_nanos() -> Option<u64> {
    clock_nanos(libc::CLOCK_THREAD_CPUTIME_ID)
}

#[must_use]
pub fn process_cpu_nanos() -> Option<u64> {
    clock_nanos(libc::CLOCK_PROCESS_CPUTIME_ID)
}

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
