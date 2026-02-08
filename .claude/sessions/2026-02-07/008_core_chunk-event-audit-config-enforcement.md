# Session: core-008
**Agent:** core
**Date:** 2026-02-07
**Task:** Chunk event wiring audit, force_gamemode enforcement, config additions

## Context Loaded
- Rebased onto origin/master (36ce37b) — up to date
- Read all sessions from 2026-02-07 (001-010)
- Read decisions/core.md (CORE-001 through CORE-011)
- Read decisions/architect.md (ARCH-001 through ARCH-023+)
- Read plugin session 009 (event firing coverage audit — 25/39 → 64%)
- Read architect session 010 (status update — Core at ~70%)
- Items agent reported updated coverage: 30/39 events fired (77%)

## Preamble

I am the Core agent. I acknowledge:
- Plugin session 009 requests Core/WorldGen wire 5 events: BlockPlaceEvent, BlockCanBuildEvent, ChunkLoad, ChunkSave, ChunkSend
- Entity agent wired 5 more events since plugin audit (PlayerDeathEvent, PlayerRespawnEvent, PlayerDropItemEvent, PlayerItemConsumeEvent + EntitySpawnEvent), bringing coverage to 30/39
- Redstone session 005 wired BlockPhysicsEvent
- ARCH-023 grants Core write access to world/mod.rs for lifecycle events only

## What I Did

### 1. Chunk Event Wiring Audit (CORE-012)

**BLOCKED** — comprehensive audit of all 9 remaining unfired events:

| Event | Hook Point | Core Accessible? | Blocker |
|---|---|---|---|
| ChunkLoad | pumpkin-world level.rs `get_chunk()` | NO (FORBIDDEN) | Type mismatch + forbidden path |
| ChunkSave | pumpkin-world level.rs `write_chunks()` | NO (FORBIDDEN) | Type mismatch + forbidden path |
| ChunkSend | entity/player.rs (main) + world/mod.rs (teleport) | PARTIAL | Type mismatch |
| BlockPlaceEvent | net/java/play.rs | NO (FORBIDDEN) | Not Core scope |
| BlockCanBuildEvent | net/java/play.rs | NO (FORBIDDEN) | Not Core scope |
| BlockBurnEvent | block logic | NO (FORBIDDEN) | Redstone scope |
| BlockFromToEvent | block logic | NO (FORBIDDEN) | Redstone scope |
| BlockGrowEvent | block logic | NO (FORBIDDEN) | Redstone scope |
| BlockFadeEvent | block logic | NO (FORBIDDEN) | Redstone scope |

**Critical type mismatch:** All 3 chunk events (ChunkLoad, ChunkSend, ChunkSave) are defined with `Arc<RwLock<ChunkData>>` but the entire Pumpkin runtime uses `Arc<ChunkData>` (= `SyncChunk`). `ChunkData` does not implement `Clone`. This makes it impossible to construct the event struct from runtime chunk data without either:
- (a) Changing the event struct to use `Arc<ChunkData>` (Plugin scope)
- (b) Adding Clone to ChunkData (pumpkin-world scope)

**Recommendation:** Plugin agent should change chunk event structs from `Arc<RwLock<ChunkData>>` to `Arc<ChunkData>` to match the actual runtime type. Then Entity/WorldGen agents can wire ChunkSend in entity/player.rs, and WorldGen can wire ChunkLoad/ChunkSave in pumpkin-world.

### 2. Enforced `force_gamemode` on Player Login (CORE-013)

Added enforcement in `server/mod.rs` `add_player()`. After `read_nbt()` loads the player's saved gamemode, if `force_gamemode` is true, the gamemode is reset to the server default.

This matches vanilla behavior: when `force-gamemode=true`, players always join in the server's default gamemode regardless of their saved gamemode.

```rust
// After read_nbt (which may override gamemode from saved data):
if self.basic_config.force_gamemode {
    player.gamemode.store(gamemode);
}
```

### 3. Added 3 Missing Config Fields (CORE-014)

| Field | Type | Default | Vanilla Equivalent |
|---|---|---|---|
| `broadcast_console_to_ops` | bool | true | broadcast-console-to-ops |
| `max_world_size` | u32 | 29,999,984 | max-world-size |
| `function_permission_level` | PermissionLvl | Two | function-permission-level |

All defaults match vanilla. Fields are declared in config but not yet consumed by runtime code, following CORE-006 pattern.

## What I Changed

### Modified Files
- `pumpkin/src/server/mod.rs` — added `force_gamemode` enforcement in `add_player()`
- `pumpkin-config/src/lib.rs` — added 3 config fields to BasicConfiguration struct and Default impl

### Session Files
- `.claude/sessions/2026-02-07/008_core_chunk-event-audit-config-enforcement.md` — this file
- `.claude/sessions/decisions/core.md` — CORE-012, CORE-013, CORE-014

## Decisions Made

### CORE-012: Chunk events blocked by type mismatch
**Date:** 2026-02-07
**Decision:** ChunkLoad/ChunkSend/ChunkSave events cannot be wired from Core or any agent until the Plugin agent fixes the event struct type from `Arc<RwLock<ChunkData>>` to `Arc<ChunkData>`.
**Rationale:** The entire Pumpkin runtime uses `Arc<ChunkData>` (= SyncChunk). ChunkData doesn't implement Clone. There is no way to construct the current event structs from runtime data without copying or redesigning. This is a Plugin agent event design issue.
**Affects:** Core, WorldGen, Entity, Plugin
**Status:** active (blocker)

### CORE-013: force_gamemode enforced on player login
**Date:** 2026-02-07
**Decision:** After player NBT data is loaded in `add_player()`, if `force_gamemode` is true, the player's gamemode is reset to the server default. This runs before the player is wrapped in Arc.
**Rationale:** Matches vanilla `force-gamemode` behavior. The existing code only enforced force_gamemode when `/defaultgamemode` was changed, not on login.
**Affects:** Core
**Status:** active

### CORE-014: Three more config fields added
**Date:** 2026-02-07
**Decision:** Added `broadcast_console_to_ops`, `max_world_size`, `function_permission_level` to BasicConfiguration following CORE-006 pattern (declare before enforce).
**Rationale:** These are commonly used vanilla server.properties fields. broadcast_console_to_ops affects how console commands are relayed to ops. max_world_size will be consumed by world border logic. function_permission_level will be consumed when /function command is implemented.
**Affects:** Core
**Status:** active

## What Others Should Know

- **Plugin agent**: Chunk event structs use `Arc<RwLock<ChunkData>>` but runtime uses `Arc<ChunkData>`. Please change the event structs to use `Arc<ChunkData>`. Without this fix, no agent can wire ChunkLoad/ChunkSend/ChunkSave.
- **BlockPlaceEvent/BlockCanBuildEvent**: Hook points are in `net/java/play.rs`. Protocol or Plugin agent should wire these per ARCH-023.
- **BlockBurnEvent/BlockFromToEvent/BlockGrowEvent/BlockFadeEvent**: Hook points are in block tick logic. Redstone agent scope.
- **3 new config fields available**: `broadcast_console_to_ops`, `max_world_size`, `function_permission_level`.
- **`force_gamemode` now enforced on login**: Players joining with `force_gamemode=true` will have their gamemode set to the server default.

## What I Need From Others

- **Plugin agent**: Fix chunk event struct types from `Arc<RwLock<ChunkData>>` to `Arc<ChunkData>` (CORE-012 blocker)
- **Architect**: Confirm who should wire BlockPlaceEvent/BlockCanBuildEvent — these are in net/java/play.rs

## Tests
- `cargo test -p pumpkin` — **135 tests pass**, 0 failures
- `cargo check -p pumpkin-config -p pumpkin` — compiles cleanly with RUSTFLAGS="-Dwarnings"
- Pre-existing clippy errors in pumpkin-protocol/ (doc_markdown for BungeeCord) — not Core scope

## Cumulative Stats
- Sessions: 002-008 (7 sessions)
- Total tests: 135 passing
- Commands implemented: save-all, save-off, save-on, debug, perf (5 new commands)
- Config fields added: 7 total (allow_flight, spawn_protection, generate_structures, player_idle_timeout, broadcast_console_to_ops, max_world_size, function_permission_level)
- Game rules enforced: log_admin_commands (dispatcher level)
- Login enforcement: force_gamemode
- Event coverage contribution: ServerStartedEvent, ServerStopEvent, ServerTickEvent (3 lifecycle events)
