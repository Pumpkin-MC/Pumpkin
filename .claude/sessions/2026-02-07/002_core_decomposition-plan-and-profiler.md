# Session: core-002
**Agent:** core
**Date:** 2026-02-07
**Task:** lib.rs decomposition plan (ARCH-004), tick profiler infrastructure

## Context Loaded
- Read .claude/sessions/2026-02-07/001_redstone_signal-propagation.md (redstone update order fix, dispenser quasi-connectivity, 28 tests)
- Read .claude/sessions/2026-02-07/001_storage_anvil-and-snbt.md (Anvil region format, SNBT parser, 48 new tests)
- Read .claude/sessions/2026-02-06/001_architect_gap-analysis.md (all 9 agent domains analyzed)
- Read .claude/sessions/2026-02-06/002_architect_restructure-sessions.md (sessions moved to .claude/)
- Read .claude/sessions/2026-02-06/003_architect_consolidate-claude.md (orchestration under .claude/)
- Read .claude/sessions/2026-02-06/004_architect_setup-validation.md (gitignore, .current-agent, workspace compiles)
- Read .claude/sessions/2026-02-06/004_protocol_packets-serialization.md (VarInt overflow validation, BitSet serialize, 21 new tests)
- Read .claude/sessions/2026-02-06/005_entity_spawning-ai-goals-mobs.md (3 AI goals, 6 mobs)
- Read .claude/sessions/decisions/core.md (empty — no prior decisions)
- Read .claude/sessions/decisions/architect.md (ARCH-001 through ARCH-007)
- Read .claude/contracts/core.toml
- No prior core sessions exist — this is the first core session

## Preamble

I am the Core agent. I acknowledge the architect's gap analysis (session 001) which estimates Core at ~65% complete with gaps in:
- lib.rs decomposition (flagged as "23K lines, god object")
- Tick loop doesn't explicitly phase subsystems
- Performance profiling infrastructure
- Graceful shutdown completeness

I acknowledge ARCH-004: Core must publish a decomposition plan as its first session. I acknowledge the architect's repeated request (sessions 001, 003, 004) for this plan.

I also acknowledge the redstone agent's request for `DispenserBlockEntity` support (session 001_redstone) — this is outside my direct scope (block entities live in pumpkin-world) but I will flag it.

**Critical finding: lib.rs is 607 lines, NOT 23K.** The gap analysis was incorrect. The actual concern is `server/mod.rs` at ~940 lines, which is the Server struct. However, this file is already well-factored — see decomposition plan below.

## What I Did

### 1. Comprehensive Structural Analysis

Analyzed the full Core scope:

**lib.rs (607 lines):**
- Module declarations (12 `pub mod` statements)
- Logger initialization (143 lines)
- `PumpkinServer` struct (wrapper for bootstrap: TCP/UDP listeners, ticker spawn, RCON, LAN broadcast, console)
- Connection listener loop (`unified_listener_task` — Java/Bedrock multiplexed accept)
- Console input handlers (stdin, readline)
- `scrub_address` helper

**Verdict:** lib.rs is well-structured. No decomposition needed. 607 lines is reasonable for a crate entry point.

**server/mod.rs (940 lines):**
- `Server` struct (23 fields)
- `Server::new()` — construction, world loading, config
- Player management (add, remove, lookup by name/uuid/ip, count)
- Entity selection (target selectors for commands, ~140 lines)
- Tick methods (tick, tick_worlds, tick_players_and_network, update_tick_times)
- Performance metrics (get_mspt, get_tps, get_average_tick_time_nanos)
- Encryption delegation (to key_store)
- Broadcasting (packets, messages)
- Status/branding getters
- Shutdown sequence

**Already extracted submodules:**
- `connection_cache.rs` (171 lines) — CachedStatus, CachedBranding
- `key_store.rs` (81 lines) — RSA key management, encryption
- `tick_rate_manager.rs` (224 lines) — tick rate, freeze, sprint control
- `ticker.rs` (61 lines) — 20 TPS tick loop
- `seasonal_events.rs` (29 lines) — April Fools

**Command system (98 files, ~13K lines):**
- Already well-organized with clean dispatcher pattern
- 49 commands across 4 permission levels
- Builder DSL for command tree construction
- 32 argument parser files

### 2. Decomposition Assessment

The Server struct has mixed concerns but is already decomposed into submodules for the cleanest extraction targets. Remaining concerns in mod.rs are tightly coupled (player management needs worlds, tick needs worlds and players, etc.). Further decomposition would create artificial indirection for marginal benefit.

**Recommended future extractions (if Server grows further):**
1. `player_manager.rs` — add_player, remove_player, get_player_*, get_all_players, for_each_player, get_player_count, has_n_players (~100 lines)
2. `entity_selector.rs` — select_entities method (~140 lines)
3. `tick_stats.rs` — tick_times_nanos, aggregated_tick_times_nanos, update_tick_times, get_mspt, get_tps, get_average_tick_time_nanos (~60 lines)

**NOT recommended to extract now:** The existing ~940 lines with 5 submodules already extracted is a healthy ratio. Moving code between files risks breaking other agents' imports and creating unnecessary churn.

### 3. Implemented Tick Profiler (`tick_profiler.rs`, ~280 lines)

Created a lock-free per-tick performance profiler that tracks:
- **Per-phase timing:** world tick, player/network tick, total tick
- **Rolling statistics:** 100-tick window with average, last, and peak values
- **Slow tick detection:** configurable threshold (default 50ms), counter
- **Snapshot API:** `TickProfileSnapshot` struct with all metrics and convenience methods (budget_usage_percent, *_ms())

Design choices:
- Lock-free atomics throughout — zero contention from the hot tick path
- Disabled by default — zero overhead when not profiling
- `RollingAverage` with fixed 100-sample window matches the existing `tick_times_nanos` approach
- No heap allocations after construction

### 4. Integrated Profiler into Server Tick Methods

Added timing instrumentation to:
- `Server::tick()` — records total tick time
- `Server::tick_worlds()` — records world phase time
- `Server::tick_players_and_network()` — records player/network phase time

The profiler is a field on Server (`tick_profiler: Arc<TickProfiler>`) initialized during `Server::new()`.

### 5. Wrote 10 Unit Tests

Tests cover:
- `RollingAverage`: empty, single sample, multiple samples, wraparound
- `TickProfiler`: disabled by default, enable/disable, records when enabled, ignores when disabled, slow tick detection, slow tick reset
- `TickProfileSnapshot`: budget usage calculation

## What I Changed

### New Files
- `pumpkin/src/server/tick_profiler.rs` — tick profiler module (~280 lines)

### Modified Files
- `pumpkin/src/server/mod.rs` — added `tick_profiler` module declaration, import, Server field, initialization, and phase timing in tick methods

## Perspectives Consulted
- **Protocol Consultant**: Examined the connection listener in lib.rs. The Java/Bedrock multiplexed accept loop is clean. No changes needed.
- **WorldGen Consultant**: Examined `tick_worlds()` which spawns world ticks via JoinSet (parallel). World tick phasing is internal to World::tick() — environment first, then chunks, then players, then entities, then flush. This is close to vanilla order.
- **Entity Consultant**: Noted that entities tick after players in World::tick(). In vanilla, entities tick before block updates but the current order (chunks/blocks -> players -> entities) differs. This is a World-level concern, not Server-level.

## What I Need From Others
- **Architect**: The gap analysis claimed lib.rs is 23K lines. It is actually 607 lines. Please update the gap analysis to reflect this. The "god object" concern is overstated — server/mod.rs at ~940 lines with 5 extracted submodules is healthy.
- **Redstone**: Acknowledged your request for `DispenserBlockEntity`. This requires pumpkin-world changes (block entity storage) which is WorldGen's domain per ARCH-002.

## What Others Should Know
- The tick profiler is **disabled by default**. Enable with `server.tick_profiler.set_enabled(true)`. A future `/tick profile` command can expose this.
- `TickProfileSnapshot` provides everything needed for a diagnostics command: per-phase averages, peaks, slow tick count, budget usage percentage.
- The profiler uses no locks — it is safe to call from any async context without risking deadlocks with the existing tick_times Mutex.
- **lib.rs does NOT need decomposition.** At 607 lines it is well within acceptable size for a crate entry point.

## Decisions Made

### CORE-001: lib.rs decomposition NOT needed
**Decision:** lib.rs at 607 lines is well-structured and does not require decomposition. The gap analysis estimate of "23K lines" was incorrect.
**Rationale:** The file contains module declarations, logger setup, PumpkinServer bootstrap, and console handlers — all appropriate for a crate root. No code moves needed.
**Affects:** All agents (removes ARCH-004 blocker for everyone waiting on this plan)
**Status:** active

### CORE-002: server/mod.rs decomposition deferred
**Decision:** server/mod.rs at ~940 lines with 5 extracted submodules is healthy. Further decomposition (player_manager, entity_selector, tick_stats) is documented but deferred until the file grows past ~1200 lines.
**Rationale:** Premature extraction creates artificial indirection. The existing module structure (connection_cache, key_store, tick_rate_manager, ticker, seasonal_events, tick_profiler) already separates the cleanest concerns.
**Affects:** Core
**Status:** active

### CORE-003: Tick profiler uses lock-free atomics
**Decision:** The tick profiler uses AtomicU64/AtomicBool throughout with no Mutex or RwLock.
**Rationale:** The tick loop is the hottest path in the server. Adding lock contention would defeat the purpose of profiling. The slight inaccuracy from relaxed ordering is acceptable for performance diagnostics.
**Affects:** Core
**Status:** active

## Tests
- `cargo test -p pumpkin` — **59 tests pass**, 0 failures (10 new + 49 existing)
- `cargo check -p pumpkin` — compiles cleanly, 0 errors
- New test breakdown:
  - RollingAverage: 4 tests (empty, single, multiple, wraparound)
  - TickProfiler: 6 tests (disabled_by_default, enable_disable, records_when_enabled, ignores_when_disabled, slow_tick_detection, slow_tick_reset, snapshot_budget_usage)

## Open Questions
1. Should a `/tick profile` command be added to expose the profiler to operators? This would be a natural next step.
2. The current world tick order in World::tick() is: environment -> chunks -> players -> entities -> flush. Vanilla order is: packets -> time/weather -> entities -> blocks -> chunks -> outgoing. Should we pursue reordering? This would be a World-level change (outside Core write paths).
3. The `DispenserBlockEntity` requested by Redstone needs WorldGen/Architect coordination.
