# Differential Conformance Runner

Replaces gut-estimate parity percentages with a measured one. Two axes, reported
separately because they answer different questions and conflating them is how the
prior "60-70%" estimate ended up 3x the community's "~30%" figure.

- **Coverage** — does a Pumpkin analogue exist at all for a given vanilla class?
  Computed mechanically (name matching, no AI) over the full enumerated population
  of gameplay-relevant vanilla 26.2 classes. Cheap, exhaustive, reproducible.
- **Fidelity** — given an analogue exists, does it actually behave like vanilla?
  Requires reading both implementations. Too expensive to run over the whole
  population, so it's measured on a stratified random sample and reported with
  its sample size `n`. Never presented as if it covers the full population.

`composite_estimate = coverage% * fidelity_pass_rate%` is an estimate that assumes
the sampled fidelity rate generalizes to subsystems that weren't sampled. Always
quote it with the sample size and the list of unchecked subsystems.

## Pipeline

1. `enumerate.py` — walks the decompiled vanilla 26.2 source
   (`/tmp/pumpkin-vanilla-26.2/decompiled`) under the gameplay-relevant packages
   (`world/level/block`, `world/entity`, `world/item`, `world/inventory`,
   `world/level/material`, `world/food`, `world/level/border`,
   `world/level/gameevent`) and extracts every public/protected class as a
   conformance unit, tagged with `milestone: "26.2"`. No AI, no Pumpkin awareness.
   -> `vanilla_units.json`

2. `map_coverage.py` — for each enumerated class, checks the Pumpkin workspace for
   an analogue by exact struct/enum name, case-insensitive struct name (catches
   e.g. vanilla `WorldBorder` vs Pumpkin `Worldborder`), or exact snake_case
   filename. Deliberately does NOT do fuzzy/substring matching — an earlier version
   did, and it matched garbage like vanilla `PlayDead` to Pumpkin's `net/bedrock/play.rs`
   purely because both strings contain "play". Exact-only understates coverage on
   genuine semantic renames but doesn't lie in the other direction.
   -> `catalog.json`, prints coverage % overall and per subsystem.

3. `sample_fidelity.py` — draws a stratified random sample (fixed seed 262, up to
   6 per subsystem) from the *covered* classes only. Fidelity can't be checked on
   classes with no analogue.
   -> `fidelity_sample.json`

4. Each sampled class gets read on both sides (vanilla Java + Pumpkin Rust) and
   compared for behavior, not just existence — timings, damage values, formulas,
   condition logic. Verdict: `pass` / `fail` / `not_found`.
   -> `fidelity_results.json` (hand-authored from agent findings this run; not yet
   scripted as an automated step)

5. `score.py` — combines catalog.json + fidelity_results.json into `score.json`
   with both numbers, the composite estimate, and the caveat text.

## Current numbers (milestone 26.2, run 2026-08-02)

```
Coverage (mechanical, full population): 28.35% (453/1598 classes)
Fidelity (sampled, n=16, subsystems=[block, entity, inventory]): 50.0%
  unchecked subsystems: [border, food, gameevent, item, material]
Composite estimate: 14.18%
```

Coverage lines up with the community's "~30%" figure — that number was almost
certainly measuring existence, not correctness. The prior "60-70%" verbal estimate
this session gave was implicitly measuring fidelity-on-audited-code, which is a
heavily biased sample (only fixed/notable findings get looked at, and most of them
get fixed on sight) — 50% on an actually-random sample is a very different picture,
and composite estimate lands far below either verbal number.

Per-subsystem coverage varies enormously: `block` 51%, `border`/`food` near 0%
(mostly semantic renames the exact-match heuristic misses, e.g. `FoodData` vs
`HungerManager`), `inventory` only 6%. Read `catalog.json`'s `by_subsystem` before
assuming any single number represents the whole game.

## What this run found

Real, currently-open behavioral gaps discovered by the sampled fidelity check (see
`fidelity_results.json` for full evidence, file:line on both sides):

- Twisting vines still checks the wrong support face (`Down` instead of `Up`) —
  the PR #2604 fix did not actually resolve this; still broken.
- Donkey breeding (donkey x horse -> mule) is entirely unimplemented — no
  `BreedGoal` registered at all.
- Pufferfish puff-state/contact-damage/poison mechanic is entirely unimplemented.
- `FollowParentGoal` uses inverted distance-check logic vs vanilla (follows a
  farther parent in a case where vanilla refuses to follow anyone that tick).
- `Attribute` values are never clamped to vanilla's per-attribute min/max range.
- `ResultSlot` never triggers recipe-book unlock tracking on craft-take.
- `ArmorSlot` equip-triggered entity callbacks (sound/stat/game-event hooks) never
  fire — `set_stack_prev` is a stub.
- `Tilt` (sculk sensor big dripleaf state) carries no vibration data, so sculk
  sensors never detect dripleaf tilt changes.

These are candidates for the next fix pass; not yet applied.

## Known limitations / next steps

- Exact-match-only coverage undercounts genuine semantic renames. A small,
  explicitly-maintained alias table (not fuzzy matching) could recover some of
  these without reintroducing false positives — not yet built.
- Fidelity sample only covers 16 of a planned 26 units across 3 of 7 populated
  subsystems (`border+gameevent`, `item`, `material` batches were not run this
  pass). Re-running `sample_fidelity.py` + the comparison step for the missing
  subsystems is the highest-value next step for tightening the composite estimate.
- Fidelity comparison (step 4) is currently manual/agent-driven per run, not a
  script. Scaling this to the full covered population (~450 classes) is a large
  job — call it out explicitly and let the user decide on scale/tooling (e.g. the
  Workflow tool) rather than silently running hundreds of agent calls.
- `food` and `gameevent` subsystems have near-zero mechanical coverage; worth a
  manual pass to check for renames before concluding those areas are actually
  unimplemented.
