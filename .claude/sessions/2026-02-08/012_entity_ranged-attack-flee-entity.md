# Session 012 — Entity: RangedAttackGoal + FleeEntityGoal Wiring

**Agent:** entity
**Date:** 2026-02-08
**Branch:** claude/entity-spawning-ai-V7oqj

## Preamble

- Read Session 011 log (BreedGoal + FollowParentGoal for 18 mobs).
- Read entity decisions ENT-001 through ENT-014.
- Read architect decisions ARCH-001 through ARCH-031.
- No pending requests from other agents to entity.
- Session 011 committed and pushed as bf22e84.

## What I Did

### 1. Created RangedAttackGoal (`ai/goal/ranged_attack.rs`)
New AI goal for ranged hostile mobs:
- Reads target from `mob.get_mob_entity().target` (set by ActiveTargetGoal)
- If out of range: navigates toward target (MOVE|LOOK controls)
- If in range: stops moving, looks at target, counts down attack interval
- When cooldown expires: fires (currently a stub — projectile spawning needs the projectile entity system)
- Parameters: `speed`, `attack_interval` (ticks), `attack_radius`
- Tracks `seen_target_ticks` — only fires when target has been continuously visible
- `should_run_every_tick = true` for responsive combat

### 2. Wired RangedAttackGoal to 6 Ranged Hostile Mobs
| Mob | Priority | Speed | Interval | Range | Notes |
|-----|----------|-------|----------|-------|-------|
| Skeleton | 4 | 1.0 | 20 | 15.0 | Bow attack (vanilla) |
| Blaze | 4 | 1.0 | 40 | 16.0 | Fireball (vanilla) |
| Ghast | 4 | 1.0 | 40 | 64.0 | Large fireball (vanilla) |
| Pillager | 3 | 1.0 | 20 | 8.0 | Crossbow (vanilla) |
| Witch | 2 | 1.0 | 60 | 10.0 | Potion throw (vanilla) |
| Drowned | 2 | 1.0 | 20 | 10.0 | Trident throw (vanilla) |

Also added missing goals to Skeleton (Swim, Wander, LookAround) and Creeper (Swim, Wander) that were absent.

### 3. Wired FleeEntityGoal to 5 Mobs (Vanilla-Accurate Flee Relationships)
FleeEntityGoal already existed in the codebase — this session wires it to the correct mobs.

| Mob | Flees From | Distance | Slow | Fast | Priority |
|-----|-----------|----------|------|------|----------|
| Creeper | Cat | 6.0 | 1.0 | 1.2 | 1 |
| Phantom | Cat | 16.0 | 1.0 | 1.2 | 1 |
| Skeleton | Wolf | 6.0 | 1.0 | 1.2 | 3 |
| Rabbit | Wolf | 10.0 | 2.2 | 2.2 | 2 |
| Fox | Wolf | 12.0 | 1.6 | 1.8 | 2 |

## Decisions Made
- **ENT-015:** RangedAttackGoal at priority 2-4 depending on mob. Lower priority than Swim/Panic/Flee, similar to MeleeAttack. Witch=2, Drowned=2, Pillager=3, Skeleton/Blaze/Ghast=4.
- **ENT-016:** FleeEntityGoal priority 1-3 depending on mob. Flee is high-priority (above attack): Creeper/Phantom flee cats at 1, Rabbit/Fox flee wolves at 2, Skeleton flees wolves at 3.

## Files Created (1)
- `pumpkin/src/entity/ai/goal/ranged_attack.rs` — RangedAttackGoal implementation

## Files Modified (11)
- `pumpkin/src/entity/ai/goal/mod.rs` — registered `ranged_attack` module
- `pumpkin/src/entity/mob/skeleton/mod.rs` — added Swim, FleeEntity(Wolf), RangedAttack, Wander, LookAround
- `pumpkin/src/entity/mob/blaze.rs` — added RangedAttack
- `pumpkin/src/entity/mob/ghast.rs` — added RangedAttack
- `pumpkin/src/entity/mob/pillager.rs` — added RangedAttack
- `pumpkin/src/entity/mob/witch.rs` — added RangedAttack
- `pumpkin/src/entity/mob/drowned.rs` — added RangedAttack
- `pumpkin/src/entity/mob/creeper.rs` — added Swim, FleeEntity(Cat), Wander
- `pumpkin/src/entity/mob/phantom.rs` — added FleeEntity(Cat)
- `pumpkin/src/entity/passive/rabbit.rs` — added FleeEntity(Wolf)
- `pumpkin/src/entity/passive/fox.rs` — added FleeEntity(Wolf)

## What Others Should Know
- **Protocol agent:** Projectile spawning (arrows, fireballs, tridents, potions) is needed to complete ranged combat. RangedAttackGoal provides correct targeting/timing but the actual projectile creation is a TODO.
- **Plugin agent:** No new events wired this session. EntityShootBowEvent would be the next event for ranged combat.

## Current AI Goal Count
- **10 goals actively used by mobs:** Swim, Panic, FleeEntity, Tempt, Breed, FollowParent, RangedAttack, MeleeAttack, LookAtEntity, LookAround
- **Plus WanderAround on most mobs**
- **Total goal structs in codebase:** 22 (21 + RangedAttack)

## Priority Layout (Updated)
```
Swim=0, Panic/Flee=1-3, Attack(Ranged/Melee)=2-4, Tempt=3, Breed=4, FollowParent=5, Wander=5-6, LookAt=6-7, LookAround=7-8
```

## Build Status
- `cargo build` — clean, no warnings
