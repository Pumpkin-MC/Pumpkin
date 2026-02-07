# Session 003 — Redstone Event Wiring

**Agent:** redstone
**Date:** 2026-02-07
**Branch:** `claude/redstone-signal-propagation-QKEoc`

## Preamble

Read all session logs for today (001-009 across all agents). Read decisions for redstone and architect. Key context:
- Plugin agent (session 003, 008) created BlockRedstoneEvent, BlockPistonExtendEvent, BlockPistonRetractEvent event structs
- ARCH-023 grants cross-agent event-firing write access
- RED-003 (from session 002) deferred event firing until Plugin created the types — now resolved

## What Changed

### 1. BlockRedstoneEvent wired into redstone_wire.rs
- **File:** `pumpkin/src/block/blocks/redstone/redstone_wire.rs:153-191`
- In `on_neighbor_update`, when wire power changes (old != new), fires `BlockRedstoneEvent`
- If event is cancelled by a plugin, the power change is skipped (early return)
- Uses `&Block::REDSTONE_WIRE` (already `&'static Block`)
- Accesses plugin manager via `args.world.server.upgrade()`

### 2. BlockPistonExtendEvent wired into piston.rs
- **File:** `pumpkin/src/block/blocks/piston/piston.rs:158-171`
- In `on_synced_block_event` type 0 (extend) path, fires event before `move_piston`
- If cancelled, returns false (piston does not extend)
- Uses `Block::from_state_id(state.id)` for `&'static Block` (avoids lifetime error with `&'a Block`)

### 3. BlockPistonRetractEvent wired into piston.rs
- **File:** `pumpkin/src/block/blocks/piston/piston.rs:198-210`
- In `on_synced_block_event` retract path, fires event before retraction logic
- If cancelled, returns false (piston does not retract)
- Same `Block::from_state_id(state.id)` pattern

### 4. Updated bukkit_api.toml registry
- Marked BlockRedstoneEvent, BlockPistonExtendEvent, BlockPistonRetractEvent as `status = "implemented"`
- Updated counts: 31 implemented (+3), 249 missing (-3), redstone backlog 10 (-3)

## Decisions Made

- **RED-004:** Block events wired via ARCH-023 cross-agent access. Supersedes RED-003.

## Technical Notes

- **Lifetime pattern:** Event structs need `&'static Block`. Block method args provide `&'a Block` (non-static). Fix: `Block::from_state_id(state.id)` returns `&'static Self` from the static registry. This is the canonical pattern for passing blocks to events.
- **Cancellation semantics:** Cancelled redstone event = power change skipped. Cancelled piston event = piston doesn't move. Both match Bukkit behavior.
- **Bukkit divergence:** Bukkit's `BlockRedstoneEvent` is NOT cancellable; Pumpkin's implementation IS cancellable (as specified by Plugin agent's struct). This gives plugin authors more control than Bukkit.

## Tests

All 58 tests pass (45 redstone + 13 piston). Build passes clean.

## Remaining Redstone Event Backlog (10 events)

| Event | Priority | Notes |
|-------|----------|-------|
| BlockPhysicsEvent | high | Fires on any block update — broad scope |
| BlockPistonEvent | - | Abstract base class, may not need impl |
| BellResonateEvent | - | Bell vibration detection |
| BellRingEvent | - | Bell activation |
| BlockDispenseEvent | - | Needs DispenserBlockEntity (Core/Entity) |
| BlockDispenseArmorEvent | - | Needs DispenserBlockEntity |
| BlockReceiveGameEvent | - | Sculk sensor/warden game event |
| NotePlayEvent | - | Note block plays |
| SculkBloomEvent | - | Sculk catalyst |
| TNTPrimeEvent | - | TNT ignition |

## What Others Should Know

- **Plugin:** Your 3 event structs are now actively fired from redstone/piston code paths. The `cancelled` field is checked and respected.
- **All agents:** Pattern for getting `&'static Block` from block args: `Block::from_state_id(state.id)` — use this when constructing events.

## Perspectives Consulted
- **Plugin:** Event struct definitions (read block_redstone.rs, block_piston_extend.rs, block_piston_retract.rs)

## Vanilla Parity Notes
- Bukkit's BlockRedstoneEvent is non-cancellable; ours is cancellable (extra feature, not a parity issue)
- Piston extend/retract cancellation matches Bukkit behavior
