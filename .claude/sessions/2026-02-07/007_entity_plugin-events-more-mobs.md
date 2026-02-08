# Session 007 — Entity: Plugin Events + More Mobs

**Agent:** entity
**Date:** 2026-02-07
**Branch:** claude/entity-spawning-ai-V7oqj

## Preamble

Read all session logs from 2026-02-06 and 2026-02-07. Read CLAUDE.md and .claude/prompts/entity.md.
Rebased to latest master. Hydrated blackboard (entity at 30%).

Previous sessions (005, 006) established 16 hostile mobs and 10 passive mobs, plus 4 AI goals and
the Navigator::is_idle() fix. This session focuses on Priority #1 (plugin events) and Priority #2 (more mobs).

ARCH-023 granted cross-agent write access to `world/mod.rs` and `world/natural_spawner.rs` for
EntitySpawnEvent wiring. Used this authorization to complete all 4 plugin events.

## What I Did

### 1. Wired All 4 Plugin Entity Events into Entity Lifecycle

**EntityDamageEvent + EntityDamageByEntityEvent** — wired into `LivingEntity::damage_with_context()` in `living.rs`:
- Fire `EntityDamageByEntityEvent` first (when source entity exists), then `EntityDamageEvent` for all damage
- If either event is cancelled, damage is not applied (return false)
- Added `damage_type_static()` helper to convert owned `DamageType` to `&'static DamageType` via lock-free `OnceLock<DamageType>` cache (max 256 entries indexed by damage type id)

**EntityDeathEvent** — wired into `LivingEntity::on_death()` in `living.rs`:
- Fire before death processing; if cancelled, skip death entirely

**EntitySpawnEvent** — wired in TWO locations per ARCH-023:
- `World::spawn_entity()` in `world/mod.rs`: Fire before spawn packet broadcast; if cancelled, return early. Changed signature from `&self` to `self: &Arc<Self>` (all callers already use `Arc<Self>`).
- Natural spawner batch spawn in `world/natural_spawner.rs`: Fire for each entity in batch before preparation; filter out cancelled entities.

### 2. Created 9 More Mob Entities (Total: 34 registered)

**Hostile mobs (3):**
- `phantom.rs` — Swim, Wander, LookAt, LookAround + player targeting (swooping TBD)
- `endermite.rs` — Wander, LookAt, LookAround + player targeting
- `magma_cube.rs` — Swim, Wander, LookAt + player targeting

**Passive mobs (6):**
- `dolphin.rs` — Swim, Panic(1.6x), Wander, LookAt, LookAround
- `fox.rs` — Swim, Panic(2.0x), Wander, LookAt(8.0), LookAround
- `bee.rs` — Swim, Panic(2.0x), Wander, LookAt, LookAround
- `goat.rs` — Swim, Panic(2.0x), Wander, LookAt, LookAround
- `frog.rs` — Swim, Panic(2.0x), Wander, LookAt, LookAround
- `cat.rs` — Swim, Panic(2.0x), Wander(0.8), LookAt(10.0), LookAround

### 3. Updated Registration

- Updated `mob/mod.rs` with module declarations for phantom, endermite, magma_cube
- Updated `passive/mod.rs` with module declarations for dolphin, fox, bee, goat, frog, cat
- Updated `type.rs` with imports and match arms for all 9 new entities

## Files Modified
- `pumpkin/src/entity/living.rs` — plugin event wiring + damage_type_static helper
- `pumpkin/src/entity/type.rs` — imports + 9 new match arms
- `pumpkin/src/entity/mob/mod.rs` — 3 new module declarations
- `pumpkin/src/entity/passive/mod.rs` — 6 new module declarations
- `pumpkin/src/world/mod.rs` — EntitySpawnEvent in spawn_entity() [ARCH-023]
- `pumpkin/src/world/natural_spawner.rs` — EntitySpawnEvent in batch spawn [ARCH-023]

## Files Created
- `pumpkin/src/entity/mob/phantom.rs`
- `pumpkin/src/entity/mob/endermite.rs`
- `pumpkin/src/entity/mob/magma_cube.rs`
- `pumpkin/src/entity/passive/dolphin.rs`
- `pumpkin/src/entity/passive/fox.rs`
- `pumpkin/src/entity/passive/bee.rs`
- `pumpkin/src/entity/passive/goat.rs`
- `pumpkin/src/entity/passive/frog.rs`
- `pumpkin/src/entity/passive/cat.rs`

## Decisions Made
- ENT-006: Use OnceLock array for DamageType static reference cache
- ENT-007: EntitySpawnEvent requires cross-agent coordination → resolved by ARCH-023
- ENT-008: EntitySpawnEvent fires in both spawn_entity() and natural_spawner batch path

## What Others Should Know
- All 4 plugin entity events (spawn, damage, damage-by-entity, death) are now wired and cancellable.
- 34 entity types are now registered in the factory (up from 25 at session start).
- `World::spawn_entity()` signature changed from `&self` to `self: &Arc<Self>` — all existing callers already use `Arc<Self>`.

## What I Need From Others
- **Architect**: Guidance on priority for remaining ~45 entity types toward the 79+ vanilla target

## Build Status
- `cargo build` — clean, no warnings
- clippy errors only in `pumpkin-inventory` (pre-existing, outside entity write_paths)
