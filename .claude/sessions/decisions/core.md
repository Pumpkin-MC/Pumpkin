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
**Status:** active (save-* implemented in CORE-008/009, 4 remaining: execute, function, schedule, return)

## CORE-008: save-all uses should_save flag for non-destructive chunk save
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/005_core_save-commands.md
**Decision:** The save-all command triggers chunk saves by setting `level.should_save = true` and notifying the chunk system, rather than calling `Level::shutdown()`. The `flush` variant additionally waits for pending IO to complete.
**Rationale:** `Level::shutdown()` cancels the chunk system and joins threads — it's destructive and can only be called once. The `should_save` flag is the designed mechanism for on-demand saves: it triggers `save_all_chunk()` in the chunk system thread without shutting anything down.
**Affects:** Core
**Status:** active

## CORE-009: save-off/save-on control autosave_enabled on Server
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/005_core_save-commands.md
**Decision:** The save-off/save-on commands toggle `server.autosave_enabled` (AtomicBool). When disabled, `tick_worlds()` skips `player_data_storage.tick()`. Chunk saves on unload still happen (managed by chunk system, not tick loop).
**Rationale:** In vanilla, save-off disables periodic autosave, not chunk unload saves. Pumpkin's chunk saves happen on unload (not on a timer), so save-off primarily affects player data autosave. `/save-all` always saves regardless of the autosave state, matching vanilla.
**Affects:** Core
**Status:** active

## CORE-010: debug and perf commands share tick profiler
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/006_core_debug-perf-commands.md
**Decision:** Both /debug and /perf commands use the same TickProfiler instance. They cannot run simultaneously (starting one while the other is active returns an error). This matches the profiler being a single server-wide resource.
**Rationale:** No need for separate profiler instances. The tick profiler is already integrated into the tick loop (session 002). Both commands are just different entry points to the same profiler data.
**Affects:** Core
**Status:** active

## CORE-011: log_admin_commands enforced at dispatcher level
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/007_core_gamerule-audit.md
**Decision:** The `log_admin_commands` game rule is checked in `CommandDispatcher::dispatch()` after permission check. When enabled, logs `"{sender} issued server command: /{cmd}"` to the server console.
**Rationale:** Centralized enforcement in the dispatcher ensures all commands are logged without per-command code. Matches vanilla behavior.
**Affects:** Core
**Status:** active

## CORE-012: Chunk events blocked by type mismatch
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/008_core_chunk-event-audit-config-enforcement.md
**Decision:** ChunkLoad/ChunkSend/ChunkSave events cannot be wired until Plugin agent fixes event struct type from `Arc<RwLock<ChunkData>>` to `Arc<ChunkData>`. The runtime universally uses `SyncChunk = Arc<ChunkData>` and ChunkData does not implement Clone.
**Rationale:** Comprehensive audit of all 9 remaining unfired events. 3 chunk events are blocked by type mismatch. 2 block events (Place, CanBuild) are in net/ (not Core scope). 4 block events (Burn, FromTo, Grow, Fade) are Redstone scope.
**Affects:** Core, WorldGen, Entity, Plugin
**Status:** active (blocker — requires Plugin agent fix)

## CORE-013: force_gamemode enforced on player login
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/008_core_chunk-event-audit-config-enforcement.md
**Decision:** After player NBT data is loaded in `add_player()`, if `force_gamemode` is true, the player's gamemode is reset to the server default. Placed after `read_nbt()` and before `Arc::new()`.
**Rationale:** Matches vanilla `force-gamemode` behavior. Previously only enforced when admin changed default gamemode via `/defaultgamemode`, not on login.
**Affects:** Core
**Status:** active

## CORE-014: Three more config fields added
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/008_core_chunk-event-audit-config-enforcement.md
**Decision:** Added `broadcast_console_to_ops` (bool, true), `max_world_size` (u32, 29999984), `function_permission_level` (PermissionLvl, Two) to BasicConfiguration following CORE-006 pattern.
**Rationale:** Commonly used vanilla server.properties fields. Declared before runtime enforcement to allow operators to configure them.
**Affects:** Core
**Status:** active
