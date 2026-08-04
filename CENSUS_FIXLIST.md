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
