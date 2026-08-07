# Brain/Memory/Activity System — Design Document

Status: design only, no mob migration. Written 2026-08-06.

## Claim tiers

Every behavioral claim below is tagged VERIFIED (read this session, file:line
given), INFERRED (reasoned from a VERIFIED fact, chain stated), or UNVERIFIED
(a command that would settle it is named). Vanilla citations are against
`/home/eshanki/pumpkin-vanilla-26.2/decompiled`, package
`net.minecraft.world.entity.ai`, version 26.2 unless marked 1.21.4 (mil1dude
mirror). All were read fresh this session.

## 1. Problem statement

VERIFIED: `crates/pumpkin/src/entity/mob/warden.rs:5-42` and
`crates/pumpkin/src/entity/passive/allay.rs:1-10` both state in comments that
Pumpkin has no Brain/Memory/Activity system, and both mobs are implemented as
ad-hoc field-and-timer ports that explicitly skip parts of vanilla behavior
because the gating framework isn't there (warden's DIGGING/SNIFFING poses are
unimplemented; the comment cites `Digging.java:36-40` ending in entity
removal as the reason an under-specified port would be worse than the gap).

This is the largest structural gap in Pumpkin: modern hostile/passive AI
(villager, piglin, hoglin, zombified piglin, axolotl, allay, warden, frog,
goat, camel, sniffer, and all NPC-adjacent mobs) is Brain-driven in vanilla,
not Goal-driven. Pumpkin has a working, idiomatic Goal system
(`crates/pumpkin/src/entity/ai/goal/`, 118 files) but nothing structurally
equivalent for Brain. That one gap gates roughly 142 of the 213 missing
classes (behavior 108 + Brain.java + memory 7 + sensing 26, per the session's
count).

## 2. Vanilla runtime shape

### 2.1 Brain (VERIFIED, `Brain.java`)

- `Brain<E>` holds: `memories: Map<MemoryModuleType<?>, MemorySlot<?>>`
  (`Brain.java:40`), `sensors: Map<SensorType, Sensor>` (`:41`),
  `availableBehaviorsByPriority: TreeMap<Integer, Map<Activity,
  Set<BehaviorControl>>>` (`:42`), `activityRequirements`,
  `activityMemoriesToEraseWhenStopped`, `coreActivities`, `activeActivities`,
  `defaultActivity` (`:43-49`).
- `tick()` (`:384-389`) does exactly four things in order: expire memories
  (`forgetOutdatedMemories`, `:397-399`), tick sensors (`:391-395`), start
  every non-running behavior whose activity is active and required-memory
  conditions are met (`startEachNonRunningBehavior`, `:409-424`), then tick
  every currently-running behavior (`tickEachRunningBehavior`, `:426-432`).
- **Behavior arbitration is not exclusive.** `startEachNonRunningBehavior`
  iterates the full `TreeMap<Integer, ...>` by priority and calls
  `tryStart` on *every* stopped behavior in every active activity whose
  memory conditions hold (`:412-423`) — there is no equivalent of the Goal
  system's `Controls`/`goals_by_control` slot exclusivity. Two behaviors can
  run concurrently within a tick as long as neither's `checkExtraStartConditions`
  fails. Contention between behaviors that would otherwise fight (e.g. two
  behaviors both wanting to walk somewhere) is resolved by *memory ownership*:
  one behavior writes `WALK_TARGET`, a single terminal "sink" behavior
  (`MoveToTargetSink`, `SetWalkTargetFromLookTarget`, etc.) is the only thing
  that reads it and drains it into the navigator. The `TreeMap` priority
  only orders the *order in which `tryStart` is attempted* within one tick,
  not mutual exclusion.
- `updateActivityFromSchedule` (`:327-335`) rate-limits schedule polling to
  once per 20 ticks and routes through `EnvironmentAttribute<Activity>` /
  `EnvironmentAttributeSystem`, an entirely separate subsystem this session
  did not read and Pumpkin has no analogue for. **Out of scope**, see §5.
- `setActiveActivity` (`:305-312`) is the actual activity switch: on change,
  it erases memories registered against the outgoing activity
  (`eraseMemoriesForOtherActivitesThan`, `:314-325`), clears
  `activeActivities`, and refills it with `coreActivities ∪ {newActivity}`.
  CORE is therefore always active alongside exactly one non-core activity.

### 2.2 Memory (VERIFIED, `memory/MemoryModuleType.java`,
`memory/MemorySlot.java`, `memory/MemoryStatus.java`, `memory/ExpirableValue.java`)

- `MemoryModuleType<U>` is a generic-typed registry key with an optional
  `Codec<ExpirableValue<U>>` for persistence (`MemoryModuleType.java:153-179`).
  ~120 concrete instances are declared (`:33-152`), value types ranging from
  `Void`/`Boolean`/`Integer`/`Long` through `GlobalPos`, `WalkTarget`,
  `Path`, `Set<GlobalPos>`, up to live entity references:
  `LivingEntity`, `Player`, `ItemEntity`, `Hoglin`, `AbstractPiglin`, `Mob`,
  `AgeableMob`, `Entity`, `List<LivingEntity>`, `List<Player>`, etc.
- `MemorySlot<T>` (`MemorySlot.java`) is `{ value: Option<T>, timeToLive: i64
  (MAX = never expire) }`. `tick()` decrements `timeToLive` and clears on
  reaching 0 (`:16-24`). No inherent per-type expiry — callers choose via
  `setMemoryWithExpiry`.
- `MemoryStatus` (VERIFIED, full enum) is exactly three variants:
  `VALUE_PRESENT, VALUE_ABSENT, REGISTERED`. `Brain.checkMemory`
  (`Brain.java:242-249`) implements `REGISTERED` as "slot exists at all"
  (used by behaviors that don't care about the value, only that the memory
  type is wired up for this brain).
- Uniform iteration matters structurally: `Brain.forEach` (`:229-235`),
  `clearMemories` (`:154-156`), and `pack()` (`:113-135`, gated on
  `type.canSerialize()`, i.e. codec presence) all walk the memory map
  homogeneously by `MemoryModuleType`. Any Rust representation that can't
  support "iterate every registered slot regardless of concrete type" breaks
  these three operations.

### 2.3 Behavior (VERIFIED, `behavior/Behavior.java`, `BehaviorControl.java`,
`declarative/BehaviorBuilder.java`, `MoveToTargetSink.java`, `GateBehavior.java`)

- `BehaviorControl<E>` is the real interface: `getStatus()`,
  `getRequiredMemories()`, `tryStart`, `tickOrStop`, `doStop`, `debugString()`
  (`BehaviorControl.java`).
- `Behavior<E>` is the common base: a fixed `entryCondition: Map<MemoryModuleType<?>,
  MemoryStatus>` checked in `tryStart` (`hasRequiredMemories`, `:96-104`),
  `Status::{STOPPED, RUNNING}`, min/max duration with a *randomized* actual
  duration rolled at start (`:46-52`), and `canStillUse`/`timedOut` gating
  continuation (`:60-66`).
- `MoveToTargetSink` (VERIFIED, full read) is the canonical example of the
  memory-ownership pattern: entry condition requires `WALK_TARGET` present
  and `PATH` absent (`:29-41`); `checkExtraStartConditions` reads
  `WALK_TARGET`, computes a path via the mob's `PathNavigation`, and writes
  `PATH` back to the brain; `start` calls `body.getNavigation().moveTo(path,
  speed)`; `stop` erases both `WALK_TARGET` and `PATH` and stops the
  navigator. **This is the load-bearing bridge between Brain memory and the
  navigation stack** — nothing else in vanilla drains `WALK_TARGET`.
- Roughly half of behavior classes are not `Behavior` subclasses at all.
  VERIFIED by direct count in `behavior/*.java` (104 files, excluding
  `declarative/`): 55 use `BehaviorBuilder.create(...)` (one-shot declarative
  triggers, no persistent per-instance state beyond captured lambdas), 33
  `extends Behavior<E>` (full start/tick/stop lifecycle), 4 implement
  `BehaviorControl` directly for custom control flow (`GateBehavior`,
  `RunOne`/`ShufflingList`-based combinators), and the remainder are shared
  infrastructure (`Behavior`, `BehaviorControl`, `BehaviorUtils`,
  `PositionTracker`, `EntityTracker`, `ShufflingList`, etc.), not behaviors
  themselves. **This matters for sizing**: over half the "108 behavior
  classes" in the gap count are small declarative triggers, not large
  stateful machines — the port is not 108 units of `MoveToTargetSink`-sized
  work.

### 2.4 Sensor (VERIFIED, `sensing/Sensor.java`, `sensing/NearestItemSensor.java`)

- `Sensor<E>` ticks on a fixed cadence (`DEFAULT_SCAN_RATE = 20` ticks,
  `Sensor.java:14,44-49`), randomized-delayed at brain creation
  (`randomlyDelayStart`, `:37-39`, called from `Brain.java:84`) so many
  mobs' sensors don't all fire the same tick. `doTick` is the per-sensor
  hook; `requires()` declares which memory types it writes, used by
  `Brain`'s constructor to auto-register those slots (`Brain.java:87-89`).
- `NearestItemSensor` (full read) is representative: scans entities in a
  bounding-box, filters/sorts, writes one `Optional<ItemEntity>` into
  `NEAREST_VISIBLE_WANTED_ITEM`. Sensors are the *only* legitimate writers
  of "sensed world state" memories; behaviors write intent/target memories
  (`WALK_TARGET`, `ATTACK_TARGET`, etc.).
- The 20-tick cadence means every sensed memory is allowed up to 20 ticks of
  staleness by vanilla's own design — this is the justification used in §4.4
  for how Pumpkin can store weak/resolved-at-read references instead of
  strong entity handles.

### 2.5 Activity / ActivityData (VERIFIED, `schedule/Activity.java`, `ActivityData.java`)

- `Activity` is a flat registered-singleton enum-like class (27 constants:
  CORE, IDLE, WORK, PLAY, REST, MEET, PANIC, RAID, FIGHT, ADMIRE_ITEM, ...).
- `ActivityData<E>` is a plain record: `(Activity, priority→behavior pairs,
  entry-memory conditions, memories-to-erase-on-exit)`, built once per mob
  type at brain-creation time and fed into `Brain.addActivity`
  (`Brain.java:350-373`).

### 2.6 AllayAi as a worked example (VERIFIED, full read)

`AllayAi.java` (165 lines) declares exactly two activities:
- CORE (`:61-74`, priority 0): `Swim`, `AnimalPanic`, `LookAtTargetSink`,
  `MoveToTargetSink`, two `CountDownCooldownTicks`. All infrastructure —
  present on essentially every land/flying mob's brain.
- IDLE (`:76-90`, priority 0): `GoToWantedItem`, `GoAndGiveItemsToTarget`,
  `StayCloseToTarget`, `SetEntityLookTargetSometimes`, one `RunOne` combinator
  wrapping `RandomStroll`/`SetWalkTargetFromLookTarget`/`DoNothing`.
- `updateActivity` (`:92-94`) calls `setActiveActivityToFirstValid(List.of(IDLE))`
  — **no schedule dependency** (contrast with villager, which routes through
  `EnvironmentAttribute<Activity>`).
- No POI, no village, no gossip. `hearNoteblock`/`getLikedPlayer` (`:96-156`)
  are self-contained memory read/writes plus one cross-cutting call from
  jukebox/noteblock code (already ported ad hoc in `allay.rs` per its own
  comment).
- 11 behavior instances total, of which 4 (Swim, AnimalPanic, LookAtTargetSink,
  MoveToTargetSink) are reusable infrastructure and only ~7 are Allay-specific.

This is the smallest real Brain-driven mob in the game and is the
recommended vertical slice (§6).

## 3. Existing Pumpkin Goal system — what to reuse, what not to

VERIFIED, `crates/pumpkin/src/entity/ai/goal/mod.rs`,
`crates/pumpkin/src/entity/ai/goal/goal_selector.rs`,
`crates/pumpkin/src/entity/mob/mod.rs:934-999`.

- `Goal` is an async trait (`GoalFuture<'a, T> = Pin<Box<dyn Future<Output=T>
  + Send + 'a>>`) with `can_start`/`should_continue`/`start`/`stop`/`tick`,
  each taking `&'a dyn Mob`. `GoalSelector` owns `Vec<PrioritizedGoal>` plus
  a fixed `[usize; 4]` `goals_by_control` array mapping the four `Controls`
  bits (MOVE/LOOK/JUMP/TARGET) to the currently-running goal that claims
  that control, giving Goals *mutual exclusion by control bit* — this is
  exactly what Brain behaviors do **not** have (§2.1). A Brain port must not
  reuse `Controls`/`goals_by_control`-style exclusivity for behaviors; doing
  so would silently change arbitration semantics from vanilla's "everything
  whose memory gate is satisfied runs" to "one behavior per control axis,"
  which is a different and wrong model. State this loudly in code comments
  wherever the two systems are made to interoperate.
- Storage and ticking pattern to imitate (`mob/mod.rs:967-985`): `MobEntity`
  holds `goals_selector: std::sync::Mutex<GoalSelector>`. Each entity tick,
  the code takes the selector out of its mutex with `std::mem::take`
  (requires `Default`), ticks it with no lock held (so `.await` inside
  `Goal` methods is legal), then puts it back. This exact shape — extract →
  unlocked-await-tick → replace — is the concurrency pattern to reuse for
  the *behavior/sensor runtime* half of Brain (see §4.2), but **not** for
  the memory store, for reasons in §4.2.
- Goal registration is per-mob, explicit, at construction (`cow.rs:45-57`):
  `goal_selector.add_goal(priority, Box::new(SomeGoal::new(...)))`. A Brain
  port should follow the same shape: a mob's constructor builds its
  `Brain` (memories to register, sensors, activities/behaviors) once, the
  same way vanilla's `*Ai.getActivities()` static methods do.

## 4. Rust-specific hard problems and recommendations

### 4.1 Typed heterogeneous memory storage

**Constraint** (from §2.2): must support (a) `brain.get::<T>(MemoryKey)` typed
at the call site with no per-call downcast, (b) uniform untyped iteration
over all registered slots for expiry/clear/pack, (c) ~120 memory types,
most mobs registering a small subset (Allay uses ~10).

Rejected: `HashMap<TypeId, Box<dyn Any>>` keyed by type. Runtime downcast
(`Any::downcast_ref`) at every read/write on a per-tick hot path, hashing
overhead, and it's easy to typo a key without the compiler catching it.
Rejected also because `Box<dyn Any>` erases the ability to cheaply enumerate
"what codec does this type have" for a future pack/unpack without a second
parallel map.

Rejected: a per-mob struct with named typed fields (e.g. `AllayMemory {
liked_player: Option<Weak<...>>, ... }`). Fast and fully typed, but breaks
uniform iteration (§2.2) entirely — `forEach`/`clearMemories`/`pack` would
each need per-mob hand-written visitor code, and shared behaviors
(`MoveToTargetSink` reading `WALK_TARGET`) couldn't be written generically
against "any brain that has a `WALK_TARGET` slot."

**Recommended**: zero-sized-type memory keys plus a fixed-size array of
type-erased slots, indexed by a compile-time constant, wrapped by a typed
accessor. Sketch (illustrative, not exhaustive — see §7 for the full
skeleton):

```rust
pub trait MemoryKey: 'static {
    type Value: Send + 'static;
    const NAME: &'static str;
}

pub struct AttackTarget;
impl MemoryKey for AttackTarget {
    type Value = EntityHandle; // see 4.4
    const NAME: &'static str = "attack_target";
}
```

Each concrete `MemoryKey` is registered once in a static registry (built
with `inventory` or a hand-maintained `const` array, decided at
implementation time — not blocking for the design) that assigns it a dense
`usize` slot index shared across all brains, mirroring how
`BuiltInRegistries.MEMORY_MODULE_TYPE` gives every `MemoryModuleType` a
stable identity in vanilla. The `Brain` struct then holds
`Vec<MemorySlot>` where `MemorySlot` internally stores `Option<Box<dyn Any +
Send>>` plus `ttl: Option<u32>`, sized to the *registered* subset for that
mob (not all ~120 — Allay registers ~10, matching vanilla's
`registerMemory` being called only for memories actually referenced by that
mob's sensors/behaviors, `Brain.java:78-89`). Typed access is:

```rust
impl Brain {
    pub fn get<K: MemoryKey>(&self) -> Option<&K::Value> { ... }
    pub fn set<K: MemoryKey>(&mut self, value: K::Value) { ... }
    pub fn set_with_expiry<K: MemoryKey>(&mut self, value: K::Value, ttl: u32) { ... }
    pub fn erase<K: MemoryKey>(&mut self) { ... }
    pub fn has_value<K: MemoryKey>(&self) -> bool { ... }
}
```

The one remaining downcast (`Box<dyn Any>` → `K::Value`) is internal to
these four methods and unreachable from behavior/sensor code, matching the
advisor guidance of keeping the downcast in exactly one place. This is not
zero-cost like a per-mob struct, but it is the only option that satisfies
uniform iteration without hand-written per-mob visitor boilerplate, which
is what makes shared behaviors like `MoveToTargetSink` portable at all.
Given ~120 memory types and most mobs registering under 15, the array is
small (a handful of pointer-sized `Option<Box<dyn Any>>` slots per mob) —
this is not the dominant cost of the system.

### 4.2 Concurrency — the memory store must not be taken out with the runtime

**This is the central Rust-specific finding of this document, and it
overrides the naive "just reuse the GoalSelector take/tick/put-back
pattern" answer.**

VERIFIED this session: Pumpkin has many non-AI-tick call sites that can
write to a living entity's state concurrently with (or interleaved between)
that entity's own tick — e.g. `damage`/`hurt` is invoked from
`crates/pumpkin/src/entity/projectile/{arrow,trident,fireball}.rs`,
`crates/pumpkin/src/block/blocks/{campfire,cauldron,magma,fire/mod.rs,plant/cactus.rs,plant/crop/sweet_berry_bush.rs}`,
`crates/pumpkin/src/entity/{item.rs,falling.rs,breath.rs,decoration/{end_crystal,painting,interaction}.rs}`,
`crates/pumpkin/src/block/entities/conduit.rs`, `crates/pumpkin/src/block/fluid/lava.rs`,
and others. Vanilla's equivalent — `HurtBySensor` populating `HURT_BY` /
`HURT_BY_ENTITY`, and (INFERRED from the existing warden anger port,
`warden.rs`) game-event/vibration code populating things like
`DISTURBANCE_LOCATION` — happens from arbitrary call sites at arbitrary
times relative to that mob's own `Brain.tick()`, not only from inside it.

If a Brain (memory map *and* behavior/sensor runtime together) is stored
behind one `std::sync::Mutex<Brain>` and taken out with `mem::take` for the
duration of the entity's tick (the `GoalSelector` pattern), every write to
that mob's memory that originates from a damage/game-event call arriving
during that window lands on the `Brain::default()` placeholder sitting in
the mutex and is silently lost. This is a correctness bug, not a
performance one, and it would be intermittent and hard to reproduce —
exactly the shape of failure CLAUDE.md's standing quality rules exist to
catch.

**Recommendation: split the Brain into two independently-locked halves.**

1. `MemoryStore` — the `Vec<MemorySlot>` from §4.1, behind its own
   `std::sync::Mutex<MemoryStore>` (or an `RwLock` if read contention from
   e.g. targeting code turns out to dominate; start with `Mutex`, the
   critical sections are all short field accesses). This lock is **taken
   for the duration of a single get/set/erase call only**, never held
   across an `.await`, and never `mem::take`n as a whole for ticking.
   External writers (damage handlers, game-event code, item pickup) call
   `entity.mob_entity.memory.lock().unwrap().set::<HurtBy>(...)` directly,
   exactly the way `mob_entity.breeding_cooldown` (an `AtomicI32`,
   VERIFIED pattern at `mob/mod.rs:944-946`) is written from both tick and
   non-tick code today. This satisfies the standing concurrency rule (never
   downgrade an atomic RMW into load/store, never hold a lock across
   `.await`) because there is no RMW here to downgrade — `MemorySlot` set/get
   are plain field writes under a short-lived guard.
2. `BrainRuntime` — sensors + the behavior priority/activity tables +
   per-behavior running state (`Behavior::Status`, timers) — behind
   `std::sync::Mutex<BrainRuntime>`, ticked with the exact
   take-with-`mem::take`-tick-unlocked-put-back pattern already proven for
   `GoalSelector` (`mob/mod.rs:967-985`). `BrainRuntime::tick` takes
   `&MemoryStore` (a short lock per access, not held across the whole tick)
   and produces reads/writes against it the same way vanilla's
   `Behavior`/`Sensor` methods call `body.getBrain().getMemory(...)`.

Net effect: `MemoryStore` is always live and accepting writes, exactly
matching vanilla where the memory map has no tick-scoped ownership window at
all (`Brain`'s `memories` field is just a plain `HashMap`, mutated
synchronously by whichever code calls `setMemory`). Only the *behavior
scheduling machinery* — which genuinely is single-owner during a tick in
vanilla too, since `Brain.tick()` is called once per entity per tick from
one thread — gets the take/tick/replace treatment. `BrainRuntime` must
implement `Default` for `mem::take` to work, matching `GoalSelector`'s
existing `Default` impl (`goal_selector.rs`, bottom).

This does mean two lock acquisitions instead of one on the fast path
(runtime tick takes `BrainRuntime`'s lock once, then acquires
`MemoryStore`'s lock per memory access inside behavior/sensor bodies). That
is the accepted cost of correctness here; do not "optimize" it by folding
memory back into the take-out structure without re-deriving this section.

### 4.3 Coexistence with the Goal system during migration

Both systems must run side by side for a long period — most mobs stay
Goal-based indefinitely (Pumpkin's own Goal port is large and correct for
non-Brain mobs; vanilla itself keeps plain `Mob`/`PathfinderMob` +
`Goal`/`GoalSelector` for most hostiles). The boundary:

- `MobEntity` gains two new optional fields, `memory: Option<Mutex<MemoryStore>>`
  and `brain: Option<Mutex<BrainRuntime>>` (or, cleaner, a single
  `Option<Brain>` newtype wrapping both halves — the `Option` is `None` for
  every mob that hasn't been migrated). `Mob::tick` (`mob/mod.rs:934-999`)
  gains one `if let Some(brain) = &mob_entity.brain { ... }` block
  positioned exactly where the existing goal/target selector tick calls
  are, ticking the memory-expiry + sensor + behavior-start + behavior-tick
  sequence from §2.1 for brain-having mobs, alongside (not instead of) the
  existing `goals_selector`/`target_selector` ticks.
- A Brain-based mob is still free to register a **reduced** `GoalSelector`
  for things vanilla itself still drives through the legacy Goal system on
  Brain mobs where applicable (in 1.21.4-era vanilla, e.g. villagers still
  use a couple of raw `Goal`s for panic/interact — UNVERIFIABLE for 26.2
  without a further read; flag as UNVERIFIED and re-check per-mob during
  migration rather than assuming). The two selectors are independent;
  neither needs to know the other exists. This mirrors vanilla's own
  `Mob` class holding both `goalSelector` and `brain` fields simultaneously
  (INFERRED from `Brain<E extends LivingEntity>`'s generic bound including
  ordinary `Mob` and from `MoveToTargetSink extends Behavior<Mob>` targeting
  the same `Mob` base class that owns goal selectors in earlier vanilla
  versions — a stronger citation should be pulled from `Mob.java` before
  this specific claim is used to justify code, hence INFERRED not VERIFIED).
- Nothing about `Navigator`/`MoveControl`/`LookControl` (the existing
  `std::sync::Mutex`-guarded controllers in `MobEntity`, `mob/mod.rs:73-78`)
  needs to change. `MoveToTargetSink`'s Rust equivalent is a `Behavior` that
  reads `WalkTarget` from `MemoryStore` and calls into the same `Navigator`
  Goal-based mobs already use — this is the concrete mechanism by which
  "Brain memories drive movement" cashes out in this codebase, matching
  §2.3's citation.

### 4.4 Entity references inside memories

Vanilla memories routinely hold live entity references (`LivingEntity`,
`Player`, `ItemEntity`, `Hoglin`, `List<LivingEntity>`, ...). Pumpkin's
equivalent is `Arc<dyn EntityBase>`. `mob_entity.target` already stores one
strong `Arc<dyn EntityBase>` in a `tokio::sync::Mutex` (`mob/mod.rs:76`),
but one precedent slot on one field is not precedent for ~20 memory types
that can each hold entity references, refreshed every 20 ticks by sensors
(§2.4). Storing strong `Arc`s in all of them means a mob can keep a fully
despawned/unloaded entity alive indefinitely if a stale memory is never
overwritten (e.g. `ATTACK_TARGET` on a mob that stops being re-sensed
because it left render distance) — a resource leak with no vanilla
equivalent, since Java GC does not have this failure mode the same way.

**Recommendation**: store `Weak<dyn EntityBase>` (or, if the codebase's
entity lookup is keyed by ID/UUID rather than object identity — verify
against `crates/pumpkin/src/entity/mod.rs` before implementing — an
`EntityId`/`Uuid` resolved through the world's entity registry at read
time) in every entity-valued memory slot. Reads that need the live entity
call `.upgrade()` (or the registry lookup) and treat a failed resolution
identically to vanilla's own "entity left the world" case, which sensors
already handle by simply not re-populating the memory (§2.4: sensors
refresh every 20 ticks, so bounded staleness up to 20 ticks is already
vanilla-correct behavior, not a Rust-specific compromise). This keeps
memory a Rust-idiomatic weak-reference pattern instead of fighting the
borrow/lifetime system with strong references that must be manually
invalidated.

### 4.5 What is deliberately deferred

- **Schedule (`EnvironmentAttribute<Activity>` / `EnvironmentAttributeSystem`,
  `Brain.java:43,327-335`)**: not read this session, no Pumpkin analogue.
  `updateActivityFromSchedule` is one call site; every mob can instead call
  `setActiveActivityIfPossible`/`setActiveActivityToFirstValid` directly
  (both fully specified, VERIFIED, `Brain.java:297-303,337-344`) the way
  `AllayAi::updateActivity` does (§2.6) — no mob needs a real day/night
  schedule to have a working brain. Villager migration specifically needs
  this and should not be attempted until schedule is designed separately.
- **`MemoryModuleType.PATH`**: couples memory storage directly to
  `net.minecraft.world.level.pathfinder.Path` internals (`MoveToTargetSink`
  stores/reads a live `Path` object, not just a target). Hold this memory
  type (and hence `MoveToTargetSink`'s exact vanilla shape) out of stage 1;
  a simplified variant that recomputes a path each start without persisting
  it across ticks is an acceptable stage-1 substitute — flag the deviation
  in the code comment.
- **POI/village/gossip/raid systems**: villager and piglin-adjacent brains
  depend on these; none exist in Pumpkin (INFERRED from CLAUDE.md's own
  parity notes referencing PARITY.md's "village 7" bucket as unaddressed).
  Villager is explicitly not the first migration target (§6).
- **Warden's 7-activity ladder**: `warden.rs`'s own comment already
  demonstrates why this is not a good first slice — DIGGING ends in entity
  removal in vanilla and an under-specified port would cause visible
  regressions. Attempt only after the core Brain machinery has proven
  itself on 2-3 simpler mobs.

## 5. Recommendation summary (rejected alternatives named inline)

| Question | Recommendation | Rejected alternative(s) |
|---|---|---|
| Memory storage | ZST typed keys + array-of-`Option<Box<dyn Any>>` slots, typed accessors on `Brain` | `HashMap<TypeId, Box<dyn Any>>` (hot-path hashing + no locality); per-mob typed struct (breaks uniform iteration needed for expiry/clear) |
| Concurrency | Split `MemoryStore` (always-live, short-lock, no `.await` inside) from `BrainRuntime` (take/tick/replace like `GoalSelector`) | Single `Mutex<Brain>` taken whole for the tick (silently drops external writes from damage/game-event code arriving mid-tick) |
| Behavior arbitration | No `Controls`-style exclusivity; behaviors run whenever their memory-status entry condition is satisfied and their activity is active, exactly per `Brain.java:409-424` | Reusing `GoalSelector`'s per-control exclusivity for behaviors (wrong semantics, would suppress legitimate concurrent behaviors) |
| Entity refs in memory | `Weak<dyn EntityBase>` or ID-resolved-at-read | Strong `Arc` in every slot (leak risk with no vanilla equivalent) |
| Coexistence | `Option<Brain>` on `MobEntity`, ticked alongside (not instead of) existing goal/target selectors | Forcing every mob through one unified system before the port is proven |
| Schedule | Deferred; `setActiveActivityIfPossible`/`ToFirstValid` called directly, no `EnvironmentAttribute` port | Blocking the whole design on porting schedule first |

## 6. Vertical slice

**Allay** (§2.6) is the recommended first migration target: 2 activities,
~7 mob-specific behavior instances plus 4 shared-infrastructure ones, no
POI/village/schedule dependency, and Pumpkin already has a working ad-hoc
port (`allay.rs`) to diff the new Brain-driven behavior against for
regression checking. Camel is the next-smallest candidate (5 distinct
memory/sensor/activity symbols by rough grep count, §"read follow-ups"
below) but was not read in full this session — treat its suitability as
UNVERIFIED pending a read of `CamelAi.java`.

The slice proving the design end-to-end means: `MemoryStore` +
`MemoryKey` machinery works for a handful of concrete keys (`LikedPlayer`,
`LikedNoteblockPosition`, `LikedNoteblockCooldownTicks`,
`NearestVisibleWantedItem`, `WalkTarget`, `Path`-or-its-stage-1-substitute,
`AttackCoolingDown`-equivalents as needed); one working `Sensor` port
(`NearestItemSensor` equivalent); the CORE+IDLE two-activity structure with
real `activityRequirementsAreMet` gating; and at minimum `Swim`,
`AnimalPanic`, `LookAtTargetSink`, `MoveToTargetSink` (simplified, no
persisted `Path` per §4.5) as reusable `Behavior` ports, since every future
mob needs these four. Getting `MoveToTargetSink`-equivalent driving the
existing `Navigator` is the one non-negotiable proof point — if brain
memory can't actually move the mob through the existing controller stack,
the design has failed regardless of how clean the memory/activity machinery
looks.

## 7. Minimal type-level skeleton (illustrative only, no mob migration)

This is intentionally small and lives here as fenced Rust, not as files
under `crates/pumpkin/src/entity/` (out of scope for this session per the
task's file-boundary rule). It shows the shape from §4.1/§4.2, not a
complete implementation.

```rust
// --- memory/mod.rs (sketch) ---

use std::any::Any;

/// A zero-sized typed key into a Brain's memory store. One impl per
/// vanilla MemoryModuleType this port covers (MemoryModuleType.java:34-152).
pub trait MemoryKey: 'static {
    type Value: Send + 'static;
    /// Dense index into MemoryStore::slots, assigned once per key at
    /// registration time (implementation detail: build-time const or a
    /// lazily-initialized registry — not fixed by this design).
    const SLOT: usize;
    const NAME: &'static str;
}

struct MemorySlot {
    value: Option<Box<dyn Any + Send>>,
    /// None == never expires (MemorySlot.java: NEVER_EXPIRE / Long.MAX_VALUE).
    ttl: Option<u32>,
}

impl MemorySlot {
    const fn empty() -> Self {
        Self { value: None, ttl: None }
    }

    /// MemorySlot.tick() (MemorySlot.java:16-24).
    fn tick(&mut self) {
        if let Some(ttl) = self.ttl {
            if ttl == 0 {
                self.value = None;
                self.ttl = None;
            } else {
                self.ttl = Some(ttl - 1);
            }
        }
    }
}

/// Always-live memory half of a Brain. Never `mem::take`n as a whole;
/// individual get/set calls take a short-lived lock only. See design
/// doc section 4.2 for why this must not be bundled with BrainRuntime.
pub struct MemoryStore {
    slots: Vec<MemorySlot>, // sized to this mob's registered subset
}

impl MemoryStore {
    pub fn get<K: MemoryKey>(&self) -> Option<&K::Value> {
        self.slots[K::SLOT]
            .value
            .as_ref()
            .and_then(|v| v.downcast_ref::<K::Value>())
    }

    pub fn set<K: MemoryKey>(&mut self, value: K::Value) {
        self.slots[K::SLOT] = MemorySlot { value: Some(Box::new(value)), ttl: None };
    }

    pub fn set_with_expiry<K: MemoryKey>(&mut self, value: K::Value, ttl_ticks: u32) {
        self.slots[K::SLOT] = MemorySlot { value: Some(Box::new(value)), ttl: Some(ttl_ticks) };
    }

    pub fn erase<K: MemoryKey>(&mut self) {
        self.slots[K::SLOT] = MemorySlot::empty();
    }

    pub fn has_value<K: MemoryKey>(&self) -> bool {
        self.slots[K::SLOT].value.is_some()
    }

    /// Brain.forgetOutdatedMemories (Brain.java:397-399).
    pub fn tick_expiry(&mut self) {
        for slot in &mut self.slots {
            slot.tick();
        }
    }
}

// --- behavior/mod.rs (sketch) ---

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum BehaviorStatus { Stopped, Running }

/// Rust analogue of BehaviorControl<E> (BehaviorControl.java). Deliberately
/// has no Controls-style exclusivity field — arbitration is memory-gated,
/// per section 2.1 and 4.2's rejected-alternative note.
pub trait Behavior: Send {
    fn status(&self) -> BehaviorStatus;
    /// Behavior.hasRequiredMemories + checkExtraStartConditions
    /// (Behavior.java:41-52,96-104), fused for the Rust port.
    fn try_start(&mut self, memory: &mut MemoryStore, game_time: i64) -> bool;
    fn tick_or_stop(&mut self, memory: &mut MemoryStore, game_time: i64);
    fn stop(&mut self, memory: &mut MemoryStore, game_time: i64);
}

/// Runtime half of a Brain: sensors + activity/behavior tables + per-
/// behavior running state. Ticked with the take/tick/replace pattern
/// already proven for GoalSelector (mob/mod.rs:967-985). Must impl
/// Default for mem::take to work.
#[derive(Default)]
pub struct BrainRuntime {
    // sensors: Vec<Box<dyn Sensor>>,
    // behaviors_by_priority: BTreeMap<u32, Vec<(ActivityId, Box<dyn Behavior>)>>,
    // active_activities: HashSet<ActivityId>,
    // ... mirrors Brain.java:40-49 minus the memory map itself.
}

impl BrainRuntime {
    /// Brain.tick (Brain.java:384-389), memory expiry pulled out to
    /// MemoryStore::tick_expiry so the caller can sequence the two
    /// independently-locked halves.
    pub fn tick(&mut self, memory: &mut MemoryStore, game_time: i64) {
        memory.tick_expiry();
        // self.tick_sensors(memory, game_time);
        // self.start_each_non_running_behavior(memory, game_time);
        // self.tick_each_running_behavior(memory, game_time);
    }
}
```

## 8. Staged plan and sizing

Sizing is rough order-of-magnitude effort for one experienced contributor
familiar with both codebases, not calendar time with review/CI overhead
factored separately.

**Stage 0 — Memory + slot machinery, no behaviors.** Implement
`MemoryKey`/`MemoryStore` (§4.1, §7) as a standalone module with unit tests
covering set/get/erase/expiry/`has_value` against ~10 concrete keys needed
for Allay. No entity integration yet. *Size: small (a few hundred lines +
tests).* Risk: the slot-indexing scheme (const generic vs. runtime registry)
is easy to over-engineer; keep it to the minimum that supports Stage 1's
mob before generalizing to all ~120 keys.

**Stage 1 — BrainRuntime skeleton + `Option<Brain>` wiring into
`MobEntity`/`Mob::tick`.** Land the split-lock structure from §4.2, the
`Behavior` trait from §7, and the coexistence wiring from §4.3, with zero
concrete behaviors — just prove the take/tick/replace sequencing compiles
and doesn't deadlock against the existing goal/target selector locks in the
same tick. *Size: small-medium.* Risk: lock-ordering interaction with the
existing `navigator`/`target`/`look_control`/`move_control` mutexes in the
same `tick()` (`mob/mod.rs:934-999`) — must acquire/release in a consistent
order to avoid a deadlock that only shows up under concurrent entity
ticking; write a regression test that ticks two mobs referencing each
other's memory concurrently.

**Stage 2 — Allay vertical slice.** Port `Swim`, `AnimalPanic`,
`LookAtTargetSink`, `MoveToTargetSink` (simplified per §4.5), one
`NearestItemSensor`-equivalent, and the Allay-specific behaviors
(`GoToWantedItem`, `GoAndGiveItemsToTarget`, `StayCloseToTarget`,
`SetEntityLookTargetSometimes`, a `RunOne` combinator). Two activities
(CORE, IDLE), matching §2.6 exactly. *Size: medium.* Risk: `MoveToTargetSink`
is the single highest-risk component — it is the only bridge to the
existing `Navigator`, and its vanilla implementation has real state
(`remainingCooldown`, `lastTargetPos`, path-recompute-on-drift) that is easy
to under-port into something that looks right but doesn't recompute paths
correctly; test against a live server with RCON per the standing
verification rig, not static review alone.

**Stage 3 — Second and third mob (Camel, Frog or Goat).** Prove the
machinery generalizes beyond Allay: more activities, more sensors, at least
one behavior requiring `RunOne`/`GateBehavior`-style combinators not needed
by Allay. *Size: medium per mob*, expected faster than Stage 2 since
infrastructure (Swim/AnimalPanic/MoveToTargetSink/LookAtTargetSink) is
reused. Risk: `GateBehavior`'s weighted-random ordering
(`ShufflingList`, seen but not read this session — UNVERIFIED, read before
implementing) may expose a gap in the §7 `Behavior` trait shape that Stage
1/2 didn't need; expect at least one trait-signature revision here.

**Stage 4 — Warden, or begin schedule design for villager.** Explicitly the
first point where either (a) a multi-activity ladder with real
priority-ordered concurrent behaviors (warden's EMERGE/DIG/ROAR/FIGHT/
INVESTIGATE/SNIFF/IDLE) or (b) the `EnvironmentAttribute<Activity>` schedule
subsystem (villager) must be tackled. Both are substantial, independent
pieces of work; do not start either until Stage 3 has shipped and been
verified live. *Size: large, and not fully scoped by this document* — a
follow-up design pass is needed once schedule's vanilla source has actually
been read (§4.5).

Total rough sizing through a working, live-verified Allay (Stages 0-2):
medium — comparable to one substantial multi-file PR, not a rewrite.
Stages 3-4 are open-ended and should each get their own sizing pass once
the prior stage's actual friction is known, per CLAUDE.md's general
preference for re-deriving rather than projecting.

## 9. Regression budget

Every stage above touches `crates/pumpkin/src/entity/mob/mod.rs` (the
shared `tick()` for *all* mobs, Goal-based or not) and, from Stage 2
onward, one already-shipped ad-hoc port (`allay.rs`) that must not regress
while it's being replaced. The two behaviors most plausibly broken by this
work: (1) any Goal-based mob's tick timing, if the new `Option<Brain>`
branch is misplaced relative to the existing selector/navigator
take-tick-replace sequence in `Mob::tick` — checked by running the existing
Goal-based mob test/verification suite unchanged before and after Stage 1
lands; (2) Allay's already-correct jukebox/noteblock interaction and
duplication logic (`allay.rs`'s own comment lists these as concretely
ported) — checked by keeping those code paths untouched until Stage 2's
Brain port is verified live via RCON to reproduce the same noteblock
following/depositing behavior, then removing the old ad-hoc path in the
same PR that adds the Brain path, not before.
