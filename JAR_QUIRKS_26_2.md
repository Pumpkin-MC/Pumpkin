# Minecraft 26.2 JAR-derived parity backlog

This backlog records behavior found by comparing the mapped classes decompiled from the
official Minecraft 26.2 server payload with Pumpkin. The payload SHA-256 is
`183c0499c5f855570ee487dd38e141a53f0121f83a0b07a3bac2d8b6698823e8`.

These are hypotheses backed by source-level branch evidence. A row is not considered
verified or fixed until it has a minimized differential scenario against the pinned
official server and a Pumpkin regression test.

| Severity | Area | Vanilla evidence | Pumpkin divergence | Minimal scenario |
| --- | --- | --- | --- | --- |
| Critical | Tick ordering | `ServerLevel.tick`, `LevelTicks.runCollectedTicks` execute block ticks, fluid ticks, then random ticks serially | `World::tick_chunks` spawns all three categories into one `JoinSet` | Same-tick opposed/side-locking repeaters; repeat and compare final state |
| Critical | Attack validation | `ServerGamePacketListenerImpl.handleAttack` checks reach, spectator, border, target class, item enablement, and attack eligibility | Attack handlers resolve any world entity ID and call `attack` | Send an attack packet against a mob 20 blocks away |
| Data corruption | Transmute recipes | `TransmuteRecipe.matches` requires one input and the material count | Matcher accepts any two slots matching input-or-material | Put two dye/material stacks in the grid |
| Data corruption | Transmute components | `TransmuteRecipe.computeResult` calls `createWithOriginalComponents` | Result construction creates a fresh stack without the input component patch | Dye a filled shulker box or bundle |
| Data corruption | Decorated pots | `DecoratedPotRecipe.assemble` writes `POT_DECORATIONS` from all four inputs | Special recipe produces a default undecorated pot | Craft with four distinct sherds and inspect after placement/break |
| High | Weather cycle | `ServerLevel.advanceWeatherCycle` advances only when `ADVANCE_WEATHER` is true | Runtime advances only when `weather_cycle_enabled` is false | Set rain timer to one and tick with rule true, then false |
| High | Clock/weather persistence | `MinecraftServer` hydrates `WeatherData` and `ServerClockManager` | Saved files are read but runtime clock/weather are constructed from defaults | Set midnight and thunder, cleanly restart, query immediately |
| High | Scheduled ticks | `ScheduledTick` uses long trigger time and preserves `subTickOrder` | Delay is `u8`, delay 300 wraps, and reload resets order to zero | Import `t=300` plus two ordered same-time ticks and restart |
| High | Random tick speed | `ServerLevel.tickChunk` samples exactly the gamerule value | Pumpkin samples three positions per section unconditionally | Compare crop/fire fixture at speeds zero and 100 |
| High | Comparator notification | `BlockEntity.setChanged` calls `updateNeighbourForOutputSignal` | Inventory dirtiness only sets an atomic bit; comparator update is TODO | Insert a stack into chest feeding comparator and lamp |
| High | Fluid containers | `FlowingFluid` delegates to `LiquidBlockContainer` and checks merged collision faces | Spread only replaces/breaks blocks; no waterlogging-container or face path | Flow water into a dry waterloggable slab/stair in a trough |
| High | Runtime skylight | `SkyLightEngine` subtracts `max(1, opacity)` and checks face occlusion | Runtime subtracts one plus opacity and ignores face shapes | Propagate skylight through a dampening-one partial block |
| High | Precipitation ticks | `ServerLevel.tickPrecipitation` freezes water, adds snow, and invokes block hooks | No precipitation sample/action exists in the world tick path | Cold rainy arena with exposed water and cauldron |
| High | Lightning selection | `tickThunder` uses heightmap, rain, difficulty, rods, spawn rule, and trap state | Random Y offset and multiple literal/TODO gates choose invalid locations/traps | Roof, open area, and lightning rod in one forced-thunder chunk |
| High | PvP configuration | Vanilla PvP filtering does not prevent player attacks on mobs | Global `pvp.enabled=false` return occurs before target classification | Disable PvP and attack a zombie |
| High | Shield melee blocking | Damage sources carry attacker position used by `applyItemBlocking` | Player melee passes no source position, so shield path is skipped | Defender raises shield facing attacker |
| High | Hurt cooldown | Vanilla compares/stores damage in one pre-reduction domain | Pumpkin compares reduced damage with stored raw damage | Armored target receives raw 10 then 15 within ten ticks |
| High | Movement authority | `handleMovePlayer` validates rate, speed, collisions, and corrections | Position handlers directly set position with TODO validation | Send a movement packet 20 blocks through a wall |
| High | Melee line of sight | `MeleeAttackGoal.canPerformAttack` requires range and sensing LOS | Goal checks cooldown and range only | Put full block between close zombie and player |
| High | Skeleton combat | `AbstractSkeleton.reassessWeaponGoal` selects ranged bow goal | Skeleton base always installs melee goal | Place skeleton and player ten blocks apart in open arena |
| Gameplay | Crossbow charge state | `CrossbowItem.use` requires charged projectile list to be non-empty | Presence of the always-present empty component counts as charged | Use fresh crossbow with arrow |
| Gameplay | Crafting remainders | `ResultSlot.onTake` restores, merges, or drops every remaining item | Crafting decrements all inputs and never applies remainders | Craft cake and count empty buckets |
| Gameplay | Stonecutter selection | `StonecutterMenu.clickMenuButton` stores selection and refreshes output | Screen handler has no button-click override | Insert stone and click stone-bricks recipe |
| Gameplay | Consumable effects | `Consumable.onConsume` invokes every configured consume effect | Only hardcoded food/potion plus clear-all effects execute | Eat chorus fruit or honey bottle |
| Gameplay | Mob buckets | `MobBucketItem.checkExtraContent` spawns saved entity | Bucket places fluid; entity spawning is TODO | Empty a cod bucket |
| Gameplay | Firework rockets | Item consumes a rocket; entity retains payload, lifetime, and explosion damage | Item is not decremented and entity is payload-free/non-damaging | Reuse one elytra rocket; explode starred rocket beside mob |
| Gameplay | Fishing retrieval | `FishingHook.retrieve` rolls loot, spawns items/XP, and damages rod | COD stack is constructed but never spawned or inserted | Reel during a bite and inspect inventory/world/XP |
| Gameplay | Piercing arrows | `AbstractArrow` tracks distinct pierced entity IDs up to level plus one | Global `has_hit` latch prevents all later entity hits | Shoot Piercing I through two aligned mobs |
| Gameplay | Furnace XP | Vanilla probabilistically rounds each fractional recipe total | Pumpkin floors aggregate XP | Repeatedly extract one iron ingot per smelt |
| Gameplay | Partial-block paths | `WalkNodeEvaluator` uses collision-shape height and partial-shape checks | Floor is integral and unrecognized non-full collisions become open | Path across slabs/stairs or a low partial-shape passage |
| Gameplay | Teleport confirms | Vanilla ignores stale or duplicate confirm IDs | Pumpkin disconnects on mismatch or no pending teleport | Replay a valid confirm packet |
| Gameplay | Critical/sweep attacks | Vanilla gates on fluids, climbing, passenger, ground, speed, and target type | Attack type uses only charge, fall/air, sprint, and sword predicates | Attack while in water or sweep while airborne/moving fast |

## Audit workflow

1. Select an independent row whose prerequisite subsystem is stable.
2. Encode its vanilla setup/action/checkpoint as a conformance case.
3. Confirm the case is deterministic or declare its statistical observation budget.
4. Run the official server and preserve the oracle transcript and state snapshot.
5. Reproduce the divergence on Pumpkin.
6. Add a minimized Rust regression test and implement one logical fix.
7. Run focused tests, strict Clippy, the workspace suite, and the differential case.
8. Commit the fix separately and mark the row with its evidence and commit.
