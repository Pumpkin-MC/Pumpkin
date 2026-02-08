# Session: redstone-001
**Agent:** redstone
**Date:** 2026-02-07
**Task:** Fix vanilla-parity signal propagation issues, add dispenser redstone activation, write unit tests

## Context Loaded
- Read .claude/sessions/2026-02-06/001_architect_gap-analysis.md (gap analysis, redstone ~60%)
- Read .claude/sessions/2026-02-06/002_architect_restructure-sessions.md (session structure)
- Read .claude/sessions/2026-02-06/003_architect_consolidate-claude.md (orchestration under .claude/)
- Read .claude/sessions/2026-02-06/004_protocol_packets-serialization.md (protocol fixes, no redstone impact)
- Read .claude/sessions/decisions/redstone.md (no prior decisions)
- Read .claude/sessions/decisions/architect.md (ARCH-001 through ARCH-007)
- No prior redstone sessions exist — this is the first redstone session

## Preamble
I am the redstone agent. I acknowledge the architect's gap analysis (session 001) which estimates redstone at ~60% complete with gaps in quasi-connectivity verification, signal propagation correctness vs vanilla update order, hopper-redstone interaction, dispenser/dropper integration, sculk sensor, and technical player relied-upon "bugs". The architect's first task recommendation was a vanilla-parity signal propagation test suite.

This session focuses on three concrete vanilla-parity issues identified through codebase analysis:
1. Wire neighbor update order using `all()` instead of vanilla `update_order()`
2. Dispenser missing all redstone activation (no `on_neighbor_update`)
3. Zero test coverage for redstone signal propagation helpers

## What I Did

### 1. Fixed `update_wire_neighbors` update order (mod.rs:34-52)
**Issue:** `update_wire_neighbors` used `BlockDirection::all()` (Down, Up, North, South, West, East) instead of `BlockDirection::update_order()` (West, East, Down, Up, North, South). In vanilla Minecraft, the neighbor update order matters for technical redstone contraptions — some builds depend on which neighbor gets updated first.

**Fix:** Changed both the outer and inner loops to use `BlockDirection::update_order()`.

### 2. Added dispenser redstone activation with quasi-connectivity (dispenser.rs)
**Issue:** The dispenser block only had `on_place` — no `on_neighbor_update`, meaning it completely ignored redstone signals. The dropper (same vanilla behavior pattern) was already fully implemented.

**Fix:** Added `on_neighbor_update` matching the dropper's pattern:
- Checks power at own position AND one block above (quasi-connectivity)
- Schedules 4-tick delay when newly powered
- Manages `triggered` property state
- Uses `BlockFlags::NOTIFY_LISTENERS` for state changes

### 3. Added 28 unit tests for signal propagation helpers
**Tests in `redstone_wire.rs` (23 tests):**
- WireConnection: `is_connected`, `is_none`
- Direction conversion roundtrips: North, South, East, West
- Cardinal wire `is_none` for all 4 directions
- `make_cross`: zero power, max power, power preservation (all 16 levels)
- `is_dot`: all-none, one-side-connected, power-independent
- `is_cross`: all-side, with-Up-connection, power-independent
- Mutual exclusivity of dot and cross
- `is_side_connected`: horizontal directions, vertical-always-false
- Power decay: saturating sub at zero, all 16 levels
- Cross all sides connected, dot no sides connected

**Tests in `mod.rs` (5 tests):**
- `is_diode`: repeater, comparator, non-diode blocks
- Vanilla update order verification (W, E, D, U, N, S)
- `all()` vs `update_order()` contain same elements

All 28 tests pass.

## What I Changed
- `pumpkin/src/block/blocks/redstone/mod.rs` — Fixed update order in `update_wire_neighbors`, added 5 unit tests
- `pumpkin/src/block/blocks/redstone/dispenser.rs` — Added `on_neighbor_update` with quasi-connectivity
- `pumpkin/src/block/blocks/redstone/redstone_wire.rs` — Added 23 unit tests

## Perspectives Consulted
- **Architect**: Gap analysis identified signal propagation correctness and quasi-connectivity as key unknowns. This session addresses both.

## Vanilla Parity Notes
- **Update order (W,E,D,U,N,S):** Matches vanilla exactly. Some redstone clocks and BUD switches depend on this order.
- **Quasi-connectivity in dispenser:** Intentionally replicates the vanilla "bug" where dispensers/droppers/pistons check power one block above themselves. Players use this extensively for hidden flush piston doors, dispensing systems, etc.
- **Quasi-connectivity in piston (should_extend):** Already implemented correctly — checks all 6 directions at self, then all directions one block above. No changes needed.
- **Quasi-connectivity in dropper:** Already implemented correctly — checks `block_receives_redstone_power` at self and at `position.up()`. No changes needed.

## What I Need From Others
- **Core**: The dispenser's `on_scheduled_tick` (actual dispensing behavior) needs a `DispenserBlockEntity` in `pumpkin-world/src/block/entities/`. This is outside my write paths — I can add the redstone triggering, but the actual dispensing needs Entity/Core support.
- **Architect**: No `pumpkin-redstone` test crate exists yet. The contract references `cargo test -p pumpkin-redstone` but tests are currently inline `#[cfg(test)]` modules. Should a separate test crate be created, or should the contract be updated?

## What Others Should Know
- The existing turbo wire propagation system (theosib's accelerator) is correct and well-implemented. The only issue was the non-turbo path's neighbor update order.
- All existing redstone component implementations are sound — repeater, comparator, observer, torch, lever, buttons, pressure plates, rails, piston, dropper, copper bulb, target block, redstone block, redstone lamp, tripwire. No functional bugs found.
- The `set_block_state` with `BlockFlags::empty()` in redstone wire is intentional — the turbo system handles neighbor propagation, and client updates go through `unsent_block_changes`.

## Decisions Made

### RED-001: Wire neighbor update uses vanilla update order
**Decision:** `update_wire_neighbors` uses `BlockDirection::update_order()` (W,E,D,U,N,S) for both outer and inner loops.
**Rationale:** Technical redstone contraptions depend on the specific order of neighbor updates. Using `all()` (D,U,N,S,W,E) would break BUD switches and other order-dependent builds.
**Affects:** Redstone

### RED-002: Dispenser quasi-connectivity matches dropper
**Decision:** Dispenser's `on_neighbor_update` checks power at self and one block above, identical to the dropper implementation.
**Rationale:** In vanilla, dispensers and droppers share the same redstone activation logic including quasi-connectivity. Players expect dispensers to activate from power sources above them.
**Affects:** Redstone

## Tests
- `cargo test -p pumpkin --lib block::blocks::redstone` — 28 tests pass, 0 failures
- `cargo check -p pumpkin` — full binary crate compiles successfully

## Open Questions
1. **Sculk sensor:** Not implemented. Is this a priority for this phase?
2. **Hopper-redstone interaction:** Hoppers lock when receiving redstone power. Not yet implemented in the hopper block. Requires changes outside redstone write_paths.
3. **Comparator container reading:** The comparator's container reading works for existing block entities but may be incomplete for all vanilla containers.
4. **TNT activation:** TNT should be ignited by redstone. Needs verification if implemented.
