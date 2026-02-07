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

## PLUGIN-004: ignore_cancelled filtering POSTPONED — needs Architect
**Date:** 2026-02-07
**Decision:** The `ignore_cancelled` field exists on handler metadata but filtering is NOT enforced in `fire()`. Postponed until Architect updates pumpkin-macros.
**Rationale:**
Bukkit's `@EventHandler(ignoreCancelled = true)` skips a handler if a higher-priority handler already cancelled the event. To implement this in `fire()`, we need to check cancellation state on a generic `E: Payload`. The problem:

1. `Payload` trait has 4 methods (`get_name_static`, `get_name`, `as_any`, `as_any_mut`). Adding `is_cancelled()` as a default method changes the vtable layout for `dyn Payload`, which **breaks binary plugin compatibility** across compilation boundaries. Existing compiled plugins would crash.
2. The `#[derive(Event)]` macro in pumpkin-macros generates the `Payload` impl. The `#[cancellable]` macro adds `cancelled: bool` field + `Cancellable` impl separately. Neither generates an `is_cancelled()` override on `Payload`.
3. We cannot check `E: Cancellable` at runtime in `fire()` because Rust has no "does this concrete type also implement trait X?" query on `dyn Payload`. `TypeId`-based downcasting won't work across compilation boundaries (plugin `.so` files have different `TypeId` values than the host).
4. Specialization (`default fn` + concrete override) is unstable Rust.

**What we tried and reverted:** Adding `fn is_cancelled(&self) -> bool { false }` as a default method to `Payload`. This was reverted because (a) it changes the trait signature which violates ARCH-011 (never rename/modify existing interfaces), and (b) existing `#[derive(Event)]` wouldn't generate the override anyway, so all events would return `false`.

**What needs to happen:** The Architect must update `pumpkin-macros` so that `#[derive(Event)]` generates an `is_cancelled()` method. Two options:
- Option A: The derive macro detects the `cancelled` field and generates `fn is_cancelled(&self) -> bool { self.cancelled }` in the Payload impl. Non-cancellable events get `fn is_cancelled(&self) -> bool { false }`.
- Option B: A new trait `MaybeCancellable` with blanket impls, kept in pumpkin-util so both pumpkin-macros and plugin code can use it.

Until then, `ignore_cancelled` is stored as metadata for forward compatibility but has no runtime effect.
**Affects:** Plugin, Architect (pumpkin-macros)
**Status:** active — blocked on Architect
