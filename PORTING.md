# Java -> Rust porting patterns for vanilla parity work

Read this before porting any vanilla behavior into Pumpkin. It exists because the
expensive bugs in this work are not "we forgot a feature" -- they are silent semantic
divergences where the Rust compiles, looks right, and behaves differently from Java.

Every entry below is either a bug actually found in this repo (cited with the file) or a
language difference that has already produced one. Add to it whenever a port surfaces a
new class of mistake; the point is that nobody rediscovers the same trap twice.

## Method: patterns first, then translate

Serialize the mapping rules before writing code. Two lookup tables carry most of the
value:

- `conformance/method_gaps.py` `METHOD_ALIASES` -- vanilla method name -> Pumpkin fn name.
  Pumpkin's naming is Yarn-derived, Mojang's is not, so a large fraction of "missing"
  methods are present under another name. A measured 18-lead sample found 10 of 18 flagged
  methods were already implemented.
- `conformance/map_coverage.py` `KNOWN_ALIASES` -- vanilla class -> Pumpkin type, same
  problem at class granularity (`FloatGoal` is `SwimGoal`, `PanicGoal` is
  `EscapeDangerGoal`, `EatBlockGoal` is `EatGrassGoal`).

A wrong alias is worse than a missing one: it makes an unimplemented method look
implemented and permanently hides a real gap. When unsure, leave it out.

## The backlog is a lead, not a fact

Prose gap-trackers go stale faster than the code moves. On 2026-08-06 a porting task was
dispatched from a backlog row claiming "Pumpkin's skeleton base always installs a melee
goal, never a ranged one". That was false: `ranged_bow_attack.rs` already existed and was
wired at `entity/mob/skeleton/mod.rs:107`, from a commit predating the claim.

Verify the premise against the current code before implementing from any written gap
description -- including one in this file. The agent that catches a stale premise and says
so has done more valuable work than one that implements a duplicate.

**Measured, 2026-08-06: the archived prose backlog is 4-for-4 stale.** Every lead dispatched
from it that day was already implemented, in whole or in part:

| Lead | Reality |
| --- | --- |
| Skeletons never get a ranged goal | `ranged_bow_attack.rs` existed and was wired; only an `isHoldingBow` gate was missing |
| Fireworks: item not consumed, entity payload-free | All correct; only a crossbow-rocket self-damage exemption was wrong |
| Fishing: COD stack built but never spawned | Loot/XP/rod-damage all worked; only spawn-vs-insert and stat gating were off |
| Piercing arrows blocked by a global has_hit latch | Fully correct already; zero changes needed |

Dispatch from the MEASURED data (`conformance/method_gaps.py` output) instead. Its precision
is only about 1 in 6, but it is regenerated from the current tree, so its errors are
name-matching noise rather than claims that were true months ago and silently stopped being
true. A stale backlog row costs a full implementation cycle; a name-match false positive
costs one read.

## Semantic traps: Java vs Rust

### Signed integers read as unsigned

Java has no unsigned integer types. NBT shorts and ints are signed, and vanilla uses
negative values as sentinels.

**Real bug** (`crates/pumpkin/src/entity/item.rs`): `nbt.get_short("Age") as u32`
sign-extended vanilla's `-6000` (extended lifetime) and `-32768` (never despawn) into
~4.29e9, which is `>= 6000`, so **every such item was deleted on its first tick** after
loading a vanilla world. Read as `i16` first, then decide.

### Zero is a valid id, not a "none" sentinel

**Real bug** (`crates/pumpkin/src/block/entities/beacon.rs`): `apply_effects` treated
`primary_id <= 0` as "no effect selected". But `StatusEffect::SPEED.id == 0`, so Speed --
the most common tier-1 beacon pick -- could never be applied. Vanilla uses `-1`, or an
`Optional`, precisely because 0 is a legitimate registry id.

Any `if id > 0` / `if id != 0` guard over a registry id is suspect. Registries in this
codebase are zero-indexed; use `-1`, `Option`, or a separate flag.

### Narrowing casts truncate instead of saturating

`as u8` on a value of 300 yields 44, silently. `PickupDelay` had exactly this bug. Use
`try_into()` or an explicit `.min()`/saturating cast, and preserve vanilla's sentinel
values deliberately.

### Field width must fit vanilla's range

Choosing `AtomicU8` for a value vanilla stores as a short caps it at 255. If vanilla's
range does not fit, the type is wrong -- do not clamp and move on without saying so.

### Inheritance is not traits

Vanilla's `Entity` -> `LivingEntity` -> `Mob` -> `Zombie` chain has no Rust equivalent.
Pumpkin uses traits with default methods plus a blanket `impl<T: Mob> EntityBase for T`.
Consequences that have already bitten:

- A blanket impl makes a direct `impl EntityBase for ConcreteType` a coherence error
  (E0119). This blocked wiring `EnderDragon::hurtServer`; the fix was to route through the
  part-hit path instead of fighting coherence.
- Overriding a trait method bypasses bookkeeping the default wrapper performed. When you
  override, check what the default did (`last_attack_time`, `on_successful_attack`) and
  either replicate it or confirm it is inert for your type.
- A vanilla `super.method()` call has no direct analogue. Call the trait default
  explicitly, or restructure.

### Vanilla stubs and client-only methods

Many vanilla methods are empty or `return false` in the base class, or run only under
`level.isClientSide()`. A dedicated server can never reach those. `Entity.hurtClient` is
literally unreachable server-side. Porting them is wasted work --
`conformance/vanilla_triage.py` filters the mechanical cases.

### Behavior expressed as data, not code

Pumpkin generates `crates/pumpkin-data` from real 26.2 game data. Attributes, block
properties, food values, loot and sounds are tables, not per-type methods. `createAttributes`
looks absent for every single mob and never is. Before hand-writing a table, check whether
the generated data already carries it -- a hand-written parallel table is a second source
of truth and will drift.

### Concurrency is real here, and Java hid it

Fields are touched by concurrent tick tasks. Never downgrade an atomic read-modify-write
into a load/store pair. Never hold a lock across an `.await`. Where vanilla relies on
single-threaded tick ordering, that assumption does not transfer.

## Verification: assume your own port is wrong

Adapted from Bun's Zig-to-Rust migration, whose most productive technique was
split-context adversarial review -- reviewers saw only the diff, never the implementer's
reasoning, and were told to assume the code was wrong. That separation is the point:
an implementer re-reading their own work confirms it.

Concretely, for this repo:

1. **Vanilla is the oracle, and it must be re-read.** Cite decompile file:line from a read
   performed in the current session. Never write behavior from memory -- numeric
   thresholds, tick timings and bit-packing layouts especially.
2. **Review from the diff alone.** A reviewer given the implementer's rationale inherits
   the implementer's blind spot.
3. **Compiler errors are a work queue, not a crisis.** Group them by crate and fix in
   batches; a port that does not compile yet is normal and cheaper to fix in bulk than to
   avoid.
4. **State a regression budget.** Name the one or two behaviors the change could plausibly
   break and how each was checked. "Nothing else could break" is not a budget.
5. **Separate "compiles" from "verified live".** Nothing is done because it type-checks.
   Entities do not tick with no player in simulation range, so mob behavior needs a live
   test, not a passing build.

## Scale expectations

Bun ported 1,448 files in 11 days using ~50 sequential workflows, peaking at 64 concurrent
model instances across 4 git worktrees, for roughly 5.9B input / 690M output tokens. The
lesson worth importing is not the parallelism -- it is that they spent preparatory time
building the pattern tables *first*, ran a 3-file trial with 1 implementer plus 2 reviewers
before scaling, and leaned on a test suite that was independent of the implementation
language. Here, the language-independent oracle is vanilla itself plus the RCON rig.
