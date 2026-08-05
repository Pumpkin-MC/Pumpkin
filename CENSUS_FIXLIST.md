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
