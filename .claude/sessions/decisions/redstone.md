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
**Status:** resolved — superseded by RED-004

## RED-004: Block events wired via ARCH-023 cross-agent access
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/003_redstone_event-wiring.md
**Decision:** Wired BlockRedstoneEvent, BlockPistonExtendEvent, and BlockPistonRetractEvent into their respective code paths. Plugin agent created the event structs; Redstone agent fires them using `server.plugin_manager.fire()`. Used `Block::from_state_id(state.id)` to get `&'static Block` for events requiring static lifetime.
**Rationale:** ARCH-023 grants cross-agent event-firing write access. Plugin created the event types, Redstone wires the fire points.
**Affects:** Redstone, Plugin
**Status:** active

## RED-005: BlockPhysicsEvent wired in update_neighbors
**Date:** 2026-02-07
**Session:** .claude/sessions/2026-02-07/005_redstone_physics-event-wiring.md
**Decision:** BlockPhysicsEvent fires in `world/mod.rs::update_neighbors` before dispatching each neighbor's `on_neighbor_update`. If cancelled, the neighbor update is skipped via `continue`. The `update_neighbor` (single) function does not fire the event because `source_position` is not available in its signature.
**Rationale:** `update_neighbors` is the main multi-direction dispatch that has both source and target positions. Adding `source_position` to `update_neighbor` would require changing all callers across the codebase.
**Affects:** Redstone, Plugin, Core
**Status:** active
