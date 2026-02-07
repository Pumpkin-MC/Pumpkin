# Session: core-006
**Agent:** core
**Date:** 2026-02-07
**Task:** Implement debug and perf commands; command audit progress

## Context Loaded
- Rebased onto origin/master (e2fa5e8) — includes PR #56 (core save commands), PR #57 (entity), PR #58 (plugin)
- Read updated .claude/prompts/core.md — P1 priority: command audit completion
- Read .claude/sessions/decisions/core.md (CORE-001 through CORE-009)
- Read .claude/sessions/decisions/architect.md (ARCH-001 through ARCH-031)
- New architect decisions: ARCH-025 through ARCH-031 (store tiers, Calcite, game mapping, SIMD vision, biome reduction, redstone benchmark)
- No new decisions affect Core's immediate work

## Preamble

I am the Core agent, session 006. I acknowledge:
- P1 priority is command audit completion (39/84 remaining)
- Lifecycle events already wired (session 003), save commands done (session 005)
- High-traffic commands (tp, gamemode, time, weather, give, kill) are already implemented
- Remaining Core-scope commands: execute, function, schedule, return (complex), debug, perf, random, advancement, scoreboard, teammsg, trigger, datapack, reload (various difficulty)
- This session targets debug and perf — leveraging the tick profiler from session 002

## What I Did

### 1. Command Audit — Identified 31 Missing Commands

Full audit against vanilla 1.21.4 commands.json:
- **Core scope:** 14 commands (debug, perf, execute, function, schedule, return, random, advancement, scoreboard, teammsg/tm, trigger, datapack, reload)
- **Entity scope:** 7 (attribute, ride, spectate, spreadplayers, tag, team, xp)
- **World scope:** 5 (clone, fillbiome, forceload, locate, place)
- **Inventory scope:** 3 (item, loot, recipe)
- **N/A:** 2 (jfr — JVM only, publish — single-player only)

### 2. Implemented `/debug start|stop` (command/commands/debug.rs)

- `/debug start` — enables tick profiler, resets slow tick counter, confirms to sender
- `/debug stop` — disables profiler, reports detailed stats:
  - Average tick time (world, player/net, total)
  - Peak tick time
  - Slow tick count
  - Budget usage percentage with color coding (green <50%, yellow 50-80%, red >80%)
- Error handling: cannot start when already running, cannot stop when not running
- Permission: OP Level 3 (matches vanilla)

### 3. Implemented `/perf start|stop` (command/commands/perf.rs)

- `/perf start` — same profiler enable as debug, separate command for vanilla compatibility
- `/perf stop` — reports performance summary and logs to server console
- Permission: OP Level 4 (matches vanilla)

Both commands share the same underlying tick profiler infrastructure (session 002).

## What I Changed

### New Files
- `pumpkin/src/command/commands/debug.rs` — debug command
- `pumpkin/src/command/commands/perf.rs` — perf command

### Modified Files
- `pumpkin/src/command/commands/mod.rs` — registered debug/perf commands with Level 3/4 permissions

## Decisions Made

### CORE-010: debug and perf commands share tick profiler
**Date:** 2026-02-07
**Decision:** Both /debug and /perf commands use the same TickProfiler instance. They cannot run simultaneously (starting one while the other is active returns an error). This matches the profiler being a single server-wide resource.
**Rationale:** No need for separate profiler instances. The tick profiler is already integrated into the tick loop (session 002). Both commands are just different entry points to the same profiler data.
**Affects:** Core
**Status:** active

## What Others Should Know
- **Command count is now 55/84** (was 53/84). 29 remaining.
- **Core-scope remaining commands:** execute (complex), function (complex), schedule (complex), return (needs function), random (needs execute context), advancement (needs advancement tree), scoreboard (major subsystem), teammsg/tm (needs team system), trigger (needs scoreboard), datapack (needs hot-reload), reload (needs hot-reload). Total: 11 Core-scope commands remaining, all moderate-to-complex.
- The **tick profiler** is now fully exposed to operators via `/debug` and `/perf`. This was the tick profiler's intended use case.

## Tests
- `cargo test -p pumpkin` — **121 tests pass**, 0 failures
- `cargo check -p pumpkin` — compiles cleanly with RUSTFLAGS="-Dwarnings"
