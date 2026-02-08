# Session 015 — Entity: Passive Mob Breeding Goals + Ocelot Flee

**Agent:** entity
**Date:** 2026-02-08
**Branch:** claude/entity-spawning-ai-V7oqj

## Preamble

- Read Session 013 log (FollowOwnerGoal + Snow Golem Goals).
- Read Session 012 log (RangedAttackGoal + FleeEntityGoal).
- Read entity decisions ENT-001 through ENT-017.
- Session 013 committed and pushed.
- Session 014 was clippy cleanup (no log file — linter fixes only).

## What I Did

### 1. Added TEMPT_LLAMA and TEMPT_BEE Constants (`ai/goal/tempt.rs`)
- `TEMPT_LLAMA`: Hay bale (vanilla-accurate)
- `TEMPT_BEE`: 18 flower types including all small/tall flowers and wither rose

### 2. Wired TemptGoal/BreedGoal/FollowParentGoal to Llama
- TemptGoal(TEMPT_LLAMA, speed 0.6) at priority 3
- BreedGoal(speed 1.0) at priority 4
- FollowParentGoal(speed 1.0) at priority 5
- Existing goals shifted: Wander→6, LookAt→7, LookAround→8

### 3. Wired TemptGoal/BreedGoal/FollowParentGoal to Bee
- TemptGoal(TEMPT_BEE, speed 0.6) at priority 3
- BreedGoal(speed 1.0) at priority 4
- FollowParentGoal(speed 1.1) at priority 5
- Existing goals kept: Wander at 6, LookAt at 7, LookAround at 8

### 4. Verified Ocelot and Parrot NOT Breedable
- Ocelot: Not breedable since 1.14. Can only be "trusted" by feeding cod/salmon (already has TemptGoal). No BreedGoal needed.
- Parrot: Not breedable in vanilla. Already has TemptGoal for seeds. No BreedGoal needed.

### 5. Added FleeEntityGoal for Ocelot to Creeper and Phantom
In vanilla, creepers and phantoms flee both cats AND ocelots. Previously only fleeing cats.
- Creeper: Added `FleeEntityGoal::new(&EntityType::OCELOT, 6.0, 1.0, 1.2)` at priority 1
- Phantom: Added `FleeEntityGoal::new(&EntityType::OCELOT, 16.0, 1.0, 1.2)` at priority 1

## Decisions Made
- **ENT-018:** Only Llama and Bee needed breeding goals among remaining passive mobs. Ocelot, Parrot, Polar Bear, Dolphin, and Allay are NOT breedable in vanilla and should not have BreedGoal/FollowParentGoal.

## Files Modified (5)
- `pumpkin/src/entity/ai/goal/tempt.rs` — added TEMPT_LLAMA and TEMPT_BEE constants
- `pumpkin/src/entity/passive/llama.rs` — added TemptGoal, BreedGoal, FollowParentGoal
- `pumpkin/src/entity/passive/bee.rs` — added TemptGoal, BreedGoal, FollowParentGoal
- `pumpkin/src/entity/mob/creeper.rs` — added FleeEntityGoal for Ocelot
- `pumpkin/src/entity/mob/phantom.rs` — added FleeEntityGoal for Ocelot

## What Others Should Know
- **FleeEntity expanded**: Creepers and Phantoms now flee both Cat AND Ocelot (7 total flee pairs instead of 5).
- **Breedable mob coverage**: All vanilla-breedable mobs that have entity files now have TemptGoal/BreedGoal/FollowParentGoal: Cow, Sheep, Pig, Chicken, Rabbit, Fox, Wolf, Cat, Goat, Llama, Bee, Turtle, Panda, Frog, Horse, Camel, Sniffer, Armadillo, Strider, Axolotl, Mooshroom.
- **Non-breedable confirmed**: Ocelot, Parrot, Polar Bear, Dolphin, Allay — no BreedGoal.

## Current AI Goal Stats
- **11 goal types actively used by mobs**: Swim, Panic, FleeEntity, Tempt, Breed, FollowParent, FollowOwner, RangedAttack, MeleeAttack, LookAtEntity, LookAround + WanderAround
- **FleeEntity pairs: 7** — Creeper→Cat, Creeper→Ocelot, Phantom→Cat, Phantom→Ocelot, Skeleton→Wolf, Rabbit→Wolf, Fox→Wolf

## Build Status
- `cargo build` — clean, no warnings
- `cargo clippy --all-targets --all-features` with `-Dwarnings` — clean
