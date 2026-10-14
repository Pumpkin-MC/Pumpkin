# pumpkin-profiler

Memory and CPU profiling for Pumpkin on x86_64 and aarch64 Windows, macOS and Linux with glibc. It records live Rust allocations, samples CPU call stacks (macOS and Linux), and times server work by subsystem, world, entity type, block type, chunk and plugin event.

The default build contains no profiler hooks or commands.

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

Start before the workload. Take a snapshot after warmup, repeat the workload, then take another after unloading and saves. Each command prints a summary and the report path. `profiler status` shows the active session and `profiler cancel` ends it without a final capture. An orderly server shutdown saves a final capture.

Wait for each command to finish. Concurrent requests are rejected, and a failed snapshot or stop can be retried. In-game use requires `pumpkin:command.profiler` (operator level 4). The old `memory-profiler` command and build feature still work.

The `profiling` build profile keeps release optimizations and debug symbols, which give readable function names and file/line attribution. Keep that binary together with its reports.

On macOS, Xcode Command Line Tools are required. On Windows, build with the MSVC toolchain.

## Read a report

Captures are written to `debug/profiler/session-*/snapshot-NNN/`.

| File | Use |
| --- | --- |
| `summary.txt` | CPU usage, RSS/heap changes, largest allocation sites and most expensive work |
| `report.json` | Full function/thread rankings, phase timings, world/type/chunk/plugin timings and server counters |
| `cpu.flamegraph.svg` | Open in a browser to explore CPU call stacks |
| `cpu.speedscope.json` | Open in Speedscope for interactive CPU analysis |
| `cpu.pb.gz` | CPU profile for pprof |
| `heap.pb.gz` | Live heap call stacks for pprof |
| `windows.json` | Working set, commit charge, private bytes, handles, I/O and page faults (Windows) |
| `darwin.json` | Physical footprint, CPU/I/O counters and malloc zone totals (macOS) |
| `glibc.xml` | Native allocator arenas (Linux) |
| `status.txt`, `smaps_rollup.txt`, `maps.txt` | Process memory and mappings from `/proc/self` (Linux) |

`session.json` records the build and collection settings. Report directories have mode `0700` and stay until you delete them.

To rank functions or compare two heap snapshots with Go's pprof tool:

```sh
cd debug/profiler/session-<id>
go tool pprof -top -cum snapshot-001/cpu.pb.gz
go tool pprof -http=127.0.0.1:8080 -base snapshot-001/heap.pb.gz snapshot-002/heap.pb.gz
```

Heap profiles show live bytes by default. Use `-sample_index=inuse_objects` for allocation counts.

## CPU and tick measurements

The CPU sampler records native thread stacks at 100 Hz on macOS and Linux. Short or idle sessions can have few samples, and Wasm guest functions are not resolved. Windows has no stack sampling. Use Windows Performance Recorder (WPR) for CPU call stacks there, and this profiler for heap and timing data.

Timed phases cover the tick and each world, players, entities, block entities, plugin tick hooks, scheduled and random block/fluid ticks, spawning, chunk generation, chunk I/O and lighting. Results are broken down by world, entity type, block type, chunk and plugin event. Up to 4,096 chunk entries and 4,096 other entries are kept, and the report counts the ones it left out.

All timings are cumulative since `start` and inclusive, so nested work overlaps. Parallel worker times cannot be added up to get the tick duration. Async chunk I/O and plugin handlers measure wall time including waits, so they have no CPU time. On Windows, thread CPU time is too coarse for single spans, so all timings are wall time only. Process CPU includes every thread and can exceed 100%, which means one core.

## Memory measurements

Each capture records process RSS and virtual memory on every platform, plus the platform files listed above. `report.json` combines them.

Heap stacks only cover Rust allocations made after `start`. Native `malloc`, direct `mmap` and older allocations are not included. Allocations are sampled every 512 KiB on average, and any allocation over 256 KiB is always recorded, so small changes can be sampling noise.

Rising RSS alone does not prove a leak. Compare heap growth with the world, entity and cache counters in the report.

The collector keeps up to 32,768 live samples with 32 frames each. If it drops samples, the estimate is incomplete. Captures read live data while the server runs, so they are not atomic snapshots.

## Limitations

- Each output file is limited to 32 MiB. Long sessions can use noticeable memory and temporary disk space while the CPU report is built.
- Load native plugins before the first session, because the CPU unwinder caches module information.
- CPU sampling uses `SIGPROF` and the process timer. Don't run another sampler that uses them at the same time.
- On macOS, stacks through system libraries and Apple Silicon frames with signed return addresses can be incomplete.
