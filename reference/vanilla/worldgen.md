# World generation: features, carvers, state providers

Source: `net.minecraft.world.level.levelgen.feature.**`, decompiled 26.2 Mojang mappings.

## Registry completeness (already verified, don't re-litigate)

A prior triage conflated file-count granularity: comparing 102 Rust files against 177 vanilla
*files* (which include `configurations/`, `foliageplacers/`, `trunkplacers/`, etc. as separate
files per class) made the gap look bigger than it is. At the actual registry level:

- Vanilla has exactly **63 registered `Feature<C>` types**. Pumpkin's `ConfiguredFeature` enum
  (`pumpkin-world/src/generation/feature/configured_features.rs`) has a match arm for all 63.
- Foliage placers 11/11, trunk placers 9/9, tree decorators 10/10, root placers 1/1, feature-size
  types 2/2 — all fully covered.
- Carvers: vanilla only has 3 carver classes (`CaveWorldCarver`, `CanyonWorldCarver`,
  `NetherWorldCarver` — there is no separate "NetherCaveCarver"). Pumpkin's `cave.rs`/`canyon.rs`
  already dispatch nether-vs-overworld carving correctly (`NetherWorldCarver`'s overridden bound
  10, thickness formula, y-scale 5.0, and `carveBlock`'s lava-below-`minGenY+31` rule all match).

**Do not spend time re-auditing "missing features/carvers" as a category — there isn't one.**
The real gaps are individual feature implementations that drifted from vanilla, and a few
genuinely missing pieces in shared helpers (state providers). Known remaining ones below.

## Fixed this session (for context, don't re-fix)

- `RandomizedIntBlockStateProvider::get` and `PillarBlockStateProvider::get`
  (`pumpkin-world/src/generation/block_state_provider.rs`) were stubs that ignored their sampled
  value / always returned a default axis. Now implemented against
  `stateproviders/{RandomizedIntStateProvider,RotatedBlockProvider}.java`.
- `TwistingVinesFeature` (`generation/feature/features/twisting_vines.rs`): was missing
  `NETHERRACK` as valid ground, used a two-call RNG offset (`nextInt(w) - nextInt(w)`) instead of
  vanilla's single ranged call (`Mth.nextInt(random, -w, w)`) — this desyncs the RNG stream for
  every subsequent chunk feature, not just this one — used the wrong shorten-chance denominator
  (10 instead of 5), and never set vine age. Fixed against `TwistingVinesFeature.java`.
- `WeepingVinesFeature` was a near-total stub (straight-down column, fixed length, no age, no
  roof). Rewritten against `WeepingVinesFeature.java`'s actual two-pass algorithm: 200 scattered
  nether-wart-block roof probes with neighbor-count checks, then 100 scattered vine probes with
  height/doubling/shortening/age logic. Note: unlike twisting vines, weeping vines' offset RNG
  *is* the two-call subtraction pattern — don't "fix" it to match twisting vines, they're
  genuinely different in vanilla.

## Known remaining gaps (not fixed, small, independent — good units of work)

Each of these is its own class, own file, no interdependency — safe to pick off individually:

- `blue_ice.rs`, `iceberg.rs`, `freeze_top_layer.rs`, `basalt_columns.rs`: hardcode
  `SEA_LEVEL = 63` instead of reading it from world generation settings
  (`generator.settings.sea_level`, already threaded through elsewhere — see
  `pumpkin-world/src/generation/carver/mod.rs:168` for the pattern). **Warning before attempting
  this**: the shared `ConfiguredFeature::generate` dispatch function in
  `configured_features.rs` does NOT currently pass `sea_level`/`settings` down to individual
  feature `generate()` calls at all — adding it means touching the dispatch signature and
  every one of the ~63 match arms, not just these 4 files. Scope this as "thread sea_level
  through the feature-generate call chain," not "fix 4 constants," or the diff will be far
  bigger than expected.
- `coral_claw.rs`, `coral_tree.rs`, `tree/decorator/attached_to_logs.rs`: missing
  `shuffle`/random branch-direction selection — an RNG-call-order divergence, same bug class as
  the twisting-vines offset issue. Worth auditing the exact vanilla RNG call sequence before
  touching, since getting the *order* of random calls right (not just the final value
  distribution) is what actually matters for seed-reproducibility.
- `simple_block.rs`, `root_system.rs`, `vines.rs`, `drip_stone/small.rs`: smaller individual
  TODOs (schedule-tick-on-place, a water check, etc.) — check each file's own comments.

## Vanilla RNG-order rule of thumb

Multiple bugs found this session were not "wrong final value" but "right value from the wrong
number/order of RNG calls" (e.g. twisting vines' two-call subtraction vs. vanilla's single
ranged call). This matters because every RNG call after the wrong one is now desynced from
vanilla for the same seed. When porting a feature, count exactly how many `random.next*()` calls
vanilla makes and in what order — don't just match the final distribution.
