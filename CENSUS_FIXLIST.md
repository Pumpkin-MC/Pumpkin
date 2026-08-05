# Follow-up items found during fresh census audit (2026-08-04)

Master at dd9849d as of this note. A 122-unit fresh stratified behavioral-fidelity
sample (independent of the original 27-chunk census) found these real bugs; five
were already fixed and merged (Mirror LeftRight/FrontBack swap, DolphinJumpGoal
should_continue logic, MerchantMenu swapped-slot trades, Sheep grass-eating age-up,
Jukebox song end-padding — fixed at `b560f26`). This file tracks what's left.

## Newly found, not yet fixed

- ~~SheepEntity NBT corruption~~ **fixed** (pre-existing, earlier this session):
  `write_nbt`/`read_nbt_non_mut` now go through `LivingEntity::write_nbt` plus
  `write_ageable_nbt`/`write_animal_nbt`, matching pig/cow/chicken.
- ~~Sheep::shear() velocity jitter is ~4x narrower than vanilla~~ **fixed** (`a3cac37`):
  `shear_drop_velocity` now computes base+jitter matching vanilla's combined range.
- ~~Sheep wool drop Y-offset~~ **fixed** (`372d2b5`): drops now spawn at `Y + 1.0` via
  `drop_spawn_pos`, matching `Entity.spawnAtLocation`'s offset argument.
- ~~MerchantMenu offer stock gating~~ **fixed** (`08f5a8e`): `offer_matches` now
  disqualifies `uses >= max_uses` offers. `is_disabled` deliberately NOT used as the
  gate — it's actually `!rewardExp`, not vanilla's out-of-stock concept.
- ~~MerchantOffer wire-protocol out-of-stock byte~~ **fixed** (`372d2b5`): the wire
  writer now sends `uses >= max_uses` in the slot vanilla uses for `isOutOfStock()`,
  instead of the unrelated `is_disabled`/`!rewardExp` flag.

## Other real gaps this fresh sample surfaced (not yet dispatched)

- MyceliumBlock — no Rust implementation at all (STUB, no file)
- ~~Monster (entity base) — no monster-spawn-darkness-gating port anywhere~~
  **already fixed** pre-existing (commit `afbe839d`, issue #2424) — verified against
  `Monster.java` during a later audit pass, no changes needed.
- ~~MagmaCube — bare Slime delegate, zero magma-specific behavior~~ **fixed** (`57388d3`):
  distinct hurt/jump/squish sounds, 4x jump delay, 0.9 squish decay, attack-damage +2 and
  armor=size*3 attribute scaling, fire-render suppression. Deferred: lava-jump extra
  velocity and size-scaled jump-from-ground boost, both needing a liquid-jump physics hook
  this codebase doesn't have for any mob yet.
- Marker entity — no implementation at all (STUB)
- ~~EntitySelector — `pushableBy`'s team/CollisionRule gating entirely absent~~
  **fixed** (`0557881`): `collision_rule_permits_push` ports the exact vanilla predicate
  (verified against `EntitySelector.java:29-56` directly), wired into every
  `push_entities` call site; pusher-side team lookup hoisted out of the per-candidate
  loop, whole check skipped when no teams exist server-wide.
- ~~Bogged — bare skeleton wrapper, no shearing/poison-arrow/sound overrides~~ **fixed** (`6802646`): shearing/mushroom-drop/poison-arrow/hard-attack-interval.
- ~~Husk — bare zombie wrapper, no sun-immunity/water-conversion/hunger-hit~~ **fixed** (`6802646`): hunger-on-hit, underwater-to-zombie conversion timer. Sun-immunity needed no change (already tag-driven). Baby-scale and eye-in-fluid detection still approximated/deferred.
- ~~Endermite — no despawn timer (life/MAX_LIFE=2400)~~ **fixed** (`6802646`): despawn timer + `MeleeAttackGoal` followEvenIfNotSeen fix. yBodyRot lock still not ported.
- ~~Silverfish — SilverfishMergeWithStoneGoal and SilverfishWakeUpFriendsGoal both
  entirely missing~~ **fixed** (`f8d96fc`): both goals implemented, extra
  LookAtEntity/RandomLookAround goals removed. Deferred: infested-block-broken-by-player
  still spawns a bare Entity with no AI (separate bug in `infested.rs`), and
  `getWalkTargetValue`'s stone-pathing bias has no counterpart.
- Giant — has AI goals (swim/melee/wander/target) that vanilla Giants don't have
  at all (vanilla giants are stationary); a behavioral over-implementation bug
- MaceItem — fall-damage-bonus formula is a flat `1.5 * fall_distance` instead of
  vanilla's tiered formula (diverges sharply: fd=8 gives 12 in Rust vs 22 in
  vanilla); also missing the unconditional knockback AoE on smash attack
- ~~AnvilBlock — falls through to generic FallingBlock with no entity-crushing
  damage, no landing/breaking sound events, no chip/damage progression~~ **fixed**
  (`0557881`): crushing damage on landing, chip/damage/destroy tier progression,
  land/break sound events, all as pure-helper-plus-orchestration matching
  `FallingBlockEntity.causeFallDamage`/`AnvilBlock`.
  SculkCatalystBlockEntity, EndCrystal (NBT/tick), DecoratedPotBlock,
  ChorusFlowerBlock growth, SpawnerBlock XP drop, BrewingStandBlock inventory
  drop-on-break, IceBlock playerDestroy, several bonemeal gaps (BushBlock,
  FlowerBedBlock, GrassBlock, CaveVines harvest-by-click) — see full chunk
  transcripts in conversation history for citations.

Sculk roadmap: steps 1-3 (MultifaceBlock framework, SculkVeinBlock, SculkBehaviour
growth math) are landed and inert. Step 4 (SculkSpreader cursor-driving + catalyst
XP-consumption wiring) is still not started — this is what would make sculk
actually spread in a running server.

## Found live on the NUC test server (2026-08-04)

- Glow Squid — no AI at all observed live (just hangs motionless in place, no
  flee/wander/light-particle behavior). Not yet audited against
  `GlowSquid.java`/`Squid.java`; needs its own pass since it overrides several
  of Squid's goals (no flee-from-player, plus the ambient glow-ink-cloud-on-death
  is squid-specific too and should be checked).
- Regular Squid appears entirely absent from ocean spawns live — only Glow
  Squid observed spawning, in daylight surface ocean where vanilla Glow Squid
  should require dark/deep-water conditions (`GlowSquid.checkGlowSquidSpawnRules`
  gates on light level and Y, doesn't just replace Squid). Suggests either
  Squid's own spawn entry is missing/broken, or GlowSquid's spawn predicate is
  missing its light/depth gate and is over-spawning into Squid's niche. Needs
  a spawn-rule audit of both together, not just an AI-goals port. Dispatched
  as part of the aquatic-mobs-batch3 agent below.
- Floating entities above water surface observed live (a Drowned and what
  looked like a loot chest, both hovering well above the water line instead of
  sitting in/on it — see screenshot in conversation). Possibly a buoyancy/swim
  vertical-motion bug shared across water-adjacent entities, or a chest-minecart
  /shipwreck-loot-entity placement bug unrelated to physics. Not yet
  root-caused. Dispatched as part of the aquatic-mobs-batch3 agent below.
- Chunk loading falls behind player flight speed — client outruns chunk
  generation/send pipeline, ends up looking at unloaded/edge-of-world chunks.
  Also produces a hard disconnect ("Connection Lost — Network Protocol
  Error") when chunks load fast enough, suggesting an actual packet-encoding
  race/bug under high chunk-send throughput, not just a perf/priority issue.
  Dispatched to agent on branch `chunk-load-pace-work` (2026-08-04).
- World appears to be overwhelmingly ocean — flew a long distance before
  finding any landmass at all on a fresh seed (1784236824834363978). Strong
  enough signal (not just "this seed happens to be oceanic") to warrant a
  dedicated continentalness/biome-distribution audit against vanilla's
  overworld noise router before assuming it's normal seed variance. A
  follow-up screenshot on the same seed showed an oddly massive, perfectly
  flat sand beach (with thin disconnected grass strips cutting through it)
  where vanilla beaches are normally narrow — consistent with the same
  erosion/continentalness-threshold hypothesis rather than a separate bug.
  Not yet dispatched (holding off on a 3rd concurrent worktree agent for
  memory reasons; pick up once mob-spawning-audit/aquatic-mobs-batch3 land).
- Ocean Monument generation does not match vanilla's shape (screenshot shows
  a plain flat prismarine wall, not the vanilla stepped/tiered pyramid
  monument structure). Needs its own audit against
  `OceanMonumentPieces`/`OceanMonumentStructure` (structure NBT/piece
  assembly, not just block palette). Not yet dispatched.
- Shipwreck generated fully on dry land (sand beach, no water anywhere
  nearby) instead of being ocean/beach-only. Needs a placement-condition
  audit against vanilla's `Shipwreck.java` heightmap/biome placement checks.
  Not yet dispatched.
- No villages found anywhere during live testing (user report, extensive
  exploration). Given the same session also found no Ocean Monuments
  shaped correctly and Shipwrecks placed wrong, this may be a broader
  structure-placement/start-check issue rather than village-specific — or
  simply that oversized oceans (see above) are pushing villages further
  apart than normal. Needs its own structure-spacing/placement audit,
  probably alongside the ocean-monument and shipwreck items. Not yet
  dispatched.
- No Pandas/Ocelots observed in jungle biomes, AND hostile mobs not
  spawning in dark areas (caves/night) — folded into the mob-spawning-audit
  agent already in flight (aa32d066a921fbdd9), since this suggests the
  bug is broader than just passive-land-mob spawning: likely all
  land-based natural spawning (CREATURE + MONSTER + AMBIENT categories),
  while aquatic spawning (confirmed via Glow Squid) still works.
- Kelp observed floating in open water with no supporting block underneath —
  vanilla kelp requires a solid block (or another kelp block) directly below
  to exist at all. Needs checking both worldgen kelp placement and the
  runtime neighbor-update-triggered break/drop logic (KelpBlock likely
  missing its `canSurvive`-equivalent check, or the check exists but isn't
  enforced during structure/feature placement). Not yet dispatched.
- Ruined Nether Portal generated underwater with what looks like a fully
  intact, undamaged frame — vanilla ruined portals are always partially
  destroyed (random block replacement/decay), never a complete portal. Worth
  checking whether Pumpkin's ruined portal generation applies the
  degradation pass at all, or only for certain variants. Not yet dispatched.
- Silverfish pathfinding "doesn't feel like vanilla" per live testing (user
  report, no specifics yet — silverfish spawning and its AI goal set were
  otherwise confirmed reasonably close). Needs a closer look at
  `Silverfish.java`'s `getWalkTargetValue`/pathing-through-stone bias
  (already flagged as unported in the earlier Silverfish AI fix note above)
  and general navigator behavior, not just the goal list. Not yet dispatched.

## Fixed this session (not yet in the numbered gap tracker)

- `use_item_on`'s generic post-use check played the tool-break sound/status
  whenever ANY held stack emptied out (not just real durability breakage) —
  hit spawn eggs on their last use, likely also the last block placed from a
  stack. Fixed: removed the redundant, wrongly-gated broadcast; real tool
  breakage already emits this correctly elsewhere (`DamageResult::Broken` in
  `Player::damage_item_in_slot`). Commit `e261c8c`.
- Glow Lichen placed `Block::GLOW_LICHEN.default_state` unconditionally
  whenever ANY neighbour was non-air (including bamboo/leaves/other
  non-full blocks) - default_state has every multiface direction flag
  false, so it rendered as an invisible/floating block with no visible
  attachment regardless of which neighbour "qualified" it, matching the
  live "lichen on the surface and on air" report exactly. Fixed: requires
  an air/water origin and a sturdy-face neighbour (matching vanilla's
  `MultifaceGrowthFeature.place`), and sets the specific face flag toward
  the qualifying neighbour instead of a bare default state. Commit `22a6048`.
- URGENT/live-crashing: entering the Nether panicked chunk generation on
  every single chunk ("index out of bounds: the len is 32768 but the
  index is 32768"). Root cause: `Chunk::build_level_sections` sized its
  section-count loop off `Dimension::THE_NETHER.height` (256) instead of
  the ProtoChunk's actual generation height (128, from
  `GenerationSettings::NETHER.shape`) - the exact same Dimension-vs-
  GenerationSettings conflation `845326f` fixed in `ProtoChunk::new`, in
  a sibling call site that fix never touched. Fixed to use
  `proto_chunk.height()`/`bottom_y()` throughout; added a regression test
  driving Nether generation to Full/Level (no prior test in this file
  exercised any non-Overworld dimension past Features/Spawn). Commit
  `a93054d`.

## Feature requests

- `/locate` command not implemented (structure/biome location for players).
  Not yet dispatched.

## Queued campaigns (user-requested scope, not yet dispatched)

- Full mob-AI vanilla-parity pass across the entire roster (explicit user
  request 2026-08-04) — this is the standing goal the many `*-ai-work`
  branches in this tracker/PARITY.md already serve; treat any new mob found
  missing goals/targeting/behavior during live testing as in-scope.
- Nether Update-specific audit (explicit user request 2026-08-04): Piglin
  bartering/zombification/reverting, Hoglin/Zoglin, Strider + warped fungus
  on a stick, Soul Speed/Soul Sand Valley soul-sand-and-soil interactions,
  Basalt Deltas, Crimson/Warped Forest fungus features, Respawn Anchor
  (already touched once this session via PR #2754, now superseded by the
  stop-upstream-contributions instruction — needs re-verifying on master
  directly, not assumed fixed), Lodestone, Target Block, Netherite tools'
  fire-immune item-drop behavior, ancient debris generation. Not yet
  dispatched — queue after the current mob-spawning/aquatic-mobs backlog
  clears.

## Village/POI audit (2026-08-04)

PARITY.md §1 calling the village/POI system "missing entirely" is **stale**. Direct
inspection of current `village-poi-audit-work` (branched from master @ `a93054d`) found
it already largely landed, apparently across two prior sessions not reflected back into
PARITY.md's summary line:

- `pumpkin/src/world/village_poi.rs` + `World::{poi_count_in_range, sections_to_village,
  is_close_to_village, acquire_poi, release_poi}` (`pumpkin/src/world/mod.rs`): real POI
  registry backed by the existing region-file `PoiStorage`, Chebyshev section-distance
  BFS, true 3D sphere range filter, ticket-based bed claiming (`Occupancy::HasSpace` vs
  `IsOccupied`), bed head/foot POI-position filtering. Cited against `PoiManager.java`,
  `PoiRecord.java`, `PoiTypes.java`, `ServerLevel.isCloseToVillage`.
- `CatSpawner` (`world/custom_spawners.rs`) already uses the real two-part vanilla gate:
  `is_close_to_village(pos, 2)` then `poi_count_in_range(HOME, pos, 48, IsOccupied) > 4`.
- `GossipContainer` (`entity/passive/villager/gossip.rs`): full port of vanilla's
  `GossipContainer.java` — weighted reputation sum, decay, add/merge/clamp, and even
  `transfer_from` (vanilla's `transferFrom`/`selectGossipsForTransfer`), though the last
  one is implemented-but-never-invoked (see below).
- Reputation-event witness propagation, the specific thing PARITY.md flagged as missing,
  is **also already there** for the one vanilla event type that's witness-based:
  `VillagerEntity::on_mob_death` (`entity/passive/villager/mod.rs`) ports
  `Villager::tellWitnessesThatIWasMurdered` (`Villager.java:615-624`), scanning a 16-block
  box (vanilla: brain's `NEAREST_VISIBLE_LIVING_ENTITIES` sensor) and applying
  `MAJOR_NEGATIVE` gossip on every nearby villager. `TRADE` (self-only) and
  `VILLAGER_HURT` (self-only) are also wired, matching vanilla's own scope (vanilla never
  broadcasts those two to witnesses either — only `VILLAGER_KILLED` does).

Real gaps found and fixed this pass (branch `village-poi-audit-work`):

- `ZombieVillager::finish_conversion` was missing the `ZOMBIE_VILLAGER_CURED` reputation
  event entirely (`ZombieVillager.java:259-262` → `Villager.java:855-858`: MAJOR_POSITIVE
  20 + MINOR_POSITIVE 25 against the curing player). The old code's own comment claimed
  this needed "per-player conversion-credit infrastructure" that didn't exist — false,
  `conversion_starter` already tracks the curing player's UUID for the advancement
  trigger sitting two lines below. Also fixed while there: the gossip grant is now gated
  on the same "player still online" check the advancement trigger already used
  (`ZombieVillager.java:259`: `if (player instanceof ServerPlayer)`), not applied
  unconditionally whenever a starter UUID existed.
- Iron Golem village defense (`DefendVillageTargetGoal`, `IronGolem.java:75` /
  `target/DefendVillageTargetGoal.java`) did not exist at all — golems only had
  `RevengeGoal` + `NearestHostileTargetGoal`, never targeting a player regardless of
  reputation. New `entity/ai/goal/defend_village_target.rs` ports it: scans a
  `(10,8,10)`-inflated box around the golem for villagers and players, targets a player
  any nearby villager holds reputation ≤ -100 against. Required adding
  `IronGolemEntity::player_created` (`IronGolem.java`'s `PlayerCreated` NBT field, unset
  before this pass) since vanilla's `IronGolem.canAttack` never lets a player-built golem
  attack players regardless of reputation — `CarvedPumpkinBlock`-spawned golems now set
  it, village-spawned ones (`Villager::spawnGolemIfNeeded`) leave it false, matching
  vanilla's two spawn paths.

## Village/POI follow-up plan (not attempted this pass — scoped for a future session)

Each item below is independently sized; none require a Brain/Memory port, only a
Goal-based approximation following the precedent above.

1. **`MoveThroughVillageGoal`** (villager wander-through-village goal,
   `ai/goal/MoveThroughVillageGoal.java`) — doesn't exist. Needs `PoiManager.find`'s
   closest-first search over `#minecraft:village`-tagged, unvisited, `IS_OCCUPIED` POIs
   (`getInRange` + a `hasNotVisited` 15-entry ring buffer), `LandRandomPos.getPos` for an
   initial random walk target, and mid-path wooden-door detection to truncate the path at
   the door node. Moderate size — path/navigation-adjacent, not just a query.
2. **`GolemRandomStrollInVillageGoal`** (`ai/goal/GolemRandomStrollInVillageGoal.java`) —
   doesn't exist (Iron Golem currently uses plain `WanderAroundGoal`). Needs three target
   strategies weighted 30/49/21ish (towards-anywhere / towards-a-golem-wanting-villager /
   towards-a-random-POI-in-a-`sections_to_village == 0` section), and
   `Villager::wantsToSpawnGolem` (`golemSpawnConditionsMet` — slept within the last 24000
   ticks — plus `!GOLEM_DETECTED_RECENTLY`) which also doesn't exist yet.
3. **`MoveBackToVillageGoal`** (`ai/goal/MoveBackToVillageGoal.java`) — doesn't exist.
   Needs `BehaviorUtils.findSectionClosestToVillage` (search a 2-section-radius cube for
   the section with the lowest `sectionsToVillage`) — smallest of the three goal ports
   above, a good first pick.
4. **`VillageSiege`** (`ai/village/VillageSiege.java`) — a `CustomSpawner` implementation,
   entirely absent (no `village_siege` match anywhere in `pumpkin/src`). Needs
   `isBrightOutside`/clock-marker gating (1-in-10 chance nightly), `isVillage` +
   `#minecraft:without_zombie_sieges` biome-tag exclusion, and a 20-zombie ring-spawn
   sequence around a random player standing in a village. Self-contained, no dependency
   on the other three goals.
5. **POI backfill at chunk load** (vanilla `PoiManager.checkConsistencyWithBlocks`) —
   freshly-generated villages register zero POIs until a bed/job-site block is placed or
   broken post-generation, since worldgen writes chunk sections directly and bypasses
   `World::set_block_state`. Fixing this means hooking `pumpkin-world`'s chunk-load path,
   a different crate than everything else here — biggest single item on this list.
6. **Job-site and meeting-point POI claiming** — only `HOME` (bed) claiming exists
   (`VillagerEntity`'s rest logic). No profession ever claims a job-site POI, no villager
   ever claims a meeting point by ringing a bell, so `sections_to_village`'s non-`HOME`
   contribution to the `#minecraft:village` tag is permanently inert — a village with
   claimed beds registers correctly, but the job-site/meeting share of vanilla's distance
   metric never does. Prerequisite for goal 2's `sections_to_village == 0` check to be
   fully accurate.
7. **`GossipContainer::transfer_from`** — fully implemented and unit-tested
   (`gossip.rs`) but never called from anywhere. Vanilla invokes it from
   `Villager::gossip`, itself only reached via the brain's periodic villager-to-villager
   meet-and-gossip behavior, which Pumpkin has no scheduler for. Wiring this in requires
   *some* periodic "two nearby villagers meet" tick, not necessarily a full brain port —
   candidate for folding into the existing 20-tick `mob_tick` cadence already used for
   gossip decay in `VillagerEntity`.

## GameEvent emission audit (2026-08-04)

PARITY.md does not exist in any worktree on this machine (`ls /home/eshanki/pumpkin-wt/*/PARITY.md`
comes back empty; only a stray `PARITY.md.bak` with unrelated lighting/potions content is
present in this worktree) — the campaign-state note describing a "section 6" flagging most
GameEvent emission call sites as missing could not be found or cited. Direct inspection found
the premise stale regardless: the engine (`pumpkin/src/world/game_event/`) plus **23** of
vanilla's 46 non-`Resonate` `GameEvent` variants (from `pumpkin-data/src/generated/game_event.rs`)
were already emitted somewhere in `pumpkin/src` before this session — BlockActivate, BlockAttach,
BlockChange, BlockDeactivate, BlockDestroy, BlockDetach, BlockPlace, ContainerClose,
ContainerOpen, Drink, Eat, EntityAction, EntityDie, EntityDismount, EntityPlace, FluidPickup,
FluidPlace, JukeboxPlay, JukeboxStopPlay, NoteBlockPlay, ProjectileLand, Shear, Splash.
(There's a 743-commits-stale local/fork branch `pr/gameevent-emissions` that landed most of
this in an earlier session under different commit hashes; `git log --cherry-pick --right-only
master...pr/gameevent-emissions` shows nothing genuinely dropped, just patch-id drift from two
commits — no free re-land available.)

This pass (branch `gameevent-audit-work`) added 4 more, each a one-line insertion at an
existing state-change site with a precise vanilla citation:

- **BlockOpen/BlockClose** — doors (`doors.rs`: `toggle_door` player-click, `on_neighbor_update`
  redstone, `set_door_open` mob-AI path), trapdoors (`trapdoor.rs`: `toggle_trapdoor`,
  `on_neighbor_update`), fence gates (`fence_gates.rs`: `toggle_fence_gate`,
  `on_neighbor_update`). Cites `DoorBlock.java:208/220/233`, `TrapDoorBlock.java:122`,
  `FenceGateBlock.java:161/198`.
- **PrimeFuse** — `TNTBlock::prime` (`tnt.rs`, covers flint & steel/fire charge ignition,
  initial and post-place redstone power, and fire spreading onto TNT via `fire.rs:201`) and
  the creeper fuse-start tick (`creeper.rs::mob_tick`, the `fuse_speed > 0 && current == 0`
  branch). Cites `TntBlock.java:92`, `Creeper.java:136`.
- **Explode** — `Explosion::explode` (`explosion.rs`), placed as the very first statement in
  the function to match `ServerExplosion.java:236`, which fires before block/entity
  interaction so an explosion that destroys an occluding block is still heard by anything on
  the far side (verified by reading `ServerExplosion.explode()` in full, not assumed from the
  grep hit).

None of the touched files had pre-existing tests exercising these call sites, and no `--lib`
test anywhere in the crate constructs a `World`, so no regression test was added for the
emissions themselves — `cargo test -p pumpkin --lib` (758 tests) still passes unchanged,
confirming no behavior outside the new calls regressed.

### Remaining missing emissions (not attempted this pass) — 19 of 46

Ranked by what a sculk sensor/Warden would plausibly want to detect, each with the specific
architectural blocker found, not just "not done yet":

1. **Step / Swim / Flap / HitGround / Bounce** — the single highest-value gap (footsteps are
   the main thing a sculk sensor listens for) and *not* a safe one-liner. Vanilla routes all
   of these through `Entity.applyMovementEmissionAndPlaySound` → `vibrationAndSoundEffectsFromBlock`,
   gated by a `MovementEmission` enum and a per-tick clipped-movement queue
   (`Entity.java:795, 872-889, 991-1012, 1042-1049`) — pumpkin has no equivalent step-sound/
   movement-emission accumulator at all. Also: the engine's listener registry is a flat
   per-world `Vec` scanned on every `emit_game_event` call (documented in
   `game_event/mod.rs`'s module comment), not chunk-sharded like vanilla's
   `EuclideanGameEventListenerRegistry` — firing STEP on raw per-tick movement without that
   sharding is a perf hazard on top of the missing gate. Needs its own dedicated pass, not a
   guessed insertion.
2. **Equip / Unequip** — `LivingEntity::send_equipment_changes` (`living.rs:286`) is the
   correct single hook (~20 call sites across the crate, all already inside `async fn`s), but
   it's currently synchronous, receives only the new stack (vanilla's
   `LivingEntity.java:689-711` `onEquipItem` also needs the old stack to decide the sound, and
   gates on `!ItemStack.isSameItemSameComponents`), and `LivingEntity` stores no self
   `Arc<dyn EntityBase>`/`Weak` to build a `GameEventContext::of_entity(self)` from inside an
   `&self` method. Fixable, but needs that self-reference added first, not a stretch to force
   through this pass.
3. **ProjectileShoot** — vanilla's single hook is `Projectile.tick()`'s `hasBeenShot` gate
   (`Projectile.java:99-104`, fires exactly once on an entity's first tick after spawn).
   Pumpkin has no equivalent: each projectile type (`arrow.rs`, `snowball.rs`, `trident.rs`,
   etc.) reimplements `EntityBase::tick` from scratch, and `ArrowEntity::tick` never calls
   the generic `impl EntityBase for Entity` tick (confirmed by reading `arrow.rs:309-324`) —
   there is no shared first-tick hook to attach to. `Entity::age == 1` inside the generic
   tick (age is incremented once per tick in `world/mod.rs:1148` right before `tick()` runs)
   would reproduce vanilla's semantics exactly, but only once projectiles actually delegate to
   it, which none currently do.
4. **Shriek** — `pumpkin/src/block/entities/sculk_shrieker.rs` is a bare NBT-only stub (no
   sound, no Warden-summoning, no sensor wiring at all). This needs the mechanic implemented,
   not an emission added to it. Cites `SculkShriekerBlockEntity.java:124`.
5. **SculkSensorTendrilsClicking** — cites `SculkSensorBlock.java:218`; not checked against
   pumpkin's `sculk_sensor.rs` activation path this pass, so unknown whether it's a one-liner
   or needs more — flagged rather than guessed.
6. **Remaining, cited but not sized/attempted**: `EntityInteract` (`Entity.java:2318`,
   `Mob.java:1132`/`1141`, `Player.java:849`), `EntityMount` (`Entity.java:2461`),
   `EntityDamage` (`LivingEntity.java:1970`, `Player.java:768`, `VehicleEntity.java:48`,
   `ArmorStand.java:332`/`393`, `ItemEntity.java:301`), `Teleport` (`EnderMan.java:291`,
   `Shulker.java:404`, `TeleportRandomlyConsumeEffect.java:56`), `InstrumentPlay`
   (`InstrumentItem.java:65`), `ElytraGlide` (`LivingEntity.java:3200`),
   `ItemInteractStart`/`ItemInteractFinish` (`LivingEntity.java:3506`/`3621`,
   `FishingRodItem.java:40`/`59`, `BoneMealItem.java:42`/`53`), `LightningStrike`.

## Features/structures audit (2026-08-05)

Branched from master @ `b0c0651`. Task was to turn the earlier "102 Rust files vs 177
vanilla feature classes" and "45 vs 32 structure files" counts into a real per-item gap
analysis, then chase this session's live-testing reports (magma in rivers, oversized
beaches, wrong ocean monument shape, shipwreck on land, kelp floating, ruined portal
fully intact underwater).

### Premise correction: there is no 58%-style feature/structure count gap

`reference/vanilla/worldgen.md` (committed 2026-08-03, same campaign) already
established this and it still holds: `pumpkin-world/src/generation/feature/configured_features.rs`
has ~58-59 match arms against vanilla's 63 registered `Feature<C>` types, with foliage
placers, trunk placers, tree decorators, root placers, and feature-size types all at
full parity, and only 3 real vanilla carver classes (all covered). The 102-vs-177 and
45-vs-32 file counts compare Rust `.rs` files against vanilla **source files**, which
include `configurations/`, `foliageplacers/`, `trunkplacers/` etc. as separate files per
class on the vanilla side, and (on the structure side) miss that village/outpost/bastion/
trail-ruins/ancient-city all arrive through the shared jigsaw system and NBT templates
and never get their own structure `.rs` file. **Do not re-litigate this as a category
gap** - the real remaining work is individual feature/structure fidelity bugs, not
missing registrations.

### Decompiled vanilla source is currently unavailable - infra note for future sessions

`/tmp/pumpkin-vanilla-26.2/decompiled` (the path `CONFORMANCE.md`/`JAR_QUIRKS_26_2.md`
assume exists) is gone - `/tmp` does not persist across sessions and there is no
committed script to rebuild it. Checked whether it's cheap to regenerate:
`/home/eshanki/Downloads/server.jar` is a genuine 26.2 server jar (`version.json` id
"26.2", matches this campaign's target), but Mojang's version manifest for 26.2
(`https://piston-meta.mojang.com/v1/packages/4b74f58f68a2baae3547d5a20274079f29cafc06/26.2.json`)
has no `client_mappings`/`server_mappings` entry in `downloads` at all - only `client`
and `server` jar URLs. Without official mappings the jar decompiles to obfuscated names
only, useless for citation. Rebuilding this pipeline is out of scope for a single
session; if someone does rebuild it, please commit a script under `conformance/` so it
isn't lost again. Per CLAUDE.md, the wiki is an acceptable citation for placement-condition
questions ("does X require Y nearby") but not for algorithm-structure questions ("what
order are the RNG calls", "how are the pieces assembled") - several items below are
blocked on the latter.

### Fixed this session

- **Kelp head-cap bug** (`pumpkin-world/src/generation/feature/features/kelp.rs`): when
  a growing kelp column hit the water surface or an obstruction before its randomly
  rolled height, the code tried to cap the last-placed body segment with a proper aged
  `KELP` head by checking whether the block *below* the failing position was still
  `WATER` - but that position had just been overwritten with `KELP_PLANT` on the
  previous loop iteration, so the check could never pass. Net effect: any column cut
  short by the surface/an obstruction was left as a bare, uncapped `KELP_PLANT` stalk
  instead of ending in a head. This is a self-contained logic bug in Pumpkin's own
  algorithm (the code's own intent, provable by tracing state transitions, not a claim
  about vanilla's algorithm), so it didn't need decompiled-source access to fix safely.
  Extracted the decision into a pure `can_cap_with_head(below_id, below_below_id)`
  helper (now also accepts `KELP_PLANT`, alongside virgin `WATER`, as a valid support to
  cap) with unit tests, plus an integration regression test
  (`chunk_system::generation::tests::generated_kelp_columns_are_supported_and_capped`,
  seed 42 chunk (2,4), verified by direct probing to contain kelp) that generates a real
  chunk through the Features stage and asserts every kelp column both rests on solid
  ground/more kelp (not air/water) and ends in a `KELP` head, not a bare `KELP_PLANT`.
  Confirmed this test fails on the pre-fix code with exactly the predicted symptom
  (`kelp column ... ends at y=61 without a KELP head cap`).
  **Caveat**: this is a real, reproducible, now-fixed bug, but I could not conclusively
  connect it to this session's specific live-testing report of kelp "floating in open
  water with no supporting block underneath" - that symptom implies the column's *base*
  has nothing solid below it, which is a different failure mode from the one this fix
  addresses (a column that rests on real ground but lacks its head cap). The base-anchor
  logic (`ocean_floor_height_exclusive` plus the incremental/downgrade heightmap
  maintenance in `proto_chunk.rs` lines ~392-432) was audited and looks deliberately
  correct (has an explicit comment citing the exact aquifer/carver-replaces-solid-with-
  fluid scenario it guards against), so the true root cause of the "floating with zero
  support" report, if it's a distinct bug, is still open. Recommend a follow-up
  live-testing pass on a fixed seed to check whether this fix already resolved the
  visible symptom before spending more time on it.

### Investigated, concluded not a bug (re-classify, don't re-dispatch)

- **"Magma in rivers"**: `underwater_magma.rs`'s own placement predicate (solid floor
  required, no open water/air on any horizontal face) already prevents floating/exposed
  placement. Cross-checked `pumpkin-data/src/generated/biome.rs` (auto-generated from
  real game data, not hand-authored) and confirmed `UnderwaterMagma` sits in the shared
  `underground_ores` decoration step (alongside `OreCopper`, `DiskSand`, `DiskClay`,
  `DiskGravel`) for the `RIVER` biome specifically, not just ocean biomes - this is a
  data-driven registration, not a Pumpkin-authored bug. The wiki
  (`https://minecraft.wiki/w/Magma_Block`, Natural Generation section) describes magma
  generating "at the bottom of water aquifers" generically across the Overworld, not as
  an ocean-exclusive mechanic, which is consistent with rivers legitimately getting
  occasional magma clusters wherever a water column meets a qualifying solid floor.
  Recommend re-verifying the original live-testing observation (biome may have been
  misidentified) rather than treating this as a confirmed parity bug.

### Investigated, root-caused but blocked on source access (scoped follow-up)

- **Ruined Portal never decayed** (`pumpkin-world/src/generation/structure/structures/ruined_portal.rs`):
  confirmed - `RuinedPortalPiece::place` calls `place_template` directly on the raw
  `ruined_portal/portal_N`/`giant_portal_N` templates with no post-processing pass at
  all. Contrast with `shipwreck.rs`, whose `TEMPLATES` list includes explicit
  `_degraded` NBT variants (damage baked into the template data itself, no runtime pass
  needed) - ruined portal has no such variants, meaning vanilla's decay must be a
  runtime pass applied after placement, and Pumpkin has never implemented it. This
  fully explains the live-testing report of a "fully intact, undamaged" underwater
  portal. **Why not fixed this session**: vanilla's actual decay pass (mossiness-based
  stone/mossy-stone block substitution, cold-biome netherrack-for-lava/magma swaps,
  obsidian→crying-obsidian chance, air-pocket carving, vine placement) needs exact
  per-block percentages and RNG call order from `RuinedPortalPiece.java`/
  `RuinedPortalStructure.java`, which the wiki only paraphrases approximately (e.g. "15%
  ...20%" for crying obsidian - not a citable exact number) and decompiled source is
  currently unavailable (see infra note above). Attempting this from memory/wiki-
  paraphrase would risk exactly the kind of unverifiable claim CLAUDE.md's verification
  discipline rules out. **Scoped as its own follow-up**: port
  `RuinedPortalPiece.postProcess`'s decay pass once decompiled source is available
  again; standalone, no interdependency with other items here.

### Audited, confirmed real but far too large for this session

- **Ocean Monument shape** (`pumpkin-world/src/generation/structure/structures/ocean_monument.rs`):
  confirmed - `OceanMonumentPiece::place` is a hand-rolled symmetric stepped-pyramid
  shell (a nested-square loop over 15 layers, prismarine/prismarine-bricks/dark-
  prismarine by simple modulo), not vanilla's actual room-based monument (a fixed,
  asymmetric layout of named rooms - entry, core, treasure room, wing corridors - each
  with guardian/elder-guardian placement and its own internal geometry). Confirmed
  `pumpkin-world/assets/structures/` has no monument NBT templates at all, meaning
  vanilla's `OceanMonumentPieces` is not template-driven either - it's a from-scratch
  procedural room-grid algorithm across many small piece classes. This is a multi-file,
  multi-week port, not a placement-condition bug like the ones already fixed this
  session (glow lichen, huge mushroom, kelp). **Explicitly not attempted** per this
  task's own scoping guidance; recorded here as a confirmed, well-understood gap for
  whoever picks up structures next.

### Cross-referenced, likely a downstream symptom rather than its own bug

- **Shipwreck on dry land**: audited `shipwreck.rs` end to end - biome-tag gating
  (`structure.biomes` checked against `#minecraft:has_structure/shipwreck` /
  `..._beached` at the structure's `start_pos`), rotation/template selection, and
  height-map sampling (`OceanFloorWg` for open-ocean, `WorldSurfaceWg` for the beached
  variant, with distinct vertical offsets) all look structurally sound and correctly
  wired - no bug found in the structure's own logic. The most parsimonious explanation
  given this session's *other* live-testing report of "an oddly massive, perfectly flat
  sand beach" (already tracked above under the continentalness/erosion investigation,
  out of scope for this feature/structure pass) is that the beached shipwreck variant is
  legitimately placed inside a real (oversized) beach biome, but the beach is so much
  wider than vanilla that no water ends up anywhere near it. Recommend re-testing
  shipwreck placement only after the oversized-beach/continentalness bug is fixed,
  rather than treating this as its own structure bug.

### Out of scope, already tracked elsewhere

- Oversized beaches / "overwhelmingly ocean" worldgen: this is a noise-router/
  continentalness issue, not a feature or structure file - already flagged above in
  this same document under the 2026-08-04 live-testing section. Not re-litigated here.

## Live-testing reports 2026-08-05 (post Nether-crash fix, build 465d2e6)

Nether entry CONFIRMED WORKING by the player after 465d2e6 deployed. The chunk-section
desync (short chunks on generation AND on disk load) is closed.

New reports from the same session, all in/around the Nether, none yet triaged:

1. **Slimes spawning in the Nether.** Screenshot shows a green slime in a Nether cave.
   Vanilla spawns magma cubes in the Nether; slimes are Overworld-only (swamps at night
   in a light/height band, plus slime chunks below y=40). Suspect the slime spawn rule is
   not dimension-gated, or the slime-chunk check ignores dimension.
2. **Slimes and magma cubes float / do not fall.** Screenshots show a slime embedded in a
   cave ceiling and a magma cube hanging in mid-air. NOTE: an earlier report this same
   session ("floating entities") may be the same underlying gravity/physics bug rather
   than a slime-family-specific one - check whether gravity is applied to these mob types
   at all before assuming it is slime-specific.
3. **Nether fungus tree generation is wrong.** Screenshot of a crimson forest shows caps
   generating as large flat single-layer plates scattered across the terrain, with sparse
   thin stems, instead of vanilla's huge-fungus shape (stem with a wrapped cap and
   shroomlight inclusions). Affects both crimson and warped variants. Likely the
   huge-fungus feature's cap placement, related in kind to the already-fixed huge mushroom
   bug.
4. **Lighting broken crossing Nether -> Overworld into a cave.** Large unlit/black regions
   with hard seams. May be an instance of the already-tracked lighting gaps (CLightUpdate
   mask/nibble order, get_sky_light_level sign bug, vanilla-import relight suppression -
   see PARITY.md), or fallout from the new section-padding on load: padded sections are
   created unlit, so if a cross-dimension load path relies on them being lit, seams would
   appear. WORTH CHECKING FIRST given the timing.

Item 4's timing correlation with the section-padding change makes it the highest priority
to investigate - a fix that trades a crash for broken lighting is not a good trade.
