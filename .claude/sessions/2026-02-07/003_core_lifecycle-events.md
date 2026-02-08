# Session: core-003
**Agent:** core
**Date:** 2026-02-07
**Task:** Wire server lifecycle events (ServerTickEvent, ServerStartedEvent, ServerStopEvent) into game loop

## Context Loaded
- Read all session logs from 2026-02-07 (001_redstone, 001_storage, 002_core, 002_items, 003_plugin, 004_architect, 006_entity)
- Read .claude/sessions/decisions/core.md (CORE-001 through CORE-003)
- Read .claude/sessions/decisions/architect.md (ARCH-001 through ARCH-015)
- Read .claude/prompts/core.md (explicit instructions to fire lifecycle events)
- This is the second core session (first was 002_core_decomposition-plan-and-profiler)

## Preamble

I am the Core agent. I acknowledge the Plugin agent's session 003 which defined 3 server lifecycle events and explicitly requested Core to fire them:
- `ServerTickEvent` — fire each tick (PLUGIN-003: not cancellable)
- `ServerStartedEvent` — fire after bootstrap completes
- `ServerStopEvent` — fire when shutdown begins

I also acknowledge:
- ARCH-015: Payload::is_cancelled() now available via Event derive macro
- Entity session 006: Navigator::is_idle() fixed (ENT-005), 9 more mobs added
- Items session 002: Stonecutter/Smithing screen handlers, 61 tests
- Architect session 004: Recipe codegen unblocking Items

The core.md prompt explicitly marks firing lifecycle events as the "#1 cross-agent blocker."

## What I Did

### 1. Wired ServerTickEvent into Server::tick() (server/mod.rs)

Added `plugin_manager.fire(ServerTickEvent::new(tick_count))` at the end of `Server::tick()`, after profiler recording. The tick_count is read from `self.tick_count` (AtomicI32) and cast to i64 to match the event's field type.

The event fires every tick regardless of freeze state — when frozen, it fires after `tick_players_and_network()`; when running normally or sprinting, it fires after `tick_worlds()`. This matches Bukkit's behavior where ServerTickEvent fires even during /tick freeze.

### 2. Added fire_started_event() to PumpkinServer (lib.rs)

Created `PumpkinServer::fire_started_event()` which:
- Gets world count from `server.worlds.load().len()`
- Gets plugin count from `server.plugin_manager.loaded_plugins().await.len()`
- Fires `ServerStartedEvent::new(world_count, plugin_count)`

Called from main.rs after `init_plugins()`, before the "Started server" log message. This ensures plugins are loaded before the event fires, so plugin_count is accurate.

### 3. Wired ServerStopEvent into shutdown sequence (lib.rs)

Added `plugin_manager.fire(ServerStopEvent::new("Server shutting down"))` at the beginning of the shutdown sequence in `PumpkinServer::start()`, immediately after "Stopped accepting incoming connections". This fires before:
- Player data is saved
- Players are kicked
- Player tasks end
- Plugins are unloaded
- Worlds are saved

This ordering lets plugins react to the stop event while players are still connected and the world is still accessible.

## What I Changed

### Modified Files
- `pumpkin/src/server/mod.rs` — added ServerTickEvent firing in `tick()`
- `pumpkin/src/lib.rs` — added `fire_started_event()` method, added ServerStopEvent firing in shutdown
- `pumpkin/src/main.rs` — added `pumpkin_server.fire_started_event().await` call after init_plugins

## Perspectives Consulted
- **PluginAPI Consultant**: Verified all 3 events are NOT cancellable (PLUGIN-003). Fire-and-forget pattern is correct — no need to check return value or cancellation state.
- **Protocol Consultant**: Not needed — events are internal plugin API.

## What I Need From Others
- **Plugin**: The 3 server lifecycle events are now wired. Please verify the firing order meets plugin expectations.
- **Entity**: The 4 entity events (spawn, damage, damage_by_entity, death) are still not wired. Entity agent should fire them from the appropriate lifecycle points.

## What Others Should Know
- **ServerTickEvent fires every tick** including during freeze. The tick_count reflects the total ticks processed by the server.
- **ServerStartedEvent fires once** after plugin init, before the server starts accepting connections.
- **ServerStopEvent fires once** at the beginning of shutdown, while players are still connected. Plugins have a window to react before players are kicked.
- All 3 events are non-cancellable. The `fire()` call is fire-and-forget.

## Decisions Made

### CORE-004: ServerTickEvent fires after profiler recording
**Date:** 2026-02-07
**Decision:** ServerTickEvent fires at the end of Server::tick(), after tick profiler recording but still within the tick method.
**Rationale:** Plugins get the most accurate tick count. Firing after profiler ensures the profiler measures actual game logic, not plugin event handling overhead. Plugin overhead will be captured in the next tick's total measurement.
**Affects:** Core, Plugin
**Status:** active

### CORE-005: ServerStopEvent fires before player disconnect
**Date:** 2026-02-07
**Decision:** ServerStopEvent fires immediately after connection acceptance stops, before player data save, player kicks, plugin unload, or world save.
**Rationale:** Matches Bukkit's ServerStopEvent which fires while the server is still operational. Plugins may want to send final messages, save data, or clean up while players are still connected and the world is accessible.
**Affects:** Core, Plugin
**Status:** active

## Tests
- `cargo test -p pumpkin` — **91 tests pass**, 0 failures
- `cargo check -p pumpkin` — compiles cleanly, 0 errors

## Open Questions
1. Should ServerTickEvent also fire during tick sprints? Currently it does — every call to `Server::tick()` fires the event. During sprints, this means many events fire with zero sleep between them. Should we throttle?
2. The entity events (EntitySpawnEvent, EntityDamageEvent, etc.) still need wiring by the Entity agent.
