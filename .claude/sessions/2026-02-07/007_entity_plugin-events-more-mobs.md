# Session 007 — Entity: Plugin Events + More Mobs

**Agent:** entity
**Date:** 2026-02-07
**Branch:** claude/entity-spawning-ai-V7oqj

## Preamble

Read all session logs from 2026-02-06 and 2026-02-07. Read CLAUDE.md and .claude/prompts/entity.md.
Rebased to latest master. Hydrated blackboard (entity at 30%).

Previous sessions (005, 006) established 16 hostile mobs and 10 passive mobs, plus 4 AI goals and
the Navigator::is_idle() fix. This session focuses on Priority #1 (plugin events) and Priority #2 (more mobs).

## What I Did

### 1. Wired Plugin Entity Events into Entity Lifecycle

**EntityDamageEvent + EntityDamageByEntityEvent** — wired into `LivingEntity::damage_with_context()` in `living.rs`:
- Fire `EntityDamageByEntityEvent` first (when source entity exists), then `EntityDamageEvent` for all damage
- If either event is cancelled, damage is not applied (return false)
- Added `damage_type_static()` helper to convert owned `DamageType` to `&'static DamageType` via lock-free `OnceLock<DamageType>` cache (max 256 entries indexed by damage type id)

**EntityDeathEvent** — wired into `LivingEntity::on_death()` in `living.rs`:
- Fire before death processing; if cancelled, skip death entirely

**EntitySpawnEvent** — ⚠️ CANNOT wire from entity code. Spawning happens in `world/mod.rs` which is outside entity write_paths. Documented as cross-agent need for world/core agent.

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
- ENT-007: EntitySpawnEvent requires cross-agent coordination

## What Others Should Know
- ⚠️ **World/Core agent**: EntitySpawnEvent needs to be fired in `world/mod.rs` during entity spawning. Entity agent cannot do this (outside write_paths).
- Plugin events (damage, death) are now live — plugins can cancel damage and death.
- 34 entity types are now registered in the factory (up from 25 at session start).

## What I Need From Others
- **World/Core agent**: Wire `EntitySpawnEvent` in `world/mod.rs` spawn_entity path
- **Architect**: If more entity types are needed beyond the current 34, guidance on priority order

## Build Status
- `cargo build` — clean, no warnings
- clippy errors only in `pumpkin-inventory` (pre-existing, outside entity write_paths)
