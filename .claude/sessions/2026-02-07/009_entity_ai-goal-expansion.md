# Session 009 — Entity: AI Goal Expansion (P0)

**Agent:** entity
**Date:** 2026-02-07
**Branch:** claude/entity-spawning-ai-V7oqj

## Preamble

Read session 008 log (mass mob implementation — 81 types registered).
Read architect session 010 (status update, P0 prioritization).
Read plugin session 008 (ignore-cancelled, 25/39 events fired at 64%).
Noted: Plugin agent found 4 entity-owned events needing my attention (PlayerDeathEvent, PlayerRespawnEvent, PlayerDropItemEvent, PlayerItemConsumeEvent).

P0 priority from architect: "AI goal expansion — 81 mob types but only 4 goals. Mobs are idle."
Checked entities.toml, items.toml, and entity-data.md for vanilla-accurate data.
Rebased to latest master.

## What I Did

### 1. Made `melee_attack` Module Public
Changed `mod melee_attack;` → `pub mod melee_attack;` in `ai/goal/mod.rs` so non-zombie mobs can use `MeleeAttackGoal` directly without the zombie arm-raising wrapper.

### 2. Wired MeleeAttackGoal to 14 Hostile Mobs
Added `MeleeAttackGoal` at priority 2 to all melee hostile mobs that had `ActiveTargetGoal` (target finding) but NO attack goal:

| Mob | Speed | Pause |
|-----|-------|-------|
| vindicator | 1.0 | false |
| wither_skeleton | 1.0 | false |
| hoglin | 1.0 | true |
| ravager | 1.0 | true |
| vex | 1.0 | true |
| warden | 0.9 | true |
| creaking | 1.0 | false |
| zombified_piglin | 1.0 | false |
| piglin | 1.0 | false |
| slime | 1.2 | false |
| magma_cube | 1.2 | false |
| phantom | 1.0 | true |
| endermite | 1.0 | false |
| silverfish | 1.9 | true |

Also upgraded endermite and silverfish with full AI goals (swim, wander, look_at, look_around) — they previously only had target_selector.

### 3. Created TemptGoal (New AI Goal)
New file: `pumpkin/src/entity/ai/goal/tempt.rs`

**Behavior:** Passive mobs follow players holding specific food items.
- Finds closest player within range holding a tempt item
- Navigates toward the player each tick
- Stops when player switches items, goes out of range, or dies
- 100-tick cooldown after stopping to prevent oscillation

**Vanilla-accurate food item lists** (static slices of item IDs):
- `TEMPT_WHEAT` — cow, sheep, goat, mooshroom
- `TEMPT_SEEDS` — chicken, parrot
- `TEMPT_PIG` — carrot, potato, beetroot
- `TEMPT_RABBIT` — carrot, golden_carrot, dandelion
- `TEMPT_CAT` — cod, salmon (also used for ocelot)
- `TEMPT_FOX` — sweet_berries, glow_berries
- `TEMPT_WOLF` — 10 raw/cooked meat items
- `TEMPT_PANDA` — bamboo
- `TEMPT_TURTLE` — seagrass
- `TEMPT_AXOLOTL` — tropical_fish_bucket
- `TEMPT_HORSE` — golden_apple, golden_carrot
- `TEMPT_STRIDER` — warped_fungus
- `TEMPT_FROG` — slime_ball
- `TEMPT_CAMEL` — cactus
- `TEMPT_ARMADILLO` — spider_eye
- `TEMPT_SNIFFER` — torchflower_seeds

### 4. Wired TemptGoal to 19 Passive Mobs
Added `TemptGoal` at priority 3 to all passive mobs that follow food in vanilla:

cow, pig, sheep, chicken, rabbit, cat, wolf, fox, goat, turtle, panda, frog, strider, camel, sniffer, armadillo, ocelot, parrot, axolotl, horse

Also gave wolf proper basic goals it was missing (swim, wander).

## Files Created (1)
- `pumpkin/src/entity/ai/goal/tempt.rs` — TemptGoal + 16 static tempt item lists

## Files Modified (33)

### AI system:
- `pumpkin/src/entity/ai/goal/mod.rs` — made melee_attack public, added tempt module

### Hostile mobs with melee attack added (14):
vindicator.rs, wither_skeleton.rs, hoglin.rs, ravager.rs, vex.rs, warden.rs,
creaking.rs, zombified_piglin.rs, piglin.rs, slime.rs, magma_cube.rs, phantom.rs,
endermite.rs (rewritten), silverfish.rs (rewritten)

### Passive mobs with TemptGoal added (19):
cow.rs, pig.rs, sheep.rs, chicken.rs, rabbit.rs, cat.rs, wolf.rs, fox.rs,
goat.rs, turtle.rs, panda.rs, frog.rs, strider.rs, camel.rs, sniffer.rs,
armadillo.rs, ocelot.rs, parrot.rs, axolotl.rs, horse.rs

## AI Goal Count After This Session
- **Before:** 4 goals used by mobs (swim, panic, wander, look_at)
- **After:** 6 goals used by mobs (swim, panic, wander, look_at, melee_attack, tempt)
- **Total goal implementations in codebase:** 17 (including zombie_attack, creeper_ignite, flee_entity, active_target, track_target, step_and_destroy_block, move_to_target_pos, ambient_stand, look_around)

## Coverage Summary
- **Hostile mobs with attack goals:** ~20 (zombie, drowned, husk, spider, cave_spider, enderman, +14 new)
- **Passive mobs with tempt goals:** 19 (all major farmable/breedable mobs)
- **Mobs still missing attack goals:** blaze, ghast, pillager (ranged — need RangedAttackGoal), evoker, illusioner (spells), shulker, breeze (projectiles), guardian, elder_guardian (laser), witch (potions)

## Decisions Made
- ENT-010: Use `MeleeAttackGoal` directly for non-zombie mobs (no arm-raising animation). `ZombieAttackGoal` reserved for zombie-type mobs.
- ENT-011: TemptGoal uses static `&[u16]` item ID slices for zero-allocation food checks.

## What Others Should Know
- **Plugin agent:** 4 entity-owned player events pending: PlayerDeathEvent, PlayerRespawnEvent, PlayerDropItemEvent, PlayerItemConsumeEvent. Will address in next session.
- **All melee hostile mobs now walk toward and attack players** (navigation + attack animation). Actual damage dealing requires combat system work (damage_with_context integration in MeleeAttackGoal tick).
- **All major passive mobs now follow food items.** This is the most visible passive mob behavior.
- **Next priorities:** RangedAttackGoal (skeletons, blazes), BreedGoal (passive breeding), FollowOwnerGoal (wolves/cats).

## Build Status
- `cargo build` — clean, no warnings
