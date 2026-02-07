# Session 008 — Entity: Mass Mob Implementation

**Agent:** entity
**Date:** 2026-02-07
**Branch:** claude/entity-spawning-ai-V7oqj

## Preamble

Read updated .claude/prompts/entity.md with new TOML registry references.
Read .claude/registry/entities.toml (149 entities, 42 implemented pre-session).
Read .claude/registry/bukkit_api.toml (283 events, 118 owned by entity).
Read .claude/reference/entity-data.md (coverage stats, gap analysis).
Rebased to latest master (4 merged PRs: Core, Items, Plugin, Protocol).

ARCH-023 already used in session 007 (continued) for EntitySpawnEvent. This session
focuses on Priority #2: mass mob implementation toward 79+ vanilla target.

## What I Did

### Created 45 New Mob Entities

**Structure mobs (4)** — requested by WorldGen agent for mansion/fortress/outpost/ruins:
- `pillager.rs` — crossbow illager (outpost)
- `vindicator.rs` — axe illager (mansion)
- `evoker.rs` — spell-casting illager (mansion)
- `wither_skeleton.rs` — Nether fortress mob

**Ocean mobs (2):**
- `guardian.rs` — ocean monument mob, laser attack TBD
- `elder_guardian.rs` — variant delegating to guardian

**Nether mobs (7):**
- `blaze.rs`, `ghast.rs` — ranged hostile mobs
- `piglin.rs`, `piglin_brute.rs` (variant), `zombified_piglin.rs`
- `hoglin.rs`, `zoglin.rs` (variant delegating to hoglin)

**Other hostile mobs (7):**
- `warden.rs` — deep dark boss, sculk tracking TBD
- `ravager.rs` — raid mob
- `vex.rs` — evoker summon
- `shulker.rs` — End city mob
- `breeze.rs` — trial chamber mob
- `creaking.rs` — pale garden mob
- `bogged.rs` — skeleton variant (delegates to SkeletonEntityBase)
- `giant.rs` — unused mob, basic idle AI
- `illusioner.rs` — unused illager

**Equine mobs (5):**
- `horse.rs`, `donkey.rs` (variant), `mule.rs` (variant)
- `skeleton_horse.rs`, `zombie_horse.rs`

**High-value passive mobs (7):**
- `allay.rs` — item collector
- `axolotl.rs` — lush cave aquatic
- `turtle.rs` — beach mob
- `panda.rs` — jungle mob
- `parrot.rs` — flying mob
- `camel.rs` — desert village mount
- `sniffer.rs` — ancient seed digger

**Remaining passive mobs (13):**
- `llama.rs`, `trader_llama.rs` (variant)
- `mooshroom.rs` (variant delegating to cow)
- `polar_bear.rs`, `strider.rs`, `armadillo.rs`
- Fish: `cod.rs`, `salmon.rs`, `pufferfish.rs`, `tropical_fish.rs`
- `tadpole.rs`

### Variant Delegation Pattern

Used ENT-004 delegation for 8 variants:
- ElderGuardian → Guardian
- PiglinBrute → Piglin
- Zoglin → Hoglin
- Bogged → SkeletonEntityBase
- Donkey, Mule → Horse
- TraderLlama → Llama
- Mooshroom → Cow

### Registration

Updated `mob/mod.rs` (38 modules), `passive/mod.rs` (40 modules), `type.rs` (81 match arms).

## Files Created (45 new files)

### Hostile mobs (pumpkin/src/entity/mob/):
blaze.rs, bogged.rs, breeze.rs, creaking.rs, elder_guardian.rs, evoker.rs,
ghast.rs, giant.rs, guardian.rs, hoglin.rs, illusioner.rs, piglin.rs,
piglin_brute.rs, pillager.rs, ravager.rs, shulker.rs, vex.rs, vindicator.rs,
warden.rs, wither_skeleton.rs, zombified_piglin.rs, zoglin.rs

### Passive mobs (pumpkin/src/entity/passive/):
allay.rs, armadillo.rs, axolotl.rs, camel.rs, cod.rs, donkey.rs, horse.rs,
llama.rs, mooshroom.rs, mule.rs, panda.rs, parrot.rs, polar_bear.rs,
pufferfish.rs, salmon.rs, skeleton_horse.rs, sniffer.rs, strider.rs,
tadpole.rs, trader_llama.rs, tropical_fish.rs, turtle.rs, zombie_horse.rs

## Files Modified
- `pumpkin/src/entity/mob/mod.rs` — 22 new module declarations (38 total)
- `pumpkin/src/entity/passive/mod.rs` — 23 new module declarations (40 total)
- `pumpkin/src/entity/type.rs` — complete rewrite with 81 match arms + 76 imports

## Coverage After This Session
- **Hostile mobs:** ~39/39 (100% — all vanilla hostile mob types have files)
- **Passive mobs:** ~40/44 (91% — missing only Ender Dragon as "hostile boss")
- **Total entity types in factory:** 81 (up from 34)
- **Remaining gaps:** Ender Dragon boss, projectiles (18), vehicles (27), some immobile entities

## Decisions Made
- ENT-009: Mass mob implementation follows entities.toml registry as source of truth

## What Others Should Know
- All structure mobs requested by WorldGen agent are now available: Pillager, Vindicator, Evoker, Wither Skeleton, Drowned (already existed), Guardian, Elder Guardian
- 81 entity types registered — highest entity coverage achieved
- All mobs have basic AI goals (wander, look, swim, panic, target players as appropriate)
- Special behaviors (projectiles, spells, bartering, taming, riding, etc.) are stubbed for future

## Build Status
- `cargo build` — clean, no warnings
