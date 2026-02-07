# Session: core-005
**Agent:** core
**Date:** 2026-02-07
**Task:** Implement save-all, save-off, save-on commands; add autosave toggle to Server

## Context Loaded
- Rebased onto origin/master (106c807) — includes ARCH-023 (cross-agent event-firing write access)
- Read all session logs from 2026-02-07 (23 files)
- Read .claude/sessions/decisions/core.md (CORE-001 through CORE-007)
- Read .claude/sessions/decisions/architect.md (ARCH-001 through ARCH-024)
- Read .claude/prompts/core.md
- This is the fourth core session (after 002_decomposition, 003_lifecycle-events, 004_command-config-audit)

## Preamble

I am the Core agent. I acknowledge:
- ARCH-023 grants Core write access to `world/mod.rs` for lifecycle events. No lifecycle events needed wiring from world/mod.rs this session.
- Plugin session 008: 7 block events + 2 player events created. Block events fire from block/ (Redstone/Entity scope). Player events fire from entity/player/ (Entity scope). Neither requires Core action.
- Protocol sessions 003-004: PlayerPosition read impl, PlayerAction type fixes. No Core impact.
- Architect session 009: pumpkin-store crate created (ARCH-020). No Core impact yet (ARCH-024 says Items should not adopt yet).
- Items session 009: 11 special recipes. No Core impact.
- CORE-007 identified save-all/save-off/save-on as simple Core-scope commands — implementing them now.
- Command aliases (tell->msg, w->msg, tp->teleport) already exist in the codebase.

## What I Did

### 1. Added `autosave_enabled` Field to Server (server/mod.rs)

Added `pub autosave_enabled: AtomicBool` field to the Server struct, initialized to `true`. This field controls whether the periodic player data autosave runs during `tick_worlds()`.

Guarded the `player_data_storage.tick()` call in `tick_worlds()` with an `autosave_enabled` check. When `save-off` disables autosave, the periodic player data save is skipped.

### 2. Added `save_all()` Method to Server (server/mod.rs)

Created `Server::save_all(flush: bool)` — a non-destructive save method that:
1. Saves all player data via `player_data_storage.save_all_players()`
2. Triggers chunk saves on all worlds by setting `level.should_save = true` and notifying the chunk system via `level.level_channel.notify()`
3. If `flush` is true, waits for all pending chunk writes to complete via `chunk_saver.block_and_await_ongoing_tasks()`
4. Writes level.dat via `world_info_writer.write_world_info()`

This mirrors the save logic from `shutdown()` without the destructive parts (task closing, thread joining, chunk system shutdown).

### 3. Implemented save-all Command (command/commands/save_all.rs)

- `/save-all` — saves player data, triggers chunk saves, writes level.dat
- `/save-all flush` — same as above but waits for all pending chunk IO to complete
- Uses vanilla translation keys: `commands.save.saving`, `commands.save.success`
- Permission: Level 4 (same as /stop in vanilla)

### 4. Implemented save-off Command (command/commands/save_off.rs)

- `/save-off` — sets `server.autosave_enabled` to false
- Uses vanilla translation key: `commands.save.disabled`
- Permission: Level 4

### 5. Implemented save-on Command (command/commands/save_on.rs)

- `/save-on` — sets `server.autosave_enabled` to true
- Uses vanilla translation key: `commands.save.enabled`
- Permission: Level 4

### 6. Command Aliases Already Exist

Discovered that command aliases were already implemented:
- `msg` has aliases `["msg", "tell", "w"]`
- `teleport` has aliases `["teleport", "tp"]`

No work needed.

## What I Changed

### New Files
- `pumpkin/src/command/commands/save_all.rs` — save-all command implementation
- `pumpkin/src/command/commands/save_off.rs` — save-off command implementation
- `pumpkin/src/command/commands/save_on.rs` — save-on command implementation

### Modified Files
- `pumpkin/src/server/mod.rs` — added `autosave_enabled` field, guarded autosave in `tick_worlds()`, added `save_all()` method
- `pumpkin/src/command/commands/mod.rs` — registered 3 new commands with Level 4 permissions

## Perspectives Consulted
- **WorldGen Consultant**: Verified `level.should_save` (public AtomicBool on Level) triggers `save_all_chunk()` in the chunk system. The `level_channel.notify()` wakes the chunk system thread to process the flag. This is the correct non-destructive way to trigger a chunk save without shutting down the level.
- **PluginAPI Consultant**: Not needed — save commands are admin commands, not plugin events.

## What I Need From Others
- **No blockers.** All save infrastructure was accessible from Core's scope.

## What Others Should Know
- **`server.autosave_enabled`** is a new `pub AtomicBool` on Server. If any agent needs to check or control autosave state (e.g., for a backup plugin), it's available.
- **`server.save_all(flush)`** is a new public async method on Server. It provides a non-destructive save-all that can be called from anywhere that has a `&Server` reference (commands, plugins, etc.).
- **Pre-existing clippy errors** in `pumpkin-protocol/` (3 errors: missing_const_for_fn, doc_markdown). These are from Protocol agent sessions, not Core changes.

## Decisions Made

### CORE-008: save-all uses should_save flag for non-destructive chunk save
**Date:** 2026-02-07
**Decision:** The save-all command triggers chunk saves by setting `level.should_save = true` and notifying the chunk system, rather than calling `Level::shutdown()`. The `flush` variant additionally waits for pending IO to complete.
**Rationale:** `Level::shutdown()` cancels the chunk system and joins threads — it's destructive and can only be called once. The `should_save` flag is the designed mechanism for on-demand saves: it triggers `save_all_chunk()` in the chunk system thread without shutting anything down. This allows save-all to be called repeatedly during normal server operation.
**Affects:** Core
**Status:** active

### CORE-009: save-off/save-on control autosave_enabled on Server
**Date:** 2026-02-07
**Decision:** The save-off/save-on commands toggle `server.autosave_enabled` (AtomicBool). When disabled, `tick_worlds()` skips `player_data_storage.tick()`. Chunk saves on unload still happen (they're managed by the chunk system, not by the tick loop).
**Rationale:** In vanilla, save-off disables periodic autosave, not chunk unload saves. Pumpkin's chunk saves happen on unload (not on a timer), so save-off primarily affects player data autosave. The `/save-all` command always saves regardless of the autosave state, matching vanilla behavior.
**Affects:** Core
**Status:** active

## Tests
- `cargo test -p pumpkin` — **121 tests pass**, 0 failures
- `cargo check -p pumpkin` — compiles cleanly with RUSTFLAGS="-Dwarnings", 0 errors in pumpkin
- Pre-existing clippy errors in pumpkin-protocol (3 errors, not from Core changes)

## Open Questions
1. Should save-all also fire a plugin event (e.g., `WorldSaveEvent`)? Currently it's fire-and-forget with no plugin notification. Low priority — can be added when Plugin agent defines the event.
2. The `entity_saver` on Level is private (`Arc<dyn FileIO<Data = SyncEntityChunk>>`). Save-all cannot flush entity saves directly. Entity saves happen via `world.save_entity()` which is also private to world/. Entity save flushing would need world/ to expose a public method. Not blocking — entities are saved on chunk unload.
