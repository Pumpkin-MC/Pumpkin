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
**Superseded by:** ENT-008 (ARCH-023 granted cross-agent access)

## ENT-008: EntitySpawnEvent Wired in Both Spawn Paths
**Date:** 2026-02-07
**Session:** entity-007 (continued)
**Decision:** EntitySpawnEvent fires in both `World::spawn_entity()` (world/mod.rs) and the natural spawner batch path (world/natural_spawner.rs). Per ARCH-023 cross-agent write authorization.
**Rationale:** Natural spawner uses its own batch spawn logic (doesn't call `spawn_entity()`), so both paths need the event. Single `fire()` call per entity, no logic changes beyond the event check — per ARCH-023 rules.

## ENT-009: Mass Mob Implementation Source of Truth
**Date:** 2026-02-07
**Session:** entity-008
**Decision:** `.claude/registry/entities.toml` is the master checklist for mob implementation. New mobs are added by following the standard mob pattern and registering in type.rs.
**Rationale:** Single source of truth prevents divergence between registry and implementation.

## ENT-010: MeleeAttackGoal for Non-Zombie Hostile Mobs
**Date:** 2026-02-07
**Session:** entity-009
**Decision:** Made `melee_attack` module public. Non-zombie hostile mobs use `MeleeAttackGoal` directly at priority 2.
**Rationale:** `ZombieAttackGoal` wraps `MeleeAttackGoal` with zombie-specific behavior. Other melee hostiles should use the base goal directly.

## ENT-011: TemptGoal Uses Static Slice Item Lists
**Date:** 2026-02-07
**Session:** entity-009
**Decision:** `TemptGoal` accepts `&'static [u16]` item ID slices. 16 static lists defined in `tempt.rs` for vanilla-accurate food following.
**Rationale:** Static slices avoid allocation and are zero-cost. Item IDs from `pumpkin_data::item::Item` constants are compile-time known.

## ENT-012: Arc<Player> Lookup for &self Contexts
**Date:** 2026-02-07
**Session:** entity-010
**Decision:** Use `world.players.load().iter().find(|p| p.entity_id() == self.entity_id()).cloned()` to get `Arc<Player>` from `&self` in event fire sites.
**Rationale:** Avoids changing `EntityBase` trait signatures. Plugin events require `Arc<Player>` but `handle_killed`/`drop_held_item` only have `&self`.

## ENT-013: BreedGoal Priority 4, FollowParentGoal Priority 5
**Date:** 2026-02-08
**Session:** entity-011
**Decision:** BreedGoal at priority 4, FollowParentGoal at priority 5. Full layout: Swim=0, Panic=1, Attack=2, Tempt=3, Breed=4, FollowParent=5, Wander=6, LookAt=7, LookAround=8.
**Rationale:** Breed has higher priority than FollowParent because mating behavior should override baby following. Both sit between Tempt (player interaction) and Wander (idle movement).

## ENT-014: BreedGoal Stores State Internally
**Date:** 2026-02-08
**Session:** entity-011
**Decision:** BreedGoal stores `love_ticks` and `breed_cooldown` in the goal struct itself, not on MobEntity. External systems call `set_in_love()` on the goal directly.
**Rationale:** Avoids modifying the shared MobEntity struct (ARCH-011). Goal-local state is sufficient since the GoalSelector owns the goal instances.

## ENT-015: RangedAttackGoal Priority Varies by Mob
**Date:** 2026-02-08
**Session:** entity-012
**Decision:** RangedAttackGoal priority is mob-dependent: Witch=2, Drowned=2, Pillager=3, Skeleton/Blaze/Ghast=4. Range varies from 8.0 (Pillager) to 64.0 (Ghast). Attack intervals: 20-60 ticks.
**Rationale:** Matches vanilla parameter differences. Witch/Drowned are primary ranged attackers (low priority number = high importance). Skeleton/Blaze/Ghast have flee/other behaviors that should take precedence.

## ENT-016: FleeEntityGoal Priority 1-3 for Vanilla Flee Relationships
**Date:** 2026-02-08
**Session:** entity-012
**Decision:** Wired 5 vanilla-accurate flee relationships: Creeper→Cat(p1), Phantom→Cat(p1), Rabbit→Wolf(p2), Fox→Wolf(p2), Skeleton→Wolf(p3). Flee priority is always above attack priority for the same mob.
**Rationale:** In vanilla, flee behavior overrides attack behavior. A creeper will always run from a cat rather than try to explode near it. Skeleton flees wolves at p3 (above its p4 ranged attack).

## ENT-017: FollowOwnerGoal at Priority 6 for Tamed Mobs
**Date:** 2026-02-08
**Session:** entity-013
**Decision:** FollowOwnerGoal stores `owner_id: Option<i32>` in the goal struct (ENT-014 pattern). Priority 6 — after FollowParent(5), before Wander(7). Teleports when >12 blocks, follows when >10 blocks, stops when <2 blocks. Wired to Wolf and Cat.
**Rationale:** MobEntity has no owner/tamed fields (ARCH-011). Storing owner state in the goal itself follows the BreedGoal pattern. External code calls `set_owner()` when taming occurs.

## ENT-018: Only Llama and Bee Need Breeding Goals Among Remaining Passives
**Date:** 2026-02-08
**Session:** entity-015
**Decision:** Among remaining passive mobs audited (Llama, Bee, Ocelot, Parrot, Polar Bear, Dolphin, Allay), only Llama and Bee are breedable in vanilla. Ocelot lost breeding in 1.14 (trust-only). Parrot, Polar Bear, Dolphin, and Allay are not breedable.
**Rationale:** Vanilla 1.21 game mechanics. Only breedable mobs should have BreedGoal/FollowParentGoal. Ocelot has TemptGoal for trust-building but no breed. Also added FleeEntityGoal(Ocelot) to Creeper/Phantom since vanilla mobs flee both Cat and Ocelot.
