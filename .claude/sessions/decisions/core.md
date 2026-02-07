# Core — Decisions

## CORE-001: lib.rs decomposition NOT needed
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_core_decomposition-plan-and-profiler.md
**Decision:** lib.rs at 607 lines is well-structured and does not require decomposition. The gap analysis estimate of "23K lines" was incorrect.
**Rationale:** The file contains module declarations, logger setup, PumpkinServer bootstrap, and console handlers — all appropriate for a crate root. No code moves needed.
**Affects:** All agents (removes ARCH-004 blocker)
**Status:** active

## CORE-002: server/mod.rs decomposition deferred
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_core_decomposition-plan-and-profiler.md
**Decision:** server/mod.rs at ~940 lines with 5 extracted submodules is healthy. Further decomposition (player_manager, entity_selector, tick_stats) documented but deferred until the file grows past ~1200 lines.
**Rationale:** Premature extraction creates artificial indirection. The existing module structure (connection_cache, key_store, tick_rate_manager, ticker, seasonal_events, tick_profiler) already separates the cleanest concerns.
**Affects:** Core
**Status:** active

## CORE-003: Tick profiler uses lock-free atomics
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_core_decomposition-plan-and-profiler.md
**Decision:** The tick profiler uses AtomicU64/AtomicBool throughout with no Mutex or RwLock.
**Rationale:** The tick loop is the hottest path in the server. Adding lock contention would defeat the purpose of profiling. The slight inaccuracy from relaxed ordering is acceptable for performance diagnostics.
**Affects:** Core
**Status:** active

## CORE-004: ServerTickEvent fires after profiler recording
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/003_core_lifecycle-events.md
**Decision:** ServerTickEvent fires at the end of Server::tick(), after tick profiler recording but still within the tick method.
**Rationale:** Plugins get the most accurate tick count. Firing after profiler ensures the profiler measures actual game logic, not plugin event handling overhead.
**Affects:** Core, Plugin
**Status:** active

## CORE-005: ServerStopEvent fires before player disconnect
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/003_core_lifecycle-events.md
**Decision:** ServerStopEvent fires immediately after connection acceptance stops, before player data save, player kicks, plugin unload, or world save.
**Rationale:** Matches Bukkit's ServerStopEvent behavior. Plugins may want to send final messages, save data, or clean up while players are still connected and the world is accessible.
**Affects:** Core, Plugin
**Status:** active

## CORE-006: Config fields added before runtime enforcement
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/004_core_command-config-audit.md
**Decision:** Config fields (allow_flight, spawn_protection, generate_structures, player_idle_timeout) added to BasicConfiguration before runtime code that consumes them.
**Rationale:** Config fields with sane defaults are harmless. They appear in TOML, allowing operators to set values before enforcement exists. Other agents can consume them.
**Affects:** Core, Entity (allow_flight, player_idle_timeout), WorldGen (generate_structures, spawn_protection)
**Status:** active

## CORE-007: Command completeness — 39 missing, 7 are Core scope
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/004_core_command-config-audit.md
**Decision:** Of 39 missing vanilla commands, 7 fall under Core scope: execute, function, schedule, return, save-all, save-off, save-on. Save-* are simple. Execute/function/schedule/return require dispatcher work.
**Rationale:** Execute is most impactful missing command but also most complex. Needs Architect guidance.
**Affects:** Core, Architect
**Status:** active
