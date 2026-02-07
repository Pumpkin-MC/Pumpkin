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

## ENT-004: Mob Variants Delegate to Parent Entity
**Date:** 2026-02-07
**Session:** entity-006
**Decision:** Mob variants (CaveSpider, Husk, Stray) wrap their parent entity type and delegate `get_mob_entity()` rather than duplicating AI setup code.
**Rationale:** Ensures AI changes to parent mobs (Spider, Zombie, Skeleton) automatically propagate to their variants. Reduces code duplication and maintenance burden.

## ENT-005: Navigator::is_idle() Fix
**Date:** 2026-02-07
**Session:** entity-006
**Decision:** `Navigator::is_idle()` returns `self.current_goal.is_none()` instead of hardcoded `false`. Per ARCH-008 authorization.
**Rationale:** The existing hardcoded `false` broke goal lifecycle — movement goals could never detect arrival. This is the minimal correct fix: idle when no goal is active.

## ENT-006: DamageType Static Reference Cache
**Date:** 2026-02-07
**Session:** entity-007
**Decision:** Use a `[OnceLock<DamageType>; 256]` array in `living.rs` to convert owned `DamageType` values to `&'static DamageType` references for plugin events. Lock-free, bounded at 256 entries.
**Rationale:** Plugin events (EntityDamageEvent, EntityDamageByEntityEvent) require `&'static DamageType` but `damage_with_context()` receives an owned `DamageType`. The OnceLock array provides O(1) lookup without mutex contention or memory leaks.

## ENT-007: EntitySpawnEvent Requires Cross-Agent Coordination
**Date:** 2026-02-07
**Session:** entity-007
**Decision:** EntitySpawnEvent cannot be wired from entity code. Entity spawning occurs in `world/mod.rs` which is outside entity agent's write_paths. Documented as a cross-agent need for the world/core agent.
**Rationale:** The entity agent's contract restricts writes to `pumpkin/src/entity/`. The spawn code path lives in `pumpkin/src/world/mod.rs`.
