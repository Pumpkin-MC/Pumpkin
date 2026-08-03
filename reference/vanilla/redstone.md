# Redstone: quasi-connectivity

Source: `net.minecraft.world.level.block.piston.PistonBaseBlock`,
`net.minecraft.world.level.block.SignalGetter`, decompiled 26.2 mappings.

## Correction: quasi-connectivity was NOT actually missing

An earlier scoping pass in this session concluded quasi-connectivity was "entirely absent" from
a keyword grep for "quasi" — which found nothing because Pumpkin's code doesn't use that word
anywhere, not because the mechanic was missing. **It was already implemented:**

- `should_extend()` in `pumpkin/src/block/blocks/piston/piston.rs` already ports
  `PistonBaseBlock#getNeighborSignal` faithfully, including the "check the block above the
  piston" loop that IS the BUD-switch/quasi-connectivity logic.
- The general case (a solid block conducts strong signal from any of its 6 neighbors, used by
  every redstone consumer, not just pistons) is already implemented in
  `pumpkin/src/block/blocks/redstone/mod.rs` via `get_redstone_power`/`get_max_strong_power`,
  mirroring `SignalGetter#getSignal` + `SignalGetter#getDirectSignalTo`.

**Do not re-implement quasi-connectivity from scratch based on a "missing" report — it isn't
missing.** If you're asked to work on it, first verify what's actually there before assuming a
whole-mechanic rewrite is needed; you'll likely find (as this session did) a narrow, specific bug
instead of an absent feature.

## The real, narrow bug that WAS found and fixed

`should_extend()`'s port of vanilla's `if (level.hasSignal(pos, Direction.DOWN)) return true;`
(`PistonBaseBlock.java:134-136`) read the block/state of the position **below** the piston
instead of the piston's own block/state, while still passing the piston's own position as the
query position for the power lookup.

Vanilla evaluates this check against the piston's own block: solidity of the piston's own block
gates whether `getDirectSignalTo(pos)` (quasi-connectivity power from the piston's own six
neighbours) is considered at all (`SignalGetter.java:61-69`). Reading the block below instead
meant the piston's own quasi-power check was silently skipped whenever the block underneath
happened to be non-solid (air, a slab, glass, etc.) — breaking some BUD-switch/piston-door style
contraptions.

Fix: use `block_pos` (the piston's own position) for both the block/state lookup and the power
query, matching vanilla exactly. One file, `piston.rs`, ~5 lines changed.

## Lesson for future audits

When a prior scoping note says a mechanic is "entirely absent," verify by reading the actual
code for the relevant vanilla class name(s) and their non-obvious Pumpkin-side equivalents
(different naming, no exact keyword match) before assuming a full reimplementation is the task.
A keyword-grep-based "missing" conclusion is not the same as confirmed-absent.
