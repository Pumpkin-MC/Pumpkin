# pumpkin-profiler

Memory and CPU profiling for Pumpkin on x86_64 and aarch64 macOS and Linux with glibc. It records live Rust allocations, samples CPU call stacks, and times server work by subsystem, world, entity type, block type, chunk and plugin event.

## Run

```sh
cargo build -p pumpkin --profile profiling --features profiler
```

Run `target/profiling/pumpkin` from your server directory. In the console:

```text
profiler start
profiler snapshot
profiler stop
```

Start before the workload. Take a snapshot after warmup, repeat the workload, then take another after unloading and saves. Each command prints a summary and the report path. `profiler status` shows the active session. `profiler cancel` ends it without a final capture. Orderly server shutdown saves a final capture.

Wait for each command to finish. Concurrent requests are rejected; a failed snapshot or stop can be retried. In-game commands require `pumpkin:command.profiler` (operator level 4). The old `memory-profiler` command and build feature still work.

## Read a report

Captures are under `debug/profiler/session-*/snapshot-NNN/`.

| File | Use |
| --- | --- |
| `summary.txt` | CPU usage, RSS/heap changes, largest allocation sites and expensive work |
| `report.json` | Full function/thread rankings, phase timings, world/type/chunk/plugin timings and server counters |
| `cpu.flamegraph.svg` | Open in a browser to explore CPU call stacks |
| `cpu.speedscope.json` | Open in Speedscope for interactive CPU analysis |
| `cpu.pb.gz` | CPU profile for pprof |
| `heap.pb.gz` | Live heap call stacks for pprof |
| `darwin.json` | Physical footprint, CPU/I/O counters and malloc zone totals (macOS) |
| `glibc.xml` | Native allocator arenas (Linux) |
| `status.txt`, `smaps_rollup.txt`, `maps.txt` | Process memory and mappings from `/proc/self` (Linux) |

`session.json` records the build and collection settings. Directories have mode `0700`. Reports remain until deleted.

For function rankings or a heap comparison with Go's pprof tool:

```sh
cd debug/profiler/session-<id>
go tool pprof -top -cum snapshot-001/cpu.pb.gz
go tool pprof -http=127.0.0.1:8080 -base snapshot-001/heap.pb.gz snapshot-002/heap.pb.gz
```

Heap profiles default to live bytes. Use `-sample_index=inuse_objects` for allocation counts. Profiler allocation frames are hidden.

On macOS, build and run the same commands with Xcode Command Line Tools installed.

Debug symbols are optional for collection. Keep them for readable function names and file/line attribution. The `profiling` build keeps release optimization and debug symbols. Keep that binary with its reports.

## CPU and tick measurements

The CPU sampler runs at 100 Hz and records native thread stacks. Reports contain self and inclusive sample counts per function, thread totals, and call stacks. Short or idle sessions can have few samples. `missed_samples` counts lock-contention misses; blocked libraries and failed unwinds are not included in that counter. Wasm guest functions are not resolved by the native sampler.

Timed phases cover:

- Tick and world totals; world setup/environment and entity tracking.
- Functions/plugin tick hooks, players, entities and block entities.
- Scheduled block/fluid ticks, random ticks and spawning.
- Chunk generation, chunk loading/saving and lighting.

Synchronous spans measure wall time and current-thread CPU time. Async chunk I/O and plugin event handlers measure elapsed wall time, including waits; their CPU time is null. The sampled CPU profile shows where CPU execution occurred separately.

World timings identify the dimension and world UUID. Entity and block entity timings identify the type. Block/fluid timings distinguish scheduled and random ticks. Chunk timings combine measured entity, block entity, block/fluid and spawning work in that chunk. Plugin timings identify the handler source and event; handlers without a source appear as `unattributed`. A maximum of 4,096 distinct detail keys is retained; omitted entries are counted in the report.

Timings and CPU samples are cumulative since start. Tick timing includes tick events and excludes sleeping before the next tick. A phase call may be one player tick, a worker batch, one generation job or one I/O batch. Timings are inclusive: nested work overlaps, and parallel worker wall times cannot be added to obtain tick duration. Main-thread CPU excludes Rayon workers. Process CPU includes every thread and profiling overhead, is measured between snapshots, and can exceed 100% because 100% means one core.

## Memory measurements

`memory-stats` reads process RSS and virtual memory on both platforms. Linux also saves `/proc` diagnostics and glibc arena XML. macOS uses Mach task information through `memory-stats`, `proc_pid_rusage(RUSAGE_INFO_V4)` for footprint, CPU time, I/O, page-ins and wakeups, and `malloc_zone_statistics` for totals across all malloc zones. Footprint, wired memory and malloc usage are current values; CPU, I/O, page-ins and wakeups accumulate over the process lifetime. These values are included in `report.json` and `darwin.json`.

Heap stacks cover Rust allocations created or successfully reallocated after `start`. Native `malloc`, direct `mmap` and older allocations are absent. Sampling uses inverse power-of-two weights with a 512 KiB interval; allocations larger than 256 KiB are always selected. Small changes can be sampling noise. Compare heap growth with world/entity/cache counters; rising RSS alone does not establish a leak.

The heap collector holds up to 32,768 live samples with 32 frames each. Dropped samples make the estimate incomplete. Heap sample storage is released on stop/cancel. Captures read live data while the server runs; they are not atomic process snapshots. A busy region-cache lock produces a null count. Unavailable `smaps_rollup` data is recorded as an error.

On macOS, CPU stack coverage can be limited in dyld shared-cache libraries and Apple Silicon frames with signed return addresses. The sampler uses available unwind data and does not provide complete coverage of those system frames. System allocator/unwinder libraries are excluded from CPU sampling.

Each output file is limited to 32 MiB. CPU collection can use temporary disk space and resolves its report in memory before the output limit is applied, so long sessions can consume more resources. Load native plugins before the first session: the CPU unwinder caches module information. CPU sampling owns `SIGPROF` and its process timer; do not run another sampler using those resources simultaneously.

The default build has no profiler hooks or commands. The profiler uses the existing `System` allocator. Its CPU dependency is `rpprof` with `framehop-unwinder`, `flamegraph` and `prost-codec`; default C++ demangling features are disabled.
