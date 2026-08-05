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
