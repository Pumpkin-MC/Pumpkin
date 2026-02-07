# Redstone — Decisions

## RED-001: Wire neighbor update uses vanilla update order
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/001_redstone_signal-propagation.md
**Decision:** `update_wire_neighbors` uses `BlockDirection::update_order()` (W,E,D,U,N,S) for both outer and inner loops, not `BlockDirection::all()`.
**Rationale:** Technical redstone contraptions depend on the specific order of neighbor updates. Using `all()` (D,U,N,S,W,E) would break BUD switches and other order-dependent builds.
**Affects:** Redstone
**Status:** active

## RED-002: Dispenser quasi-connectivity matches dropper
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/001_redstone_signal-propagation.md
**Decision:** Dispenser's `on_neighbor_update` checks power at self and one block above, identical to the dropper implementation.
**Rationale:** In vanilla, dispensers and droppers share the same redstone activation logic including quasi-connectivity. Players expect dispensers to activate from power sources above them.
**Affects:** Redstone
**Status:** active

## RED-003: Block event firing deferred until Plugin creates event types
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/002_redstone_component-verification.md
**Decision:** Redstone agent will not create BlockRedstoneEvent/BlockPistonExtend/RetractEvent since event definitions belong in `pumpkin/src/plugin/api/events/block/` which is outside redstone write_paths. Will fire them once Plugin creates them.
**Rationale:** Respect agent ownership boundaries. Plugin agent owns event type definitions.
**Affects:** Redstone, Plugin
**Status:** active — waiting on Plugin
