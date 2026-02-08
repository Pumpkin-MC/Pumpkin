# Session 011 — Entity: BreedGoal + FollowParentGoal

**Agent:** entity
**Date:** 2026-02-08
**Branch:** claude/entity-spawning-ai-V7oqj

## Preamble

- No session logs exist for 2026-02-08 — I'm the first session today.
- Read sessions 007-010 entity logs (plugin events, mass mobs, AI expansion, player events).
- Read architect session 010 (status update, P0 = AI goal expansion).
- Read entity decisions ENT-001 through ENT-012.
- Read architect decisions ARCH-001 through ARCH-031.
- No pending requests from other agents to entity.
- Plugin's 4 player events were completed in Session 010.

## What I Did

### 1. Created BreedGoal (`ai/goal/breed.rs`)
New AI goal for breedable passive mobs. When activated via `set_in_love()`:
- Searches for nearby entity of same `entity_type` within 8 blocks
- Navigates toward partner using MOVE|LOOK controls
- When within 2.5 blocks, completes breeding: resets love mode, sets 6000-tick cooldown
- Cooldown-aware: `breed_cooldown` prevents re-breeding for 5 minutes (vanilla)
- Future: baby entity spawn, EntityBreedEvent, player XP orbs

Key design: BreedGoal stores love state internally (`love_ticks`, `breed_cooldown`)
since MobEntity has no breeding-specific fields. This is activated externally when
a player feeds the mob its breeding food.

### 2. Created FollowParentGoal (`ai/goal/follow_parent.rs`)
New AI goal for baby mobs to follow a nearby adult of the same type:
- Searches within 16 blocks for same-type entity
- Navigates toward parent with MOVE control
- Stops following when close (9 blocks) or parent leaves range
- 200-tick delay between re-scans when no parent found
- Future: integrate with age/growth tracking to auto-disable for adults

### 3. Wired Both Goals to 18 Breedable Passive Mobs
All breedable mobs now have BreedGoal at priority 4 and FollowParentGoal at priority 5:

**Standard (16 mobs):** cow, pig, sheep, chicken, rabbit, goat, turtle, panda,
fox, frog, camel, sniffer, armadillo, horse, axolotl, cat

**Special import patterns:**
- wolf (no PanicGoal in imports)
- strider (no SwimGoal in imports)

## Priority Layout (Updated)
```
Swim=0, Panic=1, Attack=2, Tempt=3, Breed=4, FollowParent=5, Wander=6, LookAt=7, LookAround=8
```

## Files Created (2)
- `pumpkin/src/entity/ai/goal/breed.rs` — BreedGoal implementation
- `pumpkin/src/entity/ai/goal/follow_parent.rs` — FollowParentGoal implementation

## Files Modified (19)
- `pumpkin/src/entity/ai/goal/mod.rs` — registered `breed` and `follow_parent` modules
- 18 passive mob files (cow, pig, sheep, chicken, rabbit, cat, wolf, fox, goat, turtle,
  panda, frog, strider, camel, sniffer, armadillo, horse, axolotl) — added imports + goals

## Decisions Made
- **ENT-013:** BreedGoal priority is 4, FollowParentGoal priority is 5. Between Tempt (3) and Wander (6). Breed has higher priority than FollowParent because mating behavior should override baby following.
- **ENT-014:** BreedGoal stores love/cooldown state internally (not on MobEntity) to avoid modifying the shared MobEntity struct. External systems (player feeding) call `set_in_love()` on the goal directly.

## What Others Should Know
- **Plugin agent:** EntityBreedEvent is not yet wired (no event definition exists). When Plugin creates it, entity will wire the fire() call in BreedGoal.
- **Items agent:** Player feeding integration not yet complete — the bridge between "player uses food item on mob" and "BreedGoal::set_in_love()" needs the interaction handler in net/java/play.rs (Protocol scope).

## Current AI Goal Count
- **8 goals actively used by mobs:** Swim, Panic, Tempt, Breed, FollowParent, Wander, LookAtEntity, LookAround
- **Plus MeleeAttack on 14 hostile mobs** (Session 009)
- **Total goal structs in codebase:** 21 (19 original + Breed + FollowParent)

## Build Status
- `cargo build` — clean, no warnings
