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
