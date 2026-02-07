# Entity — Decisions

*Append-only. Never delete; only supersede with rationale.*

## ENT-001: AI Goal Priority Convention
**Date:** 2026-02-06
**Session:** entity-005
**Decision:** AI goal priorities follow vanilla Minecraft conventions. Lower number = higher priority.
Standard scale: Swim=0, Panic=1, Attack=2, Special=4-5, Wander=6, LookAt=7, LookAround=8.
**Rationale:** Matches vanilla behavior and ensures consistent mob AI across all entity implementations.

## ENT-002: WanderAroundGoal Parameters
**Date:** 2026-02-06
**Session:** entity-005
**Decision:** WanderAroundGoal uses 8-block horizontal radius, 1/120 tick chance by default.
**Rationale:** Simplified version of vanilla WanderAroundFarGoal. Provides natural-looking idle movement without overloading pathfinding.

## ENT-003: PanicGoal Damage Detection via Health Comparison
**Date:** 2026-02-06
**Session:** entity-005
**Decision:** PanicGoal detects damage by comparing current health to last-known health each tick, rather than relying on a damage event callback.
**Rationale:** The Goal trait's `can_start()` method doesn't receive damage events. Health comparison is a pragmatic approach that works within the existing goal system.
