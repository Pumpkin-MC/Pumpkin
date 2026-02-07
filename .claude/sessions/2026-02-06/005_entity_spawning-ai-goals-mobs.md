# Session: entity-005
**Agent:** entity
**Date:** 2026-02-06 UTC
**Task:** Add foundational AI goals (WanderAround, Swim, Panic) and 6 common overworld mob entities

## Context Loaded
- Read all 4 session logs from today (001-004): architect gap analysis, restructure, consolidation, protocol packets
- No yesterday sessions exist (first day)
- Read `.claude/sessions/decisions/entity.md` — empty, no prior decisions
- Read `.claude/sessions/decisions/architect.md` — 7 decisions (ARCH-001 through ARCH-007)
- Read entity contract in `.claude/contracts/entity.toml`
- Read `.claude/ORCHESTRATOR.md` for full constitution
- No entity specs exist yet (Phase 1 pending from Architect)
- No reference code populated yet

## What I Did
1. **Created 3 new AI goals** in `pumpkin/src/entity/ai/goal/`:
   - `wander_around.rs` — WanderAroundGoal: random idle movement within 8 blocks, configurable chance (default 1/120 ticks)
   - `swim.rs` — SwimGoal: makes mobs jump when submerged in water to stay afloat
   - `panic.rs` — PanicGoal: detects health drop and runs to random position 5-10 blocks away at increased speed

2. **Created 4 passive mob entities** in `pumpkin/src/entity/passive/`:
   - `chicken.rs` — ChickenEntity with Swim, Panic(1.4x), Wander, LookAt, LookAround goals
   - `cow.rs` — CowEntity with Swim, Panic(2.0x), Wander, LookAt, LookAround goals
   - `pig.rs` — PigEntity with Swim, Panic(1.25x), Wander, LookAt, LookAround goals
   - `sheep.rs` — SheepEntity with Swim, Panic(1.25x), Wander, LookAt, LookAround goals

3. **Created 2 hostile mob entities** in `pumpkin/src/entity/mob/`:
   - `spider.rs` — SpiderEntity with Swim, MeleeAttack, Wander, LookAt, LookAround + player targeting
   - `enderman.rs` — EndermanEntity with Swim, MeleeAttack, Wander, LookAt, LookAround + player targeting (placeholder; enderman look-targeting TBD)

4. **Registered all new mobs** in `pumpkin/src/entity/type.rs` factory function

5. **Updated mod.rs** files for both `mob/` and `passive/` directories

## What I Learned
- The mob pattern is consistent: struct wrapping MobEntity, async `new()` that sets up goals, impl NBTStorage + Mob trait
- Goal priorities follow vanilla conventions: lower number = higher priority
- SwimGoal (priority 0), PanicGoal (priority 1), Attack (priority 2), Wander (priority 6-7), Look (priority 7-8)
- Navigator uses simple A* pathfinding over a 3x3 grid; `is_idle()` always returns false (TODO in codebase)
- `rand::RngExt` must be explicitly imported for `.random::<T>()` calls

## What I Changed
### New Files
- `pumpkin/src/entity/ai/goal/wander_around.rs`
- `pumpkin/src/entity/ai/goal/swim.rs`
- `pumpkin/src/entity/ai/goal/panic.rs`
- `pumpkin/src/entity/passive/chicken.rs`
- `pumpkin/src/entity/passive/cow.rs`
- `pumpkin/src/entity/passive/pig.rs`
- `pumpkin/src/entity/passive/sheep.rs`
- `pumpkin/src/entity/mob/spider.rs`
- `pumpkin/src/entity/mob/enderman.rs`

### Modified Files
- `pumpkin/src/entity/ai/goal/mod.rs` — added `panic`, `swim`, `wander_around` module declarations
- `pumpkin/src/entity/mob/mod.rs` — added `enderman`, `spider` module declarations
- `pumpkin/src/entity/passive/mod.rs` — added `chicken`, `cow`, `pig`, `sheep` module declarations
- `pumpkin/src/entity/type.rs` — added imports and factory match arms for all 6 new entities

## Perspectives Consulted
- **protocol**: Not needed this session. Entity metadata for passive mobs is minimal (no special tracked data beyond base mob flags). Spider and Enderman have metadata (climbing flags, carried block) but we use base behavior for now.
- **worldgen**: Passive mobs need light level >= 9 and grass block spawning surface per vanilla rules. The natural_spawner.rs in pumpkin/src/world/ already handles biome-based spawn lists. Our entities just need to be registered in assets/entities.json (which they already are).

## What I Need From Others
- **Architect**: `Navigator::is_idle()` always returns `false` (line 82-85 of `ai/path/mod.rs`). This means `should_continue` for WanderAroundGoal and PanicGoal will never return idle=true from the navigator. The navigator needs proper idle detection when destination is reached. This is a pre-existing issue, not introduced by this session.

## What Others Should Know
- ⚠️ The 3 new AI goals (WanderAround, Swim, Panic) are designed to be reusable across all mob types. Future mob implementations should use these rather than re-implementing.
- Goal priority conventions used: 0=Swim, 1=Panic, 2=Attack, 4-5=Special, 6=Wander, 7=LookAt, 8=LookAround
- Panic speed multipliers match vanilla: chicken=1.4, cow=2.0, pig=1.25, sheep=1.25
- Enderman targeting is simplified (always targets nearby players). Vanilla behavior (only targets players who look at them) requires custom TargetPredicate work in a future session.

## Decisions Made
- **ENT-001**: AI goal priorities follow vanilla conventions (lower=higher priority). Swim=0, Panic=1, Attack=2, Wander=6, LookAt=7, LookAround=8.
- **ENT-002**: WanderAroundGoal uses 8-block radius, 1/120 tick chance. Matches vanilla WanderAroundFarGoal simplified.
- **ENT-003**: PanicGoal detects damage via health comparison (not damage event). This is a pragmatic choice since we don't have a damage event callback in the Goal trait.

## Tests
- `cargo build -p pumpkin` — passes
- `cargo clippy -p pumpkin` — passes with no warnings

## Open Questions
- Should enderman teleportation be part of the entity module or does it require world-level block manipulation (outside entity scope)?
- Navigator `is_idle()` returning false breaks goal lifecycle for movement goals. Who owns the fix? (Filed as need from Architect)
