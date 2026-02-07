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
