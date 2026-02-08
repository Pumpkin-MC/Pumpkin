# Session 013 — Entity: FollowOwnerGoal + Snow Golem Goals

**Agent:** entity
**Date:** 2026-02-08
**Branch:** claude/entity-spawning-ai-V7oqj

## Preamble

- Read Session 012 log (RangedAttackGoal + FleeEntityGoal).
- Read entity decisions ENT-001 through ENT-016.
- Read architect decisions (ARCH-011, ARCH-023 relevant).
- Session 012 committed and pushed as 565768b.

## What I Did

### 1. Created FollowOwnerGoal (`ai/goal/follow_owner.rs`)
New AI goal for tamed mobs (Wolf, Cat) to follow their owner player:
- Stores `owner_id: Option<i32>` internally (ENT-014 pattern — state in goal, not MobEntity)
- External activation via `set_owner(entity_id)` when taming occurs
- Start following when owner is >10 blocks away
- Stop following when <2 blocks away
- Teleport to owner when >12 blocks away
- Uses `world.get_player_by_id()` to find owner
- Uses `player.position()` for owner location
- Controls: MOVE | LOOK, `should_run_every_tick = true`
- Priority 6 — after FollowParent(5), before Wander(7)

### 2. Wired FollowOwnerGoal to Wolf and Cat
- Wolf: Added at priority 6, shifted Wander to 7, LookAt to 9, LookAround to 10
- Cat: Added at priority 6, shifted Wander to 7, LookAt to 8, LookAround to 9

### 3. Snow Golem Goal Expansion
Added missing goals to Snow Golem:
- SwimGoal at priority 0
- RangedAttackGoal at priority 2 (interval 20, range 10.0) — snowball throwing
- WanderAroundGoal at priority 6
- Already had LookAtEntity and LookAround

### 4. Verified Hostile Mob Goal Coverage
Audited all hostile mob files — most already had complete goal sets from Sessions 009/012.
Iron Golem already had SwimGoal, MeleeAttackGoal, WanderAroundGoal from prior work.

## Decisions Made
- **ENT-017:** FollowOwnerGoal at priority 6 for tamed mobs. After FollowParent(5), before Wander(7). Uses ENT-014 pattern — owner state stored in goal struct, not MobEntity.

## Files Created (1)
- `pumpkin/src/entity/ai/goal/follow_owner.rs` — FollowOwnerGoal implementation

## Files Modified (4)
- `pumpkin/src/entity/ai/goal/mod.rs` — registered `follow_owner` module
- `pumpkin/src/entity/passive/wolf.rs` — added FollowOwnerGoal, adjusted priorities
- `pumpkin/src/entity/passive/cat.rs` — added FollowOwnerGoal, adjusted priorities
- `pumpkin/src/entity/passive/snow_golem.rs` — added SwimGoal, RangedAttackGoal

## What Others Should Know
- **Protocol agent:** FollowOwnerGoal is wired but inactive until taming is implemented. When taming system is added, call `goal.set_owner(player_entity_id)` on the mob's FollowOwnerGoal.
- **Plugin agent:** No new events this session. EntityTameEvent will be needed when taming is implemented.

## Current AI Goal Count
- **11 goals actively used by mobs:** Swim, Panic, FleeEntity, Tempt, Breed, FollowParent, FollowOwner, RangedAttack, MeleeAttack, LookAtEntity, LookAround
- **Plus WanderAround on most mobs**
- **Total goal structs in codebase:** 23 (22 + FollowOwner)

## Build Status
- `cargo build` — clean, no warnings
