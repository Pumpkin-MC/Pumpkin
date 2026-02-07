# Plugin — Decisions

## PLUGIN-001: Entity events use primitive entity_id (i32)
**Date:** 2026-02-07
**Decision:** Entity events carry `entity_id: i32` and `entity_type: &'static EntityType` rather than `Arc<LivingEntity>`.
**Rationale:** Keeps plugin API decoupled from internal entity implementation. Avoids exposing internal state.
**Affects:** Plugin
**Status:** active

## PLUGIN-002: Monitor priority is Bukkit-compatible observe-only
**Date:** 2026-02-07
**Decision:** Added `EventPriority::Monitor` as 6th priority level. Handlers at Monitor MUST NOT modify the event.
**Rationale:** Matches Bukkit's `EventPriority.MONITOR`. Enables logging/metrics plugins.
**Affects:** Plugin
**Status:** active

## PLUGIN-003: Non-cancellable lifecycle events
**Date:** 2026-02-07
**Decision:** ServerStartedEvent, ServerStopEvent, ServerTickEvent are NOT cancellable.
**Rationale:** These represent facts, not proposals. Matches Bukkit (ServerLoadEvent not cancellable).
**Affects:** Plugin
**Status:** active

## PLUGIN-004: ignore_cancelled filtering — IMPLEMENTED
**Date:** 2026-02-07
**Decision:** Implemented Bukkit-compatible `ignore_cancelled` filtering in `PluginManager::fire()`. Blocking handlers check `handler.ignore_cancelled() && event.is_cancelled()` per-iteration (since earlier handlers can cancel mid-loop). Non-blocking handlers are filtered once before `join_all` (they receive immutable refs, so cancellation state is stable during concurrent execution). Non-cancellable events always return `false` from `is_cancelled()`, so filtering is a no-op for them.
**Rationale:** Bukkit's `@EventHandler(ignoreCancelled = true)` skips a handler if a higher-priority handler already cancelled the event.
**Affects:** Plugin
**Status:** IMPLEMENTED (commit a5f1dbc, merged as PR #46)

## PLUGIN-005: Multi-version data harvest with delta annotations
**Date:** 2026-02-07
**Decision:** Harvested PrismarineJS data for MC 1.12.2, 1.14.4, 1.16.5, 1.18.2, 1.21.4 into `.claude/specs/data/`. Built TOML registries: bukkit_api.toml (283 events), entities.toml, items.toml, blocks.toml, protocol.toml (237 packets). Delta registries annotate version presence for multi-version plugin compatibility.
**Rationale:** Agents need canonical data to implement features. Cross-version deltas enable the DTO multi-version strategy.
**Affects:** All agents
**Status:** active — FLOW

## PLUGIN-006: Block events for Redstone agent (RED-003 unblock)
**Date:** 2026-02-07
**Decision:** Created 7 new block event types: BlockRedstoneEvent, BlockPistonExtendEvent, BlockPistonRetractEvent, BlockPhysicsEvent, BlockFromToEvent, BlockGrowEvent, BlockFadeEvent. All cancellable, all implement BlockEvent trait.
**Rationale:** RED-003 blocked Redstone agent from firing events until Plugin created the type definitions. These match Bukkit's block event hierarchy.
**Affects:** Plugin, Redstone
**Status:** IMPLEMENTED (commit a5f1dbc, merged as PR #46)

## PLUGIN-007: Player item events (drop, consume)
**Date:** 2026-02-07
**Decision:** Created PlayerDropItemEvent and PlayerItemConsumeEvent. Both carry `ItemStack` (from pumpkin-world) directly rather than `Arc<Mutex<ItemStack>>` since the events represent a snapshot, not a live reference.
**Rationale:** Matches Bukkit's PlayerDropItemEvent and PlayerItemConsumeEvent. Snapshot semantics prevent race conditions.
**Affects:** Plugin
**Status:** IMPLEMENTED (commit a5f1dbc, merged as PR #46)

## PLUGIN-008: Core lifecycle events verified
**Date:** 2026-02-07
**Decision:** Verified that Core agent correctly wired all 3 lifecycle events: ServerStartedEvent fires after init_plugins() in main.rs, ServerTickEvent fires at end of each tick in server/mod.rs, ServerStopEvent fires during shutdown in lib.rs. Firing order matches Bukkit semantics.
**Rationale:** Core requested Plugin verification of lifecycle event wiring (session 003_core).
**Affects:** Plugin, Core
**Status:** VERIFIED

## PLUGIN-009: Event firing coverage audit — 25/39 (64%)
**Date:** 2026-02-07
**Decision:** Comprehensive audit of all 39 event types. 25 are fired (via `plugin_manager.fire()` or `send_cancellable!` macro). 14 remain unfired — all require wiring from code outside Plugin's write_paths:

**Wiring roadmap for remaining 14 events:**
- **Entity agent** → PlayerDeathEvent (entity/player.rs death handling), PlayerRespawnEvent (entity/player.rs respawn), PlayerDropItemEvent (entity/player.rs drop_held_item), PlayerItemConsumeEvent (entity/living.rs food consumption)
- **Core/WorldGen agent** → BlockPlaceEvent (world/mod.rs set_block), BlockCanBuildEvent (world/mod.rs place validation), ChunkLoad/ChunkSave/ChunkSend (world/mod.rs chunk lifecycle)
- **Redstone/Block agent** → BlockBurnEvent (fire spread), BlockPhysicsEvent (neighbor updates), BlockFromToEvent (liquid flow), BlockGrowEvent (crop ticks), BlockFadeEvent (ice/snow decay)

**Coverage by category:** Server 7/7 (100%), Entity 4/4 (100%), Player 10/14 (71%), Block 4/11 (36%), World 0/3 (0%)
**Rationale:** Plugin agent has maximized coverage within ARCH-023 scope. Remaining events require cross-agent coordination.
**Affects:** All agents
**Status:** active — handovers needed
