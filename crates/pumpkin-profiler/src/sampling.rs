use std::{
    cmp::Reverse,
    collections::{HashMap, HashSet},
    io::{self, Write},
    path::Path,
};

use flate2::{Compression, write::GzEncoder};
use prost::Message;
use serde::Serialize;

use crate::capture::{BoundedWriter, write_file};

pub const CPU_FREQUENCY: i32 = 100;

#[derive(Serialize)]
pub struct Function {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub self_samples: u64,
    pub inclusive_samples: u64,
}

#[derive(Serialize)]
pub struct Thread {
    pub name: String,
    pub id: u64,
    pub samples: u64,
}

#[derive(Serialize)]
pub struct Analysis {
    pub frequency: i32,
    pub samples: usize,
    pub missed_samples: u64,
    pub functions: Vec<Function>,
    pub threads: Vec<Thread>,
}

pub fn start() -> io::Result<rpprof::ProfilerGuard<'static>> {
    #[cfg(target_os = "linux")]
    let blocked = &["libc", "libgcc", "pthread", "vdso"][..];
    #[cfg(target_os = "macos")]
    let blocked = &[
        "libsystem_",
        "libSystem.",
        "libunwind",
        "libdyld",
        "libobjc",
        "libdispatch",
        "pthread",
    ][..];
    rpprof::ProfilerGuardBuilder::default()
        .frequency(CPU_FREQUENCY)
        .blocklist(blocked)
        .build()
        .map_err(io::Error::other)
}

pub fn capture(guard: &rpprof::ProfilerGuard<'_>, path: &Path) -> io::Result<Analysis> {
    let report = guard.report().build().map_err(io::Error::other)?;
    let mut functions = HashMap::<(String, String, u32), (u64, u64)>::new();
    let mut threads = HashMap::<(String, u64), u64>::new();
    for (frames, &count) in &report.data {
        let count = u64::try_from(count).map_err(io::Error::other)?;
        *threads
            .entry((frames.thread_name.clone(), frames.thread_id))
            .or_default() += count;
        let mut seen = HashSet::new();
        for (index, symbol) in frames.frames.iter().flatten().enumerate() {
            let key = (
                symbol.name(),
                symbol.filename().into_owned(),
                symbol.lineno(),
            );
            if !seen.insert(key.clone()) {
                continue;
            }
            let value = functions.entry(key).or_default();
            value.0 += u64::from(index == 0) * count;
            value.1 += count;
        }
    }
    let mut functions: Vec<_> = functions
        .into_iter()
        .map(
            |((name, file, line), (self_samples, inclusive_samples))| Function {
                name,
                file,
                line,
                self_samples,
                inclusive_samples,
            },
        )
        .collect();
    functions.sort_unstable_by_key(|function| {
        Reverse((function.self_samples, function.inclusive_samples))
    });
    let mut threads: Vec<_> = threads
        .into_iter()
        .map(|((name, id), samples)| Thread { name, id, samples })
        .collect();
    threads.sort_unstable_by_key(|thread| Reverse(thread.samples));
    let mut flamegraph = BoundedWriter(Vec::new());
    if report.total_samples() == 0 {
        flamegraph.write_all(br#"<svg xmlns="http://www.w3.org/2000/svg" width="600" height="40"><text x="10" y="25">No CPU samples yet.</text></svg>"#)?;
    } else {
        report
            .flamegraph(&mut flamegraph)
            .map_err(io::Error::other)?;
    }
    write_file(&path.join("cpu.flamegraph.svg"), &flamegraph.0)?;
    let mut speedscope = BoundedWriter(Vec::new());
    report.write_speedscope(&mut speedscope, "Pumpkin CPU")?;
    write_file(&path.join("cpu.speedscope.json"), &speedscope.0)?;
    let profile = report.pprof().map_err(io::Error::other)?;
    if profile.encoded_len() > crate::capture::MAX_FILE_BYTES {
        return Err(io::Error::other("CPU profile exceeds output size limit"));
    }
    let mut compressed = GzEncoder::new(BoundedWriter(Vec::new()), Compression::fast());
    compressed.write_all(&profile.encode_to_vec())?;
    write_file(&path.join("cpu.pb.gz"), &compressed.finish()?.0)?;
    Ok(Analysis {
        frequency: CPU_FREQUENCY,
        samples: report.total_samples(),
        missed_samples: report.missed_samples,
        functions,
        threads,
    })
}
