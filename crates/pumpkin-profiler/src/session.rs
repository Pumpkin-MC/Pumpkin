use std::{
    fs::{self, Permissions},
    io,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    path::{Path, PathBuf},
    time::Instant,
};

use serde_json::{Value, json};
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    allocator::{self, Suppress},
    capture,
    collector::{COLLECTOR, MAX_FRAMES, MAX_SAMPLES},
    cpu, sampling,
};

pub static PROFILER: Profiler = Profiler {
    state: Mutex::const_new(State {
        closed: false,
        session: None,
        next_generation: 1,
    }),
};

#[derive(Clone, Copy)]
pub enum Operation {
    Start,
    Snapshot,
    Stop,
    Status,
    Cancel,
}

pub struct Profiler {
    state: Mutex<State>,
}

pub struct Request<'a> {
    state: MutexGuard<'a, State>,
}

struct State {
    closed: bool,
    session: Option<Session>,
    next_generation: u64,
}

struct Session {
    directory: PathBuf,
    started: Instant,
    snapshots: usize,
    previous: Option<capture::Snapshot>,
    sampler: rpprof::ProfilerGuard<'static>,
}

impl Profiler {
    /// Rejects overlapping requests instead of queuing blocking workers.
    pub fn try_reserve(&self) -> io::Result<Request<'_>> {
        let state = self.state.try_lock().map_err(|_| {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "another profiler operation is in progress",
            )
        })?;
        if state.closed {
            return Err(io::Error::other("profiler is shutting down"));
        }
        Ok(Request { state })
    }

    /// Executes a profiler operation on a blocking worker.
    pub fn execute(
        &self,
        operation: Operation,
        root: &Path,
        build: &str,
        counters: &Value,
    ) -> io::Result<String> {
        self.try_reserve()?
            .execute(operation, root, build, counters)
    }

    /// Saves an active session and disables collection even when output fails.
    /// Call from a blocking worker after command tasks have finished.
    pub fn shutdown(&self, counters: &Value) -> io::Result<()> {
        let _guard = Suppress::enter();
        let mut state = self.state.blocking_lock();
        state.closed = true;
        let result = state.session.as_mut().map_or(Ok(()), |session| {
            session.capture("shutdown", counters).map(|_| ())
        });
        COLLECTOR.stop();
        cpu::stop();
        state.session = None;
        result
    }
}

impl Request<'_> {
    pub fn execute(
        mut self,
        operation: Operation,
        root: &Path,
        build: &str,
        counters: &Value,
    ) -> io::Result<String> {
        let _guard = Suppress::enter();
        let state = &mut self.state;
        if matches!(operation, Operation::Start) {
            return state.start(root, build, counters);
        }
        let Some(session) = state.session.as_mut() else {
            return if matches!(operation, Operation::Status) {
                Ok("Profiling is inactive".to_owned())
            } else {
                Err(io::Error::other("profiling is not running"))
            };
        };
        match operation {
            Operation::Start => Err(io::Error::other("profiling is already running")),
            Operation::Snapshot | Operation::Stop => {
                let stopping = matches!(operation, Operation::Stop);
                let (path, summary) =
                    session.capture(if stopping { "final" } else { "snapshot" }, counters)?;
                if stopping {
                    COLLECTOR.stop();
                    cpu::stop();
                    state.session = None;
                }
                Ok(format!(
                    "Profiler {} saved: {}\n{summary}",
                    if stopping {
                        "stopped; final snapshot"
                    } else {
                        "snapshot"
                    },
                    path.display()
                ))
            }
            Operation::Status => Ok(format!(
                "Profiling active for {:.1}s; {} snapshots: {}",
                session.started.elapsed().as_secs_f64(),
                session.snapshots,
                session.directory.display()
            )),
            Operation::Cancel => {
                COLLECTOR.stop();
                cpu::stop();
                state.session = None;
                Ok("Profiling cancelled; previously saved snapshots are retained".to_owned())
            }
        }
    }
}

impl State {
    fn start(&mut self, root: &Path, build: &str, counters: &Value) -> io::Result<String> {
        if self.session.is_some() {
            return Err(io::Error::other("profiling is already running"));
        }
        let generation = self.next_generation;
        self.next_generation = generation
            .checked_add(1)
            .ok_or_else(|| io::Error::other("profiler generation exhausted"))?;
        fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(root)?;
        let directory = tempfile::Builder::new()
            .prefix("session-")
            .permissions(Permissions::from_mode(0o700))
            .tempdir_in(root)?;
        let mut session = Session {
            directory: directory.path().to_owned(),
            started: Instant::now(),
            snapshots: 0,
            previous: None,
            sampler: sampling::start()?,
        };
        capture::write_json(
            &session.directory.join("session.json"),
            &json!({
                "schema_version": 2, "build": build, "pid": std::process::id(),
                "os": std::env::consts::OS, "arch": std::env::consts::ARCH,
                "debug_assertions": cfg!(debug_assertions), "allocator": "System",
                "sample_bytes": allocator::SAMPLE_BYTES, "sampling": "inverse_power_of_two",
                "max_frames": MAX_FRAMES, "max_live_samples": MAX_SAMPLES,
                "max_file_bytes": capture::MAX_FILE_BYTES,
                "max_native_xml_bytes": crate::platform::MAX_NATIVE_BYTES,
                "cpu_sampling_frequency": sampling::CPU_FREQUENCY,
                "coverage": "Rust allocations created or successfully reallocated after start; native malloc and mmap excluded"
            }),
        )?;
        allocator::start(generation)?;
        cpu::start(generation);
        let (_, summary) = match session.capture("baseline", counters) {
            Ok(capture) => capture,
            Err(error) => {
                COLLECTOR.stop();
                cpu::stop();
                return Err(error);
            }
        };
        let path = directory.keep();
        self.session = Some(session);
        Ok(format!("Profiling started: {}\n{summary}", path.display()))
    }
}

impl Session {
    fn capture(&mut self, reason: &str, counters: &Value) -> io::Result<(PathBuf, String)> {
        let destination = self
            .directory
            .join(format!("snapshot-{:03}", self.snapshots));
        let snapshot = capture::save(
            &self.directory,
            &destination,
            reason,
            counters,
            self.started,
            self.previous.as_ref(),
            &self.sampler,
        )?;
        self.snapshots += 1;
        let summary = snapshot.summary.clone();
        self.previous = Some(snapshot);
        Ok((destination, summary))
    }
}
