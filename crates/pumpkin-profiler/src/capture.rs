use std::{
    fs::{self, File, Permissions},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use flate2::{Compression, write::GzEncoder};
use prost::Message;
use serde_json::{Value, json};

use crate::{collector::COLLECTOR, cpu, platform, profile, sampling};

pub const MAX_FILE_BYTES: usize = 32 * 1024 * 1024;

pub struct Snapshot {
    rss_bytes: Option<u64>,
    heap_bytes: usize,
    captured: Instant,
    cpu_nanos: Option<u64>,
    process_cpu_percent: Option<f64>,
    pub summary: String,
}

pub fn save(
    root: &Path,
    destination: &Path,
    reason: &str,
    counters: &Value,
    started: Instant,
    previous: Option<&Snapshot>,
    sampler: &rpprof::ProfilerGuard<'_>,
) -> io::Result<Snapshot> {
    let capture_started = unix_nanos();
    let directory = tempfile::Builder::new()
        .prefix(".snapshot-")
        .permissions(Permissions::from_mode(0o700))
        .tempdir_in(root)?;
    let path = directory.path();
    let process = platform::capture(path)?;
    let (samples, dropped) = COLLECTOR.snapshot();
    let sampled_at = unix_nanos();
    let (profile, heap) = profile::build(&samples, dropped, sampled_at, started.elapsed())?;
    let timings = cpu::snapshot();
    let details = cpu::detail_snapshot();
    if profile.encoded_len() > MAX_FILE_BYTES {
        return Err(io::Error::other("heap profile exceeds output size limit"));
    }
    let cpu_samples = sampling::capture(sampler, path)?;
    let snapshot = Snapshot::new(
        &process,
        &heap,
        &timings,
        previous,
        &cpu_samples,
        &details,
    );
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&profile.encode_to_vec())?;
    write_file(&path.join("heap.pb.gz"), &encoder.finish()?)?;
    write_file(&path.join("summary.txt"), snapshot.summary.as_bytes())?;
    write_json(
        &path.join("report.json"),
        &json!({
            "schema_version": 2, "reason": reason, "capture_started_unix_nanos": capture_started,
            "heap_sampled_unix_nanos": sampled_at, "capture_finished_unix_nanos": unix_nanos(),
            "elapsed_seconds": started.elapsed().as_secs_f64(), "heap": heap,
            "rss_bytes": snapshot.rss_bytes,
            "virtual_bytes": process.virtual_bytes, "native_memory": process.native,
            "cpu_timings": timings.iter().map(|(name, timing)| ((*name).to_owned(), json!(timing))).collect::<serde_json::Map<_, _>>(),
            "process_cpu_percent": snapshot.process_cpu_percent,
            "cpu_samples": cpu_samples,
            "cpu_details": details,
            "smaps_rollup_error": process.smaps_rollup_error, "pumpkin": counters
        }),
    )?;
    fs::rename(path, destination)?;
    Ok(snapshot)
}

impl Snapshot {
    fn new(
        process: &platform::Process,
        heap: &profile::HeapStats,
        timings: &[(&str, cpu::Timing)],
        previous: Option<&Self>,
        cpu_samples: &sampling::Analysis,
        details: &cpu::Details,
    ) -> Self {
        let rss_bytes = Some(process.rss_bytes);
        let heap_bytes = heap.estimated_live_bytes;
        let captured = Instant::now();
        let cpu_nanos = cpu::process_cpu_nanos();
        let process_cpu_percent = previous.and_then(|before| {
            let elapsed = captured.duration_since(before.captured).as_secs_f64();
            let used = cpu_nanos?.checked_sub(before.cpu_nanos?)?;
            (elapsed > 0.0).then(|| used as f64 / 1_000_000_000.0 / elapsed * 100.0)
        });
        let mut lines = Vec::new();
        if let Some(percent) = process_cpu_percent {
            lines.push(format!(
                "Process CPU: {percent:.1}% of one core since previous snapshot"
            ));
        }
        if let Some(rss) = rss_bytes {
            lines.push(format!(
                "RSS: {:.2} MiB{}",
                mib(rss as f64),
                delta(
                    rss as f64,
                    previous
                        .and_then(|value| value.rss_bytes)
                        .map(|value| value as f64)
                )
            ));
        }
        lines.push(format!(
            "Virtual memory: {:.2} MiB",
            mib(process.virtual_bytes as f64)
        ));
        if let Some(footprint) = process
            .native
            .get("physical_footprint_bytes")
            .and_then(Value::as_u64)
        {
            lines.push(format!(
                "Darwin physical footprint: {:.2} MiB",
                mib(footprint as f64)
            ));
        }
        lines.push(format!(
            "Estimated live Rust heap: {:.2} MiB{}",
            mib(heap_bytes as f64),
            delta(
                heap_bytes as f64,
                previous.map(|value| value.heap_bytes as f64)
            )
        ));
        if heap.dropped_samples != 0 {
            lines.push(format!(
                "Incomplete heap estimate: {} samples dropped",
                heap.dropped_samples
            ));
        }
        if !heap.top_allocations.is_empty() {
            lines.push("Largest retained allocation sites:".to_owned());
            for site in &heap.top_allocations {
                lines.push(format!(
                    "  {:.2} MiB  {}",
                    mib(site.bytes as f64),
                    site.name
                ));
            }
        }
        lines.extend(cpu_summary(timings, cpu_samples, details));
        Self {
            rss_bytes,
            heap_bytes,
            captured,
            cpu_nanos,
            process_cpu_percent,
            summary: lines.join("\n"),
        }
    }
}

fn cpu_summary(
    timings: &[(&str, cpu::Timing)],
    cpu_samples: &sampling::Analysis,
    details: &cpu::Details,
) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(format!(
        "CPU stack samples: {}, lock-contention misses: {}",
        cpu_samples.samples, cpu_samples.missed_samples
    ));
    for function in cpu_samples.functions.iter().take(5) {
        let percent = if cpu_samples.samples == 0 {
            0.0
        } else {
            function.self_samples as f64 / cpu_samples.samples as f64 * 100.0
        };
        lines.push(format!(
            "  {percent:.1}% of CPU samples  {} ({}:{})",
            function.name, function.file, function.line
        ));
    }
    lines.push("Tick/worker timings since start (inclusive):".to_owned());
    for (name, timing) in timings {
        if timing.calls == 0 {
            continue;
        }
        let cpu = timing.cpu_nanos.map_or_else(
            || "unavailable".to_owned(),
            |value| format!("{:.2} ms", millis(value)),
        );
        lines.push(format!(
            "  {name}: {} calls, {:.2} ms wall, {cpu} CPU, {:.2} ms avg, {:.2} ms max{}",
            timing.calls,
            millis(timing.wall_nanos),
            millis(timing.wall_nanos) / timing.calls as f64,
            millis(timing.max_wall_nanos),
            if *name == "tick" {
                format!(", {} over budget", timing.over_budget)
            } else {
                String::new()
            }
        ));
    }
    if !details.entries.is_empty() {
        lines.push("Most expensive world/type/chunk/plugin entries (inclusive):".to_owned());
        for entry in details.entries.iter().take(10) {
            let timing = entry.timing;
            let cpu = timing.cpu_nanos.map_or_else(
                || "CPU unavailable".to_owned(),
                |value| format!("{:.2} ms CPU", millis(value)),
            );
            lines.push(format!(
                "  {}: {} calls, {:.2} ms wall, {cpu}",
                entry.detail.label(),
                timing.calls,
                millis(timing.wall_nanos)
            ));
        }
    }
    if details.dropped_keys != 0 {
        lines.push(format!(
            "Detail capacity reached: {} unseen entries omitted",
            details.dropped_keys
        ));
    }
    lines
}

fn millis(nanos: u64) -> f64 {
    nanos as f64 / 1_000_000.0
}

fn mib(bytes: f64) -> f64 {
    bytes / (1024.0 * 1024.0)
}

fn delta(current: f64, previous: Option<f64>) -> String {
    previous.map_or_else(String::new, |before| {
        format!(
            " ({:+.2} MiB since previous snapshot)",
            mib(current - before)
        )
    })
}

fn unix_nanos() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .min(i64::MAX as u128) as i64
}

pub fn write_json(path: &Path, value: &Value) -> io::Result<()> {
    let mut bytes = BoundedWriter(Vec::new());
    serde_json::to_writer_pretty(&mut bytes, value)?;
    write_file(path, &bytes.0)
}

pub struct BoundedWriter(pub Vec<u8>);

impl Write for BoundedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_FILE_BYTES - self.0.len() {
            return Err(io::Error::other(
                "profiler report exceeds output size limit",
            ));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn write_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if bytes.len() > MAX_FILE_BYTES {
        return Err(io::Error::other(
            "profiler report exceeds output size limit",
        ));
    }
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
